mod config;
mod error;
mod executor;
mod steps;
pub mod ui;
mod widgets;

pub use config::OnboardConfig;
pub use steps::StepResult;
pub use widgets::StatusBarState;

use tokio::sync::mpsc;

#[derive(Debug)]
pub enum ExecutionMessage {
    TaskStarted(usize),
    TaskSuccess(usize, Option<String>),
    TaskFailed(usize, String),
    UserCreated(Option<String>),
    ReviewComplete { any_failed: bool },
    UpdateComplete { any_failed: bool },
    StepComplete { step_result: StepResult },
}

use crate::ui::Theme;
use crate::vim::{InputBuffer, ModeAction, VimMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use steps::StepId;
use tracing::warn;

/// Actions that can be triggered by the onboard app
#[derive(Debug)]
pub enum OnboardAction {
    /// Launch an external program (program, args)
    LaunchExternal(String, Vec<String>),
    /// Reboot the system
    Reboot,
    /// Power off the system
    Poweroff,
    /// Execute current step (user form submission, updates, etc.)
    ExecuteStep,
    /// Execute Review step - apply all configuration
    ExecuteReview,
    /// Execute Update step - run system updates
    ExecuteUpdate,
    /// Exit onboard and go to greeter login
    ExitToLogin,
    /// In dryrun mode: show fake reboot animation then transition to login
    TransitionToLogin,
}

/// Which panel is focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    /// Welcome screen (before setup starts)
    Welcome,
    /// Sidebar with step list
    Sidebar,
    /// Main content panel
    Content,
}

/// What is currently focused in the content panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFocus {
    /// Focused on a picker list
    Picker,
    /// Focused on an input field (0=username, 1=password, 2=confirm)
    InputField(usize),
    /// No specific focus (viewing tasks/info)
    None,
}

/// Message displayed to the user
pub struct Message {
    pub text: String,
    pub is_error: bool,
}

/// Confirm action dialog state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Reboot,
    Poweroff,
    Cancel,
}

/// Main onboard application state
pub struct OnboardApp {
    pub config: OnboardConfig,
    pub theme: Theme,

    // Vim mode state
    pub vim_mode: VimMode,
    pub command_buffer: InputBuffer,

    // Panel navigation
    pub panel_focus: PanelFocus,
    pub content_focus: ContentFocus,

    // Sidebar state
    pub menu_items: Vec<MenuItem>,
    pub selected_step: usize,

    // Inline picker state (for locale/keyboard/timezone)
    pub picker_items: Vec<String>,
    pub picker_selected: usize,
    pub picker_filter: InputBuffer,

    // Form fields for user creation
    pub username: InputBuffer,
    pub password: InputBuffer,
    pub password_confirm: InputBuffer,

    // Sudo password for commands that require it (entered on Update step)
    pub sudo_password: InputBuffer,
    pub sudo_password_needed: bool,
    pub sudo_password_entered: bool,

    // Task execution state
    pub tasks: Vec<TaskStatus>,
    pub current_task: Option<usize>,
    pub is_executing: bool,

    // Selected values for display
    pub selected_locale: Option<String>,
    pub selected_keyboard: Option<String>,
    pub selected_timezone: Option<String>,

    // UI state
    pub message: Option<Message>,
    pub confirm_action: Option<ConfirmAction>,
    pub show_help: bool,
    pub should_exit: bool,
    pub setup_started: bool,
    pub setup_complete: bool,

    // Spinner for async operations
    spinner_frame: usize,

    // Network status (cached)
    pub network_connected: bool,

    // Created user info (set after user creation step)
    pub created_username: Option<String>,

    // Step results tracking
    pub step_results: Vec<StepResult>,

    // Review completion unlocks Update step
    pub review_completed: bool,

    // Update completion (or skip) unlocks Reboot step
    pub update_completed: bool,

    // Update package selection: update_package_selected[category_idx][package_idx]
    pub update_package_selected: Vec<Vec<bool>>,
    // Navigation: which category is focused
    pub update_category_cursor: usize,
    // Navigation: which package within the category (None = on the category header)
    pub update_package_cursor: Option<usize>,
    // Per-category scroll offset (number of packages to skip from top)
    pub update_category_scroll: Vec<usize>,

    // Status bar state - updated by content panels
    pub status_bar: StatusBarState,

    // Dryrun simulation state for progress animation
    dryrun_sim_active: bool,
    dryrun_sim_task_idx: usize,
    dryrun_sim_progress: u8,
    dryrun_sim_callback: Option<DryrunCallback>,
}

/// Callback to execute after dryrun simulation completes
#[derive(Debug, Clone, Copy)]
pub enum DryrunCallback {
    CompleteUpdate,
    CompleteReview,
}

/// A menu item in the setup wizard
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: StepId,
    pub required: bool,
    pub has_picker: bool,
    pub has_form: bool,
}

/// Status of a task being executed
#[derive(Debug, Clone)]
pub struct TaskStatus {
    pub name: String,
    pub status: TaskState,
    pub output: Option<String>,
    /// Progress percentage (0-100) for dryrun simulation
    pub progress: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Running,
    Success,
    Failed,
}

impl OnboardApp {
    pub fn new(config: OnboardConfig) -> Self {
        let menu_items = Self::build_menu_items(&config);
        let step_results = Self::initial_step_results(&menu_items);

        // Check network status immediately
        let network_connected = executor::check_network(config.general.dryrun);

        // Initialize per-package selection based on defaults (before moving config)
        let update_package_selected: Vec<Vec<bool>> = config.updates.iter()
            .map(|cat| {
                cat.packages.iter()
                    .map(|pkg| pkg.is_default_enabled(cat.enabled_by_default))
                    .collect()
            })
            .collect();

        Self {
            config,
            theme: Theme::default(),
            vim_mode: VimMode::Normal,
            command_buffer: InputBuffer::new(),
            panel_focus: PanelFocus::Welcome,
            content_focus: ContentFocus::None,
            menu_items,
            selected_step: 0,
            picker_items: Vec::new(),
            picker_selected: 0,
            picker_filter: InputBuffer::new(),
            username: InputBuffer::new(),
            password: InputBuffer::masked(),
            password_confirm: InputBuffer::masked(),
            sudo_password: InputBuffer::masked(),
            sudo_password_needed: false,
            sudo_password_entered: false,
            tasks: Vec::new(),
            current_task: None,
            is_executing: false,
            selected_locale: None,
            selected_keyboard: None,
            selected_timezone: None,
            message: None,
            confirm_action: None,
            show_help: false,
            should_exit: false,
            setup_started: false,
            setup_complete: false,
            spinner_frame: 0,
            network_connected,
            created_username: None,
            step_results,
            review_completed: false,
            update_completed: false,
            update_category_scroll: vec![0; update_package_selected.len()],
            update_package_selected,
            update_category_cursor: 0,
            update_package_cursor: None,
            status_bar: StatusBarState::welcome(),
            dryrun_sim_active: false,
            dryrun_sim_task_idx: 0,
            dryrun_sim_progress: 0,
            dryrun_sim_callback: None,
        }
    }

