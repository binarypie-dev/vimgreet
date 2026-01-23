pub mod ui;
pub mod widgets;

use crate::ipc::{AuthResponse, GreetdClient};
use crate::system::{discover_sessions, discover_users, Session, User};
use crate::ui::Theme;
use crate::vim::{parse_command, Command, InputBuffer, ModeAction, VimMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusField {
    #[default]
    Username,
    Password,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Reboot,
    Poweroff,
}

pub struct Message {
    pub text: String,
    pub is_error: bool,
}

pub struct App {
    pub vim_mode: VimMode,
    pub focus: FocusField,
    pub username: InputBuffer,
    pub password: InputBuffer,
    pub command_buffer: InputBuffer,
    pub sessions: Vec<Session>,
    pub selected_session: usize,
    pub users: Vec<User>,
    pub selected_user: usize,
    pub message: Option<Message>,
    pub working: bool,
    pub should_exit: bool,
    pub exit_success: bool,
    pub show_session_picker: bool,
    pub show_user_picker: bool,
    pub show_help: bool,
    pub confirm_action: Option<ConfirmAction>,
    pub theme: Theme,
    pub demo_mode: bool,
    pending_dd: bool,
}

impl App {
    pub fn new(demo_mode: bool) -> Self {
        let sessions = discover_sessions();
        let users = discover_users();

        info!(
            "Initialized app with {} sessions and {} users",
            sessions.len(),
            users.len()
        );

        Self {
            vim_mode: VimMode::Insert,
            focus: FocusField::Username,
            username: InputBuffer::new(),
            password: InputBuffer::masked(),
            command_buffer: InputBuffer::new(),
            sessions,
            selected_session: 0,
            users,
            selected_user: 0,
            message: None,
            working: false,
            should_exit: false,
            exit_success: false,
            show_session_picker: false,
            show_user_picker: false,
            show_help: false,
            confirm_action: None,
            theme: Theme::default(),
            demo_mode,
            pending_dd: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        // Clear message on any key
        if self.message.is_some() && !self.working {
            self.message = None;
        }

        // Handle confirm dialog
        if let Some(action) = &self.confirm_action {
            return self.handle_confirm_key(key, *action);
        }

        // Handle popups
        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                self.show_help = false;
            }
            return None;
        }

        if self.show_session_picker {
            return self.handle_picker_key(key, true);
        }

        if self.show_user_picker {
            return self.handle_picker_key(key, false);
        }

        // Handle based on vim mode
        match self.vim_mode {
            VimMode::Normal => self.handle_normal_mode(key),
            VimMode::Insert => self.handle_insert_mode(key),
            VimMode::Command => self.handle_command_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            // Mode transitions
            KeyCode::Char('i') => {
                self.vim_mode = self.vim_mode.transition(ModeAction::EnterInsert);
                self.pending_dd = false;
            }
            KeyCode::Char('a') => {
                self.vim_mode = self.vim_mode.transition(ModeAction::EnterInsert);
                self.current_input_mut().move_right();
                self.pending_dd = false;
            }
            KeyCode::Char('A') => {
                self.vim_mode = self.vim_mode.transition(ModeAction::EnterInsert);
                self.current_input_mut().move_end();
                self.pending_dd = false;
            }
            KeyCode::Char('I') => {
                self.vim_mode = self.vim_mode.transition(ModeAction::EnterInsert);
                self.current_input_mut().move_start();
                self.pending_dd = false;
            }
            KeyCode::Char(':') => {
                self.vim_mode = self.vim_mode.transition(ModeAction::EnterCommand);
                self.command_buffer.clear();
                self.pending_dd = false;
            }

            // Navigation
            KeyCode::Char('h') | KeyCode::Left => {
                self.current_input_mut().move_left();
                self.pending_dd = false;
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.current_input_mut().move_right();
                self.pending_dd = false;
            }
            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                self.next_field();
                self.pending_dd = false;
            }
            KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab => {
                self.prev_field();
                self.pending_dd = false;
            }
            KeyCode::Char('0') => {
                self.current_input_mut().move_start();
                self.pending_dd = false;
            }
            KeyCode::Char('$') => {
                self.current_input_mut().move_end();
                self.pending_dd = false;
            }

            // Editing
            KeyCode::Char('x') => {
                self.current_input_mut().delete_forward();
                self.pending_dd = false;
            }
            KeyCode::Char('d') => {
                if self.pending_dd {
                    self.current_input_mut().clear();
                    self.pending_dd = false;
                } else {
                    self.pending_dd = true;
                }
            }

            // Actions
            KeyCode::Enter => {
                self.pending_dd = false;
                return Some(AppAction::Login);
            }

            // Function keys
            KeyCode::F(2) => {
                self.show_user_picker = true;
                self.pending_dd = false;
            }
            KeyCode::F(3) => {
                self.show_session_picker = true;
                self.pending_dd = false;
            }
            KeyCode::F(12) => {
                self.confirm_action = Some(ConfirmAction::Poweroff);
                self.pending_dd = false;
            }

            _ => {
                self.pending_dd = false;
            }
        }
        None
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Esc => {
                self.vim_mode = self.vim_mode.transition(ModeAction::Escape);
            }
            KeyCode::Enter => {
                if self.focus == FocusField::Username && !self.username.is_empty() {
                    self.focus = FocusField::Password;
                    self.vim_mode = VimMode::Normal;
                } else if self.focus == FocusField::Password {
                    self.vim_mode = VimMode::Normal;
                    return Some(AppAction::Login);
                }
            }
            KeyCode::Tab => {
                self.next_field();
            }
            KeyCode::BackTab => {
                self.prev_field();
            }
            KeyCode::Backspace => {
                self.current_input_mut().delete_back();
            }
            KeyCode::Delete => {
                self.current_input_mut().delete_forward();
            }
            KeyCode::Left => {
                self.current_input_mut().move_left();
            }
            KeyCode::Right => {
                self.current_input_mut().move_right();
            }
            KeyCode::Home => {
                self.current_input_mut().move_start();
            }
            KeyCode::End => {
                self.current_input_mut().move_end();
            }
            KeyCode::Char(c) => {
                // Handle Ctrl shortcuts
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'u' => self.current_input_mut().clear(),
                        'a' => self.current_input_mut().move_start(),
                        'e' => self.current_input_mut().move_end(),
                        'w' => {
                            // Delete word backward
                            let input = self.current_input_mut();
                            while input.cursor() > 0 {
                                let content = input.content().to_string();
                                let cursor = input.cursor();
                                if cursor > 0 {
                                    let prev_char = content.chars().nth(cursor - 1);
                                    if prev_char.is_some_and(|c| c.is_whitespace()) && cursor > 1 {
                                        input.delete_back();
                                    } else if prev_char.is_some_and(|c| !c.is_whitespace()) {
                                        input.delete_back();
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    self.current_input_mut().insert(c);
                }
            }
            _ => {}
        }
        None
    }

    fn handle_command_mode(&mut self, key: KeyEvent) -> Option<AppAction> {
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

    fn handle_picker_key(&mut self, key: KeyEvent, is_session: bool) -> Option<AppAction> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                if is_session {
                    self.show_session_picker = false;
                } else {
                    self.show_user_picker = false;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if is_session {
                    if self.selected_session < self.sessions.len().saturating_sub(1) {
                        self.selected_session += 1;
                    }
                } else if self.selected_user < self.users.len().saturating_sub(1) {
                    self.selected_user += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if is_session {
                    self.selected_session = self.selected_session.saturating_sub(1);
                } else {
                    self.selected_user = self.selected_user.saturating_sub(1);
                }
            }
            KeyCode::Enter => {
                if is_session {
                    self.show_session_picker = false;
                } else {
                    if let Some(user) = self.users.get(self.selected_user) {
                        self.username.set(&user.username);
                    }
                    self.show_user_picker = false;
                }
            }
            _ => {}
        }
        None
    }

    fn handle_confirm_key(&mut self, key: KeyEvent, action: ConfirmAction) -> Option<AppAction> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.confirm_action = None;
                match action {
                    ConfirmAction::Reboot => return Some(AppAction::Reboot),
                    ConfirmAction::Poweroff => return Some(AppAction::Poweroff),
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.confirm_action = None;
            }
            _ => {}
        }
        None
    }

    fn execute_command(&mut self, cmd: &str) -> Option<AppAction> {
        match parse_command(cmd) {
            Ok(Command::Reboot) => {
                self.confirm_action = Some(ConfirmAction::Reboot);
            }
            Ok(Command::Poweroff) => {
                self.confirm_action = Some(ConfirmAction::Poweroff);
            }
            Ok(Command::Session(name)) => {
                if let Some(name) = name {
                    if let Some(idx) = self.sessions.iter().position(|s| {
                        s.name.to_lowercase().contains(&name.to_lowercase())
                            || s.slug.to_lowercase() == name.to_lowercase()
                    }) {
                        self.selected_session = idx;
                    } else {
                        self.set_error(format!("Session not found: {}", name));
                    }
                } else {
                    self.show_session_picker = true;
                }
            }
            Ok(Command::User(name)) => {
                if let Some(name) = name {
                    if let Some(idx) = self.users.iter().position(|u| {
                        u.username.to_lowercase() == name.to_lowercase()
                    }) {
                        self.selected_user = idx;
                        self.username.set(&self.users[idx].username.clone());
                    } else {
                        self.set_error(format!("User not found: {}", name));
                    }
                } else {
                    self.show_user_picker = true;
                }
            }
            Ok(Command::Login) | Ok(Command::Quit) => {
                return Some(AppAction::Login);
            }
            Ok(Command::Cancel) => {
                return Some(AppAction::Cancel);
            }
            Ok(Command::Help) => {
                self.show_help = true;
            }
            Err(e) => {
                self.set_error(e.to_string());
            }
        }
        None
    }

    fn current_input_mut(&mut self) -> &mut InputBuffer {
        match self.focus {
            FocusField::Username => &mut self.username,
            FocusField::Password => &mut self.password,
        }
    }

    fn next_field(&mut self) {
        self.focus = match self.focus {
            FocusField::Username => FocusField::Password,
            FocusField::Password => FocusField::Username,
        };
    }

    fn prev_field(&mut self) {
        self.focus = match self.focus {
            FocusField::Username => FocusField::Password,
            FocusField::Password => FocusField::Username,
        };
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

    pub async fn login(&mut self, client: &mut GreetdClient) {
        if self.username.is_empty() {
            self.set_error("Username is required".to_string());
            return;
        }

        self.working = true;
        self.message = None;

        let username = self.username.content().to_string();
        debug!("Creating session for user: {}", username);

        match client.create_session(&username).await {
            Ok(AuthResponse::PromptSecret(_)) => {
                // Password prompt expected, send password
                let password = self.password.content().to_string();
                match client.post_auth_response(Some(password)).await {
                    Ok(AuthResponse::Success) => {
                        info!("Authentication successful");
                        self.start_session(client).await;
                    }
                    Ok(AuthResponse::Error(msg)) => {
                        warn!("Authentication failed: {}", msg);
                        self.working = false;
                        self.set_error(msg);
                        self.password.clear();
                        let _ = client.cancel_session().await;
                    }
                    Ok(AuthResponse::PromptSecret(prompt)) => {
                        // MFA or additional prompt
                        self.working = false;
                        self.set_info(prompt);
                    }
                    Ok(other) => {
                        warn!("Unexpected auth response: {:?}", other);
                        self.working = false;
                        self.set_error("Unexpected authentication response".to_string());
                        let _ = client.cancel_session().await;
                    }
                    Err(e) => {
                        error!("Auth error: {}", e);
                        self.working = false;
                        self.set_error(e.to_string());
                        let _ = client.cancel_session().await;
                    }
                }
            }
            Ok(AuthResponse::Success) => {
                // No password required
                info!("Authentication successful (no password)");
                self.start_session(client).await;
            }
            Ok(AuthResponse::Error(msg)) => {
                warn!("Session creation failed: {}", msg);
                self.working = false;
                self.set_error(msg);
            }
            Ok(other) => {
                warn!("Unexpected response: {:?}", other);
                self.working = false;
                self.set_error("Unexpected response from greetd".to_string());
            }
            Err(e) => {
                error!("IPC error: {}", e);
                self.working = false;
                self.set_error(e.to_string());
            }
        }
    }

    async fn start_session(&mut self, client: &mut GreetdClient) {
        let session = match self.sessions.get(self.selected_session) {
            Some(s) => s,
            None => {
                self.set_error("No session selected".to_string());
                self.working = false;
                return;
            }
        };

        let cmd = session.build_cmd();
        let env = session.build_env();

        info!("Starting session: {:?} with env: {:?}", cmd, env);

        match client.start_session(cmd, env).await {
            Ok(()) => {
                self.should_exit = true;
                self.exit_success = true;
            }
            Err(e) => {
                error!("Failed to start session: {}", e);
                self.set_error(e.to_string());
                self.working = false;
            }
        }
    }
}

#[derive(Debug)]
pub enum AppAction {
    Login,
    Cancel,
    Reboot,
    Poweroff,
}

impl std::fmt::Debug for AuthResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthResponse::PromptSecret(_) => write!(f, "PromptSecret(...)"),
            AuthResponse::PromptVisible(s) => write!(f, "PromptVisible({})", s),
            AuthResponse::Info(s) => write!(f, "Info({})", s),
            AuthResponse::Error(s) => write!(f, "Error({})", s),
            AuthResponse::Success => write!(f, "Success"),
        }
    }
}
