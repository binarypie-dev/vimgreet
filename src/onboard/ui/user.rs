use ratatui::{prelude::*, widgets::Paragraph};

use super::super::{ContentFocus, OnboardApp, PanelFocus};
use crate::vim::VimMode;

pub fn draw_user_form(frame: &mut Frame, area: Rect, app: &OnboardApp) {
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
