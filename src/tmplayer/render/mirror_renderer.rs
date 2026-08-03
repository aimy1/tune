use crate::tmplayer::app::state::AppState;
use crate::tmplayer::data::config::BarChannels;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn render(f: &mut Frame, area: Rect, app: &mut AppState) {
    let h = area.height as usize;
    let w = area.width as usize;
    if h == 0 || w == 0 {
        return;
    }

    let mid_row = h / 2;
    let max_half_h = mid_row.max(1);

    let bars = &app.spectrum.bars;
    let mono_count = bars.len().max(1);
    if app.spectrum_render_grid.len() != h {
        app.spectrum_render_grid.resize_with(h, Vec::new);
    }
    for row in &mut app.spectrum_render_grid {
        if row.len() != w {
            row.resize(w, ' ');
        } else {
            row.fill(' ');
        }
    }

    let (bar_widths, gap_width, draw_total, x_offset) =
        compute_bar_layout(w, app.config.bars_gap, mono_count, app.config.bar_channels);
    if draw_total == 0 || bar_widths.is_empty() {
        return;
    }

    let draw_vals = build_display_vals(
        bars,
        draw_total,
        app.config.bar_channels,
        app.config.bar_channel_reverse,
    );

    let mut x_cursor = x_offset.min(w);
    for (i, &val) in draw_vals.iter().enumerate() {
        if x_cursor >= w {
            break;
        }
        let bar_width = bar_widths.get(i).copied().unwrap_or(1);
        let val = apply_height_curve(val);

        if app.config.super_smooth_bar {
            let fill = val * max_half_h as f32;
            let full = fill.floor().clamp(0.0, max_half_h as f32) as usize;
            let frac = (fill - full as f32).clamp(0.0, 1.0);

            for y in 0..=max_half_h {
                let ch_up = if y < full {
                    '█'
                } else if y == full {
                    smooth_char(frac)
                } else {
                    ' '
                };

                let ch_down = if y < full {
                    '█'
                } else if y == full {
                    downward_smooth_char(frac)
                } else {
                    ' '
                };

                for x in x_cursor..(x_cursor + bar_width).min(w) {
                    if mid_row >= y && ch_up != ' ' {
                        app.spectrum_render_grid[mid_row - y][x] = ch_up;
                    }
                    if mid_row + y < h && ch_down != ' ' {
                        app.spectrum_render_grid[mid_row + y][x] = ch_down;
                    }
                }
            }
        } else {
            let bar_h = (val * max_half_h as f32).round() as usize;
            for y in 0..bar_h.min(max_half_h) {
                let ch = density_char(y, bar_h.max(1));
                for x in x_cursor..(x_cursor + bar_width).min(w) {
                    if mid_row >= y {
                        app.spectrum_render_grid[mid_row - y][x] = ch;
                    }
                    if mid_row + y < h {
                        app.spectrum_render_grid[mid_row + y][x] = ch;
                    }
                }
            }
        }

        x_cursor = x_cursor.saturating_add(bar_width);
        if i + 1 < draw_total {
            x_cursor = x_cursor.saturating_add(gap_width);
        }
    }

    // Fill center line empty cells with horizontal divider line
    for x in x_offset..x_cursor.min(w) {
        if app.spectrum_render_grid[mid_row][x] == ' ' {
            app.spectrum_render_grid[mid_row][x] = '─';
        }
    }

    // Render per-line vertical gradient using theme colors.
    let mut lines: Vec<Line> = Vec::with_capacity(h);
    for (row_idx, row) in app.spectrum_render_grid.iter().enumerate() {
        let fg = if row_idx == mid_row {
            app.theme.color_text()
        } else if row_idx < mid_row {
            let t = if mid_row == 0 {
                1.0
            } else {
                row_idx as f32 / mid_row as f32
            };
            mix(app.theme.color_accent2(), app.theme.color_text(), t)
        } else {
            let t = if h - 1 == mid_row {
                0.0
            } else {
                (row_idx - mid_row) as f32 / (h - 1 - mid_row) as f32
            };
            mix(app.theme.color_text(), app.theme.color_accent3(), t)
        };

        let s = row.iter().collect::<String>();
        lines.push(Line::from(Span::styled(s, Style::default().fg(fg))));
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn compute_bar_layout(
    width: usize,
    gap: bool,
    data_len: usize,
    mode: BarChannels,
) -> (Vec<usize>, usize, usize, usize) {
    if width == 0 {
        return (Vec::new(), 0, 0, 0);
    }

    let mut desired_total = match mode {
        BarChannels::Mono => data_len,
        BarChannels::Stereo => data_len.saturating_mul(2),
    };

    let max_total = if gap {
        width.div_ceil(2).max(1)
    } else {
        (width / 2).max(1)
    };
    if desired_total > max_total {
        desired_total = max_total;
    }
    if mode == BarChannels::Stereo && desired_total % 2 == 1 {
        desired_total = desired_total.saturating_sub(1).max(2);
    }

    let mut bars = desired_total.max(1);
    loop {
        if !gap {
            let bar_w = width / bars;
            if bar_w >= 2 {
                let used = bars * bar_w;
                let mut widths = vec![bar_w; bars];
                let mut remainder = width.saturating_sub(used);
                for w in &mut widths {
                    if remainder == 0 {
                        break;
                    }
                    *w += 1;
                    remainder -= 1;
                }
                let used = widths.iter().sum::<usize>();
                let offset = width.saturating_sub(used) / 2;
                return (widths, 0, bars, offset);
            }
        } else {
            let mut bar_w = width / bars;
            while bar_w >= 1 {
                let gap_w = bar_w.div_ceil(2);
                let needed = bars * bar_w + (bars.saturating_sub(1)) * gap_w;
                if needed <= width {
                    let mut widths = vec![bar_w; bars];
                    let mut remainder = width.saturating_sub(needed);
                    for w in &mut widths {
                        if remainder == 0 {
                            break;
                        }
                        *w += 1;
                        remainder -= 1;
                    }
                    let used = widths.iter().sum::<usize>() + (bars.saturating_sub(1)) * gap_w;
                    let offset = width.saturating_sub(used) / 2;
                    return (widths, gap_w, bars, offset);
                }
                if bar_w == 1 {
                    break;
                }
                bar_w -= 1;
            }
        }

        if bars <= 1 {
            let used = width.max(1);
            let offset = width.saturating_sub(used) / 2;
            return (vec![used], 0, 1, offset);
        }
        bars -= 1;
    }
}

fn build_display_vals(
    data: &[f32],
    draw_total: usize,
    mode: BarChannels,
    reverse: bool,
) -> Vec<f32> {
    let data_len = data.len().max(1);
    if draw_total == 0 {
        return Vec::new();
    }

    match mode {
        BarChannels::Mono => (0..draw_total)
            .map(|i| {
                if reverse {
                    sample_val(data, data_len, draw_total, draw_total - 1 - i)
                } else {
                    sample_val(data, data_len, draw_total, i)
                }
            })
            .collect(),
        BarChannels::Stereo => {
            let per_side = (draw_total / 2).max(1);
            let mut right: Vec<f32> = (0..per_side)
                .map(|i| {
                    if reverse {
                        sample_val(data, data_len, per_side, per_side - 1 - i)
                    } else {
                        sample_val(data, data_len, per_side, i)
                    }
                })
                .collect();
            let mut left = right.clone();
            left.reverse();
            left.append(&mut right);
            left
        }
    }
}

fn sample_val(data: &[f32], data_len: usize, draw_len: usize, i: usize) -> f32 {
    let idx =
        ((i as u32) * (data_len as u32) / (draw_len as u32)).min((data_len - 1) as u32) as usize;
    data.get(idx).copied().unwrap_or(0.0).clamp(0.0, 1.0)
}

fn apply_height_curve(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    v.powf(0.72)
}

fn density_char(level: usize, height: usize) -> char {
    if height == 0 {
        return ' ';
    }
    if height == 1 {
        return '░';
    }
    let ratio = level as f32 / height as f32;
    if ratio < 0.25 {
        '█'
    } else if ratio < 0.50 {
        '▓'
    } else if ratio < 0.75 {
        '▒'
    } else {
        '░'
    }
}

fn smooth_char(frac: f32) -> char {
    if frac <= 0.0 {
        ' '
    } else if frac < 1.0 / 7.0 {
        '▂'
    } else if frac < 2.0 / 7.0 {
        '▃'
    } else if frac < 3.0 / 7.0 {
        '▄'
    } else if frac < 4.0 / 7.0 {
        '▅'
    } else if frac < 5.0 / 7.0 {
        '▆'
    } else if frac < 6.0 / 7.0 {
        '▇'
    } else {
        '█'
    }
}

fn downward_smooth_char(frac: f32) -> char {
    if frac <= 0.0 {
        ' '
    } else if frac < 0.25 {
        '▔'
    } else if frac < 0.65 {
        '▀'
    } else {
        '█'
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    match (a, b) {
        (Color::Rgb(ar, ag, ab), Color::Rgb(br, bg, bb)) => {
            let r = (ar as f32 + (br as f32 - ar as f32) * t) as u8;
            let g = (ag as f32 + (bg as f32 - ag as f32) * t) as u8;
            let b = (ab as f32 + (bb as f32 - ab as f32) * t) as u8;
            Color::Rgb(r, g, b)
        }
        _ => a,
    }
}
