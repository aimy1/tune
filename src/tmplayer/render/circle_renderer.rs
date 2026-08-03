use crate::tmplayer::app::state::AppState;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::f32::consts::TAU;

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let cx = (w as f32) / 2.0;
    let cy = (h as f32) / 2.0;

    let bars = &app.spectrum.bars;
    let num_bars = bars.len().max(1);

    // Aspect ratio correction: terminal cells are ~2x taller than wide.
    let aspect = 2.0;
    let max_r = (cx.min(cy * aspect) * 0.85).max(3.0);

    // Pulse inner core radius with bass energy
    let bass_energy = bars.iter().take(4).sum::<f32>() / 4.0;
    let base_r = (max_r * (0.28 + bass_energy.clamp(0.0, 1.0) * 0.12)).max(2.0);

    let mut lines: Vec<Line> = Vec::with_capacity(h);

    for row in 0..h {
        let dy = row as f32 - cy;
        let mut spans: Vec<Span> = Vec::with_capacity(w);

        for col in 0..w {
            let dx = (col as f32 - cx) * aspect;
            let r = (dx * dx + dy * dy).sqrt();

            if r <= max_r {
                // Calculate angle in [0, TAU)
                let mut angle = dy.atan2(dx);
                if angle < 0.0 {
                    angle += TAU;
                }

                let bar_idx = ((angle / TAU) * num_bars as f32) as usize % num_bars;
                let val = bars.get(bar_idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
                let target_r = base_r + val * (max_r - base_r);

                if r <= base_r {
                    // Inner pulsing core color block
                    let ch = if r < base_r * 0.6 { '█' } else { '▓' };
                    let color = mix(app.theme.color_text(), app.theme.color_accent(), r / base_r);
                    spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                } else if r <= target_r {
                    // Radial ray color blocks
                    let ray_ratio = (r - base_r) / (max_r - base_r).max(0.1);
                    let is_tip = (target_r - r) < 1.0;

                    let ch = if is_tip {
                        '█'
                    } else if ray_ratio > 0.6 {
                        '▓'
                    } else {
                        '█'
                    };

                    let color = if is_tip {
                        app.theme.color_text()
                    } else if ray_ratio > 0.6 {
                        mix(app.theme.color_accent(), app.theme.color_accent3(), ray_ratio)
                    } else {
                        mix(app.theme.color_accent2(), app.theme.color_accent(), ray_ratio)
                    };

                    spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                } else {
                    spans.push(Span::raw(" "));
                }
            } else {
                spans.push(Span::raw(" "));
            }
        }

        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), area);
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
