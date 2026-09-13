use crate::app::App;
use crate::data::config::Language;
use crate::tmplayer::data::about::{BrailleImage, about_info};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Modern About modal: elegant card layout with vinyl braille art,
/// structured identity header, concise description, metadata specs,
/// tech stack pills, and responsive resizing.
pub fn draw_about_modal(frame: &mut Frame, app: &App, size: Rect) {
    if size.width < 40 || size.height < 14 {
        draw_compact(frame, app, size);
        return;
    }

    let area = modal_area(size);
    frame.render_widget(Clear, area);

    let title = match app.config.language {
        Language::Zh => " 󰎆 关于 Tune ",
        Language::En => " 󰎆 About Tune ",
    };

    let back_hint = match app.config.language {
        Language::Zh => " 返回 ",
        Language::En => " Back ",
    };

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(title)
            .title_bottom(Line::from(vec![
                Span::styled(
                    " Esc / q ",
                    Style::default()
                        .fg(app.theme.color_base())
                        .bg(app.theme.color_buff())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    back_hint,
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

    let header_h = if inner.height >= 18 { 2 } else { 1 };
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
        size.height.saturating_sub(2).max(10),
        size,
    );
    frame.render_widget(Clear, area);

    let title = match app.config.language {
        Language::Zh => " 󰎆 关于 Tune ",
        Language::En => " 󰎆 About Tune ",
    };
    let back_hint = match app.config.language {
        Language::Zh => " 返回 ",
        Language::En => " Back ",
    };

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(title)
            .title_bottom(Line::from(vec![
                Span::styled(
                    " Esc / q ",
                    Style::default()
                        .fg(app.theme.color_base())
                        .bg(app.theme.color_buff())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    back_hint,
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
        horizontal: 1,
        vertical: 1,
    });
    draw_text_column(frame, app, inner);
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
        Language::Zh => "终端网易云音乐 · 用键盘听歌",
        Language::En => "NetEase Cloud Music · TUI Player",
    };

    let top = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("󰎆 Tune", name_style),
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

    if area.width >= 62 {
        // Horizontal two-column layout: Left is project name logo, Right is structured info
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(34),
                Constraint::Length(1),
                Constraint::Min(20),
            ])
            .split(area);

        draw_art_panel(frame, app, cols[0]);

        // Vertical separator line
        let separator = (0..cols[1].height)
            .map(|_| Line::from(Span::styled("│", Style::default().fg(app.theme.color_buff()))))
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(separator), cols[1]);

        draw_text_column(
            frame,
            app,
            cols[2].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 0,
            }),
        );
    } else {
        // Vertical stacked layout for narrow terminals
        let show_art = area.height >= 14 && area.width >= 24;
        if show_art {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(7),
                    Constraint::Min(6),
                ])
                .split(area);
            draw_art_panel(frame, app, rows[0]);
            draw_text_column(frame, app, rows[1]);
        } else {
            draw_text_column(frame, app, area);
        }
    }
}

