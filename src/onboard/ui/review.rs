use ratatui::{prelude::*, widgets::Paragraph};

use super::super::{OnboardApp, PanelFocus, TaskState};

pub fn draw_review_step(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    if area.height < 10 {
        return;
    }

    let is_content_focused = app.panel_focus == PanelFocus::Content;
    let mut y = area.y + 1;

    // Title
    frame.render_widget(
        Paragraph::new("Review & Apply Configuration")
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Show summary of selections
    frame.render_widget(
        Paragraph::new("Configuration Summary:").style(app.theme.style()),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Username
    let username = app.username.content();
    let username_display = if username.is_empty() { "(not set)" } else { username };
    frame.render_widget(
        Paragraph::new(format!("  User: {}", username_display))
            .style(if username.is_empty() { app.theme.error_style() } else { app.theme.secondary_style() }),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 1;

    // Locale
    let locale_display = app.selected_locale.as_deref().unwrap_or("(system default)");
    frame.render_widget(
        Paragraph::new(format!("  Locale: {}", locale_display)).style(app.theme.style()),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 1;

    // Keyboard
    let keyboard_display = app.selected_keyboard.as_deref().unwrap_or("(system default)");
    frame.render_widget(
        Paragraph::new(format!("  Keyboard: {}", keyboard_display)).style(app.theme.style()),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 1;

    // Timezone
    let timezone_display = app.selected_timezone.as_deref().unwrap_or("(system default)");
    frame.render_widget(
        Paragraph::new(format!("  Timezone: {}", timezone_display)).style(app.theme.style()),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 1;

    // Network
    let network_display = if app.network_connected { "Connected" } else { "Not connected" };
    frame.render_widget(
        Paragraph::new(format!("  Network: {}", network_display))
            .style(if app.network_connected { app.theme.secondary_style() } else { app.theme.muted_style() }),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Show tasks if executing
    if !app.tasks.is_empty() {
        frame.render_widget(
            Paragraph::new("Applying configuration:").style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );
        y += 1;

        for task in &app.tasks {
            if y >= area.y + area.height - 4 {
                break;
            }

            let (status_char, style) = match task.status {
                TaskState::Pending => (' ', app.theme.muted_style()),
                TaskState::Running => (app.spinner_char(), app.theme.primary_style()),
                TaskState::Success => ('x', app.theme.secondary_style()),
                TaskState::Failed => ('!', app.theme.error_style()),
            };

            // Show progress bar if available
            if let Some(progress) = task.progress {
                let bar_width = 20;
                let filled = (progress as usize * bar_width / 100).min(bar_width);
                let empty = bar_width - filled;
                let bar = format!("[{}{}]", "=".repeat(filled), " ".repeat(empty));
                let line = format!("  [{status_char}] {} {bar} {progress}%", task.name);
                frame.render_widget(
                    Paragraph::new(line).style(style),
                    Rect::new(area.x + 2, y, area.width - 4, 1),
                );
            } else {
                let line = format!("  [{status_char}] {}", task.name);
                frame.render_widget(
                    Paragraph::new(line).style(style),
                    Rect::new(area.x + 2, y, area.width - 4, 1),
                );
            }
            y += 1;
        }
    } else {
        frame.render_widget(
            Paragraph::new("Press Enter to apply configuration and create user.")
                .style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );
    }

    // Validation check
    let is_valid = !app.username.content().is_empty()
        && !app.password.content().is_empty()
        && app.password.content() == app.password_confirm.content();

    // Action button
    if !app.is_executing {
        let button_y = area.y + area.height - 4;
        let button_text = " [Enter] Apply & Create User ";
        let button_width = button_text.len() as u16;
        let button_x = area.x + 2;

        let button_style = if is_valid && is_content_focused {
            app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            app.theme.muted_style().add_modifier(Modifier::REVERSED)
        };

        frame.render_widget(
            Paragraph::new(button_text).style(button_style),
            Rect::new(button_x, button_y, button_width, 1),
        );
    }
}
