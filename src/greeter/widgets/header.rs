use crate::greeter::App;
use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let time = Local::now().format("%H:%M").to_string();
    let date = Local::now().format("%A, %B %d").to_string();

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "hypercube".to_string());

    // Left side: hostname
    let left = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled(&hostname, app.theme.primary_style().add_modifier(Modifier::BOLD)),
    ]));
    frame.render_widget(left, area);

    // Right side: date and time
    let right = Paragraph::new(Line::from(vec![
        Span::styled(&date, app.theme.muted_style()),
        Span::raw("  "),
        Span::styled(&time, app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Span::raw(" "),
    ]))
    .alignment(Alignment::Right);
    frame.render_widget(right, area);
}