    fn initial_step_results(menu_items: &[MenuItem]) -> Vec<StepResult> {
        menu_items.iter().map(|item| {
            match item.id {
                // Update and Login start locked
                StepId::Update | StepId::Reboot => StepResult::Locked,
                _ => StepResult::Pending,
            }
        }).collect()
    }

    fn build_menu_items(config: &OnboardConfig) -> Vec<MenuItem> {
        let mut items = Vec::new();

        // 1. User creation (required)
        items.push(MenuItem {
            id: StepId::User,
            required: true,
            has_picker: false,
            has_form: true,
        });

        // 2. Locale (optional)
        if config.locale.enabled {
            items.push(MenuItem {
                id: StepId::Locale,
                required: false,
                has_picker: true,
                has_form: false,
            });
        }

        // 3. Keyboard (optional)
        if config.keyboard.enabled {
            items.push(MenuItem {
                id: StepId::Keyboard,
                required: false,
                has_picker: true,
                has_form: false,
            });
        }

        // 4. Network (optional, for WiFi setup)
        if config.network.enabled {
            items.push(MenuItem {
                id: StepId::Network,
                required: false,
                has_picker: false,
                has_form: false,
            });
        }

        // 5. Preferences/Timezone (optional)
        if config.preferences.timezone_enabled {
            items.push(MenuItem {
                id: StepId::Preferences,
                required: false,
                has_picker: true,
                has_form: false,
            });
        }

        // 6. Review - always present, shows summary and applies config
        items.push(MenuItem {
            id: StepId::Review,
            required: true,
            has_picker: false,
            has_form: false,
        });

        // 7. Update - locked until Review completes, only shown if update categories are configured
        if !config.updates.is_empty() {
            items.push(MenuItem {
                id: StepId::Update,
                required: false,
                has_picker: false,
                has_form: true, // has_form for sudo password input
            });
        }

        // 8. Login - locked until Update completes or is skipped
        items.push(MenuItem {
            id: StepId::Reboot,
            required: true,
            has_picker: false,
            has_form: false,
        });

        items
    }

    /// Get the currently selected menu item
    pub fn current_item(&self) -> Option<&MenuItem> {
        self.menu_items.get(self.selected_step)
    }

    /// Get the currently selected step ID
    pub fn current_step_id(&self) -> Option<StepId> {
        self.current_item().map(|item| item.id)
    }

    /// Check if a step is locked
    pub fn is_step_locked(&self, idx: usize) -> bool {
        self.step_results.get(idx).map(|r| *r == StepResult::Locked).unwrap_or(false)
    }

    /// Check if current step is locked
    pub fn is_current_step_locked(&self) -> bool {
        self.is_step_locked(self.selected_step)
    }

    /// Check if running in dry run mode (demo/testing without real system changes)
    pub fn is_dryrun(&self) -> bool {
        self.config.general.dryrun
    }

    /// Unlock the Update step (called after Review completes)
    fn unlock_update_step(&mut self) {
        for (idx, item) in self.menu_items.iter().enumerate() {
            if item.id == StepId::Update && self.step_results[idx] == StepResult::Locked {
                self.step_results[idx] = StepResult::Pending;
            }
        }
    }

