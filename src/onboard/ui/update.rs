use ratatui::{prelude::*, widgets::Paragraph};

use super::super::{ContentFocus, OnboardApp, PanelFocus, TaskState};
use crate::vim::VimMode;

pub fn draw_update_step(frame: &mut Frame, area: Rect, app: &mut OnboardApp) {
    if area.height < 8 {
        return;
    }

    let is_content_focused = app.panel_focus == PanelFocus::Content;
    let is_password_focused = is_content_focused && matches!(app.content_focus, ContentFocus::InputField(0));
    let is_insert = app.vim_mode == VimMode::Insert && is_password_focused;
    let mut y = area.y + 1;

    // Title
    frame.render_widget(
        Paragraph::new("Install Packages")
            .style(app.theme.primary_style().add_modifier(Modifier::BOLD)),
        Rect::new(area.x + 2, y, area.width - 4, 1),
    );
    y += 2;

    // Show running tasks if executing
    if !app.tasks.is_empty() {
        frame.render_widget(
            Paragraph::new("Installing:").style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );
        y += 1;

        for task in &app.tasks {
            if y >= area.y + area.height - 2 {
                break;
            }

            let (status_char, style) = match task.status {
                TaskState::Pending => (' ', app.theme.muted_style()),
                TaskState::Running => (app.spinner_char(), app.theme.primary_style()),
                TaskState::Success => ('x', app.theme.secondary_style()),
                TaskState::Failed => ('!', app.theme.error_style()),
            };

            // Show progress bar if available
            if let Some(progress) = task.progress {
                let bar_width = 20;
                let filled = (progress as usize * bar_width / 100).min(bar_width);
                let empty = bar_width - filled;
                let bar = format!("[{}{}]", "=".repeat(filled), " ".repeat(empty));
                let line = format!("  [{status_char}] {} {bar} {progress}%", task.name);
                frame.render_widget(
                    Paragraph::new(line).style(style),
                    Rect::new(area.x + 2, y, area.width - 4, 1),
                );
            } else {
                let line = format!("  [{status_char}] {}", task.name);
                frame.render_widget(
                    Paragraph::new(line).style(style),
                    Rect::new(area.x + 2, y, area.width - 4, 1),
                );
            }
            y += 1;
        }
    } else {
        // Show package selection list
        frame.render_widget(
            Paragraph::new("Select packages to install:")
                .style(app.theme.style()),
            Rect::new(area.x + 2, y, area.width - 4, 1),
        );
        y += 2;

        let max_y = area.y + area.height - 4;
        let num_categories = app.config.updates.len();

        if num_categories > 0 {
            // Divide available height equally among categories
            let available_height = max_y.saturating_sub(y) as usize;
            let per_category_height = if num_categories > 0 {
                available_height / num_categories
            } else {
                0
            };

            for (cat_idx, category) in app.config.updates.iter().enumerate() {
                let section_start = y;
                let section_end = if cat_idx == num_categories - 1 {
                    max_y
                } else {
                    (section_start + per_category_height as u16).min(max_y)
                };

                if section_start >= max_y {
                    break;
                }

                // Category header with aggregate checkbox
                let is_cat_cursor = is_content_focused
                    && cat_idx == app.update_category_cursor
                    && app.update_package_cursor.is_none();

                let checkbox = if app.is_category_fully_selected(cat_idx) {
                    "[x]"
                } else if app.is_category_partially_selected(cat_idx) {
                    "[~]"
                } else {
                    "[ ]"
                };

                let cursor = if is_cat_cursor { ">" } else { " " };

                let cat_style = if is_cat_cursor {
                    app.theme.primary_style().add_modifier(Modifier::BOLD)
                } else if app.is_category_any_selected(cat_idx) {
                    app.theme.secondary_style()
                } else {
                    app.theme.style()
                };

                frame.render_widget(
                    Paragraph::new(format!("{cursor} {checkbox} {}", category.name))
                        .style(cat_style),
                    Rect::new(area.x + 2, y, area.width - 4, 1),
                );
                y += 1;

                // Package list area starts after the header
                let pkg_area_start = y;
                let pkg_area_end = section_end;
                let pkg_area_lines = pkg_area_end.saturating_sub(pkg_area_start) as usize;

                // Each package takes 2 lines (title + description) or 1 if no description
                let lines_per_pkg: usize = 2;
                let visible_count = pkg_area_lines / lines_per_pkg;
                let total_packages = category.packages.len();

                // Adjust scroll to keep cursor visible for the active category
                let scroll = if cat_idx == app.update_category_cursor {
                    let current_scroll = app.update_category_scroll
                        .get(cat_idx).copied().unwrap_or(0);
                    if let Some(pkg_idx) = app.update_package_cursor {
                        // Ensure cursor is within visible window
                        if pkg_idx < current_scroll {
                            pkg_idx
                        } else if visible_count > 0 && pkg_idx >= current_scroll + visible_count {
                            pkg_idx - visible_count + 1
                        } else {
                            current_scroll
                        }
                    } else {
                        current_scroll
                    }
                } else {
                    app.update_category_scroll.get(cat_idx).copied().unwrap_or(0)
                };

                // Write back adjusted scroll
                if cat_idx < app.update_category_scroll.len() {
                    app.update_category_scroll[cat_idx] = scroll;
                }

                // Show scroll-up indicator
                if scroll > 0 && y < pkg_area_end {
                    frame.render_widget(
                        Paragraph::new("       \u{2191} more above")
                            .style(app.theme.muted_style()),
                        Rect::new(area.x + 2, y, area.width - 4, 1),
                    );
                    y += 1;
                }

                // Render visible packages
                let mut rendered = 0;
                for (pkg_idx, pkg) in category.packages.iter().enumerate() {
                    if pkg_idx < scroll {
                        continue;
                    }
                    if y >= pkg_area_end {
                        break;
                    }
                    // Reserve 1 line for "more below" indicator if not at end
                    let needs_more_indicator = pkg_idx + 1 < total_packages
                        && (y + lines_per_pkg as u16) >= pkg_area_end;
                    if needs_more_indicator && y + 1 >= pkg_area_end {
                        break;
                    }

                    let is_pkg_cursor = is_content_focused
                        && cat_idx == app.update_category_cursor
                        && app.update_package_cursor == Some(pkg_idx);

                    let pkg_selected = app.update_package_selected
                        .get(cat_idx)
                        .and_then(|pkgs| pkgs.get(pkg_idx))
                        .copied()
                        .unwrap_or(false);

                    let pkg_checkbox = if pkg.required || pkg_selected {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    let pkg_cursor_str = if is_pkg_cursor { ">" } else { " " };
                    let required_marker = if pkg.required { " *" } else { "" };

                    let pkg_style = if is_pkg_cursor {
                        app.theme.primary_style().add_modifier(Modifier::BOLD)
                    } else if pkg_selected {
                        app.theme.secondary_style()
                    } else {
                        app.theme.style()
                    };

                    frame.render_widget(
                        Paragraph::new(format!("  {pkg_cursor_str} {pkg_checkbox} {}{required_marker}", pkg.title))
                            .style(pkg_style),
                        Rect::new(area.x + 2, y, area.width - 4, 1),
                    );
                    y += 1;

                    // Package description
                    if !pkg.description.is_empty() && y < pkg_area_end {
                        frame.render_widget(
                            Paragraph::new(format!("           {}", pkg.description))
                                .style(app.theme.muted_style()),
                            Rect::new(area.x + 2, y, area.width - 4, 1),
                        );
                        y += 1;
                    }

                    rendered += 1;
                }

                // Show scroll-down indicator
                if scroll + rendered < total_packages && y < pkg_area_end {
                    frame.render_widget(
                        Paragraph::new(format!("       \u{2193} {} more below", total_packages - scroll - rendered))
                            .style(app.theme.muted_style()),
                        Rect::new(area.x + 2, y, area.width - 4, 1),
                    );
                }

                // Advance y to section end for consistent spacing
                y = section_end;
            }
        }

        // Show sudo password input if needed
        if app.sudo_password_needed && !app.sudo_password_entered && !app.is_dryrun() {
            if y < max_y {
                y += 1;
                frame.render_widget(
                    Paragraph::new("Password required for sudo commands:")
                        .style(app.theme.primary_style()),
                    Rect::new(area.x + 2, y, area.width - 4, 1),
                );
                y += 1;

                let label = "Password: ";
                frame.render_widget(
                    Paragraph::new(label).style(app.theme.style()),
                    Rect::new(area.x + 2, y, label.len() as u16, 1),
                );

                let field_x = area.x + 2 + label.len() as u16;
                let field_width = area.width.saturating_sub(label.len() as u16 + 4);
                let password_display = "*".repeat(app.sudo_password.content().len());

                if is_insert {
                    let cursor_pos = app.sudo_password.cursor().min(password_display.len());
                    let before: String = password_display.chars().take(cursor_pos).collect();
                    let after: String = password_display.chars().skip(cursor_pos).collect();

                    let line = Line::from(vec![
                        Span::styled(before, app.theme.style()),
                        Span::styled("|", app.theme.primary_style().add_modifier(Modifier::BOLD)),
                        Span::styled(after, app.theme.style()),
                    ]);
                    frame.render_widget(Paragraph::new(line), Rect::new(field_x, y, field_width, 1));
                } else if is_password_focused {
                    let chars: Vec<char> = password_display.chars().collect();
                    let cursor_pos = app.sudo_password.cursor().min(chars.len());
                    let mut spans = Vec::new();

                    for (i, ch) in chars.iter().enumerate() {
                        if i == cursor_pos {
                            spans.push(Span::styled(ch.to_string(), app.theme.style().add_modifier(Modifier::REVERSED)));
                        } else {
                            spans.push(Span::styled(ch.to_string(), app.theme.style()));
                        }
                    }
                    if cursor_pos >= chars.len() {
                        spans.push(Span::styled(" ", app.theme.style().add_modifier(Modifier::REVERSED)));
                    }

                    frame.render_widget(Paragraph::new(Line::from(spans)), Rect::new(field_x, y, field_width, 1));
                } else {
                    let display = if password_display.is_empty() { "(enter password)" } else { &password_display };
                    frame.render_widget(
                        Paragraph::new(display).style(app.theme.muted_style()),
                        Rect::new(field_x, y, field_width, 1),
                    );
                }
            }
        }
    }

    // Action button
    let can_run = !app.is_executing && app.tasks.is_empty();
    let needs_password = app.commands_need_sudo() && !app.sudo_password_entered && !app.is_dryrun();
    let any_selected = app.any_package_selected();

    if can_run {
        let button_y = area.y + area.height - 2;
        let button_text = if needs_password && app.sudo_password.content().is_empty() {
            " [i] Enter Password "
        } else if any_selected {
            " [Enter] Install Selected "
        } else {
            " [Enter] Skip & Continue "
        };
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
}
