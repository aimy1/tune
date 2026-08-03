use crate::tmplayer::app::state::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::f32::consts::TAU;

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

    let cx = (w_px as f32) / 2.0;
    let cy = (h_px as f32) / 2.0;

    let bars = &app.spectrum.bars;
    let num_bars = bars.len().max(1);

    // Fill the available box height/width comfortably
    let max_r = (cx.min(cy * 1.25) * 0.90).max(4.0);

    // Pulse inner core radius with bass energy
    let bass_energy = bars.iter().take(4).sum::<f32>() / 4.0;
    let inner_r = (max_r * (0.35 + bass_energy.clamp(0.0, 1.0) * 0.10)).max(2.0);

    let mut cell_bits = vec![0u8; w_cells * h_cells];

    // 1. Draw inner pulsing disc border ring
    let disc_steps = 64;
    for i in 0..disc_steps {
        let angle = (i as f32 / disc_steps as f32) * TAU;
        let px = (cx + inner_r * angle.cos()).round() as i32;
        let py = (cy + inner_r * angle.sin()).round() as i32;
        set_pixel(&mut cell_bits, w_cells, w_px, h_px, px, py);
    }

    // 2. Draw 48 distinct, crisp radial equalizer bars
    let num_rays = 48;
    for i in 0..num_rays {
        let angle = (i as f32 / num_rays as f32) * TAU - std::f32::consts::FRAC_PI_2;
        let bar_idx = (i * num_bars) / num_rays;
        let val = bars.get(bar_idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let ray_len = inner_r + val * (max_r - inner_r);

        // Ray line
        let steps = (ray_len - inner_r).max(1.0) as usize;
        for s in 1..=steps {
            let r = inner_r + (s as f32 / steps as f32) * (ray_len - inner_r);
            let px = (cx + r * angle.cos()).round() as i32;
            let py = (cy + r * angle.sin()).round() as i32;
            set_pixel(&mut cell_bits, w_cells, w_px, h_px, px, py);
        }

        // Floating peak dot
        if val > 0.05 {
            let peak_r = (ray_len + 1.5).min(max_r + 2.0);
            let px = (cx + peak_r * angle.cos()).round() as i32;
            let py = (cy + peak_r * angle.sin()).round() as i32;
            set_pixel(&mut cell_bits, w_cells, w_px, h_px, px, py);
        }
    }

    let mut lines: Vec<Line> = Vec::with_capacity(h_cells);
    let center_cell_x = w_cells / 2;
    let center_cell_y = h_cells / 2;
    let max_cell_dist = ((center_cell_x * center_cell_x + center_cell_y * center_cell_y) as f32).sqrt().max(1.0);

    for row in 0..h_cells {
        let dy = (row as i32 - center_cell_y as i32).abs() as f32;
        let mut spans: Vec<Span> = Vec::with_capacity(w_cells);

        for col in 0..w_cells {
            let dx = (col as i32 - center_cell_x as i32).abs() as f32;
            let dist = (dx * dx + dy * dy).sqrt();
            let norm_dist = (dist / max_cell_dist).clamp(0.0, 1.0);

            let bits = cell_bits[row * w_cells + col];
            let ch = if bits == 0 {
                ' '
            } else {
                char::from_u32(0x2800 + bits as u32).unwrap_or(' ')
            };

            let fg = if norm_dist < 0.25 {
                mix(app.theme.color_text(), app.theme.color_accent(), norm_dist / 0.25)
            } else if norm_dist < 0.60 {
                mix(app.theme.color_accent(), app.theme.color_accent2(), (norm_dist - 0.25) / 0.35)
            } else {
                mix(app.theme.color_accent2(), app.theme.color_accent3(), (norm_dist - 0.60) / 0.40)
            };

            spans.push(Span::styled(ch.to_string(), Style::default().fg(fg)));
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn set_pixel(cell_bits: &mut [u8], w_cells: usize, w_px: usize, h_px: usize, px: i32, py: i32) {
    if px >= 0 && px < w_px as i32 && py >= 0 && py < h_px as i32 {
        let cell_x = (px as usize) / 2;
        let cell_y = (py as usize) / 4;
        let sub_x = (px as usize) % 2;
        let sub_y = (py as usize) % 4;

        let bit = braille_bit(sub_x, sub_y);
        let idx = cell_y * w_cells + cell_x;
        if idx < cell_bits.len() {
            cell_bits[idx] |= bit;
        }
    }
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
