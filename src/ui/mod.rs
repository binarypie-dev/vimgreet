mod layout;
mod theme;
pub mod widgets;

pub use layout::Layout;
pub use theme::Theme;

use crate::app::App;
use ratatui::Frame;

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
