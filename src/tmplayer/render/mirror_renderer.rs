use crate::tmplayer::app::state::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

const BLOCK_LEVELS: [char; 7] = ['█', '▇', '▆', '▅', '▄', '▃', '▂'];

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mid_row = h / 2;
    let max_half_h = mid_row.max(1);
    let bars = &app.spectrum.bars;
    let num_bars = bars.len().max(1);

    let mut grid = vec![(' ', app.theme.color_subtext()); w * h];

    // 1. Draw glowing laser horizon line across the exact center
    for col in 0..w {
        let idx = mid_row * w + col;
        if idx < grid.len() {
            grid[idx] = ('━', app.theme.color_text());
        }
    }

    // 2. Draw symmetric mirrored frequency bars expanding upwards & downwards
    for col in 0..w {
        let bar_idx = (col * num_bars) / w;
        let val = bars.get(bar_idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let bar_h = (val * max_half_h as f32).round() as usize;

        if bar_h > 0 {
            for dy in 1..=bar_h {
                let ratio = dy as f32 / max_half_h as f32;
                let char_idx = (ratio * (BLOCK_LEVELS.len() - 1) as f32)
                    .round()
                    .clamp(0.0, (BLOCK_LEVELS.len() - 1) as f32) as usize;

                let is_peak = dy == bar_h;
                let (sym_up, sym_down) = if is_peak {
                    ('▔', ' ')
                } else {
                    (BLOCK_LEVELS[char_idx], BLOCK_LEVELS[char_idx])
                };

                let color_up = if is_peak {
                    app.theme.color_text()
                } else if dy > max_half_h / 2 {
                    app.theme.color_accent3()
                } else {
                    app.theme.color_accent()
                };

                let color_down = if is_peak {
                    app.theme.color_text()
                } else if dy > max_half_h / 2 {
                    app.theme.color_accent2()
                } else {
                    app.theme.color_subtext()
                };

                // Upward bar (Warm Fire palette)
                if mid_row >= dy {
                    let up_row = mid_row - dy;
                    let idx = up_row * w + col;
                    if idx < grid.len() {
                        grid[idx] = (sym_up, color_up);
                    }
                }

                // Downward mirrored bar (Cool Ice palette)
                let down_row = mid_row + dy;
                if down_row < h {
                    let idx = down_row * w + col;
                    if idx < grid.len() {
                        grid[idx] = (sym_down, color_down);
                    }
                }
            }
        }
    }

    let mut lines: Vec<Line> = Vec::with_capacity(h);
    for row in 0..h {
        let mut spans: Vec<Span> = Vec::with_capacity(w);
        let row_offset = row * w;
        for col in 0..w {
            let (ch, color) = grid[row_offset + col];
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), area);
}
