use crate::greeter::ConfirmAction;
use crate::greeter::ui::Layout;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn draw_confirm_dialog(frame: &mut Frame, area: Rect, action: &ConfirmAction) {
    let (title, message) = match action {
        ConfirmAction::Reboot => ("Reboot", "Are you sure you want to reboot?"),
        ConfirmAction::Poweroff => ("Shutdown", "Are you sure you want to shut down?"),
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" - Yes    "),
            Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" - No"),
        ]),
    ];

    let dialog_area = Layout::centered_box(area, 40, 6);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(format!(" {} ", title))
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    frame.render_widget(Clear, dialog_area);
    frame.render_widget(
        Paragraph::new(text).block(block).alignment(Alignment::Center),
        dialog_area,
    );
}
