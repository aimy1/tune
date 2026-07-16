use crate::app::App;
use crate::data::config::Language;
use crate::tmplayer::data::about::{BrailleImage, about_info};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Redesigned About overlay: card layout with art panel, identity header,
/// structured description / links / tech stack, and a soft footer.
pub fn draw_about_modal(frame: &mut Frame, app: &App, size: Rect) {
    if size.width < 36 || size.height < 12 {
        draw_compact(frame, app, size);
        return;
    }

    let area = modal_area(size);
    frame.render_widget(Clear, area);

    let title = match app.config.language {
        Language::Zh => " 󰎆  关于 ",
        Language::En => " 󰎆  About ",
    };

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(title)
            .title_bottom(Line::from(vec![
                Span::styled(
                    " Esc ",
                    Style::default()
                        .fg(app.theme.color_base())
                        .bg(app.theme.color_buff())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    match app.config.language {
                        Language::Zh => " 返回 ",
                        Language::En => " Back ",
                    },
                    Style::default().fg(app.theme.color_subtext()),
                ),
            ]))
            .border_style(
                Style::default()
                    .fg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD),
            )
            .style(surface_style(app)),
        area,
    );

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });
    if inner.width < 20 || inner.height < 6 {
        return;
    }

    // Compact header · logo-first body · footer
    let header_h = if inner.height >= 22 { 2 } else { 1 };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_h),
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(inner);

    draw_identity_header(frame, app, rows[0]);
    draw_body(frame, app, rows[1]);
    draw_footer(frame, app, rows[2]);
}

fn draw_compact(frame: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(
        size.width.saturating_sub(2).max(20),
        size.height.saturating_sub(2).max(8),
        size,
    );
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(match app.config.language {
                Language::Zh => " 关于 ",
                Language::En => " About ",
            })
            .border_style(Style::default().fg(app.theme.color_accent()))
            .style(surface_style(app)),
        area,
    );
    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    let info = about_info();
    let text = format!("Tune  v{}\n{}", info.version, info.description);
    frame.render_widget(
        Paragraph::new(text)
            .style(
                Style::default()
                    .fg(app.theme.color_text())
                    .bg(app.theme.color_surface()),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_identity_header(frame: &mut Frame, app: &App, area: Rect) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let info = about_info();
    let name_style = Style::default()
        .fg(app.theme.color_accent())
        .bg(app.theme.color_surface())
        .add_modifier(Modifier::BOLD);
    let badge_style = Style::default()
        .fg(app.theme.color_base())
        .bg(app.theme.color_accent2())
        .add_modifier(Modifier::BOLD);
    let tagline_style = Style::default()
        .fg(app.theme.color_subtext())
        .bg(app.theme.color_surface());

    let tagline = match app.config.language {
        Language::Zh => "终端网易云 · 用键盘听歌",
        Language::En => "NetEase Cloud Music · TUI player",
    };

    // Single-line: name · version · tagline (leave vertical space for logo)
    let top = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("󰎆  Tune", name_style),
            Span::styled("  ", Style::default().bg(app.theme.color_surface())),
            Span::styled(format!(" v{} ", info.version), badge_style),
            Span::styled("  ·  ", Style::default().fg(app.theme.color_buff()).bg(app.theme.color_surface())),
            Span::styled(tagline, tagline_style),
        ])),
        top,
    );

    if area.height >= 2 {
        frame.render_widget(
            Paragraph::new("─".repeat(area.width as usize)).style(
                Style::default()
                    .fg(app.theme.color_buff())
                    .bg(app.theme.color_surface()),
            ),
            Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: 1,
            },
        );
    }
}

