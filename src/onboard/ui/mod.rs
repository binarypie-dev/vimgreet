mod network;
mod picker;
mod reboot;
mod review;
mod update;
mod user;
mod welcome;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use super::steps::StepId;
use super::{ConfirmAction, OnboardApp, PanelFocus};
use crate::vim::VimMode;

/// Main draw function for the onboard wizard
pub fn draw(frame: &mut Frame, app: &mut OnboardApp) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    match app.panel_focus {
        PanelFocus::Welcome => draw_welcome_screen(frame, area, app),
        PanelFocus::Sidebar | PanelFocus::Content => draw_setup_screen(frame, area, app),
    }

    // Overlays
    if let Some(action) = app.confirm_action {
        draw_confirm_dialog(frame, action, app);
    }

    if app.show_help {
        draw_help(frame, app);
    }
}

fn draw_welcome_screen(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    // Full-width welcome screen with centered content
    // Match login screen layout: 1-line header, content, 3-line message, 1-line status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header (no border)
            Constraint::Min(10),    // Content
            Constraint::Length(3),  // Message panel
            Constraint::Length(1),  // Status bar
        ])
        .split(area);

    // Header - aligned title (no border, like login screen)
    draw_header(frame, chunks[0], app);

    // Welcome content - full width, centered
    welcome::draw_welcome_content(frame, chunks[1], app);

    // Message area
    draw_message(frame, chunks[2], app);

    // Status bar
    draw_status_bar(frame, chunks[3], app);
}

/// Draw header bar (1 line, no borders) - matches login screen style
fn draw_header(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    frame.render_widget(Clear, area);

    // Left side: title with version
    let title = format!(" {} (v{}) ", app.config.general.title, env!("CARGO_PKG_VERSION"));
    frame.render_widget(
        Paragraph::new(title)
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        area,
    );

    // Right side: network status
    let network_status = if app.network_connected {
        "[Network: OK] "
    } else {
        "[Network: --] "
    };
    frame.render_widget(
        Paragraph::new(network_status)
            .style(if app.network_connected {
                app.theme.secondary_style()
            } else {
                app.theme.muted_style()
            })
            .alignment(Alignment::Right),
        area,
    );
}

fn draw_setup_screen(frame: &mut Frame, area: Rect, app: &mut OnboardApp) {
    // Match login screen layout: 1-line header, content, 3-line message, 1-line status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header (no border)
            Constraint::Min(10),    // Content (sidebar + main)
            Constraint::Length(3),  // Message panel
            Constraint::Length(1),  // Status bar
        ])
        .split(area);

    draw_header(frame, chunks[0], app);
    draw_setup_content(frame, chunks[1], app);
    draw_message(frame, chunks[2], app);
    draw_status_bar(frame, chunks[3], app);
}


fn draw_setup_content(frame: &mut Frame, area: Rect, app: &mut OnboardApp) {
    // Split: sidebar (25%) and main content (75%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    draw_sidebar(frame, chunks[0], app);
    draw_main_content(frame, chunks[1], app);
}

