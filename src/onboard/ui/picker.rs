use ratatui::{
    prelude::*,
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use super::super::{ContentFocus, OnboardApp, PanelFocus};
use crate::vim::VimMode;

pub fn draw_picker(frame: &mut Frame, area: Rect, app: &OnboardApp, title: &str, default: &str) {
    if area.height < 5 || area.width < 20 {
        return;
    }

    let is_content_focused = app.panel_focus == PanelFocus::Content;
    let is_picker_focused = is_content_focused && matches!(app.content_focus, ContentFocus::Picker);
    let is_insert = app.vim_mode == VimMode::Insert && is_picker_focused;

    let mut y = area.y + 1;

    // Title
    frame.render_widget(
        Paragraph::new(title)
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Filter input
    let filter_label = "Filter: ";
    frame.render_widget(
        Paragraph::new(filter_label).style(app.theme.style()),
        Rect::new(area.x + 2, y, filter_label.len() as u16, 1),
    );

    let filter_x = area.x + 2 + filter_label.len() as u16;
    let filter_width = area.width.saturating_sub(filter_label.len() as u16 + 4);
    let filter_content = app.picker_filter.content();

    if is_insert {
        // Show cursor in filter
        let cursor_pos = app.picker_filter.cursor();
        let before: String = filter_content.chars().take(cursor_pos).collect();
        let after: String = filter_content.chars().skip(cursor_pos).collect();

        let line = Line::from(vec![
            Span::styled(before, app.theme.style()),
            Span::styled("|", app.theme.primary_style().add_modifier(Modifier::BOLD)),
            Span::styled(after, app.theme.style()),
        ]);
        frame.render_widget(Paragraph::new(line), Rect::new(filter_x, y, filter_width, 1));
    } else {
        let display = if filter_content.is_empty() {
            format!("(default: {default})")
        } else {
            filter_content.to_string()
        };
        frame.render_widget(
            Paragraph::new(display).style(if is_picker_focused {
                app.theme.primary_style()
            } else {
                app.theme.muted_style()
            }),
            Rect::new(filter_x, y, filter_width, 1),
        );
    }
    y += 2;

    // Picker list
    let filtered = app.filtered_picker_items();
    let button_y = area.y + area.height.saturating_sub(4);
    let list_height = button_y.saturating_sub(y + 1) as usize;

    // Calculate scroll
    let scroll_offset = if app.picker_selected >= list_height {
        app.picker_selected - list_height + 1
    } else {
        0
    };

    for (i, item) in filtered.iter().skip(scroll_offset).take(list_height).enumerate() {
        let idx = i + scroll_offset;
        let is_selected = idx == app.picker_selected;

        let prefix = if is_selected { ">" } else { " " };
        let line = format!("{prefix} {item}");

        let style = if is_selected && is_picker_focused {
            app.theme.primary_style().add_modifier(Modifier::BOLD)
        } else if is_selected {
            app.theme.secondary_style()
        } else {
            app.theme.style()
        };

        frame.render_widget(
            Paragraph::new(line).style(style),
            Rect::new(area.x + 2, y + i as u16, area.width - 4, 1),
        );
    }

    // Scrollbar
    if filtered.len() > list_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));

        let mut scrollbar_state = ScrollbarState::new(filtered.len())
            .position(app.picker_selected);

        let scrollbar_area = Rect::new(
            area.x + area.width - 2,
            y,
            1,
            list_height as u16,
        );

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Action button area
    let button_y = area.y + area.height - 4;
    let has_selection = !filtered.is_empty();

    if has_selection {
        let button_text = " [Enter] Save & Next ";
        let button_width = button_text.len() as u16;
        let button_x = area.x + 2;

        let button_style = if is_picker_focused {
            app.theme.primary_style().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            app.theme.muted_style().add_modifier(Modifier::REVERSED)
        };

        frame.render_widget(
            Paragraph::new(button_text).style(button_style),
            Rect::new(button_x, button_y, button_width, 1),
        );
    }
}
