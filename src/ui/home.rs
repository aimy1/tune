use crate::app::{App, HomeSidebarHit, HomeSidebarSection};
use crate::data::config::Language;
use crate::ui::page_lyrics;
use crate::ui::player_bar;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn draw_home(frame: &mut Frame, app: &mut App) {
    app.clear_player_bar_hits();
    app.clear_content_hits();

    let size = frame.area();
    frame.render_widget(Block::default().style(base_bg_style(app)), size);

    if size.width < 32 || size.height < 12 {
        frame.render_widget(
            Paragraph::new(match app.config.language {
                Language::Zh => "终端窗口过小",
                Language::En => "Terminal too small",
            })
            .style(Style::default().fg(app.theme.color_subtext())),
            size,
        );
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(player_bar::PLAYER_BAR_HEIGHT),
        ])
        .split(size);

    let (content_area, hint_area) = if app.config.show_hints {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(rows[0]);
        (split[0], split[1])
    } else {
        (rows[0], Rect::default())
    };

    let content_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(content_area);
    let header_area = content_split[0];
    let body_area = content_split[1];

    crate::ui::draw_header_bar(frame, app, header_area);

    draw_tiles(frame, app, body_area);
    if app.config.page_lyrics {
        let panel_area = page_lyrics::overlay_panel_area(content_area);
        page_lyrics::draw_page_lyrics_panel(frame, app, panel_area);
    }
    if app.config.show_hints {
        draw_home_hint(frame, app, hint_area);
    }
    if app.home_sidebar.is_visible() {
        draw_home_sidebar(frame, app, rows[0]);
    }

    player_bar::draw_collapsed_player_bar(frame, app, rows[1]);
}

