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

    let bars = &app.spectrum.bars;
    let num_bars = bars.len().max(1);
    let mut cell_bits = vec![0u8; w_cells * h_cells];

    for x_px in 0..w_px {
        let bar_idx = (x_px * num_bars) / w_px;
        let val = bars.get(bar_idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let hill_h = (val * (h_px as f32 * 0.85)).round() as usize;

        // 1. Smooth Braille hill foundation
        if hill_h > 0 {
            for py_inv in 0..hill_h.min(h_px) {
                let py = h_px - 1 - py_inv;
                let cell_x = x_px / 2;
                let cell_y = py / 4;
                let sub_x = x_px % 2;
                let sub_y = py % 4;

                let bit = braille_bit(sub_x, sub_y);
                let idx = cell_y * w_cells + cell_x;
                if idx < cell_bits.len() {
                    cell_bits[idx] |= bit;
                }
            }
        }

        // 2. Floating sub-pixel particle dots in the air above hills
        if val > 0.15 {
            let spark_count = (val * 4.0) as usize;
            for s in 0..spark_count {
                let py_offset = hill_h + (s * 3 + (x_px % 4)) % 6 + 1;
                if py_offset < h_px {
                    let py = h_px - 1 - py_offset;
                    let cell_x = x_px / 2;
                    let cell_y = py / 4;
                    let sub_x = x_px % 2;
                    let sub_y = py % 4;

                    let bit = braille_bit(sub_x, sub_y);
                    let idx = cell_y * w_cells + cell_x;
                    if idx < cell_bits.len() {
                        cell_bits[idx] |= bit;
                    }
                }
            }
        }
    }

    let mut lines: Vec<Line> = Vec::with_capacity(h_cells);
    for row in 0..h_cells {
        let t = if h_cells <= 1 {
            1.0
        } else {
            row as f32 / (h_cells - 1) as f32
        };
        // Color gradient from top (Rose / Accent3) -> mid (Violet / Accent) -> bottom (Cyan / Accent2)
        let fg = if t < 0.35 {
            mix(app.theme.color_text(), app.theme.color_accent3(), t / 0.35)
        } else if t < 0.70 {
            mix(app.theme.color_accent3(), app.theme.color_accent(), (t - 0.35) / 0.35)
        } else {
            mix(app.theme.color_accent(), app.theme.color_accent2(), (t - 0.70) / 0.30)
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
