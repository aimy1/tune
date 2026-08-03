use crate::tmplayer::app::state::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

const SPARKLE_STARS: [char; 6] = ['★', '✶', '✦', '✧', '✺', '·'];
const WAVE_BASE: [char; 8] = ['⣀', '⣤', '⣴', '⣶', '⣾', '⣿', '⡿', '⠿'];

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let bars = &app.spectrum.bars;
    let num_bars = bars.len().max(1);
    let mut grid = vec![(' ', app.theme.color_subtext()); w * h];

    for col in 0..w {
        let bar_idx = (col * num_bars) / w;
        let val = bars.get(bar_idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let column_h = (val * h as f32).round() as usize;

        if column_h > 0 {
            let base_h = (column_h / 2).max(1);
            let particle_h = column_h;

            // 1. Fluid liquid wave foundation at the bottom
            for y in 0..base_h.min(h) {
                let row = h - 1 - y;
                let char_idx = (y * (WAVE_BASE.len() - 1)) / base_h.max(1);
                let sym = WAVE_BASE[char_idx.min(WAVE_BASE.len() - 1)];

                let color = if y > base_h / 2 {
                    app.theme.color_accent()
                } else {
                    app.theme.color_accent2()
                };

                let idx = row * w + col;
                if idx < grid.len() {
                    grid[idx] = (sym, color);
                }
            }

            // 2. Sparkling particle stars floating into the air above foundation
            for y in base_h..particle_h.min(h) {
                let row = h - 1 - y;
                let star_idx = (col + y * 3) % SPARKLE_STARS.len();
                let sym = SPARKLE_STARS[star_idx];

                let color = if y == particle_h.saturating_sub(1) {
                    if val > 0.6 {
                        app.theme.color_text()
                    } else {
                        app.theme.color_accent3()
                    }
                } else if y > (particle_h + base_h) / 2 {
                    app.theme.color_accent3()
                } else {
                    app.theme.color_accent()
                };

                let idx = row * w + col;
                if idx < grid.len() {
                    grid[idx] = (sym, color);
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
