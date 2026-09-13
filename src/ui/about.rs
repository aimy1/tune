use crate::app::App;
use crate::data::config::Language;
use crate::tmplayer::data::about::{BrailleImage, about_info};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Minimalist About modal: clean rounded border with no border text,
/// centered TUNE logo, concise essential info, and soft exit hint.
pub fn draw_about_modal(frame: &mut Frame, app: &App, size: Rect) {
    if size.width < 32 || size.height < 10 {
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
    draw_about_content(frame, app, inner);
}

fn draw_compact(frame: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(
        size.width.saturating_sub(2).max(20),
        size.height.saturating_sub(2).max(8),
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
    draw_about_content(frame, app, inner);
}

fn draw_about_content(frame: &mut Frame, app: &App, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let info = about_info();
    let mut lines: Vec<Line<'static>> = Vec::new();

    // 1. Logo Art (if height allows)
    let show_art = area.height >= 9 && area.width >= 26;
    if show_art {
        let max_logo_h = if area.height >= 12 && area.width >= 48 {
            6
        } else if area.height >= 10 && area.width >= 38 {
            5
        } else {
            4
        };
        let logo_lines = about_logo_lines(app, area.width as usize, max_logo_h);
        lines.extend(logo_lines);
        lines.push(blank_line(app));
    }

    // 2. Version
    let badge_style = Style::default()
        .fg(app.theme.color_base())
        .bg(app.theme.color_accent2())
        .add_modifier(Modifier::BOLD);
    lines.push(Line::from(vec![
        Span::styled(format!(" 󰎆 v{} ", info.version), badge_style),
    ]));

    // 3. Tagline & Author
    let author = if info.author.is_empty() {
        "Asniya (@aimy1)"
    } else {
        info.author.as_str()
    };
    let tagline = match app.config.language {
        Language::Zh => format!("终端网易云音乐 · By {author}"),
        Language::En => format!("NetEase Cloud Music TUI Player · By {author}"),
    };
    lines.push(Line::from(Span::styled(
        tagline,
        Style::default()
            .fg(app.theme.color_text())
            .add_modifier(Modifier::BOLD),
    )));

    // 4. Repo Link
    let repo_url = info
        .links
        .get("github_url")
        .map(|s| s.as_str())
        .unwrap_or("https://github.com/aimy1/tune");
    lines.push(Line::from(Span::styled(
        repo_url.to_string(),
        Style::default()
            .fg(app.theme.color_accent())
            .add_modifier(Modifier::UNDERLINED),
    )));

    // 5. Exit Hint
    if area.height as usize > lines.len() + 1 {
        lines.push(blank_line(app));
    }
    let exit_hint = match app.config.language {
        Language::Zh => "Esc / q  返回",
        Language::En => "Esc / q  Back",
    };
    lines.push(Line::from(Span::styled(
        exit_hint,
        Style::default().fg(app.theme.color_subtext()),
    )));

    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(app.theme.color_surface()))
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
    let want_w = 58u16;
    let want_h = 16u16;

    let max_w = size.width.saturating_sub(2);
    let max_h = size.height.saturating_sub(1);
    let w = want_w.min(max_w).max(32.min(max_w));
    let h = want_h.min(max_h).max(10.min(max_h));
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

/// Center the logo in the panel; if the panel is smaller, crop from the center.
fn about_logo_lines(app: &App, width: usize, height: usize) -> Vec<Line<'static>> {
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

    let total_rows = grid.len();
    let c1 = app.theme.palette.accent;
    let c2 = app.theme.palette.accent2;

    grid.into_iter()
        .enumerate()
        .map(|(r_idx, row)| {
            let color = match app.theme.capability {
                crate::ui::theme::ColorCapability::TrueColor => {
                    let t = if total_rows <= 1 {
                        0.0
                    } else {
                        r_idx as f32 / (total_rows - 1) as f32
                    };
                    let r = (c1.0 as f32 * (1.0 - t) + c2.0 as f32 * t).round() as u8;
                    let g = (c1.1 as f32 * (1.0 - t) + c2.1 as f32 * t).round() as u8;
                    let b = (c1.2 as f32 * (1.0 - t) + c2.2 as f32 * t).round() as u8;
                    ratatui::style::Color::Rgb(r, g, b)
                }
                _ => {
                    if r_idx < total_rows / 2 {
                        app.theme.color_accent()
                    } else {
                        app.theme.color_accent2()
                    }
                }
            };
            let text: String = row.into_iter().collect();
            Line::from(Span::styled(
                text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ))
        })
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
        assert!(area.width >= 48);
        assert!(area.height >= 12);
    }

    #[test]
    fn test_modal_area_clamps_to_small_terminal() {
        let terminal = Rect::new(0, 0, 50, 16);
        let area = modal_area(terminal);
        assert!(area.width <= terminal.width);
        assert!(area.height <= terminal.height);
    }
}