    /// Unlock the Login step (called after Update completes or is skipped)
    fn unlock_login_step(&mut self) {
        for (idx, item) in self.menu_items.iter().enumerate() {
            if item.id == StepId::Reboot && self.step_results[idx] == StepResult::Locked {
                self.step_results[idx] = StepResult::Pending;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<OnboardAction> {
        // Clear message on any key (unless working)
        if self.message.is_some() && !self.is_executing {
            self.message = None;
        }

        // Handle confirm dialog first
        if let Some(action) = self.confirm_action {
            let result = self.handle_confirm_key(key, action);
            self.update_status_bar();
            return result;
        }

        // Handle help popup
        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                self.show_help = false;
            }
            self.update_status_bar();
            return None;
        }

        // Don't handle input while executing
        if self.is_executing {
            return None;
        }

        // Handle based on vim mode
        let result = match self.vim_mode {
            VimMode::Normal => self.handle_normal_mode(key),
            VimMode::Insert => self.handle_insert_mode(key),
            VimMode::Command => self.handle_command_mode(key),
        };

        // Update status bar after any state change
        self.update_status_bar();
        result
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> Option<OnboardAction> {
        // Handle Ctrl+h/l for panel navigation
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('h') => {
                    self.focus_sidebar();
                    return None;
                }
                KeyCode::Char('l') => {
                    self.focus_content();
                    return None;
                }
                _ => {}
            }
        }

        match key.code {
            // Enter command mode
            KeyCode::Char(':') => {
                self.vim_mode = self.vim_mode.transition(ModeAction::EnterCommand);
                self.command_buffer.clear();
            }

            // Navigation
            KeyCode::Char('j') | KeyCode::Down => {
                self.navigate_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.navigate_up();
            }

            // Enter insert mode for text input
            KeyCode::Char('i') | KeyCode::Char('a') => {
                if self.panel_focus == PanelFocus::Content {
                    match self.content_focus {
                        ContentFocus::InputField(_) | ContentFocus::Picker => {
                            self.vim_mode = self.vim_mode.transition(ModeAction::EnterInsert);
                        }
                        _ => {}
                    }
                }
            }

            // Action / Select
            KeyCode::Enter => {
                return self.handle_enter();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.panel_focus == PanelFocus::Sidebar {
                    self.focus_content();
                } else {
                    return self.handle_enter();
                }
            }

            // Go back
            KeyCode::Char('h') | KeyCode::Left => {
                if self.panel_focus == PanelFocus::Content {
                    self.focus_sidebar();
                }
            }
            KeyCode::Esc => {
                if self.panel_focus == PanelFocus::Content {
                    self.focus_sidebar();
                }
            }

            // Tab to cycle through fields
            KeyCode::Tab => {
                self.navigate_down();
            }
            KeyCode::BackTab => {
                self.navigate_up();
            }

            // Function keys
            KeyCode::Char('?') | KeyCode::F(1) => {
                self.show_help = true;
            }
            KeyCode::F(12) => {
                self.confirm_action = Some(ConfirmAction::Poweroff);
            }

            // Quick select by number
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if self.setup_started && self.panel_focus == PanelFocus::Sidebar {
                    let num = c.to_digit(10).unwrap() as usize;
                    if num > 0 && num <= self.menu_items.len() {
                        self.selected_step = num - 1;
                        self.load_step_content();
                    }
                }
            }

            // Space to toggle selection (for Update step packages)
            KeyCode::Char(' ') => {
                if self.panel_focus == PanelFocus::Content {
                    if let Some(StepId::Update) = self.current_step_id() {
                        self.toggle_update_item();
                    }
                }
            }

            KeyCode::Char(c) => {
                if self.panel_focus == PanelFocus::Content
                    && self.content_focus == ContentFocus::Picker
                {
                    self.vim_mode = self.vim_mode.transition(ModeAction::EnterInsert);
                    self.picker_filter.insert(c);
                    self.picker_selected = 0;
                }
            }

            _ => {}
        }
        None
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> Option<OnboardAction> {
        match key.code {
            KeyCode::Esc => {
                self.vim_mode = self.vim_mode.transition(ModeAction::Escape);
            }
            KeyCode::Enter => {
                match self.content_focus {
                    ContentFocus::InputField(field) => {
                        let step_id = self.current_step_id();

                        match step_id {
                            Some(StepId::User) => {
                                // Move to next field, or submit on last field
                                if field < 2 {
                                    self.content_focus = ContentFocus::InputField(field + 1);
                                } else {
                                    self.vim_mode = VimMode::Normal;
                                    return Some(OnboardAction::ExecuteStep);
                                }
                            }
                            Some(StepId::Update) => {
                                // Submit password and run commands
                                if !self.sudo_password.content().is_empty() {
                                    self.sudo_password_entered = true;
                                    self.vim_mode = VimMode::Normal;
                                    return Some(OnboardAction::ExecuteUpdate);
                                }
                            }
                            _ => {
                                self.vim_mode = VimMode::Normal;
                            }
                        }
                    }
                    ContentFocus::Picker => {
                        // Select current picker item
                        self.select_picker_item();
                        self.vim_mode = VimMode::Normal;
                    }
                    _ => {
                        self.vim_mode = VimMode::Normal;
                    }
                }
            }
            KeyCode::Tab => {
                if let ContentFocus::InputField(field) = self.content_focus {
                    if field < 2 {
                        self.content_focus = ContentFocus::InputField(field + 1);
                    }
                }
            }
            KeyCode::BackTab => {
                if let ContentFocus::InputField(field) = self.content_focus {
                    if field > 0 {
                        self.content_focus = ContentFocus::InputField(field - 1);
                    }
                }
            }
            KeyCode::Backspace => {
                self.handle_backspace();
            }
            KeyCode::Delete => {
                self.handle_delete();
            }
            KeyCode::Left => {
                self.handle_cursor_left();
            }
            KeyCode::Right => {
                self.handle_cursor_right();
            }
            KeyCode::Home => {
                self.handle_cursor_home();
            }
            KeyCode::End => {
                self.handle_cursor_end();
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'u' => self.handle_clear_line(),
                        'a' => self.handle_cursor_home(),
                        'e' => self.handle_cursor_end(),
                        'h' => {
                            // Ctrl+H in insert mode goes to sidebar
                            self.vim_mode = VimMode::Normal;
                            self.focus_sidebar();
                        }
                        _ => {}
                    }
                } else {
                    self.handle_char_input(c);
                }
            }
            _ => {}
        }
        None
    }

    fn handle_command_mode(&mut self, key: KeyEvent) -> Option<OnboardAction> {
        match key.code {
            KeyCode::Esc => {
                self.vim_mode = self.vim_mode.transition(ModeAction::Escape);
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                let cmd = self.command_buffer.content().to_string();
                self.vim_mode = self.vim_mode.transition(ModeAction::Execute);
                self.command_buffer.clear();
                return self.execute_command(&cmd);
            }
            KeyCode::Backspace => {
                if self.command_buffer.is_empty() {
                    self.vim_mode = self.vim_mode.transition(ModeAction::Escape);
                } else {
                    self.command_buffer.delete_back();
                }
            }
            KeyCode::Char(c) => {
                self.command_buffer.insert(c);
            }
            _ => {}
        }
        None
    }

    fn handle_confirm_key(&mut self, key: KeyEvent, action: ConfirmAction) -> Option<OnboardAction> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.confirm_action = None;
                match action {
                    ConfirmAction::Reboot => {
                        // In dryrun mode, transition to login instead of rebooting
                        if self.is_dryrun() && self.setup_complete {
                            return Some(OnboardAction::TransitionToLogin);
                        }
                        return Some(OnboardAction::Reboot);
                    }
                    ConfirmAction::Poweroff => {
                        // In dryrun mode, transition to login instead of poweroff
                        if self.is_dryrun() && self.setup_complete {
                            return Some(OnboardAction::TransitionToLogin);
                        }
                        return Some(OnboardAction::Poweroff);
                    }
                    ConfirmAction::Cancel => {
                        self.should_exit = true;
                    }
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.confirm_action = None;
            }
            _ => {}
        }
        None
    }

    fn focus_sidebar(&mut self) {
        if self.setup_started && !self.setup_complete {
            self.panel_focus = PanelFocus::Sidebar;
            self.content_focus = ContentFocus::None;
        }
    }

    fn focus_content(&mut self) {
        if self.setup_started && !self.setup_complete {
            self.panel_focus = PanelFocus::Content;
            // Set appropriate content focus based on current step
            if let Some(item) = self.current_item() {
                if item.has_picker {
                    self.content_focus = ContentFocus::Picker;
                    self.vim_mode = self.vim_mode.transition(ModeAction::EnterInsert);
                } else if item.has_form {
                    self.content_focus = ContentFocus::InputField(0);
                } else {
                    self.content_focus = ContentFocus::None;
                }
            }
        }
    }

    fn navigate_down(&mut self) {
        match self.panel_focus {
            PanelFocus::Welcome => {
                // Nothing to navigate on welcome screen
            }
            PanelFocus::Sidebar => {
                if self.selected_step < self.menu_items.len() - 1 {
                    self.selected_step += 1;
                    self.load_step_content();
                }
            }
            PanelFocus::Content => {
                // Special case for Update step - navigate categories and packages
                if let Some(StepId::Update) = self.current_step_id() {
                    if self.tasks.is_empty() && !self.config.updates.is_empty() {
                        self.navigate_update_down();
                        return;
                    }
                }

                match self.content_focus {
                    ContentFocus::Picker => {
                        let filtered_count = self.filtered_picker_items().len();
                        if self.picker_selected < filtered_count.saturating_sub(1) {
                            self.picker_selected += 1;
                        }
                    }
                    ContentFocus::InputField(field) => {
                        if field < 2 {
                            self.content_focus = ContentFocus::InputField(field + 1);
                        }
                    }
                    ContentFocus::None => {}
                }
            }
        }
    }

    fn navigate_up(&mut self) {
        match self.panel_focus {
            PanelFocus::Welcome => {
                // Nothing to navigate on welcome screen
            }
            PanelFocus::Sidebar => {
                if self.selected_step > 0 {
                    self.selected_step -= 1;
                    self.load_step_content();
                }
            }
            PanelFocus::Content => {
                // Special case for Update step - navigate categories and packages
                if let Some(StepId::Update) = self.current_step_id() {
                    if self.tasks.is_empty() && !self.config.updates.is_empty() {
                        self.navigate_update_up();
                        return;
                    }
                }

                match self.content_focus {
                    ContentFocus::Picker => {
                        if self.picker_selected > 0 {
                            self.picker_selected -= 1;
                        }
                    }
                    ContentFocus::InputField(field) => {
                        if field > 0 {
                            self.content_focus = ContentFocus::InputField(field - 1);
                        }
                    }
                    ContentFocus::None => {}
                }
            }
        }
    }

    /// Navigate down through update categories and packages
    fn navigate_update_down(&mut self) {
        let cat_idx = self.update_category_cursor;
        let cat_count = self.config.updates.len();

        match self.update_package_cursor {
            None => {
                // On category header - move to first package in this category
                if let Some(cat) = self.config.updates.get(cat_idx) {
                    if !cat.packages.is_empty() {
                        self.update_package_cursor = Some(0);
                    } else if cat_idx < cat_count - 1 {
                        // Empty category, move to next category header
                        self.update_category_cursor += 1;
                    }
                }
            }
            Some(pkg_idx) => {
                if let Some(cat) = self.config.updates.get(cat_idx) {
                    if pkg_idx < cat.packages.len() - 1 {
                        // Move to next package in same category
                        self.update_package_cursor = Some(pkg_idx + 1);
                    } else if cat_idx < cat_count - 1 {
                        // Move to next category header
                        self.update_category_cursor += 1;
                        self.update_package_cursor = None;
                    }
                }
            }
        }
    }

    /// Navigate up through update categories and packages
    fn navigate_update_up(&mut self) {
        let cat_idx = self.update_category_cursor;

        match self.update_package_cursor {
            None => {
                // On category header
                if cat_idx > 0 {
                    // Move to last package of previous category
                    let prev_cat = cat_idx - 1;
                    if let Some(cat) = self.config.updates.get(prev_cat) {
                        self.update_category_cursor = prev_cat;
                        if !cat.packages.is_empty() {
                            self.update_package_cursor = Some(cat.packages.len() - 1);
                        }
                    }
                }
            }
            Some(pkg_idx) => {
                if pkg_idx > 0 {
                    // Move to previous package
                    self.update_package_cursor = Some(pkg_idx - 1);
                } else {
                    // Move to category header
                    self.update_package_cursor = None;
                }
            }
        }
    }

    /// Toggle the currently focused update item (category or package)
    fn toggle_update_item(&mut self) {
        let cat_idx = self.update_category_cursor;

        match self.update_package_cursor {
            None => {
                // Toggle all non-required packages in this category
                if let Some(packages) = self.update_package_selected.get(cat_idx) {
                    let all_selected = packages.iter().all(|&s| s);
                    let new_value = !all_selected;
                    if let Some(cat) = self.config.updates.get(cat_idx) {
                        if let Some(pkg_list) = self.update_package_selected.get_mut(cat_idx) {
                            for (idx, pkg) in pkg_list.iter_mut().enumerate() {
                                if cat.packages.get(idx).map(|p| p.required).unwrap_or(false) {
                                    // Required packages stay selected
                                    *pkg = true;
                                } else {
                                    *pkg = new_value;
                                }
                            }
                        }
                    }
                }
            }
            Some(pkg_idx) => {
                // Don't toggle required packages
                let is_required = self.config.updates.get(cat_idx)
                    .and_then(|cat| cat.packages.get(pkg_idx))
                    .map(|pkg| pkg.required)
                    .unwrap_or(false);

                if !is_required {
                    if let Some(pkg_list) = self.update_package_selected.get_mut(cat_idx) {
                        if let Some(pkg) = pkg_list.get_mut(pkg_idx) {
                            *pkg = !*pkg;
                        }
                    }
                }
            }
        }
    }

    /// Check if all packages in a category are selected
    pub fn is_category_fully_selected(&self, cat_idx: usize) -> bool {
        self.update_package_selected.get(cat_idx)
            .map(|pkgs| !pkgs.is_empty() && pkgs.iter().all(|&s| s))
            .unwrap_or(false)
    }

    /// Check if some (but not all) packages in a category are selected
    pub fn is_category_partially_selected(&self, cat_idx: usize) -> bool {
        self.update_package_selected.get(cat_idx)
            .map(|pkgs| {
                let any = pkgs.iter().any(|&s| s);
                let all = pkgs.iter().all(|&s| s);
                any && !all
            })
            .unwrap_or(false)
    }

    /// Check if any package in a category is selected
    pub fn is_category_any_selected(&self, cat_idx: usize) -> bool {
        self.update_package_selected.get(cat_idx)
            .map(|pkgs| pkgs.iter().any(|&s| s))
            .unwrap_or(false)
    }

    fn handle_enter(&mut self) -> Option<OnboardAction> {
        match self.panel_focus {
            PanelFocus::Welcome => {
                // Start setup
                self.start_setup();
            }
            PanelFocus::Sidebar => {
                // Check if step is locked
                if self.is_current_step_locked() {
                    self.set_error("This step is locked. Complete previous steps first.".to_string());
                    return None;
                }

                // Move to content panel
                self.focus_content();
                // If it's a form, enter insert mode
                if let Some(item) = self.current_item() {
                    if item.has_form {
                        self.vim_mode = VimMode::Insert;
                    }
                }
            }
            PanelFocus::Content => {
                // Check if step is locked
                if self.is_current_step_locked() {
                    self.set_error("This step is locked. Complete previous steps first.".to_string());
                    return None;
                }

                // Special case: Update step in dryrun mode doesn't need password
                if let Some(StepId::Update) = self.current_step_id() {
                    if self.is_dryrun() {
                        return Some(OnboardAction::ExecuteUpdate);
                    }
                }

                match self.content_focus {
                    ContentFocus::Picker => {
                        self.select_picker_item();
                    }
                    ContentFocus::InputField(_) => {
                        // Enter insert mode
                        self.vim_mode = VimMode::Insert;
                    }
                    ContentFocus::None => {
                        if let Some(item) = self.current_item() {
                            match item.id {
                                StepId::Network if !self.network_connected => {
                                    let program = self.config.network.program.clone();
                                    let args = self.config.network.args.clone();
                                    return Some(OnboardAction::LaunchExternal(program, args));
                                }
                                StepId::Network if self.network_connected => {
                                    // Network already connected, advance
                                    self.advance_to_next_step();
                                }
                                StepId::Review => {
                                    // Trigger review execution
                                    return Some(OnboardAction::ExecuteReview);
                                }
                                StepId::Update => {
                                    // Trigger update execution
                                    return Some(OnboardAction::ExecuteUpdate);
                                }
                                StepId::Reboot => {
                                    // Exit to greeter login
                                    return Some(OnboardAction::ExitToLogin);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn start_setup(&mut self) {
        self.setup_started = true;
        self.selected_step = 0;
        self.load_step_content();

        // Auto-complete network step if already connected
        if self.network_connected {
            for (idx, item) in self.menu_items.iter().enumerate() {
                if item.id == StepId::Network {
                    self.step_results[idx] = StepResult::Completed;
                }
            }
        }

        // Start with content focused on the User step, ready to type
        self.focus_content();
        self.update_status_bar();
    }

    fn load_step_content(&mut self) {
        if let Some(item) = self.current_item() {
            match item.id {
                StepId::Locale => {
                    self.picker_items = executor::list_locales(self.is_dryrun());
                    self.picker_selected = 0;
                    self.picker_filter.clear();
                }
                StepId::Keyboard => {
                    self.picker_items = executor::list_keymaps(self.is_dryrun());
                    self.picker_selected = 0;
                    self.picker_filter.clear();
                }
                StepId::Preferences => {
                    self.picker_items = executor::list_timezones(self.is_dryrun());
                    self.picker_selected = 0;
                    self.picker_filter.clear();
                }
                StepId::Update => {
                    // Check if any commands need sudo
                    self.sudo_password_needed = self.commands_need_sudo();
                    self.sudo_password.clear();
                    self.sudo_password_entered = false;
                }
                _ => {}
            }
        }
    }

    fn select_picker_item(&mut self) {
        let filtered = self.filtered_picker_items();
        if let Some(item) = filtered.get(self.picker_selected) {
            let item = item.clone();
            if let Some(step_id) = self.current_step_id() {
                match step_id {
                    StepId::Locale => {
                        self.selected_locale = Some(item.clone());
                        self.step_results[self.selected_step] = StepResult::Completed;
                        self.set_info(format!("Locale selected: {item}"));
                    }
                    StepId::Keyboard => {
                        self.selected_keyboard = Some(item.clone());
                        self.step_results[self.selected_step] = StepResult::Completed;
                        self.set_info(format!("Keyboard selected: {item}"));
                    }
                    StepId::Preferences => {
                        self.selected_timezone = Some(item.clone());
                        self.step_results[self.selected_step] = StepResult::Completed;
                        self.set_info(format!("Timezone selected: {item}"));
                    }
                    _ => {}
                }
                self.advance_to_next_step();
            }
        }
    }

    fn advance_to_next_step(&mut self) {
        if self.selected_step < self.menu_items.len() - 1 {
            self.selected_step += 1;
            self.load_step_content();
        }
        // Focus content panel so user can interact with the new step
        self.focus_content();
    }

    /// Get the input buffer for the current field based on step and field index
    fn current_input_buffer(&mut self) -> Option<&mut InputBuffer> {
        let step_id = self.current_step_id();
        match self.content_focus {
            ContentFocus::InputField(idx) => {
                match step_id {
                    Some(StepId::User) => match idx {
                        0 => Some(&mut self.username),
                        1 => Some(&mut self.password),
                        2 => Some(&mut self.password_confirm),
                        _ => None,
                    },
                    Some(StepId::Update) => match idx {
                        0 => Some(&mut self.sudo_password),
                        _ => None,
                    },
                    _ => None,
                }
            }
            ContentFocus::Picker => Some(&mut self.picker_filter),
            _ => None,
        }
    }

    fn handle_char_input(&mut self, c: char) {
        if let ContentFocus::Picker = self.content_focus {
            self.picker_filter.insert(c);
            self.picker_selected = 0;
        } else if let Some(buffer) = self.current_input_buffer() {
            buffer.insert(c);
        }
    }

    fn handle_backspace(&mut self) {
        if let ContentFocus::Picker = self.content_focus {
            self.picker_filter.delete_back();
            self.picker_selected = 0;
        } else if let Some(buffer) = self.current_input_buffer() {
            buffer.delete_back();
        }
    }

    fn handle_delete(&mut self) {
        if let Some(buffer) = self.current_input_buffer() {
            buffer.delete_forward();
        }
    }

    fn handle_cursor_left(&mut self) {
        if let Some(buffer) = self.current_input_buffer() {
            buffer.move_left();
        }
    }

    fn handle_cursor_right(&mut self) {
        if let Some(buffer) = self.current_input_buffer() {
            buffer.move_right();
        }
    }

    fn handle_cursor_home(&mut self) {
        if let Some(buffer) = self.current_input_buffer() {
            buffer.move_start();
        }
    }

    fn handle_cursor_end(&mut self) {
        if let Some(buffer) = self.current_input_buffer() {
            buffer.move_end();
        }
    }

    fn handle_clear_line(&mut self) {
        if let Some(buffer) = self.current_input_buffer() {
            buffer.clear();
        }
    }

    fn execute_command(&mut self, cmd: &str) -> Option<OnboardAction> {
        let cmd = cmd.trim().to_lowercase();
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let cmd_name = parts.first().copied().unwrap_or("");

        match cmd_name {
            "start" | "run" => {
                if !self.setup_started {
                    self.start_setup();
                }
            }
            "next" | "n" => {
                self.advance_to_next_step();
            }
            "skip" | "s" => {
                let (is_required, step_id) = self.current_item()
                    .map(|item| (item.required, item.id))
                    .unwrap_or((true, StepId::User));

                if !is_required {
                    self.step_results[self.selected_step] = StepResult::Skipped;
                    // If skipping Update, unlock Login
                    if step_id == StepId::Update {
                        self.update_completed = true;
                        self.unlock_login_step();
                    }
                    self.advance_to_next_step();
                } else {
                    self.set_error("This step is required".to_string());
                }
            }
            "cancel" | "q" | "quit" => {
                self.confirm_action = Some(ConfirmAction::Cancel);
            }
            "reboot" => {
                self.confirm_action = Some(ConfirmAction::Reboot);
            }
            "poweroff" | "shutdown" => {
                self.confirm_action = Some(ConfirmAction::Poweroff);
            }
            "help" | "h" => {
                self.show_help = true;
            }
            "submit" | "create" | "install" | "update" => {
                // Trigger async step execution
                return Some(OnboardAction::ExecuteStep);
            }
            "finish" | "done" | "login" => {
                if self.review_completed {
                    return Some(OnboardAction::ExitToLogin);
                } else {
                    self.set_error("Complete the Review step first".to_string());
                }
            }
            _ => {
                self.set_error(format!("Unknown command: {cmd_name}"));
            }
        }
        None
    }

    pub fn filtered_picker_items(&self) -> Vec<String> {
        let filter = self.picker_filter.content().to_lowercase();
        if filter.is_empty() {
            self.picker_items.clone()
        } else {
            self.picker_items
                .iter()
                .filter(|item| item.to_lowercase().contains(&filter))
                .cloned()
                .collect()
        }
    }

    pub fn validate_user_form(&mut self) -> bool {
        let username = self.username.content().to_string();
        let password = self.password.content().to_string();
        let confirm = self.password_confirm.content().to_string();

        if username.is_empty() {
            self.set_error("Username is required".to_string());
            return false;
        }

        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            self.set_error("Username can only contain letters, numbers, underscore, and dash".to_string());
            return false;
        }

        if username.len() > 32 {
            self.set_error("Username must be 32 characters or less".to_string());
            return false;
        }

        if password.is_empty() {
            self.set_error("Password is required".to_string());
            return false;
        }

        let min_len = self.config.user.min_password_length;
        if password.len() < min_len {
            self.set_error(format!("Password must be at least {min_len} characters"));
            return false;
        }

        if password != confirm {
            self.set_error("Passwords do not match".to_string());
            return false;
        }

        true
    }

    /// Start execution of the current step (User form only).
    /// Returns a receiver for execution messages, or None if handled synchronously (dryrun/validation failure).
    pub fn start_step_execution(&mut self) -> Option<mpsc::UnboundedReceiver<ExecutionMessage>> {
        let step_id = self.current_step_id()?;

        if step_id != StepId::User {
            return None;
        }

        if !self.validate_user_form() {
            return None;
        }

        let username = self.username.content().to_string();
        let password = self.password.content().to_string();

        self.is_executing = true;
        self.tasks = vec![
            TaskStatus {
                name: format!("Creating user '{username}'"),
                status: TaskState::Running,
                output: None,
                progress: None,
            },
        ];
        self.current_task = Some(0);

        if self.is_dryrun() {
            // Dryrun: immediately succeed
            self.tasks[0].status = TaskState::Success;
            self.created_username = Some(username);
            self.step_results[0] = StepResult::Completed;
            self.current_task = None;
            self.is_executing = false;
            self.advance_to_next_step();
            return None;
        }

        let groups = self.config.user.groups.clone();
        let shell = self.config.user.shell.clone();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                executor::create_user(&username, &password, &groups, &shell)
                    .map(|()| username)
            }).await;

            match result {
                Ok(Ok(uname)) => {
                    let _ = tx.send(ExecutionMessage::TaskSuccess(0, None));
                    let _ = tx.send(ExecutionMessage::UserCreated(Some(uname)));
                    let _ = tx.send(ExecutionMessage::StepComplete { step_result: StepResult::Completed });
                }
                Ok(Err(e)) => {
                    let _ = tx.send(ExecutionMessage::TaskFailed(0, e.to_string()));
                    let _ = tx.send(ExecutionMessage::UserCreated(None));
                    let _ = tx.send(ExecutionMessage::StepComplete { step_result: StepResult::Failed });
                }
                Err(e) => {
                    let _ = tx.send(ExecutionMessage::TaskFailed(0, e.to_string()));
                    let _ = tx.send(ExecutionMessage::UserCreated(None));
                    let _ = tx.send(ExecutionMessage::StepComplete { step_result: StepResult::Failed });
                }
            }
        });

        Some(rx)
    }

    /// Start Review step execution - create user and apply all configuration.
    /// Returns a receiver for execution messages, or None if handled synchronously (dryrun/validation failure).
    pub fn start_review_execution(&mut self) -> Option<mpsc::UnboundedReceiver<ExecutionMessage>> {
        // Validate user form first
        if !self.validate_user_form() {
            return None;
        }

        // In dryrun mode, use tick-based simulation with progress bars
        if self.is_dryrun() {
            self.tasks.clear();

            let username = self.username.content().to_string();
            self.tasks.push(TaskStatus {
                name: format!("Creating user '{username}'"),
                status: TaskState::Pending,
                output: None,
                progress: Some(0),
            });

            if let Some(ref locale) = self.selected_locale {
                self.tasks.push(TaskStatus {
                    name: format!("Setting locale to {locale}"),
                    status: TaskState::Pending,
                    output: None,
                    progress: Some(0),
                });
            }

            if let Some(ref keymap) = self.selected_keyboard {
                self.tasks.push(TaskStatus {
                    name: format!("Setting keyboard to {keymap}"),
                    status: TaskState::Pending,
                    output: None,
                    progress: Some(0),
                });
            }

            if let Some(ref tz) = self.selected_timezone {
                self.tasks.push(TaskStatus {
                    name: format!("Setting timezone to {tz}"),
                    status: TaskState::Pending,
                    output: None,
                    progress: Some(0),
                });
            }

            self.created_username = Some(username);
            self.start_dryrun_simulation(DryrunCallback::CompleteReview);
            return None;
        }

        // Real execution
        self.is_executing = true;
        self.tasks.clear();

        let username = self.username.content().to_string();
        let password = self.password.content().to_string();
        let groups = self.config.user.groups.clone();
        let shell = self.config.user.shell.clone();
        let locale = self.selected_locale.clone();
        let keymap = self.selected_keyboard.clone();
        let timezone = self.selected_timezone.clone();

        // Build task list for UI
        self.tasks.push(TaskStatus {
            name: format!("Creating user '{username}'"),
            status: TaskState::Running,
            output: None,
            progress: None,
        });

        if let Some(ref l) = locale {
            self.tasks.push(TaskStatus {
                name: format!("Setting locale to {l}"),
                status: TaskState::Pending,
                output: None,
                progress: None,
            });
        }
        if let Some(ref k) = keymap {
            self.tasks.push(TaskStatus {
                name: format!("Setting keyboard to {k}"),
                status: TaskState::Pending,
                output: None,
                progress: None,
            });
        }
        if let Some(ref tz) = timezone {
            self.tasks.push(TaskStatus {
                name: format!("Setting timezone to {tz}"),
                status: TaskState::Pending,
                output: None,
                progress: None,
            });
        }

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            // 1. Create user
            let _ = tx.send(ExecutionMessage::TaskStarted(0));
            let user_result = tokio::task::spawn_blocking({
                let username = username.clone();
                let password = password.clone();
                let groups = groups.clone();
                let shell = shell.clone();
                move || executor::create_user(&username, &password, &groups, &shell)
            }).await;

            let user_ok = match user_result {
                Ok(Ok(())) => {
                    let _ = tx.send(ExecutionMessage::TaskSuccess(0, None));
                    let _ = tx.send(ExecutionMessage::UserCreated(Some(username)));
                    true
                }
                Ok(Err(e)) => {
                    let _ = tx.send(ExecutionMessage::TaskFailed(0, e.to_string()));
                    let _ = tx.send(ExecutionMessage::UserCreated(None));
                    false
                }
                Err(e) => {
                    let _ = tx.send(ExecutionMessage::TaskFailed(0, e.to_string()));
                    let _ = tx.send(ExecutionMessage::UserCreated(None));
                    false
                }
            };

            if !user_ok {
                let _ = tx.send(ExecutionMessage::ReviewComplete { any_failed: true });
                return;
            }

            let mut any_failed = false;
            let mut idx: usize = 1;

            // 2. Apply locale
            if let Some(locale) = locale {
                let _ = tx.send(ExecutionMessage::TaskStarted(idx));
                let result = tokio::task::spawn_blocking({
                    let locale = locale.clone();
                    move || executor::set_locale(&locale)
                }).await;
                match result {
                    Ok(Ok(())) => { let _ = tx.send(ExecutionMessage::TaskSuccess(idx, None)); }
                    Ok(Err(e)) => { any_failed = true; let _ = tx.send(ExecutionMessage::TaskFailed(idx, e.to_string())); }
                    Err(e) => { any_failed = true; let _ = tx.send(ExecutionMessage::TaskFailed(idx, e.to_string())); }
                }
                idx += 1;
            }

            // 3. Apply keymap
            if let Some(keymap) = keymap {
                let _ = tx.send(ExecutionMessage::TaskStarted(idx));
                let result = tokio::task::spawn_blocking({
                    let keymap = keymap.clone();
                    move || executor::set_keymap(&keymap)
                }).await;
                match result {
                    Ok(Ok(())) => { let _ = tx.send(ExecutionMessage::TaskSuccess(idx, None)); }
                    Ok(Err(e)) => { any_failed = true; let _ = tx.send(ExecutionMessage::TaskFailed(idx, e.to_string())); }
                    Err(e) => { any_failed = true; let _ = tx.send(ExecutionMessage::TaskFailed(idx, e.to_string())); }
                }
                idx += 1;
            }

            // 4. Apply timezone
            if let Some(tz) = timezone {
                let _ = tx.send(ExecutionMessage::TaskStarted(idx));
                let result = tokio::task::spawn_blocking({
                    let tz = tz.clone();
                    move || executor::set_timezone(&tz)
                }).await;
                match result {
                    Ok(Ok(())) => { let _ = tx.send(ExecutionMessage::TaskSuccess(idx, None)); }
                    Ok(Err(e)) => { any_failed = true; let _ = tx.send(ExecutionMessage::TaskFailed(idx, e.to_string())); }
                    Err(e) => { any_failed = true; let _ = tx.send(ExecutionMessage::TaskFailed(idx, e.to_string())); }
                }
            }

            let _ = tx.send(ExecutionMessage::ReviewComplete { any_failed });
        });

        Some(rx)
    }

    /// Check if any selected packages have commands that require sudo
    pub fn commands_need_sudo(&self) -> bool {
        for (cat_idx, cat) in self.config.updates.iter().enumerate() {
            for (pkg_idx, pkg) in cat.packages.iter().enumerate() {
                let selected = self.update_package_selected
                    .get(cat_idx)
                    .and_then(|pkgs| pkgs.get(pkg_idx))
                    .copied()
                    .unwrap_or(false);
                if selected && pkg.commands.iter().any(|cmd| cmd.sudo) {
                    return true;
                }
            }
        }
        false
    }

    /// Get all commands from selected packages
    fn selected_commands(&self) -> Vec<config::CommandConfig> {
        let mut commands = Vec::new();
        for (cat_idx, cat) in self.config.updates.iter().enumerate() {
            for (pkg_idx, pkg) in cat.packages.iter().enumerate() {
                let selected = self.update_package_selected
                    .get(cat_idx)
                    .and_then(|pkgs| pkgs.get(pkg_idx))
                    .copied()
                    .unwrap_or(false);
                if selected {
                    commands.extend(pkg.commands.iter().cloned());
                }
            }
        }
        commands
    }

    /// Check if any package is selected across all categories
    pub fn any_package_selected(&self) -> bool {
        self.update_package_selected.iter()
            .any(|pkgs| pkgs.iter().any(|&s| s))
    }

    /// Start Update step execution - run commands from selected packages as the created user.
    /// Returns a receiver for execution messages, or None if handled synchronously.
    pub fn start_update_execution(&mut self) -> Option<mpsc::UnboundedReceiver<ExecutionMessage>> {
        let commands = self.selected_commands();

        // If no packages selected, skip
        if commands.is_empty() {
            if let Some(idx) = self.step_index_by_id(StepId::Update) {
                self.step_results[idx] = StepResult::Skipped;
            }
            self.update_completed = true;
            self.unlock_login_step();
            self.set_info("No packages selected. Continuing to finish.".to_string());
            self.advance_to_next_step();
            return None;
        }

        // In dryrun mode, use the tick-based simulation for progress animation
        if self.is_dryrun() {
            self.tasks.clear();
            for cmd_config in &commands {
                self.tasks.push(TaskStatus {
                    name: cmd_config.name.clone(),
                    status: TaskState::Pending,
                    output: None,
                    progress: Some(0),
                });
            }
            self.start_dryrun_simulation(DryrunCallback::CompleteUpdate);
            return None;
        }

        // Real execution requires a user
        let username = match &self.created_username {
            Some(u) => u.clone(),
            None => {
                self.set_error("User must be created before running commands".to_string());
                return None;
            }
        };

        // Check if sudo password is needed but not provided
        if self.commands_need_sudo() && !self.sudo_password_entered {
            self.sudo_password_needed = true;
            self.set_error("Enter your password for sudo commands".to_string());
            return None;
        }

        self.tasks.clear();
        self.is_executing = true;

        // Build task list for UI
        for cmd_config in &commands {
            self.tasks.push(TaskStatus {
                name: cmd_config.name.clone(),
                status: TaskState::Pending,
                output: None,
                progress: None,
            });
        }

        let sudo_pass = self.sudo_password.content().to_string();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut any_failed = false;

            for (idx, cmd_config) in commands.iter().enumerate() {
                let _ = tx.send(ExecutionMessage::TaskStarted(idx));

                let username = username.clone();
                let sudo_pass = sudo_pass.clone();
                let command = cmd_config.command.clone();
                let use_sudo = cmd_config.sudo;

                let result = tokio::task::spawn_blocking(move || {
                    if use_sudo {
                        executor::run_command_as_user_with_sudo(&username, &command, &sudo_pass)
                    } else {
                        executor::run_command_as_user(&username, &command)
                    }
                }).await;

                match result {
                    Ok(Ok(output)) => {
                        let _ = tx.send(ExecutionMessage::TaskSuccess(idx, Some(output)));
                    }
                    Ok(Err(e)) => {
                        any_failed = true;
                        let _ = tx.send(ExecutionMessage::TaskFailed(idx, e.to_string()));
                    }
                    Err(e) => {
                        any_failed = true;
                        let _ = tx.send(ExecutionMessage::TaskFailed(idx, e.to_string()));
                    }
                }
            }

            let _ = tx.send(ExecutionMessage::UpdateComplete { any_failed });
        });

        Some(rx)
    }

    fn step_index_by_id(&self, id: StepId) -> Option<usize> {
        self.menu_items.iter().position(|item| item.id == id)
    }

    /// Finish setup - run completion tasks and trigger final action
    pub async fn finish_setup(&mut self) {
        self.is_executing = true;
        self.tasks.clear();
        self.execute_completion().await;
        self.is_executing = false;
        self.setup_complete = true;

        // In dryrun mode, always transition to login instead of rebooting/poweroff
        if self.is_dryrun() {
            self.confirm_action = Some(ConfirmAction::Reboot);
            return;
        }

        // Trigger the completion action
        match self.config.completion.action.as_str() {
            "reboot" => {
                self.confirm_action = Some(ConfirmAction::Reboot);
            }
            "poweroff" => {
                self.confirm_action = Some(ConfirmAction::Poweroff);
            }
            _ => {
                self.should_exit = true;
            }
        }
    }

    /// Handle an execution message from a background task
    pub fn handle_execution_message(&mut self, msg: ExecutionMessage) {
        match msg {
            ExecutionMessage::TaskStarted(idx) => {
                if let Some(task) = self.tasks.get_mut(idx) {
                    task.status = TaskState::Running;
                }
            }
            ExecutionMessage::TaskSuccess(idx, output) => {
                if let Some(task) = self.tasks.get_mut(idx) {
                    task.status = TaskState::Success;
                    task.output = output;
                }
            }
            ExecutionMessage::TaskFailed(idx, error) => {
                if let Some(task) = self.tasks.get_mut(idx) {
                    task.status = TaskState::Failed;
                    task.output = Some(error.clone());
                }
                self.set_error(error);
            }
            ExecutionMessage::UserCreated(username) => {
                self.created_username = username;
            }
            ExecutionMessage::ReviewComplete { any_failed } => {
                self.is_executing = false;
                self.current_task = None;

                if any_failed {
                    let failed_count = self.tasks.iter().filter(|t| t.status == TaskState::Failed).count();
                    self.set_error(format!("{} task(s) failed during configuration", failed_count));
                } else {
                    self.set_info("Configuration applied! You can now install packages.".to_string());
                }

                if let Some(idx) = self.step_index_by_id(StepId::Review) {
                    self.step_results[idx] = StepResult::Completed;
                }
                self.review_completed = true;
                self.unlock_update_step();
                self.advance_to_next_step();
            }
            ExecutionMessage::UpdateComplete { any_failed } => {
                self.is_executing = false;
                self.current_task = None;

                if any_failed {
                    let failed_count = self.tasks.iter().filter(|t| t.status == TaskState::Failed).count();
                    self.set_error(format!("{} task(s) failed during configuration", failed_count));
                } else {
                    self.set_info("Commands completed! Click Reboot to finish setup.".to_string());
                }

                if let Some(idx) = self.step_index_by_id(StepId::Update) {
                    self.step_results[idx] = StepResult::Completed;
                }
                self.update_completed = true;
                self.unlock_login_step();
                self.advance_to_next_step();
            }
            ExecutionMessage::StepComplete { step_result } => {
                self.is_executing = false;
                self.current_task = None;

                // User step is always index 0
                self.step_results[0] = step_result;

                if step_result == StepResult::Completed {
                    self.advance_to_next_step();
                }
            }
        }
    }

    async fn execute_completion(&mut self) {
        self.tasks.push(TaskStatus {
            name: "Finishing setup".to_string(),
            status: TaskState::Running,
            output: None,
            progress: None,
        });
        self.current_task = Some(self.tasks.len() - 1);

        if !self.is_dryrun() && self.config.completion.remove_initial_session {
            if let Err(e) = executor::remove_initial_session() {
                warn!("Failed to remove initial session: {e}");
            }
        }

        if self.is_dryrun() {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        self.tasks.last_mut().unwrap().status = TaskState::Success;
        self.current_task = None;
    }

    pub fn set_error(&mut self, text: String) {
        self.message = Some(Message {
            text,
            is_error: true,
        });
    }

    pub fn set_info(&mut self, text: String) {
        self.message = Some(Message {
            text,
            is_error: false,
        });
    }

    pub fn tick(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % 4;

        // Check network periodically
        if self.spinner_frame == 0 {
            self.network_connected = executor::check_network(self.is_dryrun());
        }

        // Advance dryrun simulation if active
        if self.dryrun_sim_active {
            self.advance_dryrun_simulation();
        }

        // Keep status bar in sync
        self.update_status_bar();
    }

    /// Advance the dryrun progress simulation by one step
    fn advance_dryrun_simulation(&mut self) {
        if self.tasks.is_empty() {
            self.dryrun_sim_active = false;
            return;
        }

        let task_idx = self.dryrun_sim_task_idx;
        if task_idx >= self.tasks.len() {
            // All tasks complete
            self.dryrun_sim_active = false;
            self.is_executing = false;

            // Execute callback
            if let Some(callback) = self.dryrun_sim_callback.take() {
                match callback {
                    DryrunCallback::CompleteUpdate => {
                        if let Some(idx) = self.step_index_by_id(StepId::Update) {
                            self.step_results[idx] = StepResult::Completed;
                        }
                        self.update_completed = true;
                        self.unlock_login_step();
                        self.tasks.clear();
                        self.set_info("Installation complete! Reboot to finish setup.".to_string());
                        self.advance_to_next_step();
                    }
                    DryrunCallback::CompleteReview => {
                        if let Some(idx) = self.step_index_by_id(StepId::Review) {
                            self.step_results[idx] = StepResult::Completed;
                        }
                        self.review_completed = true;
                        self.unlock_update_step();
                        self.tasks.clear();
                        self.set_info("Configuration applied! Select packages to install.".to_string());
                        self.advance_to_next_step();
                    }
                }
            }
            return;
        }

        // Mark current task as running
        self.tasks[task_idx].status = TaskState::Running;

        // Advance progress
        self.dryrun_sim_progress += 10;
        self.tasks[task_idx].progress = Some(self.dryrun_sim_progress);

        if self.dryrun_sim_progress >= 100 {
            // Task complete, move to next
            self.tasks[task_idx].status = TaskState::Success;
            self.dryrun_sim_task_idx += 1;
            self.dryrun_sim_progress = 0;
        }
    }

    /// Start a dryrun simulation for the given tasks
    fn start_dryrun_simulation(&mut self, callback: DryrunCallback) {
        self.dryrun_sim_active = true;
        self.dryrun_sim_task_idx = 0;
        self.dryrun_sim_progress = 0;
        self.dryrun_sim_callback = Some(callback);
        self.is_executing = true;
    }

    pub fn spinner_char(&self) -> char {
        const SPINNER: [char; 4] = ['|', '/', '-', '\\'];
        SPINNER[self.spinner_frame]
    }

    /// Update status bar based on current application state
    pub fn update_status_bar(&mut self) {
        // Handle special states first
        if self.is_executing {
            self.status_bar = StatusBarState::executing();
            return;
        }

        // Command mode overrides everything
        if self.vim_mode == VimMode::Command {
            self.status_bar = StatusBarState::command_mode();
            return;
        }

        // Update based on panel focus and current step
        self.status_bar = match self.panel_focus {
            PanelFocus::Welcome => StatusBarState::welcome(),
            PanelFocus::Sidebar => StatusBarState::sidebar_normal(),
            PanelFocus::Content => self.content_status_bar(),
        };
    }

    /// Get status bar state for content panel based on current step and mode
    fn content_status_bar(&self) -> StatusBarState {
        // Check if step is locked
        if self.is_current_step_locked() {
            return StatusBarState::locked_step();
        }

        let step_id = match self.current_step_id() {
            Some(id) => id,
            None => return StatusBarState::default(),
        };

        let is_insert = self.vim_mode == VimMode::Insert;

        match step_id {
            StepId::User => {
                if is_insert {
                    StatusBarState::content_form_insert()
                } else {
                    StatusBarState::content_form_normal()
                }
            }
            StepId::Locale | StepId::Keyboard | StepId::Preferences => {
                if is_insert {
                    StatusBarState::content_picker_insert()
                } else {
                    StatusBarState::content_picker_normal()
                }
            }
            StepId::Network => StatusBarState::network_step(self.network_connected),
            StepId::Review => StatusBarState::review_step(),
            StepId::Update => {
                let needs_password = self.commands_need_sudo()
                    && !self.sudo_password_entered
                    && !self.is_dryrun();
                if is_insert && needs_password {
                    StatusBarState::content_form_insert()
                } else {
                    StatusBarState::update_step(needs_password)
                }
            }
            StepId::Reboot => StatusBarState::reboot_step(),
        }
    }
}
