use ratatui::{prelude::*, widgets::Paragraph};

use super::super::{OnboardApp, PanelFocus};

pub fn draw_network_status(frame: &mut Frame, area: Rect, app: &OnboardApp) {
    if area.height < 5 {
        return;
    }

    let mut y = area.y + 1;

    // Title
    frame.render_widget(
        Paragraph::new("Network Configuration")
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Status
    let (status_text, status_style) = if app.network_connected {
        ("Status: Connected", app.theme.secondary_style())
    } else {
        ("Status: Not connected", app.theme.error_style())
    };

    frame.render_widget(
        Paragraph::new(status_text).style(status_style),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    let is_content_focused = app.panel_focus == PanelFocus::Content;

    if app.network_connected {
        frame.render_widget(
            Paragraph::new("Network is already connected. This step is complete.")
                .style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );

        // Show "Next" button when connected
        let button_y = area.y + area.height - 4;
        let button_text = " [Enter] Next ";
        let button_width = button_text.len() as u16;

        let button_style = if is_content_focused {
            app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            app.theme.muted_style().add_modifier(Modifier::REVERSED)
        };

        frame.render_widget(
            Paragraph::new(button_text).style(button_style),
            Rect::new(area.x + 2, button_y, button_width, 1),
        );
    } else {
        frame.render_widget(
            Paragraph::new(format!("Press Enter to launch {} for WiFi setup", app.config.network.program))
                .style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );
        y += 1;

        frame.render_widget(
            Paragraph::new("Or use :skip to continue without network")
                .style(app.theme.muted_style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );

        // Show "Configure WiFi" button
        let button_y = area.y + area.height - 4;
        let button_text = " [Enter] Configure WiFi ";
        let button_width = button_text.len() as u16;

        let button_style = if is_content_focused {
            app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            app.theme.muted_style().add_modifier(Modifier::REVERSED)
        };

        frame.render_widget(
            Paragraph::new(button_text).style(button_style),
            Rect::new(area.x + 2, button_y, button_width, 1),
        );
    }
}