fn draw_sidebar(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    let is_focused = app.panel_focus == PanelFocus::Sidebar;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_focused {
            app.theme.primary_style()
        } else {
            app.theme.border_style()
        })
        .title(" Steps ");

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.height < 3 {
        return;
    }

    // Draw each step
    for (idx, item) in app.menu_items.iter().enumerate() {
        if idx as u16 >= inner.height {
            break;
        }

        let is_selected = idx == app.selected_step;
        let result = app.step_results.get(idx).copied().unwrap_or(super::steps::StepResult::Pending);

        let is_locked = result == super::steps::StepResult::Locked;

        // Status indicator
        let status = match result {
            super::steps::StepResult::Completed => "[x]",
            super::steps::StepResult::Skipped => "[-]",
            super::steps::StepResult::Failed => "[!]",
            super::steps::StepResult::Pending => "[ ]",
            super::steps::StepResult::Locked => "[#]",
        };

        // Required marker (shown at end)
        let required = if item.required { " *" } else { "" };

        // Step name
        let name = item.id.short_name();

        // Build the line text with required marker at end
        let line_text = format!(" {status} {name}{required}");

        // Determine style - horizontal highlight for selected
        let style = if is_locked {
            // Locked steps are dimmed
            app.theme.muted_style()
        } else if is_selected && is_focused {
            // Full horizontal highlight with primary color
            app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else if is_selected {
            // Selected but not focused - secondary highlight
            app.theme.secondary_style().add_modifier(Modifier::REVERSED)
        } else if result == super::steps::StepResult::Completed {
            app.theme.secondary_style()
        } else if result == super::steps::StepResult::Failed {
            app.theme.error_style()
        } else {
            app.theme.style()
        };

        // Draw full-width highlight
        let line_area = Rect::new(inner.x, inner.y + idx as u16, inner.width, 1);

        // Clear the line first for highlight
        if is_selected {
            frame.render_widget(Clear, line_area);
        }

        frame.render_widget(
            Paragraph::new(line_text).style(style),
            line_area,
        );
    }

    // Hint at bottom
    if is_focused && inner.height > app.menu_items.len() as u16 + 2 {
        let hint = "j/k:nav l/Enter:edit";
        let hint_y = inner.y + inner.height - 1;
        frame.render_widget(
            Paragraph::new(hint).style(app.theme.muted_style()),
            Rect::new(inner.x, hint_y, inner.width, 1),
        );
    }
}

fn draw_main_content(frame: &mut Frame, area: Rect, app: &mut OnboardApp) {
    let is_focused = app.panel_focus == PanelFocus::Content;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_focused {
            app.theme.primary_style()
        } else {
            app.theme.border_style()
        });

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    // Check if step is locked
    if app.is_current_step_locked() {
        draw_locked_step(frame, inner, app);
        return;
    }

    // Draw content based on current step
    if let Some(item) = app.current_item() {
        match item.id {
            StepId::User => user::draw_user_form(frame, inner, app),
            StepId::Locale => picker::draw_picker(frame, inner, app, "Select Locale", &app.config.locale.default_locale),
            StepId::Keyboard => picker::draw_picker(frame, inner, app, "Select Keyboard Layout", &app.config.keyboard.default_layout),
            StepId::Preferences => picker::draw_picker(frame, inner, app, "Select Timezone", &app.config.preferences.default_timezone),
            StepId::Network => network::draw_network_status(frame, inner, app),
            StepId::Review => review::draw_review_step(frame, inner, app),
            StepId::Update => update::draw_update_step(frame, inner, app),
            StepId::Reboot => reboot::draw_reboot_step(frame, inner, app),
        }
    }
}

fn draw_locked_step(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    let mut y = area.y + area.height / 2 - 2;

    frame.render_widget(
        Paragraph::new("Step Locked")
            .style(app.theme.muted_style().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center),
        Rect::new(area.x, y, area.width, 1),
    );
    y += 2;

    frame.render_widget(
        Paragraph::new("Complete the previous steps to unlock this step.")
            .style(app.theme.muted_style())
            .alignment(Alignment::Center),
        Rect::new(area.x, y, area.width, 1),
    );
}

