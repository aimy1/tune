use crate::tmplayer::app::state::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let w_cells = area.width as usize;
    let h_cells = area.height as usize;
    if w_cells == 0 || h_cells == 0 {
        return;
    }

    let w_px = w_cells * 2;
    let h_px = h_cells * 4;
    if w_px == 0 || h_px == 0 {
        return;
    }

    let mid_y = ((h_px as i32) - 1) / 2;
    let max_amp = (mid_y.max(1) as f32) * 0.92;
    let bars = &app.spectrum.bars;
    let num_bars = bars.len().max(1);

    let mut cell_bits = vec![0u8; w_cells * h_cells];

    for x_px in 0..w_px {
        let bar_idx = (x_px * num_bars) / w_px;
        let val = bars.get(bar_idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let amp = (val * max_amp).round() as i32;

        let y_top = (mid_y - amp).max(0);
        let y_bottom = (mid_y + amp).min(h_px as i32 - 1);

        for py in y_top..=y_bottom {
            let cell_x = x_px / 2;
            let cell_y = (py as usize) / 4;
            let sub_x = x_px % 2;
            let sub_y = (py as usize) % 4;

            let bit = braille_bit(sub_x, sub_y);
            let idx = cell_y * w_cells + cell_x;
            if idx < cell_bits.len() {
                cell_bits[idx] |= bit;
            }
        }
    }

    let mut lines: Vec<Line> = Vec::with_capacity(h_cells);
    for row in 0..h_cells {
        let t = if h_cells <= 1 {
            0.5
        } else {
            row as f32 / (h_cells - 1) as f32
        };
        let fg = if (t - 0.5).abs() < 0.15 {
            app.theme.color_text()
        } else if t < 0.5 {
            mix(app.theme.color_accent(), app.theme.color_text(), t * 2.0)
        } else {
            mix(app.theme.color_text(), app.theme.color_accent2(), (t - 0.5) * 2.0)
        };

        let mut s = String::with_capacity(w_cells);
        let base = row * w_cells;
        for col in 0..w_cells {
            let bits = cell_bits[base + col];
            s.push(if bits == 0 {
                ' '
            } else {
                char::from_u32(0x2800 + bits as u32).unwrap_or(' ')
            });
        }
        lines.push(Line::from(Span::styled(s, Style::default().fg(fg))));
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn braille_bit(sub_x: usize, sub_y: usize) -> u8 {
    match (sub_x, sub_y) {
        (0, 0) => 0x01,
        (0, 1) => 0x02,
        (0, 2) => 0x04,
        (0, 3) => 0x40,
        (1, 0) => 0x08,
        (1, 1) => 0x10,
        (1, 2) => 0x20,
        (1, 3) => 0x80,
        _ => 0,
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