fn draw_tiles(frame: &mut Frame, app: &mut App, area: Rect) {
    let margin = ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    };
    let inner = area.inner(margin);
    if inner.width < 14 || inner.height < 8 {
        return;
    }

    let tile_h = 12_u16.min(inner.height.saturating_sub(1)).max(6);
    let tile_w = tile_h.saturating_mul(2).saturating_add(4);
    let col_step = tile_w.saturating_add(2);
    let row_step = tile_h.saturating_add(1);
    let columns = usize::from((inner.width / col_step).max(1));
    app.home.set_columns(columns);

    let visible_rows = usize::from((inner.height / row_step).max(1));
    app.home.set_visible_rows(visible_rows);
    let row_offset = app.home.effective_scroll_row_offset();

    for index in 0..app.home.tiles.len() {
        let virtual_index = home_real_to_virtual_index(index, columns);
        let row = virtual_index / columns;
        if row < row_offset {
            continue;
        }
        let visual_row = row - row_offset;
        if visual_row >= visible_rows {
            continue;
        }
        let col = virtual_index % columns;
        let x = inner.x + (col as u16) * col_step;
        let y = inner.y + (visual_row as u16) * row_step;
        if x >= inner.x + inner.width || y >= inner.y + inner.height {
            continue;
        }

        let rect = Rect {
            x,
            y,
            width: tile_w.min(inner.x + inner.width - x),
            height: tile_h.min(inner.y + inner.height - y),
        };

        app.push_home_tile_hit(
            crate::app::HitRect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            },
            index,
        );

        let focused = index == app.home.focused_idx;
        let tile_bg = if focused {
            app.theme.color_surface()
        } else {
            app.theme.color_base()
        };
        let tile_style = if app.config.transparent_background {
            Style::default()
        } else {
            Style::default().bg(tile_bg)
        };

        let border_style = if focused {
            Style::default()
                .fg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.color_buff())
        };

        let border_type = if focused {
            ratatui::widgets::BorderType::Double
        } else {
            ratatui::widgets::BorderType::Rounded
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style)
            .style(tile_style);

        if focused {
            let focus_label = match app.config.language {
                Language::Zh => "选中 ",
                Language::En => "Focus ",
            };
            block = block.title(Line::from(vec![
                Span::styled(
                    " 󰐊 ",
                    Style::default()
                        .fg(app.theme.color_accent2())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    focus_label,
                    Style::default()
                        .fg(app.theme.color_accent())
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        frame.render_widget(block, rect);

        let inner_rect = rect.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner_rect.width < 2 || inner_rect.height < 2 {
            continue;
        }

        let text_rows = if inner_rect.height >= 4 { 2 } else { 1 };
        let cover_height = inner_rect.height.saturating_sub(text_rows);
        let cover_rect = Rect {
            x: inner_rect.x,
            y: inner_rect.y,
            width: inner_rect.width,
            height: cover_height,
        };
        let text_rect = Rect {
            x: inner_rect.x,
            y: inner_rect.y + cover_height,
            width: inner_rect.width,
            height: text_rows,
        };

        if !cover_rect.is_empty() {
            let draw_ascii = app.draw_ascii();
            let text_style = if focused {
                Style::default().fg(app.theme.color_accent2())
            } else {
                Style::default().fg(app.theme.color_text())
            };
            app.home.tiles[index].cover.render(
                frame,
                &mut app.graphics_picker,
                cover_rect,
                text_style,
                None,
                draw_ascii,
            );
        }

        let (title, subtitle) = {
            let tile = &app.home.tiles[index];
            (tile.title.clone(), tile.subtitle.clone())
        };

        let title_style = if focused {
            Style::default()
                .fg(app.theme.color_accent2())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.color_text())
        };
        let subtitle_style = Style::default().fg(app.theme.color_subtext());

        let mut lines = vec![Line::from(Span::styled(title, title_style))];
        if text_rows > 1 {
            lines.push(Line::from(Span::styled(subtitle, subtitle_style)));
        }

        let content = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        frame.render_widget(content, text_rect);
    }
}

fn home_real_to_virtual_index(index: usize, columns: usize) -> usize {
    let cols = columns.max(1);
    if cols <= 3 || index < 3 {
        index
    } else {
        index.saturating_add(cols - 3)
    }
}

fn draw_home_hint(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let text = match app.config.language {
        Language::Zh => format!(
            "{} 搜索  {} 设置  {} 侧边栏  {} 全屏  {} 退出",
            app.config.keybind_search_box,
            app.config.keybind_settings,
            app.config.keybind_sidebar,
            app.config.keybind_fullscreen,
            app.config.keybind_quit
        ),
        Language::En => format!(
            "{} Search  {} Settings  {} Sidebar  {} Fullscreen  {} Quit",
            app.config.keybind_search_box,
            app.config.keybind_settings,
            app.config.keybind_sidebar,
            app.config.keybind_fullscreen,
            app.config.keybind_quit
        ),
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" 󰌌 ", Style::default().fg(app.theme.color_buff())),
            Span::styled(text, Style::default().fg(app.theme.color_subtext())),
        ]))
        .alignment(Alignment::Left),
        area,
    );
}

