use ratatui::{prelude::*, widgets::{Block, Borders, Clear, Paragraph}};

use super::super::OnboardApp;
use super::center_rect;

pub fn draw_welcome_content(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    let content_width = 60.min(area.width - 4);
    let content_height = 14.min(area.height - 2);
    let centered = center_rect(area, content_width, content_height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(format!(" {} (v{}) ", app.config.general.title, env!("CARGO_PKG_VERSION")));

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
