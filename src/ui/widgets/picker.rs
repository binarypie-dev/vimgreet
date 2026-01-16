use crate::app::App;
use crate::ui::Layout;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};

pub fn draw_session_picker(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.selected_session {
                app.theme.primary_style().add_modifier(Modifier::REVERSED)
            } else {
                app.theme.style()
            };
            let marker = if i == app.selected_session { ">" } else { " " };
            let session_type = match s.session_type {
                crate::system::SessionType::Wayland => "[W]",
                crate::system::SessionType::X11 => "[X]",
            };
            ListItem::new(format!("{} {} {}", marker, session_type, s.name)).style(style)
        })
        .collect();

    let height = (items.len() as u16 + 2).min(area.height.saturating_sub(4)).max(5);
    let width = 40u16.min(area.width.saturating_sub(4));
    let picker_area = Layout::centered_box(area, width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(" Sessions (j/k to select, Enter to confirm) ")
        .title_style(app.theme.primary_style());

    frame.render_widget(Clear, picker_area);

    let list = List::new(items).block(block);
    let mut state = ListState::default().with_selected(Some(app.selected_session));

    frame.render_stateful_widget(list, picker_area, &mut state);
}

pub fn draw_user_picker(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .users
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let style = if i == app.selected_user {
                app.theme.primary_style().add_modifier(Modifier::REVERSED)
            } else {
                app.theme.style()
            };
            let marker = if i == app.selected_user { ">" } else { " " };
            let display = if let Some(ref name) = u.display_name {
                format!("{} {} ({})", marker, name, u.username)
            } else {
                format!("{} {}", marker, u.username)
            };
            ListItem::new(display).style(style)
        })
        .collect();

    let height = (items.len() as u16 + 2).min(area.height.saturating_sub(4)).max(5);
    let width = 40u16.min(area.width.saturating_sub(4));
    let picker_area = Layout::centered_box(area, width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(app.theme.border_style())
        .title(" Users (j/k to select, Enter to confirm) ")
        .title_style(app.theme.primary_style());

    frame.render_widget(Clear, picker_area);

    let list = List::new(items).block(block);
    let mut state = ListState::default().with_selected(Some(app.selected_user));

    frame.render_stateful_widget(list, picker_area, &mut state);
}
