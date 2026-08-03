use crate::tmplayer::app::state::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

const PARTICLE_CHARS: [char; 6] = ['█', '▓', '▒', '░', '•', '·'];

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
        let target_h = (val * h as f32).round() as usize;

        if target_h > 0 {
            for y in 0..target_h.min(h) {
                let row = h - 1 - y;

                let (symbol, color) = if y == target_h.saturating_sub(1) {
                    if val > 0.7 {
                        ('★', app.theme.color_text())
                    } else if val > 0.4 {
                        ('✦', app.theme.color_accent3())
                    } else {
                        ('✧', app.theme.color_accent())
                    }
                } else {
                    let level_ratio = y as f32 / h.max(1) as f32;
                    let char_idx = ((1.0 - level_ratio) * (PARTICLE_CHARS.len() - 1) as f32)
                        .round()
                        .clamp(0.0, (PARTICLE_CHARS.len() - 1) as f32)
                        as usize;
                    let sym = PARTICLE_CHARS[char_idx];

                    let col_color = if level_ratio > 0.70 {
                        app.theme.color_accent3()
                    } else if level_ratio > 0.40 {
                        app.theme.color_accent()
                    } else if level_ratio > 0.15 {
                        app.theme.color_accent2()
                    } else {
                        app.theme.color_subtext()
                    };
                    (sym, col_color)
                };

                let idx = row * w + col;
                if idx < grid.len() {
                    grid[idx] = (symbol, color);
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