fn draw_home_sidebar(frame: &mut Frame, app: &mut App, area: Rect) {
    if area.width < 20 || area.height < 8 {
        return;
    }

    let max_width = (area.width * 45 / 100).max(36).min(area.width).min(56);
    app.set_home_sidebar_anim_span_cells(max_width);
    let progress = app.home_sidebar.anim_progress.clamp(0.0, 1.0);
    // EaseOutQuad: f(t) = 1 - (1 - t)^2
    let eased_progress = 1.0 - (1.0 - progress) * (1.0 - progress);
    let width = ((max_width as f32) * eased_progress).round() as u16;
    if width < 12 {
        return;
    }

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let height = ((area.height * 85) / 100).max(12).min(area.height);
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    let sidebar = Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, sidebar);

    app.set_home_sidebar_panel_hit(Some(crate::app::HitRect {
        x: sidebar.x,
        y: sidebar.y,
        width: sidebar.width,
        height: sidebar.height,
    }));

    let title = match app.config.language {
        Language::Zh => " 󰀄 个人中心 ",
        Language::En => " 󰀄 Personal Center ",
    };

    let panel_style = if app.config.transparent_sidebar {
        Style::default().fg(app.theme.color_subtext())
    } else {
        Style::default()
            .fg(app.theme.color_subtext())
            .bg(app.theme.color_surface())
    };

    let border_style = if app.home_sidebar.expanded {
        Style::default()
            .fg(app.theme.color_accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(app.theme.color_buff())
    };

    let border_type = if app.home_sidebar.expanded {
        ratatui::widgets::BorderType::Double
    } else {
        ratatui::widgets::BorderType::Rounded
    };

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .title(Line::from(Span::styled(
                title,
                Style::default()
                    .fg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD),
            )))
            .border_style(border_style)
            .style(panel_style),
        sidebar,
    );

    let inner = sidebar.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });
    if inner.width < 8 || inner.height < 5 {
        return;
    }

    let header_height = if inner.height >= 8 { 2 } else { 1 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Length(1), // Divider
            Constraint::Min(1),    // Playlists
        ])
        .split(inner);

    let user_name = if app.home_sidebar.user_name.trim().is_empty() {
        match app.config.language {
            Language::Zh => "未登录用户".to_string(),
            Language::En => "Guest User".to_string(),
        }
    } else {
        app.home_sidebar.user_name.clone()
    };

    let status = if app.home_sidebar.loading {
        match app.config.language {
            Language::Zh => "正在同步歌单...".to_string(),
            Language::En => "Syncing playlists...".to_string(),
        }
    } else if app.home_sidebar.status_line.trim().is_empty() {
        match app.config.language {
            Language::Zh => "󰌌 ⇅/⇄ 切换分区  ↵ 进入  Esc 关闭".to_string(),
            Language::En => "󰌌 ⇅/⇄ Switch  ↵ Open  Esc Close".to_string(),
        }
    } else {
        app.home_sidebar.status_line.clone()
    };

    let mut header_lines = vec![Line::from(vec![
        Span::styled(
            "󰀄 ",
            Style::default().fg(app.theme.color_accent2()),
        ),
        Span::styled(
            user_name,
            Style::default()
                .fg(app.theme.color_text())
                .add_modifier(Modifier::BOLD),
        ),
    ])];
    if header_height > 1 {
        header_lines.push(Line::from(Span::styled(
            status,
            Style::default().fg(app.theme.color_subtext()),
        )));
    }
    frame.render_widget(
        Paragraph::new(header_lines)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center),
        chunks[0],
    );

    // Draw horizontal divider line
    let divider_line = "─".repeat(usize::from(inner.width));
    frame.render_widget(
        Paragraph::new(divider_line).style(Style::default().fg(app.theme.color_buff())),
        chunks[1],
    );

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let created_items = app.home_sidebar.created_playlists.clone();
    let collected_items = app.home_sidebar.collected_playlists.clone();

    draw_home_sidebar_section(
        frame,
        app,
        sections[0],
        match app.config.language {
            Language::Zh => "用户创建的歌单",
            Language::En => "Created Playlists",
        },
        &created_items,
        HomeSidebarSection::Created,
    );

    draw_home_sidebar_section(
        frame,
        app,
        sections[1],
        match app.config.language {
            Language::Zh => "用户收藏的歌单",
            Language::En => "Collected Playlists",
        },
        &collected_items,
        HomeSidebarSection::Collected,
    );
}