fn draw_body(frame: &mut Frame, app: &App, area: Rect) {
    if area.width < 20 || area.height < 3 {
        draw_text_column(frame, app, area);
        return;
    }

    let logo = preferred_logo_size();
    // Give the logo as much room as possible: top-centered full-width panel.
    let show_art = area.height >= 10 && area.width >= 28;
    if !show_art {
        draw_text_column(frame, app, area);
        return;
    }

    // Reserve a compact info strip under the logo when height allows.
    let text_min = 6u16;
    let art_h = if area.height > text_min + 8 {
        area.height.saturating_sub(text_min)
    } else {
        // Short terminal: almost all body for logo, skip text strip.
        area.height
    };

    // Prefer height needed by logo, but never exceed available.
    let target_art_h = logo
        .map(|(_, h)| (h as u16).saturating_add(2))
        .unwrap_or(art_h)
        .min(art_h)
        .max(8);

    let use_text_below = area.height.saturating_sub(target_art_h) >= text_min;
    if use_text_below {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(target_art_h),
                Constraint::Min(text_min),
            ])
            .split(area);
        draw_art_panel(frame, app, rows[0]);
        draw_text_column(
            frame,
            app,
            rows[1].inner(ratatui::layout::Margin {
                horizontal: 0,
                vertical: 0,
            }),
        );
    } else {
        draw_art_panel(frame, app, area);
    }
}

fn preferred_logo_size() -> Option<(usize, usize)> {
    let info = about_info();
    info.braille_images
        .iter()
        .filter(|a| a.width > 0 && a.height > 0)
        .max_by_key(|a| (a.width as u128) * (a.height as u128))
        .map(|a| (a.width, a.height))
}

fn draw_art_panel(frame: &mut Frame, app: &App, area: Rect) {
    if area.width < 8 || area.height < 4 {
        return;
    }

    // No inner border box — logo is the visual; keep outer modal frame only.
    let lines = about_logo_lines(area.width as usize, area.height as usize);
    frame.render_widget(
        Paragraph::new(lines)
            .style(
                Style::default()
                    .fg(app.theme.color_accent())
                    .bg(app.theme.color_surface()),
            )
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_text_column(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let info = about_info();
    let max_w = area.width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();

    // —— Description ——
    lines.push(section_title(
        app,
        "󰈙",
        match app.config.language {
            Language::Zh => "简介",
            Language::En => "About",
        },
    ));

    let description = if info.description.trim().is_empty() {
        match app.config.language {
            Language::Zh => "Tune：终端里的网易云音乐客户端。".to_string(),
            Language::En => "Tune: a NetEase Cloud Music client for the terminal.".to_string(),
        }
    } else {
        info.description.clone()
    };
    for row in wrap_display_width(&description, max_w.saturating_sub(2)) {
        lines.push(Line::from(Span::styled(
            format!("  {row}"),
            Style::default()
                .fg(app.theme.color_text())
                .bg(app.theme.color_surface()),
        )));
    }

    lines.push(blank_line(app));

    // —— Links ——
    lines.push(section_title(
        app,
        "󰌹",
        match app.config.language {
            Language::Zh => "链接",
            Language::En => "Links",
        },
    ));

    if info.links.is_empty() {
        lines.push(Line::from(Span::styled(
            match app.config.language {
                Language::Zh => "  暂无链接",
                Language::En => "  No links",
            },
            Style::default()
                .fg(app.theme.color_subtext())
                .bg(app.theme.color_surface()),
        )));
    } else {
        for (key, value) in &info.links {
            let (icon, label) = link_meta(app, key);
            // Label pill + value
            let label_text = format!(" {label} ");
            let label_w = UnicodeWidthStr::width(label_text.as_str());
            let icon_w = UnicodeWidthStr::width(icon) + 1; // icon + space
            let value_budget = max_w
                .saturating_sub(2 + icon_w + label_w + 1)
                .max(8);
            let clipped = clip_to_display_width(value, value_budget);

            lines.push(Line::from(vec![
                Span::styled(
                    "  ",
                    Style::default().bg(app.theme.color_surface()),
                ),
                Span::styled(
                    format!("{icon} "),
                    Style::default()
                        .fg(app.theme.color_accent2())
                        .bg(app.theme.color_surface()),
                ),
                Span::styled(
                    label_text,
                    Style::default()
                        .fg(app.theme.color_base())
                        .bg(app.theme.color_buff())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " ",
                    Style::default().bg(app.theme.color_surface()),
                ),
                Span::styled(
                    clipped,
                    Style::default()
                        .fg(app.theme.color_text())
                        .bg(app.theme.color_surface()),
                ),
            ]));
        }
    }

    lines.push(blank_line(app));

    // —— Stack ——
    lines.push(section_title(
        app,
        "󰏖",
        match app.config.language {
            Language::Zh => "技术栈",
            Language::En => "Stack",
        },
    ));

    let chips = ["Rust", "ratatui", "ncm-api", "rodio"];
    let mut chip_spans = vec![Span::styled(
        "  ",
        Style::default().bg(app.theme.color_surface()),
    )];
    for (i, chip) in chips.iter().enumerate() {
        if i > 0 {
            chip_spans.push(Span::styled(
                " ",
                Style::default().bg(app.theme.color_surface()),
            ));
        }
        chip_spans.push(Span::styled(
            format!(" {chip} "),
            Style::default()
                .fg(app.theme.color_accent())
                .bg(app.theme.color_buff()),
        ));
    }
    lines.push(Line::from(chip_spans));

    // License line when space allows
    if area.height as usize > lines.len() + 2 {
        lines.push(blank_line(app));
        lines.push(Line::from(vec![
            Span::styled(
                "  󰿃 ",
                Style::default()
                    .fg(app.theme.color_subtext())
                    .bg(app.theme.color_surface()),
            ),
            Span::styled(
                match app.config.language {
                    Language::Zh => "协议 AGPL-3.0",
                    Language::En => "License AGPL-3.0",
                },
                Style::default()
                    .fg(app.theme.color_subtext())
                    .bg(app.theme.color_surface()),
            ),
        ]));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(app.theme.color_surface()))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let text = match app.config.language {
        Language::Zh => "感谢开源社区 · 欢迎 Star / Issue",
        Language::En => "Built with open source · Stars & issues welcome",
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "󰓎 ",
                Style::default()
                    .fg(app.theme.color_accent3())
                    .bg(app.theme.color_surface()),
            ),
            Span::styled(
                text,
                Style::default()
                    .fg(app.theme.color_subtext())
                    .bg(app.theme.color_surface()),
            ),
        ]))
        .alignment(Alignment::Center),
        area,
    );
}

