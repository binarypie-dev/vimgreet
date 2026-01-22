use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use super::steps::StepId;
use super::{ConfirmAction, ContentFocus, OnboardApp, PanelFocus, TaskState};
use crate::vim::VimMode;

/// Main draw function for the onboard wizard
pub fn draw(frame: &mut Frame, app: &OnboardApp) {
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
    draw_welcome_content(frame, chunks[1], app);

    // Message area
    draw_message(frame, chunks[2], app);

    // Status bar
    draw_status_bar(frame, chunks[3], app);
}

/// Draw header bar (1 line, no borders) - matches login screen style
fn draw_header(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    frame.render_widget(Clear, area);

    // Left side: title
    let title = " System Setup ";
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

fn draw_welcome_content(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    let content_width = 60.min(area.width - 4);
    let content_height = 14.min(area.height - 2);
    let centered = center_rect(area, content_width, content_height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(format!(" {} ", app.config.general.title));

    let inner = block.inner(centered);
    frame.render_widget(Clear, centered);
    frame.render_widget(block, centered);

    let welcome_text = [
        "",
        "This wizard will help you set up your system:",
        "",
        "  * Create your user account",
        "  * Configure language and keyboard",
        "  * Set your timezone",
        "  * Connect to the network (if needed)",
        "  * Install applications",
        "",
    ];

    let mut y = inner.y;
    for line in &welcome_text {
        if y >= inner.y + inner.height {
            break;
        }
        frame.render_widget(
            Paragraph::new(*line).style(app.theme.style()),
            Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1),
        );
        y += 1;
    }

    // Start Setup button - centered at bottom
    let button_y = inner.y + inner.height - 2;
    let button_text = "[ Start Setup ]";
    let button_width = button_text.len() as u16;
    let button_x = inner.x + (inner.width.saturating_sub(button_width)) / 2;

    frame.render_widget(
        Paragraph::new(button_text)
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)),
        Rect::new(button_x, button_y, button_width, 1),
    );

    // Hint
    let hint = "Press Enter to begin";
    let hint_x = inner.x + (inner.width.saturating_sub(hint.len() as u16)) / 2;
    frame.render_widget(
        Paragraph::new(hint).style(app.theme.muted_style()),
        Rect::new(hint_x, button_y + 1, hint.len() as u16, 1),
    );
}

fn draw_setup_screen(frame: &mut Frame, area: Rect, app: &OnboardApp) {
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


fn draw_setup_content(frame: &mut Frame, area: Rect, app: &OnboardApp) {
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

fn draw_main_content(frame: &mut Frame, area: Rect, app: &OnboardApp) {
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
            StepId::User => draw_user_form(frame, inner, app),
            StepId::Locale => draw_picker(frame, inner, app, "Select Locale", &app.config.locale.default_locale),
            StepId::Keyboard => draw_picker(frame, inner, app, "Select Keyboard Layout", &app.config.keyboard.default_layout),
            StepId::Preferences => draw_picker(frame, inner, app, "Select Timezone", &app.config.preferences.default_timezone),
            StepId::Network => draw_network_status(frame, inner, app),
            StepId::Review => draw_review_step(frame, inner, app),
            StepId::Update => draw_update_step(frame, inner, app),
            StepId::Reboot => draw_reboot_step(frame, inner, app),
        }
    }
}

