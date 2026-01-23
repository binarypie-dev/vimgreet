use crate::greeter::App;
use crate::vim::VimMode;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let mode_name = app.vim_mode.display_name();
    let mode_style = app.theme.mode_style(mode_name);

    let mut left_spans = vec![
        Span::raw(" "),
        Span::styled(format!(" {} ", mode_name), mode_style),
        Span::raw(" "),
    ];

    // Show command buffer in command mode
    let cmd_content;
    if app.vim_mode == VimMode::Command {
        cmd_content = app.command_buffer.content().to_string();
        left_spans.push(Span::styled(":", app.theme.primary_style()));
        left_spans.push(Span::raw(cmd_content.as_str()));
        left_spans.push(Span::styled("│", app.theme.primary_style()));
    }

    let left = Line::from(left_spans);

    // Right side: keybinding hints
    let demo_indicator = if app.demo_mode {
        Span::styled(" [DEMO] ", app.theme.error_style())
    } else {
        Span::raw("")
    };

    let hints = if app.vim_mode == VimMode::Normal {
        vec![
            Span::styled("F2", app.theme.secondary_style()),
            Span::styled(":users ", app.theme.muted_style()),
            Span::styled("F3", app.theme.secondary_style()),
            Span::styled(":sessions ", app.theme.muted_style()),
            Span::styled("F12", app.theme.secondary_style()),
            Span::styled(":power ", app.theme.muted_style()),
        ]
    } else {
        vec![]
    };

    let right_spans: Vec<Span> = std::iter::once(demo_indicator)
        .chain(hints.into_iter())
        .collect();
    let right = Line::from(right_spans);

    // Render left-aligned
    frame.render_widget(
        Paragraph::new(left).style(app.theme.style()),
        area,
    );

    // Render right-aligned
    let right_width = right.width() as u16;
    if area.width > right_width {
        let right_area = Rect {
            x: area.x + area.width - right_width - 1,
            y: area.y,
            width: right_width + 1,
            height: 1,
        };
        frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), right_area);
    }
}