fn section_title(app: &App, icon: &str, title: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{icon} "),
            Style::default()
                .fg(app.theme.color_accent2())
                .bg(app.theme.color_surface())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            title.to_string(),
            Style::default()
                .fg(app.theme.color_accent2())
                .bg(app.theme.color_surface())
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn blank_line(app: &App) -> Line<'static> {
    Line::from(Span::styled(
        " ",
        Style::default().bg(app.theme.color_surface()),
    ))
}

fn link_meta(app: &App, key: &str) -> (&'static str, String) {
    let k = key.to_lowercase();
    if k.contains("github") {
        return (
            "󰊤",
            match app.config.language {
                Language::Zh => "GitHub".to_string(),
                Language::En => "GitHub".to_string(),
            },
        );
    }
    if k.contains("home") || k.contains("website") || k.contains("url") {
        return (
            "󰖟",
            match app.config.language {
                Language::Zh => "主页".to_string(),
                Language::En => "Home".to_string(),
            },
        );
    }
    if k.contains("issue") {
        return (
            "󰋼",
            match app.config.language {
                Language::Zh => "反馈".to_string(),
                Language::En => "Issues".to_string(),
            },
        );
    }
    ("󰌹", key.to_string())
}

fn surface_style(app: &App) -> Style {
    Style::default()
        .fg(app.theme.color_subtext())
        .bg(app.theme.color_surface())
}

