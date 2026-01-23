use crate::greeter::ui::Layout;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn draw_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Normal Mode", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  h/l      Move cursor left/right"),
        Line::from("  j/k      Move between fields"),
        Line::from("  i        Enter insert mode"),
        Line::from("  a        Enter insert mode (after cursor)"),
        Line::from("  :        Enter command mode"),
        Line::from("  x        Delete character"),
        Line::from("  dd       Clear field"),
        Line::from("  Enter    Login"),
        Line::from(""),
        Line::from(Span::styled("Insert Mode", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Escape   Return to normal mode"),
        Line::from("  Enter    Submit / next field"),
        Line::from(""),
        Line::from(Span::styled("Commands", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  :session [name]   Select session"),
        Line::from("  :user [name]      Select user"),
        Line::from("  :reboot           Reboot system"),
        Line::from("  :poweroff         Shutdown system"),
        Line::from("  :help             Show this help"),
        Line::from("  :q                Login / quit"),
        Line::from(""),
        Line::from(Span::styled("Press Escape to close", Style::default().fg(Color::DarkGray))),
    ];

    let height = help_text.len() as u16 + 2;
    let width = 45u16.min(area.width.saturating_sub(4));
    let help_area = Layout::centered_box(area, width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Help ")
        .title_style(Style::default().fg(Color::Yellow));

    frame.render_widget(Clear, help_area);
    frame.render_widget(
        Paragraph::new(help_text).block(block),
        help_area,
    );
}
