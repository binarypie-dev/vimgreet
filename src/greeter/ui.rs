use crate::greeter::App;
use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};
use ratatui::Frame;

use super::widgets;

pub struct Layout {
    pub full: Rect,
    pub header: Rect,
    pub content: Rect,
    pub message: Rect,
    pub status: Rect,
}

impl Layout {
    pub fn new(area: Rect) -> Self {
        let chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Header
                Constraint::Min(10),    // Content
                Constraint::Length(3),  // Message panel (always reserved)
                Constraint::Length(1),  // Status bar
            ])
            .split(area);

        Self {
            full: area,
            header: chunks[0],
            content: chunks[1],
            message: chunks[2],
            status: chunks[3],
        }
    }

    pub fn centered_box(area: Rect, width: u16, height: u16) -> Rect {
        let horizontal = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(width),
                Constraint::Fill(1),
            ])
            .split(area);

        let vertical = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(height),
                Constraint::Fill(1),
            ])
            .split(horizontal[1]);

        vertical[1]
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let layout = Layout::new(frame.area());

    widgets::draw_background(frame, layout.full, &app.theme);
    widgets::draw_header(frame, layout.header, app);
    widgets::draw_login_form(frame, layout.content, app);

    // Always draw message panel area (shows content only when there's a message)
    widgets::draw_message_panel(frame, layout.message, app);

    widgets::draw_status_bar(frame, layout.status, app);

    // Popups render on top of everything
    if app.show_session_picker {
        widgets::draw_session_picker(frame, layout.content, app);
    }

    if app.show_user_picker {
        widgets::draw_user_picker(frame, layout.content, app);
    }

    if app.show_help {
        widgets::draw_help(frame, layout.content);
    }

    if let Some(ref confirm) = app.confirm_action {
        widgets::draw_confirm_dialog(frame, layout.content, confirm);
    }
}