fn modal_area(size: Rect) -> Rect {
    // Wide card so the 60-col logo can breathe.
    let logo = preferred_logo_size();
    let want_w = logo.map(|(w, _)| (w as u16).saturating_add(8)).unwrap_or(72);
    let want_h = logo.map(|(_, h)| (h as u16).saturating_add(12)).unwrap_or(34);

    let max_w = size.width.saturating_sub(2);
    let max_h = size.height.saturating_sub(1);
    let w = want_w.min(max_w).max(36.min(max_w));
    let h = want_h.min(max_h).max(14.min(max_h));
    centered_rect(w, h, size)
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect {
        x: area.x + area.width.saturating_sub(w) / 2,
        y: area.y + area.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    }
}

fn wrap_display_width(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return Vec::new();
    }
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut out = Vec::new();
    let mut buf = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        if ch == '\n' {
            out.push(std::mem::take(&mut buf));
            used = 0;
            continue;
        }
        let w = ch.width().unwrap_or(0);
        if used + w > max_width && !buf.is_empty() {
            out.push(std::mem::take(&mut buf));
            used = 0;
        }
        buf.push(ch);
        used += w;
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
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

/// Center the logo in the panel; if the panel is smaller, crop from the center.
fn about_logo_lines(width: usize, height: usize) -> Vec<Line<'static>> {
    let blank = " ".repeat(width);
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let info = about_info();
    let Some(selected) = select_logo_art(width, height, &info.braille_images) else {
        return (0..height).map(|_| Line::from(blank.clone())).collect();
    };

    let mut rows: Vec<String> = selected
        .art
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();

    let mut start = 0usize;
    let mut end = rows.len();
    while start < end && rows[start].trim().is_empty() {
        start += 1;
    }
    while end > start && rows[end - 1].trim().is_empty() {
        end -= 1;
    }
    rows = rows[start..end].to_vec();

    let art_h = rows.len();
    let art_w = rows
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    // Center or center-crop.
    let (src_y0, dst_y0, copy_h) = if art_h <= height {
        (0, (height - art_h) / 2, art_h)
    } else {
        ((art_h - height) / 2, 0, height)
    };
    let (src_x0, dst_x0, copy_w) = if art_w <= width {
        (0, (width - art_w) / 2, art_w)
    } else {
        ((art_w - width) / 2, 0, width)
    };

    let mut grid: Vec<Vec<char>> = vec![vec![' '; width]; height];
    for row_i in 0..copy_h {
        let src_row = &rows[src_y0 + row_i];
        let src_chars: Vec<char> = src_row.chars().collect();
        let gy = dst_y0 + row_i;
        for col_i in 0..copy_w {
            let sx = src_x0 + col_i;
            let gx = dst_x0 + col_i;
            if sx < src_chars.len() {
                grid[gy][gx] = src_chars[sx];
            }
        }
    }

    grid.into_iter()
        .map(|row| Line::from(row.into_iter().collect::<String>()))
        .collect()
}

fn select_logo_art(
    width: usize,
    height: usize,
    arts: &[BrailleImage],
) -> Option<&BrailleImage> {
    // Prefer the largest art that still fits; otherwise the closest oversize piece
    // (center-cropped by about_logo_lines).
    let mut best_fit: Option<(&BrailleImage, u128)> = None;
    for art in arts {
        if art.width == 0 || art.height == 0 {
            continue;
        }
        if art.width <= width && art.height <= height {
            let score = (art.width as u128) * (art.height as u128);
            let should_replace = best_fit
                .as_ref()
                .map(|(_, best_score)| score > *best_score)
                .unwrap_or(true);
            if should_replace {
                best_fit = Some((art, score));
            }
        }
    }

    if let Some((art, _)) = best_fit {
        return Some(art);
    }

    arts.iter()
        .filter(|art| art.width > 0 && art.height > 0)
        .max_by_key(|art| (art.width as u128) * (art.height as u128))
}
