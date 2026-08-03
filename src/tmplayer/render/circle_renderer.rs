use crate::tmplayer::app::state::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
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
    let cx = (w_px as f32) / 2.0;
    let cy = (h_px as f32) / 2.0;

    let bars = &app.spectrum.bars;
    let num_bars = bars.len().max(1);
    let max_r = (cx.min(cy) * 0.88).max(2.0);

    // Calculate bass energy to pulse the center core
    let bass_energy = bars.iter().take(4).sum::<f32>() / 4.0;
    let base_r = (max_r * (0.25 + bass_energy.clamp(0.0, 1.0) * 0.15)).max(1.0);

    let mut grid = vec![0u8; w_cells * h_cells];
    let num_rays = 128.min(w_px * 2);

    for i in 0..num_rays {
        let angle = (i as f32 / num_rays as f32) * TAU - std::f32::consts::FRAC_PI_2;
        let bar_idx = (i * num_bars) / num_rays;
        let amp = bars.get(bar_idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let ray_len = base_r + amp * (max_r - base_r);

        let steps = (ray_len as usize).max(1);
        for s in 0..=steps {
            let r = base_r + (s as f32 / steps as f32) * (ray_len - base_r);
            let px = (cx + r * angle.cos()).round() as i32;
            let py = (cy + r * angle.sin()).round() as i32;

            if px >= 0 && px < w_px as i32 && py >= 0 && py < h_px as i32 {
                let cell_x = (px as usize) / 2;
                let cell_y = (py as usize) / 4;
                let sub_x = (px as usize) % 2;
                let sub_y = (py as usize) % 4;

                let bit_index = match (sub_x, sub_y) {
                    (0, 0) => 0,
                    (0, 1) => 1,
                    (0, 2) => 2,
                    (0, 3) => 6,
                    (1, 0) => 3,
                    (1, 1) => 4,
                    (1, 2) => 5,
                    (1, 3) => 7,
                    _ => 0,
                };

                let cell_idx = cell_y * w_cells + cell_x;
                if cell_idx < grid.len() {
                    grid[cell_idx] |= 1 << bit_index;
                }
            }
        }
    }

    let mut lines: Vec<Line> = Vec::with_capacity(h_cells);
    let center_cell_x = w_cells / 2;
    let center_cell_y = h_cells / 2;

    for row in 0..h_cells {
        let dy = (row as i32 - center_cell_y as i32).abs() as f32;
        let mut spans: Vec<Span> = Vec::with_capacity(w_cells);

        for col in 0..w_cells {
            let dx = (col as i32 - center_cell_x as i32).abs() as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            let bits = grid[row * w_cells + col];
            let ch = if bits == 0 {
                ' '
            } else {
                char::from_u32(0x2800 + bits as u32).unwrap_or(' ')
            };

            let fg = if dist < (w_cells as f32 * 0.15) {
                app.theme.color_accent()
            } else if dist < (w_cells as f32 * 0.35) {
                app.theme.color_accent2()
            } else if dist < (w_cells as f32 * 0.50) {
                app.theme.color_accent3()
            } else {
                app.theme.color_text()
            };

            spans.push(Span::styled(ch.to_string(), Style::default().fg(fg)));
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), area);
}
