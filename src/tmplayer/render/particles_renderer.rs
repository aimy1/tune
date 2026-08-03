use crate::tmplayer::app::state::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

const PARTICLE_CHARS: [char; 5] = ['✦', '✧', '•', '*', '·'];

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let bars = &app.spectrum.bars;
    let num_bars = bars.len().max(1);
    let mut grid = vec![(' ', app.theme.color_subtext()); w * h];

    let num_columns = w;
    for col in 0..num_columns {
        let bar_idx = (col * num_bars) / num_columns;
        let val = bars.get(bar_idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let target_h = (val * h as f32).round() as usize;

        if target_h > 0 {
            for y in 0..target_h.min(h) {
                let row = h - 1 - y;
                let char_idx = (y % PARTICLE_CHARS.len()).min(PARTICLE_CHARS.len() - 1);
                let symbol = if y == target_h.saturating_sub(1) {
                    '✦'
                } else {
                    PARTICLE_CHARS[char_idx]
                };

                let color = if y > (h * 2) / 3 {
                    app.theme.color_accent()
                } else if y > h / 3 {
                    app.theme.color_accent2()
                } else {
                    app.theme.color_subtext()
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
