use ratatui::{prelude::*, widgets::Paragraph};

use super::super::{OnboardApp, PanelFocus};

pub fn draw_reboot_step(frame: &mut Frame, area: Rect, app: &OnboardApp) {
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