fn draw_home_sidebar_section(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    title: &str,
    items: &[crate::app::HomeSidebarPlaylist],
    section: HomeSidebarSection,
) {
    if area.width < 6 || area.height < 3 {
        return;
    }

    let section_focused = app.home_sidebar.expanded && app.home_sidebar.focused_section == section;
    let section_title_style = if section_focused {
        Style::default()
            .fg(app.theme.color_accent2())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(app.theme.color_subtext())
    };

    let bg_style = if app.config.transparent_sidebar {
        Style::default()
    } else {
        Style::default().bg(app.theme.color_surface())
    };

    frame.render_widget(
        Block::default().style(bg_style),
        area,
    );

    let title_line_area = Rect {
        x: area.x.saturating_add(1),
        y: area.y,
        width: area.width.saturating_sub(2),
        height: 1,
    };
    if title_line_area.width > 2 {
        let title_icon = match section {
            HomeSidebarSection::Created => " 󰓏 ",
            HomeSidebarSection::Collected => " 󱉼 ",
        };
        let line_width = usize::from(title_line_area.width);
        let title_max = line_width.saturating_sub(6);
        let clipped_title = clip_to_display_width(title, title_max.max(1));
        
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(title_icon, section_title_style),
                Span::styled(clipped_title, section_title_style),
            ]))
            .style(bg_style),
            title_line_area,
        );
    }

    let inner = Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(1),
    };
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut lines = Vec::new();
    if items.is_empty() {
        lines.push(Line::from(Span::styled(
            match app.config.language {
                Language::Zh => "暂无歌单",
                Language::En => "No playlists",
            },
            Style::default().fg(app.theme.color_subtext()),
        )));
    } else {
        let max_rows = inner.height as usize;
        if max_rows == 0 {
            return;
        }
        let total = items.len();
        let focus_idx = if section_focused {
            app.home_sidebar.focused_index.min(total.saturating_sub(1))
        } else {
            0
        };
        let mut start = if total <= max_rows {
            0
        } else {
            app.home_sidebar
                .section_scroll_offset(section)
                .min(total.saturating_sub(max_rows))
        };

        if total > max_rows {
            if focus_idx < start {
                start = focus_idx;
            } else if focus_idx >= start.saturating_add(max_rows) {
                start = focus_idx + 1 - max_rows;
            }
            start = start.min(total.saturating_sub(max_rows));
        }
        app.home_sidebar.set_section_scroll_offset(section, start);

        for (visual_idx, item) in items.iter().skip(start).take(max_rows).enumerate() {
            let idx = start + visual_idx;
            let is_focused = section_focused && idx == app.home_sidebar.focused_index;
            
            let is_liked = app.home_sidebar.liked_playlist_id.as_ref() == item.id.as_ref();
            let icon = if is_liked {
                " "
            } else {
                match section {
                    HomeSidebarSection::Created => "󰓏 ",
                    HomeSidebarSection::Collected => "󱉼 ",
                }
            };
            let prefix = if is_focused {
                format!("▌ {}", icon)
            } else {
                format!("  {}", icon)
            };

            let title_text = if item.creator.trim().is_empty() {
                item.title.clone()
            } else {
                format!("{} - {}", item.title, item.creator)
            };
            let left = format!("{}{}", prefix, title_text);

            let right = match app.config.language {
                Language::Zh => format!("{}首", item.track_count),
                Language::En => format!("{}", item.track_count),
            };

            let reserved = display_width(&right) + 1;
            let left_max = usize::from(inner.width).saturating_sub(reserved);
            let clipped_left = clip_to_display_width(&left, left_max);
            let used = display_width(&clipped_left) + display_width(&right);
            let spaces = usize::from(inner.width).saturating_sub(used).max(1);

            app.push_home_sidebar_playlist_hit(
                crate::app::HitRect {
                    x: inner.x,
                    y: inner.y + visual_idx as u16,
                    width: inner.width,
                    height: 1,
                },
                HomeSidebarHit {
                    section,
                    index: idx,
                },
            );

            let row_bg = if is_focused {
                Some(app.theme.color_buff())
            } else if app.config.transparent_sidebar {
                None
            } else {
                Some(app.theme.color_surface())
            };

            let text_style = {
                let mut s = if is_focused {
                    Style::default()
                        .fg(app.theme.color_accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(app.theme.color_text())
                };
                if let Some(bg) = row_bg {
                    s = s.bg(bg);
                }
                s
            };
            let right_style = {
                let mut s = if is_focused {
                    Style::default()
                        .fg(app.theme.color_accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(app.theme.color_subtext())
                };
                if let Some(bg) = row_bg {
                    s = s.bg(bg);
                }
                s
            };

            lines.push(Line::from(vec![
                Span::styled(clipped_left, text_style),
                Span::styled(" ".repeat(spaces), text_style),
                Span::styled(right, right_style),
            ]));
        }
    }

    frame.render_widget(
        Paragraph::new(lines).style(bg_style),
        inner,
    );
}

fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn clip_to_display_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > max_width {
            break;
        }
        out.push(ch);
        used += w;
    }
    out
}

fn base_bg_style(app: &App) -> Style {
    if app.config.transparent_background {
        Style::default()
    } else {
        Style::default().bg(app.theme.color_base())
    }
}