fn draw_art_panel(frame: &mut Frame, app: &App, area: Rect) {
    if area.width < 8 || area.height < 4 {
        return;
    }

    if area.height >= 8 {
        let art_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(area);

        let lines = about_logo_lines(art_rows[0].width as usize, art_rows[0].height as usize);
        frame.render_widget(
            Paragraph::new(lines)
                .style(
                    Style::default()
                        .fg(app.theme.color_accent())
                        .bg(app.theme.color_surface()),
                )
                .alignment(Alignment::Center),
            art_rows[0],
        );

        let badge_text = match app.config.language {
            Language::Zh => "󰎆 键盘上的高保真音乐",
            Language::En => "󰎆 Hi-Fi Audio in TUI",
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    badge_text,
                    Style::default()
                        .fg(app.theme.color_accent2())
                        .bg(app.theme.color_surface())
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .alignment(Alignment::Center),
            art_rows[1],
        );
    } else {
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
}

fn draw_text_column(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let info = about_info();
    let max_w = area.width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let compact_space = area.height < 14;

    // ── 1. 简介 (Description) ──
    lines.push(section_header(
        app,
        "󰈙",
        match app.config.language {
            Language::Zh => "简介",
            Language::En => "About",
        },
    ));

    let description = match app.config.language {
        Language::Zh => {
            if info.description.trim().is_empty() {
                "Tune：终端里的网易云音乐客户端。".to_string()
            } else {
                info.description.clone()
            }
        }
        Language::En => {
            "A modern NetEase Cloud Music TUI player crafted in Rust, featuring lossless streaming, MPRIS integration, and vinyl visualization.".to_string()
        }
    };
    for row in wrap_display_width(&description, max_w.saturating_sub(2)) {
        lines.push(Line::from(Span::styled(
            format!("  {row}"),
            Style::default()
                .fg(app.theme.color_text())
                .bg(app.theme.color_surface()),
        )));
    }

    if !compact_space {
        lines.push(blank_line(app));
    }

    // ── 2. 项目信息 (Specifications) ──
    lines.push(section_header(
        app,
        "󰈀",
        match app.config.language {
            Language::Zh => "项目信息",
            Language::En => "Specifications",
        },
    ));

    let author_val = if info.author.is_empty() {
        "Asniya (@aimy1)"
    } else {
        info.author.as_str()
    };
    let license_val = if info.license.is_empty() {
        "GNU AGPL-3.0"
    } else {
        info.license.as_str()
    };

    let specs: Vec<(&str, &str, String)> = vec![
        (
            "󰏖",
            match app.config.language {
                Language::Zh => "版本",
                Language::En => "Version",
            },
            format!("v{}", info.version),
        ),
        (
            "󰑣",
            match app.config.language {
                Language::Zh => "作者",
                Language::En => "Author",
            },
            author_val.to_string(),
        ),
        (
            "󰊤",
            match app.config.language {
                Language::Zh => "源码",
                Language::En => "Repo",
            },
            "https://github.com/aimy1/tune".to_string(),
        ),
        (
            "󰋼",
            match app.config.language {
                Language::Zh => "反馈",
                Language::En => "Issues",
            },
            "https://github.com/aimy1/tune/issues".to_string(),
        ),
        (
            "󰿃",
            match app.config.language {
                Language::Zh => "协议",
                Language::En => "License",
            },
            license_val.to_string(),
        ),
    ];

    for (icon, label, val) in specs {
        let label_text = format!(" {label} ");
        let label_w = UnicodeWidthStr::width(label_text.as_str());
        let icon_w = UnicodeWidthStr::width(icon) + 1;
        let val_budget = max_w.saturating_sub(2 + icon_w + label_w + 1).max(8);
        let clipped = clip_to_display_width(&val, val_budget);

        lines.push(Line::from(vec![
            Span::styled("  ", Style::default().bg(app.theme.color_surface())),
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
            Span::styled(" ", Style::default().bg(app.theme.color_surface())),
            Span::styled(
                clipped,
                Style::default()
                    .fg(app.theme.color_text())
                    .bg(app.theme.color_surface()),
            ),
        ]));
    }

    if !compact_space {
        lines.push(blank_line(app));
    }

    // ── 3. 技术栈 (Tech Stack) ──
    lines.push(section_header(
        app,
        "󰏖",
        match app.config.language {
            Language::Zh => "技术栈",
            Language::En => "Tech Stack",
        },
    ));

    let chips = ["Rust 2024", "Ratatui", "Tokio", "Rodio", "MPRIS"];
    let mut chip_spans = vec![Span::styled("  ", Style::default().bg(app.theme.color_surface()))];
    for (i, chip) in chips.iter().enumerate() {
        if i > 0 {
            chip_spans.push(Span::styled(" ", Style::default().bg(app.theme.color_surface())));
        }
        chip_spans.push(Span::styled(
            format!(" {chip} "),
            Style::default()
                .fg(app.theme.color_accent())
                .bg(app.theme.color_buff())
                .add_modifier(Modifier::BOLD),
        ));
    }
    lines.push(Line::from(chip_spans));

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
        Language::Zh => "为终端音乐爱好者打造 · 欢迎 Star / Issue 反馈",
        Language::En => "Crafted for terminal music lovers · Stars & Issues welcome",
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

fn section_header(app: &App, icon: &str, title: &str) -> Line<'static> {
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

fn surface_style(app: &App) -> Style {
    Style::default()
        .fg(app.theme.color_subtext())
        .bg(app.theme.color_surface())
}

fn modal_area(size: Rect) -> Rect {
    let want_w = 76u16;
    let want_h = 22u16;

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

fn select_logo_art<'a>(
    width: usize,
    height: usize,
    arts: &'a [BrailleImage],
) -> Option<&'a BrailleImage> {
    let mut best_fit: Option<(&'a BrailleImage, u128)> = None;
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
        .min_by_key(|art| {
            let dw = art.width.saturating_sub(width);
            let dh = art.height.saturating_sub(height);
            dw * dw + dh * dh
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_area_fits_within_standard_80x24() {
        let terminal = Rect::new(0, 0, 80, 24);
        let area = modal_area(terminal);
        assert!(area.width <= terminal.width);
        assert!(area.height <= terminal.height);
        assert!(area.width >= 64);
        assert!(area.height >= 18);
    }

    #[test]
    fn test_modal_area_clamps_to_small_terminal() {
        let terminal = Rect::new(0, 0, 50, 16);
        let area = modal_area(terminal);
        assert!(area.width <= terminal.width);
        assert!(area.height <= terminal.height);
    }
}