fn draw_message(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    // Only draw message panel if there's a message (matches login screen behavior)
    let msg = match &app.message {
        Some(m) => m,
        None if app.is_executing => {
            // Show executing message
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(app.theme.secondary_style())
                .title(" Info ")
                .title_style(app.theme.secondary_style().add_modifier(Modifier::BOLD));

            let content = Line::from(vec![
                Span::styled("Please wait...", app.theme.style()),
            ]);

            let paragraph = Paragraph::new(content)
                .block(block)
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, area);
            return;
        }
        None => return,
    };

    let (title, border_style, text_style) = if msg.is_error {
        (" Error ", app.theme.error_style(), app.theme.error_style())
    } else {
        (" Info ", app.theme.secondary_style(), app.theme.style())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .title_style(border_style.add_modifier(Modifier::BOLD));

    let content = Line::from(vec![
        Span::styled(msg.text.as_str(), text_style),
    ]);

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    frame.render_widget(Clear, area);

    // Mode indicator on left
    let mode_name = app.vim_mode.display_name();
    let mode_style = app.theme.mode_style(mode_name);
    let mode_span = Span::styled(format!(" {mode_name} "), mode_style);

    // Command buffer in command mode, otherwise show left hint
    let after_mode = if app.vim_mode == VimMode::Command {
        Span::styled(
            format!(":{}", app.command_buffer.content()),
            app.theme.style(),
        )
    } else if !app.status_bar.left_hint.is_empty() {
        Span::styled(
            app.status_bar.left_hint.clone(),
            app.theme.muted_style(),
        )
    } else {
        Span::raw("")
    };

    let left_line = Line::from(vec![mode_span, Span::raw(" "), after_mode]);
    frame.render_widget(
        Paragraph::new(left_line),
        Rect::new(area.x, area.y, area.width * 2 / 3, 1),
    );

    // Progress and right hints
    let completed = app.step_results.iter().filter(|r| **r == super::steps::StepResult::Completed).count();
    let total = app.menu_items.len();

    let right_text = if app.status_bar.right_hint.is_empty() {
        format!("{completed}/{total}")
    } else {
        format!("{completed}/{total}  {}", app.status_bar.right_hint)
    };

    frame.render_widget(
        Paragraph::new(right_text)
            .style(app.theme.muted_style())
            .alignment(Alignment::Right),
        Rect::new(area.x + area.width / 3, area.y, area.width * 2 / 3, 1),
    );
}

fn draw_confirm_dialog(frame: &mut Frame, action: ConfirmAction, app: &OnboardApp) {
    let (title, message) = match action {
        ConfirmAction::Reboot => ("Reboot", "Are you sure you want to reboot?"),
        ConfirmAction::Poweroff => ("Power Off", "Are you sure you want to power off?"),
        ConfirmAction::Cancel => ("Cancel Setup", "Are you sure you want to cancel setup?"),
    };

    let width = 44.min(frame.area().width - 4);
    let height = 7;
    let area = center_rect(frame.area(), width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.primary_style())
        .title(format!(" {title} "));

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    frame.render_widget(
        Paragraph::new(message)
            .style(app.theme.style().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center),
        Rect::new(inner.x, inner.y + 1, inner.width, 1),
    );

    let hints = Line::from(vec![
        Span::styled("[", app.theme.style()),
        Span::styled("Y", app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Span::styled("]es / [", app.theme.style()),
        Span::styled("N", app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Span::styled("]o", app.theme.style()),
    ]);

    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        Rect::new(inner.x, inner.y + 3, inner.width, 1),
    );
}

fn draw_help(frame: &mut Frame, app: &OnboardApp) {
    let width = 60.min(frame.area().width - 4);
    let height = 20.min(frame.area().height - 4);
    let area = center_rect(frame.area(), width, height);

    let help_text = [
        "",
        "Navigation:",
        "",
        "  Ctrl+h         Focus sidebar",
        "  Ctrl+l         Focus content",
        "  j/k            Navigate up/down",
        "  h/l            Collapse/Expand",
        "  Enter          Select / Edit",
        "  1-9            Quick select step",
        "",
        "Vim Modes:",
        "",
        "  i              Enter insert mode",
        "  Esc            Return to normal",
        "  :              Command mode",
        "",
        "Commands: :skip :next :finish :help",
        "",
        "Press q or Esc to close",
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(" Help ");

    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    for (i, line) in help_text.iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        frame.render_widget(
            Paragraph::new(*line).style(app.theme.style()),
            Rect::new(inner.x, inner.y + i as u16, inner.width, 1),
        );
    }
}

pub(crate) fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let x = if area.width > width {
        area.x + (area.width - width) / 2
    } else {
        area.x
    };

    let y = if area.height > height {
        area.y + (area.height - height) / 2
    } else {
        area.y
    };

    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
