use crate::app::{App, SearchItem};
use crate::data::config::Language;
use crate::ui::page_lyrics;
use crate::ui::player_bar;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn draw_search(frame: &mut Frame, app: &mut App) {
    app.clear_player_bar_hits();
    app.clear_content_hits();

    let size = frame.area();
    frame.render_widget(Block::default().style(base_bg_style(app)), size);

    if size.width < 42 || size.height < 14 {
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

    let content_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(rows[0]);
    let header_area = content_split[0];
    let body_area = content_split[1];

    crate::ui::draw_header_bar(frame, app, header_area);

    draw_result_panel(frame, app, body_area);
    if app.config.page_lyrics {
        let panel_area = page_lyrics::overlay_panel_area(body_area);
        page_lyrics::draw_page_lyrics_panel(frame, app, panel_area);
    }

    player_bar::draw_collapsed_player_bar(frame, app, rows[1]);
}

fn draw_result_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if !app.home_sidebar.expanded {
        Style::default()
            .fg(app.theme.color_accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(app.theme.color_buff())
    };

    frame.render_widget(
        Block::default()
            .borders(Borders::TOP)
            .title(match app.config.language {
                Language::Zh => " 搜索结果 ",
                Language::En => " Search Results ",
            })
            .border_style(border_style),
        area,
    );

    let inner = Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(1),
    };

    if inner.width < 10 || inner.height < 2 {
        return;
    }

    let list_height = if app.config.show_hints {
        inner.height.saturating_sub(1)
    } else {
        inner.height
    };
    if list_height == 0 {
        return;
    }

    let list_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: list_height,
    };
    let hint_rect = Rect {
        x: inner.x,
        y: inner.y + list_height,
        width: inner.width,
        height: 1,
    };

    let visible = list_area.height as usize;
    app.search.set_visible_rows(visible);
    let offset = app.search.effective_scroll_offset();

    for (line_idx, item_idx) in (offset..app.search.results.len()).take(visible).enumerate() {
        let row_y = list_area.y + line_idx as u16;
        let row = Rect {
            x: list_area.x,
            y: row_y,
            width: list_area.width,
            height: 1,
        };

        app.push_search_item_hit(
            crate::app::HitRect {
                x: row.x,
                y: row.y,
                width: row.width,
                height: row.height,
            },
            item_idx,
        );

        let item = &app.search.results[item_idx];
        let focused = item_idx == app.search.focused_idx;
        render_search_row(frame, app, row, item_idx, item, focused);
    }

    if app.config.show_hints && list_height < inner.height {
        let hint = match app.config.language {
            Language::Zh => {
                "Enter 打开/播放  Esc 返回  后缀: @single 单曲 | @album 专辑 | @list 歌单 | 仅 @author: 关注作者"
            }
            Language::En => {
                "Enter open/play  Esc back  Suffix: @single | @album | @list | only @author: followed authors"
            }
        };
        frame.render_widget(
            Paragraph::new(hint).style(Style::default().fg(app.theme.color_subtext())),
            hint_rect,
        );
    }
}

fn render_search_row(
    frame: &mut Frame,
    app: &App,
    row: Rect,
    item_idx: usize,
    item: &SearchItem,
    focused: bool,
) {
    let is_now_playing = app.is_now_playing_song(item.song_id.as_deref());
    let zebra_bg = if app.config.transparent_background {
        None
    } else if item_idx % 2 == 0 {
        Some(app.theme.color_base())
    } else {
        Some(app.theme.color_surface())
    };

    let row_bg = if focused {
        Some(app.theme.color_buff())
    } else {
        zebra_bg
    };

    let bg_style = |s: Style| {
        if app.config.transparent_background {
            s
        } else if let Some(bg) = row_bg {
            s.bg(bg)
        } else {
            s
        }
    };

    let prefix_style = if focused {
        Style::default().fg(app.theme.color_accent())
    } else {
        Style::default().fg(app.theme.color_surface())
    };

    let index_style = if focused {
        Style::default().fg(app.theme.color_accent()).add_modifier(Modifier::BOLD)
    } else if is_now_playing {
        Style::default().fg(app.theme.color_accent3()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(app.theme.color_subtext())
    };

    let index_label = if is_now_playing {
        " 󰎆".to_string()
    } else {
        format!(" {:>2}", item_idx + 1)
    };

    let right = item
        .type_tag
        .clone()
        .filter(|tag| !tag.trim().is_empty())
        .unwrap_or_else(|| item.right_label.clone());

    let right_style = if focused {
        Style::default().fg(app.theme.color_accent())
    } else if is_now_playing {
        Style::default().fg(app.theme.color_accent3())
    } else {
        Style::default().fg(app.theme.color_subtext())
    };

    let left_style = if focused {
        Style::default().fg(app.theme.color_accent()).add_modifier(Modifier::BOLD)
    } else if is_now_playing {
        Style::default().fg(app.theme.color_accent3()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(app.theme.color_text())
    };

    let prefix_width = 1;
    let index_width = 5;
    let right_w = display_width(&right);
    let right_col_width = right_w + 2;

    let rem_w = usize::from(row.width)
        .saturating_sub(prefix_width + index_width + right_col_width);

    // Pad index column
    let idx_w = display_width(&index_label);
    let idx_pad = index_width.saturating_sub(idx_w);
    let padded_idx = format!("{}{}", index_label, " ".repeat(idx_pad));

    let mut spans = Vec::new();
    spans.push(Span::styled(if focused { "▌" } else { " " }, bg_style(prefix_style)));
    spans.push(Span::styled(padded_idx, bg_style(index_style)));

    if let (Some(title), Some(artist)) = (&item.title, &item.artist) {
        let title_max = (rem_w * 60) / 100;
        let artist_max = rem_w.saturating_sub(title_max);

        // Title Column
        let clipped_title = clip_to_display_width(title, title_max);
        let title_len = display_width(&clipped_title);
        let title_pad = title_max.saturating_sub(title_len);
        let title_col = format!("{}{}", clipped_title, " ".repeat(title_pad));

        // Artist Column
        let clipped_artist = clip_to_display_width(artist, artist_max);
        let artist_len = display_width(&clipped_artist);
        let artist_pad = artist_max.saturating_sub(artist_len);
        let artist_col = format!("{}{}", clipped_artist, " ".repeat(artist_pad));

        spans.push(Span::styled(title_col, bg_style(left_style)));
        spans.push(Span::styled(artist_col, bg_style(right_style)));
    } else {
        let left_max = rem_w;
        let clipped_left = clip_to_display_width(&item.left_label, left_max);
        let left_len = display_width(&clipped_left);
        let left_pad = left_max.saturating_sub(left_len);
        let left_col = format!("{}{}", clipped_left, " ".repeat(left_pad));

        spans.push(Span::styled(left_col, bg_style(left_style)));
    }

    // Right Column
    let right_pad = right_col_width.saturating_sub(right_w);
    let right_col = format!("{}{}", " ".repeat(right_pad), right);
    spans.push(Span::styled(right_col, bg_style(right_style)));

    frame.render_widget(
        Paragraph::new(Line::from(spans)),
        row,
    );
}

fn base_bg_style(app: &App) -> Style {
    if app.config.transparent_background {
        Style::default()
    } else {
        Style::default().bg(app.theme.color_base())
    }
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
