use crate::app::App;
use crate::data::config::Language;
use crate::tmplayer::data::about::{BrailleImage, about_info};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::UnicodeWidthChar;

/// Modern concise About modal: clean rounded border with no border text,
/// centered TUNE logo, airy metadata rows, tech stack badges, and soft exit hint.
pub fn draw_about_modal(frame: &mut Frame, app: &App, size: Rect) {
    if size.width < 40 || size.height < 14 {
        draw_compact(frame, app, size);
        return;
    }

    let area = modal_area(size);
    frame.render_widget(Clear, area);

    // Clean border with NO text on borders
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
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

    let top_h = if inner.height >= 16 { 7 } else { 2 };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_h),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(inner);

    draw_top_section(frame, app, rows[0]);
    draw_content_section(frame, app, rows[1]);
    draw_footer(frame, app, rows[2]);
}

fn draw_compact(frame: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(
        size.width.saturating_sub(2).max(20),
        size.height.saturating_sub(2).max(10),
        size,
    );
    frame.render_widget(Clear, area);

    // Clean border with NO text on borders
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
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
    draw_content_section(frame, app, inner);
}

fn draw_top_section(frame: &mut Frame, app: &App, area: Rect) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let info = about_info();
    let show_logo_art = area.height >= 6 && area.width >= 26;

    if show_logo_art {
        let logo_h = area.height.saturating_sub(2);
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(logo_h),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        // 1. Centered logo art
        let logo_lines = about_logo_lines(rows[0].width as usize, rows[0].height as usize);
        frame.render_widget(
            Paragraph::new(logo_lines)
                .style(
                    Style::default()
                        .fg(app.theme.color_accent())
                        .bg(app.theme.color_surface())
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center),
            rows[0],
        );

        // 2. Tagline with version
        draw_tagline_line(frame, app, rows[1], info);

        // 3. Horizontal divider line
        draw_divider_line(frame, app, rows[2]);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        draw_tagline_line(frame, app, rows[0], info);
        if area.height >= 2 {
            draw_divider_line(frame, app, rows[1]);
        }
    }
}

fn draw_tagline_line(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    info: &crate::tmplayer::data::about::AboutInfo,
) {
    let badge_style = Style::default()
        .fg(app.theme.color_base())
        .bg(app.theme.color_accent2())
        .add_modifier(Modifier::BOLD);
    let tagline_style = Style::default()
        .fg(app.theme.color_subtext())
        .bg(app.theme.color_surface());

    let tagline = match app.config.language {
        Language::Zh => "终端网易云音乐播放器",
        Language::En => "NetEase Cloud Music TUI Player",
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" v{} ", info.version), badge_style),
            Span::styled(
                "  ·  ",
                Style::default()
                    .fg(app.theme.color_buff())
                    .bg(app.theme.color_surface()),
            ),
            Span::styled(tagline, tagline_style),
        ]))
        .alignment(Alignment::Center),
        area,
    );
}

fn draw_divider_line(frame: &mut Frame, app: &App, area: Rect) {
    let sep_len = (area.width as usize).saturating_sub(12).max(10);
    let sep = "─".repeat(sep_len);
    frame.render_widget(
        Paragraph::new(sep)
            .style(
                Style::default()
                    .fg(app.theme.color_buff())
                    .bg(app.theme.color_surface()),
            )
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_content_section(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let info = about_info();
    let max_w = area.width as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let compact_space = area.height < 9;

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

    let spec_pill = |icon: &'static str, label: &str, val: &str| -> Vec<Span<'static>> {
        vec![
            Span::styled(
                format!("{icon} "),
                Style::default()
                    .fg(app.theme.color_accent2())
                    .bg(app.theme.color_surface()),
            ),
            Span::styled(
                format!(" {label} "),
                Style::default()
                    .fg(app.theme.color_base())
                    .bg(app.theme.color_buff())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default().bg(app.theme.color_surface())),
            Span::styled(
                val.to_string(),
                Style::default()
                    .fg(app.theme.color_text())
                    .bg(app.theme.color_surface()),
            ),
        ]
    };

    let (a_lbl, l_lbl) = match app.config.language {
        Language::Zh => ("作者", "协议"),
        Language::En => ("Author", "License"),
    };
    let repo_lbl = match app.config.language {
        Language::Zh => "源码",
        Language::En => "Repo",
    };
    let issues_lbl = match app.config.language {
        Language::Zh => "反馈",
        Language::En => "Issues",
    };

    if !compact_space {
        lines.push(blank_line(app));
    }

    if max_w >= 60 {
        let mut r1_spans = vec![Span::styled("    ", Style::default().bg(app.theme.color_surface()))];
        r1_spans.extend(spec_pill("󰑣", a_lbl, author_val));
        r1_spans.push(Span::styled("            ", Style::default().bg(app.theme.color_surface())));
        r1_spans.extend(spec_pill("󰿃", l_lbl, license_val));
        lines.push(Line::from(r1_spans));

        let mut r2_spans = vec![Span::styled("    ", Style::default().bg(app.theme.color_surface()))];
        r2_spans.extend(spec_pill("󰊤", repo_lbl, "https://github.com/aimy1/tune"));
        lines.push(Line::from(r2_spans));

        let mut r3_spans = vec![Span::styled("    ", Style::default().bg(app.theme.color_surface()))];
        r3_spans.extend(spec_pill("󰋼", issues_lbl, "https://github.com/aimy1/tune/issues"));
        lines.push(Line::from(r3_spans));
    } else {
        let items = [
            ("󰑣", a_lbl, author_val.to_string()),
            ("󰊤", repo_lbl, "https://github.com/aimy1/tune".to_string()),
            ("󰋼", issues_lbl, "https://github.com/aimy1/tune/issues".to_string()),
            ("󰿃", l_lbl, license_val.to_string()),
        ];
        for (icon, label, val) in items {
            let mut spans = vec![Span::styled("  ", Style::default().bg(app.theme.color_surface()))];
            let val_budget = max_w.saturating_sub(16).max(8);
            let clipped = clip_to_display_width(&val, val_budget);
            spans.extend(spec_pill(icon, label, &clipped));
            lines.push(Line::from(spans));
        }
    }

    if !compact_space {
        lines.push(blank_line(app));
    }

    // Centered Tech Stack pills
    let chips = ["Rust 2024", "Ratatui", "Tokio", "Rodio", "MPRIS"];
    let mut chip_spans = Vec::new();
    for (i, chip) in chips.iter().enumerate() {
        if i > 0 {
            chip_spans.push(Span::styled("  ", Style::default().bg(app.theme.color_surface())));
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
            .alignment(if max_w >= 60 { Alignment::Left } else { Alignment::Left })
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let text = match app.config.language {
        Language::Zh => "Esc / q  返回",
        Language::En => "Esc / q  to return",
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
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
    let want_w = 68u16;
    let want_h = 19u16;

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


