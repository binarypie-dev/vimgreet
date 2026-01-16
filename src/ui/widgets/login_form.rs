use crate::app::{App, FocusField};
use crate::ui::Layout;
use crate::vim::VimMode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn draw_login_form(frame: &mut Frame, area: Rect, app: &App) {
    let form_width = 50u16.min(area.width.saturating_sub(4));
    let form_height = 12u16;
    let form_area = Layout::centered_box(area, form_width, form_height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(" Login ")
        .title_style(app.theme.primary_style());

    frame.render_widget(Clear, form_area);
    frame.render_widget(block, form_area);

    let inner = form_area.inner(Margin::new(2, 1));

    let chunks = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Session
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Username label
            Constraint::Length(1), // Username input
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Password label
            Constraint::Length(1), // Password input
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Hint
        ])
        .split(inner);

    // Session indicator
    let session_name = app
        .sessions
        .get(app.selected_session)
        .map(|s| s.name.as_str())
        .unwrap_or("(no session)");
    let session_line = Line::from(vec![
        Span::styled("Session: ", app.theme.muted_style()),
        Span::styled(session_name, app.theme.secondary_style()),
        Span::styled(" (F3)", app.theme.muted_style()),
    ]);
    frame.render_widget(Paragraph::new(session_line), chunks[0]);

    // Username field
    let username_focused = app.focus == FocusField::Username;
    let username_style = if username_focused {
        app.theme.primary_style()
    } else {
        app.theme.muted_style()
    };

    let username_label = Line::from(vec![
        Span::styled("Username", username_style),
        if username_focused {
            Span::styled(" (i to edit)", app.theme.muted_style())
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(username_label), chunks[2]);

    let username_content = render_input_field(
        app.username.content(),
        app.username.cursor(),
        username_focused,
        app.vim_mode == VimMode::Insert,
        &app.theme,
    );
    frame.render_widget(Paragraph::new(username_content), chunks[3]);

    // Password field
    let password_focused = app.focus == FocusField::Password;
    let password_style = if password_focused {
        app.theme.primary_style()
    } else {
        app.theme.muted_style()
    };

    let password_label = Line::from(vec![
        Span::styled("Password", password_style),
        if password_focused {
            Span::styled(" (i to edit)", app.theme.muted_style())
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(password_label), chunks[5]);

    let password_display = app.password.display('*');
    let password_content = render_input_field(
        &password_display,
        app.password.cursor(),
        password_focused,
        app.vim_mode == VimMode::Insert,
        &app.theme,
    );
    frame.render_widget(Paragraph::new(password_content), chunks[6]);

    // Hint line (only show when no message panel is visible)
    if app.message.is_none() && !app.working {
        let hint = Line::from(Span::styled(
            "Press Enter to login, :help for commands",
            app.theme.muted_style(),
        ));
        frame.render_widget(Paragraph::new(hint), chunks[8]);
    }
}

fn render_input_field<'a>(
    content: &'a str,
    cursor: usize,
    focused: bool,
    insert_mode: bool,
    theme: &crate::ui::Theme,
) -> Line<'a> {
    let prefix = if focused { "> " } else { "  " };

    if !focused {
        return Line::from(vec![
            Span::styled(prefix, theme.muted_style()),
            Span::styled(content.to_string(), theme.muted_style()),
        ]);
    }

    let mut spans = vec![Span::styled(prefix, theme.primary_style())];

    if insert_mode {
        let (before, after) = if cursor <= content.chars().count() {
            let before: String = content.chars().take(cursor).collect();
            let after: String = content.chars().skip(cursor).collect();
            (before, after)
        } else {
            (content.to_string(), String::new())
        };

        spans.push(Span::raw(before));
        spans.push(Span::styled("│", theme.primary_style())); // Cursor
        spans.push(Span::raw(after));
    } else {
        // Normal mode - show block cursor on character
        if content.is_empty() {
            spans.push(Span::styled(" ", Style::default().bg(theme.primary).fg(theme.background)));
        } else {
            let chars: Vec<char> = content.chars().collect();
            let cursor_pos = cursor.min(chars.len().saturating_sub(1));

            let before: String = chars[..cursor_pos].iter().collect();
            let cursor_char = chars.get(cursor_pos).copied().unwrap_or(' ');
            let after: String = chars[cursor_pos + 1..].iter().collect();

            spans.push(Span::raw(before));
            spans.push(Span::styled(
                cursor_char.to_string(),
                Style::default().bg(theme.primary).fg(theme.background),
            ));
            spans.push(Span::raw(after));
        }
    }

    Line::from(spans)
}
