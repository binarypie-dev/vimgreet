use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw_message_panel(frame: &mut Frame, area: Rect, app: &App) {
    let (text, is_error) = match &app.message {
        Some(m) => (m.text.as_str(), m.is_error),
        None if app.working => ("Authenticating...", false),
        None => return,
    };

    let (title, border_style, text_style) = if is_error {
        (
            " Error ",
            app.theme.error_style(),
            app.theme.error_style(),
        )
    } else {
        (
            " Info ",
            app.theme.secondary_style(),
            app.theme.style(),
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .title_style(border_style.add_modifier(Modifier::BOLD));

    let content = Line::from(vec![
        Span::styled(text, text_style),
        Span::styled(" (press any key to dismiss)", app.theme.muted_style()),
    ]);

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(paragraph, area);
}
