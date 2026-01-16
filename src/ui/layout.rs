use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

pub struct Layout {
    pub full: Rect,
    pub header: Rect,
    pub content: Rect,
    pub message: Rect,
    pub status: Rect,
}

impl Layout {
    pub fn new(area: Rect) -> Self {
        // Always use the same layout - message panel space is always reserved
        // This prevents the login box from jumping when messages appear/disappear
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