fn draw_user_form(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    if area.height < 10 || area.width < 30 {
        return;
    }

    let is_content_focused = app.panel_focus == PanelFocus::Content;
    let mut y = area.y + 1;

    // Title
    frame.render_widget(
        Paragraph::new("Create User Account")
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Form fields - vim-style with cursor
    let fields = [
        ("Username", &app.username, false, 0),
        ("Password", &app.password, true, 1),
        ("Confirm", &app.password_confirm, true, 2),
    ];

    for (label, buffer, masked, field_idx) in &fields {
        let is_field_focused = is_content_focused && matches!(app.content_focus, ContentFocus::InputField(i) if i == *field_idx);
        let is_insert = app.vim_mode == VimMode::Insert && is_field_focused;

        // Label
        let label_style = if is_field_focused {
            app.theme.primary_style()
        } else {
            app.theme.style()
        };
        frame.render_widget(
            Paragraph::new(*label).style(label_style),
            Rect::new(area.x + 2, y, 12, 1),
        );

        // Input field with vim-style display
        let content = buffer.content();
        let display_content = if *masked && !content.is_empty() {
            "*".repeat(content.len())
        } else {
            content.to_string()
        };

        let field_x = area.x + 14;
        let field_width = area.width.saturating_sub(18);

        // Draw field background/border
        let field_style = if is_field_focused {
            app.theme.primary_style()
        } else {
            app.theme.muted_style()
        };

        // Draw the content with cursor
        if is_insert {
            // Insert mode - show cursor as |
            let cursor_pos = buffer.cursor();
            let before: String = display_content.chars().take(cursor_pos).collect();
            let after: String = display_content.chars().skip(cursor_pos).collect();

            let line = Line::from(vec![
                Span::styled(before, app.theme.style()),
                Span::styled("|", app.theme.primary_style().add_modifier(Modifier::BOLD)),
                Span::styled(after, app.theme.style()),
            ]);
            frame.render_widget(Paragraph::new(line), Rect::new(field_x, y, field_width, 1));
        } else if is_field_focused {
            // Normal mode - show cursor as block
            let cursor_pos = buffer.cursor();
            let chars: Vec<char> = display_content.chars().collect();
            let mut spans = Vec::new();

            for (i, ch) in chars.iter().enumerate() {
                if i == cursor_pos {
                    spans.push(Span::styled(ch.to_string(), app.theme.style().add_modifier(Modifier::REVERSED)));
                } else {
                    spans.push(Span::styled(ch.to_string(), app.theme.style()));
                }
            }
            if cursor_pos >= chars.len() {
                spans.push(Span::styled(" ", app.theme.style().add_modifier(Modifier::REVERSED)));
            }

            frame.render_widget(Paragraph::new(Line::from(spans)), Rect::new(field_x, y, field_width, 1));
        } else {
            // Not focused - just show content
            let display = if display_content.is_empty() { "(empty)" } else { &display_content };
            frame.render_widget(
                Paragraph::new(display).style(field_style),
                Rect::new(field_x, y, field_width, 1),
            );
        }

        y += 2;
    }

    // Action button area
    let button_y = area.y + area.height - 4;
    let is_form_ready = !app.username.content().is_empty()
        && !app.password.content().is_empty()
        && !app.password_confirm.content().is_empty();

    let button_text = " [Enter] Create User ";
    let button_width = button_text.len() as u16;
    let button_x = area.x + 2;

    let button_style = if is_form_ready && is_content_focused {
        app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        app.theme.muted_style().add_modifier(Modifier::REVERSED)
    };

    frame.render_widget(
        Paragraph::new(button_text).style(button_style),
        Rect::new(button_x, button_y, button_width, 1),
    );

}

fn draw_picker(frame: &mut Frame, area: Rect, app: &OnboardApp, title: &str, default: &str) {
    if area.height < 5 || area.width < 20 {
        return;
    }

    let is_content_focused = app.panel_focus == PanelFocus::Content;
    let is_picker_focused = is_content_focused && matches!(app.content_focus, ContentFocus::Picker);
    let is_insert = app.vim_mode == VimMode::Insert && is_picker_focused;

    let mut y = area.y + 1;

    // Title
    frame.render_widget(
        Paragraph::new(title)
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Filter input
    let filter_label = "Filter: ";
    frame.render_widget(
        Paragraph::new(filter_label).style(app.theme.style()),
        Rect::new(area.x + 2, y, filter_label.len() as u16, 1),
    );

    let filter_x = area.x + 2 + filter_label.len() as u16;
    let filter_width = area.width.saturating_sub(filter_label.len() as u16 + 4);
    let filter_content = app.picker_filter.content();

    if is_insert {
        // Show cursor in filter
        let cursor_pos = app.picker_filter.cursor();
        let before: String = filter_content.chars().take(cursor_pos).collect();
        let after: String = filter_content.chars().skip(cursor_pos).collect();

        let line = Line::from(vec![
            Span::styled(before, app.theme.style()),
            Span::styled("|", app.theme.primary_style().add_modifier(Modifier::BOLD)),
            Span::styled(after, app.theme.style()),
        ]);
        frame.render_widget(Paragraph::new(line), Rect::new(filter_x, y, filter_width, 1));
    } else {
        let display = if filter_content.is_empty() {
            format!("(default: {default})")
        } else {
            filter_content.to_string()
        };
        frame.render_widget(
            Paragraph::new(display).style(if is_picker_focused {
                app.theme.primary_style()
            } else {
                app.theme.muted_style()
            }),
            Rect::new(filter_x, y, filter_width, 1),
        );
    }
    y += 2;

    // Picker list
    let filtered = app.filtered_picker_items();
    let list_height = area.height.saturating_sub(7) as usize;

    // Calculate scroll
    let scroll_offset = if app.picker_selected >= list_height {
        app.picker_selected - list_height + 1
    } else {
        0
    };

    for (i, item) in filtered.iter().skip(scroll_offset).take(list_height).enumerate() {
        let idx = i + scroll_offset;
        let is_selected = idx == app.picker_selected;

        let prefix = if is_selected { ">" } else { " " };
        let line = format!("{prefix} {item}");

        let style = if is_selected && is_picker_focused {
            app.theme.primary_style().add_modifier(Modifier::BOLD)
        } else if is_selected {
            app.theme.secondary_style()
        } else {
            app.theme.style()
        };

        frame.render_widget(
            Paragraph::new(line).style(style),
            Rect::new(area.x + 2, y + i as u16, area.width - 4, 1),
        );
    }

    // Scrollbar
    if filtered.len() > list_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));

        let mut scrollbar_state = ScrollbarState::new(filtered.len())
            .position(app.picker_selected);

        let scrollbar_area = Rect::new(
            area.x + area.width - 2,
            y,
            1,
            list_height as u16,
        );

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Action button area
    let button_y = area.y + area.height - 4;
    let has_selection = !filtered.is_empty();

    if has_selection {
        let button_text = " [Enter] Save & Next ";
        let button_width = button_text.len() as u16;
        let button_x = area.x + 2;

        let button_style = if is_picker_focused {
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

fn draw_network_status(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    if area.height < 5 {
        return;
    }

    let mut y = area.y + 1;

    // Title
    frame.render_widget(
        Paragraph::new("Network Configuration")
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Status
    let (status_text, status_style) = if app.network_connected {
        ("Status: Connected", app.theme.secondary_style())
    } else {
        ("Status: Not connected", app.theme.error_style())
    };

    frame.render_widget(
        Paragraph::new(status_text).style(status_style),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    let is_content_focused = app.panel_focus == PanelFocus::Content;

    if app.network_connected {
        frame.render_widget(
            Paragraph::new("Network is already connected. This step is complete.")
                .style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );

        // Show "Next" button when connected
        let button_y = area.y + area.height - 4;
        let button_text = " [Enter] Next ";
        let button_width = button_text.len() as u16;

        let button_style = if is_content_focused {
            app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            app.theme.muted_style().add_modifier(Modifier::REVERSED)
        };

        frame.render_widget(
            Paragraph::new(button_text).style(button_style),
            Rect::new(area.x + 2, button_y, button_width, 1),
        );
    } else {
        frame.render_widget(
            Paragraph::new(format!("Press Enter to launch {} for WiFi setup", app.config.network.program))
                .style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );
        y += 1;

        frame.render_widget(
            Paragraph::new("Or use :skip to continue without network")
                .style(app.theme.muted_style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );

        // Show "Configure WiFi" button
        let button_y = area.y + area.height - 4;
        let button_text = " [Enter] Configure WiFi ";
        let button_width = button_text.len() as u16;

        let button_style = if is_content_focused {
            app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            app.theme.muted_style().add_modifier(Modifier::REVERSED)
        };

        frame.render_widget(
            Paragraph::new(button_text).style(button_style),
            Rect::new(area.x + 2, button_y, button_width, 1),
        );
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

fn draw_review_step(frame: &mut Frame, area: Rect, app: &OnboardApp) {
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

fn draw_update_step(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    if area.height < 8 {
        return;
    }

    let is_content_focused = app.panel_focus == PanelFocus::Content;
    let is_password_focused = is_content_focused && matches!(app.content_focus, ContentFocus::InputField(0));
    let is_insert = app.vim_mode == VimMode::Insert && is_password_focused;
    let mut y = area.y + 1;

    // Title
    frame.render_widget(
        Paragraph::new("Install Packages")
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Show running tasks if executing
    if !app.tasks.is_empty() {
        frame.render_widget(
            Paragraph::new("Installing:").style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );
        y += 1;

        for task in &app.tasks {
            if y >= area.y + area.height - 2 {
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
        // Show package selection list
        frame.render_widget(
            Paragraph::new("Select packages to install:")
                .style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );
        y += 2;

        let max_y = area.y + area.height - 4;

        for (cat_idx, category) in app.config.updates.iter().enumerate() {
            if y >= max_y {
                break;
            }

            // Category header with aggregate checkbox
            let is_cat_cursor = is_content_focused
                && cat_idx == app.update_category_cursor
                && app.update_package_cursor.is_none();

            let checkbox = if app.is_category_fully_selected(cat_idx) {
                "[x]"
            } else if app.is_category_partially_selected(cat_idx) {
                "[~]"
            } else {
                "[ ]"
            };

            let cursor = if is_cat_cursor { ">" } else { " " };

            let cat_style = if is_cat_cursor {
                app.theme.primary_style().add_modifier(Modifier::BOLD)
            } else if app.is_category_any_selected(cat_idx) {
                app.theme.secondary_style()
            } else {
                app.theme.style()
            };

            frame.render_widget(
                Paragraph::new(format!("{cursor} {checkbox} {}", category.name))
                    .style(cat_style),
                Rect::new(area.x + 2, y, area.width - 4, 1),
            );
            y += 1;

            // Category description
            if !category.description.is_empty() && y < max_y {
                frame.render_widget(
                    Paragraph::new(format!("       {}", category.description))
                        .style(app.theme.muted_style()),
                    Rect::new(area.x + 2, y, area.width - 4, 1),
                );
                y += 1;
            }

            // Packages within this category
            for (pkg_idx, pkg) in category.packages.iter().enumerate() {
                if y >= max_y {
                    break;
                }

                let is_pkg_cursor = is_content_focused
                    && cat_idx == app.update_category_cursor
                    && app.update_package_cursor == Some(pkg_idx);

                let pkg_selected = app.update_package_selected
                    .get(cat_idx)
                    .and_then(|pkgs| pkgs.get(pkg_idx))
                    .copied()
                    .unwrap_or(false);

                let pkg_checkbox = if pkg.required {
                    "[x]"
                } else if pkg_selected {
                    "[x]"
                } else {
                    "[ ]"
                };
                let pkg_cursor = if is_pkg_cursor { ">" } else { " " };
                let required_marker = if pkg.required { " *" } else { "" };

                let pkg_style = if is_pkg_cursor {
                    app.theme.primary_style().add_modifier(Modifier::BOLD)
                } else if pkg_selected {
                    app.theme.secondary_style()
                } else {
                    app.theme.style()
                };

                frame.render_widget(
                    Paragraph::new(format!("  {pkg_cursor} {pkg_checkbox} {}{required_marker}", pkg.title))
                        .style(pkg_style),
                    Rect::new(area.x + 2, y, area.width - 4, 1),
                );
                y += 1;

                // Package description
                if !pkg.description.is_empty() && y < max_y {
                    frame.render_widget(
                        Paragraph::new(format!("           {}", pkg.description))
                            .style(app.theme.muted_style()),
                        Rect::new(area.x + 2, y, area.width - 4, 1),
                    );
                    y += 1;
                }
            }

            y += 1; // Spacing between categories
        }

        // Show sudo password input if needed
        if app.sudo_password_needed && !app.sudo_password_entered && !app.is_dryrun() {
            if y < max_y {
                y += 1;
                frame.render_widget(
                    Paragraph::new("Password required for sudo commands:")
                        .style(app.theme.primary_style()),
                    Rect::new(area.x + 2, y, area.width - 4, 1),
                );
                y += 1;

                let label = "Password: ";
                frame.render_widget(
                    Paragraph::new(label).style(app.theme.style()),
                    Rect::new(area.x + 2, y, label.len() as u16, 1),
                );

                let field_x = area.x + 2 + label.len() as u16;
                let field_width = area.width.saturating_sub(label.len() as u16 + 4);
                let password_display = "*".repeat(app.sudo_password.content().len());

                if is_insert {
                    let cursor_pos = app.sudo_password.cursor().min(password_display.len());
                    let before: String = password_display.chars().take(cursor_pos).collect();
                    let after: String = password_display.chars().skip(cursor_pos).collect();

                    let line = Line::from(vec![
                        Span::styled(before, app.theme.style()),
                        Span::styled("|", app.theme.primary_style().add_modifier(Modifier::BOLD)),
                        Span::styled(after, app.theme.style()),
                    ]);
                    frame.render_widget(Paragraph::new(line), Rect::new(field_x, y, field_width, 1));
                } else if is_password_focused {
                    let chars: Vec<char> = password_display.chars().collect();
                    let cursor_pos = app.sudo_password.cursor().min(chars.len());
                    let mut spans = Vec::new();

                    for (i, ch) in chars.iter().enumerate() {
                        if i == cursor_pos {
                            spans.push(Span::styled(ch.to_string(), app.theme.style().add_modifier(Modifier::REVERSED)));
                        } else {
                            spans.push(Span::styled(ch.to_string(), app.theme.style()));
                        }
                    }
                    if cursor_pos >= chars.len() {
                        spans.push(Span::styled(" ", app.theme.style().add_modifier(Modifier::REVERSED)));
                    }

                    frame.render_widget(Paragraph::new(Line::from(spans)), Rect::new(field_x, y, field_width, 1));
                } else {
                    let display = if password_display.is_empty() { "(enter password)" } else { &password_display };
                    frame.render_widget(
                        Paragraph::new(display).style(app.theme.muted_style()),
                        Rect::new(field_x, y, field_width, 1),
                    );
                }
            }
        }
    }

    // Action button
    let can_run = !app.is_executing && app.tasks.is_empty();
    let needs_password = app.commands_need_sudo() && !app.sudo_password_entered && !app.is_dryrun();
    let any_selected = app.any_package_selected();

    if can_run {
        let button_y = area.y + area.height - 2;
        let button_text = if needs_password && app.sudo_password.content().is_empty() {
            " [i] Enter Password "
        } else if any_selected {
            " [Enter] Install Selected "
        } else {
            " [Enter] Skip & Continue "
        };
        let button_width = button_text.len() as u16;
        let button_x = area.x + 2;

        let button_style = if is_content_focused {
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

fn draw_reboot_step(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    if area.height < 8 {
        return;
    }

    let is_content_focused = app.panel_focus == PanelFocus::Content;
    let mut y = area.y + 1;

    // Title
    frame.render_widget(
        Paragraph::new("Setup Complete!")
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    frame.render_widget(
        Paragraph::new("Your system is configured and ready.")
            .style(app.theme.style()),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Show created user
    if let Some(ref username) = app.created_username {
        frame.render_widget(
            Paragraph::new(format!("User '{}' has been created.", username))
                .style(app.theme.secondary_style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );
        y += 1;
    }

    y += 1;
    frame.render_widget(
        Paragraph::new("A reboot is required to apply all changes.")
            .style(app.theme.style()),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 1;

    frame.render_widget(
        Paragraph::new("After rebooting, you will see the login screen.")
            .style(app.theme.muted_style()),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );

    // Action button
    let button_y = area.y + area.height - 4;
    let button_text = " [Enter] Reboot Now ";
    let button_width = button_text.len() as u16;
    let button_x = area.x + 2;

    let button_style = if is_content_focused {
        app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        app.theme.muted_style().add_modifier(Modifier::REVERSED)
    };

    frame.render_widget(
        Paragraph::new(button_text).style(button_style),
        Rect::new(button_x, button_y, button_width, 1),
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

fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
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
