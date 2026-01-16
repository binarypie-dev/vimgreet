mod confirm;
mod header;
mod help;
mod login_form;
mod message_panel;
mod picker;
mod status_bar;

pub use confirm::draw_confirm_dialog;
pub use header::draw_header;
pub use help::draw_help;
pub use login_form::draw_login_form;
pub use message_panel::draw_message_panel;
pub use picker::{draw_session_picker, draw_user_picker};
pub use status_bar::draw_status_bar;

use crate::ui::Theme;
use ratatui::prelude::*;
use ratatui::widgets::Block;

pub fn draw_background(frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default().style(theme.style());
    frame.render_widget(block, area);
}
