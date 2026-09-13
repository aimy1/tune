use crate::app::{App, HitRect, PlaybackRuntimeState, PlayerBarHitTargets};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use std::time::Duration;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub const PLAYER_BAR_HEIGHT: u16 = 5;

pub fn draw_collapsed_player_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    let transparent = app.config.transparent_background;
    let surface = app.theme.color_surface();
    let bar_bg = app.theme.style_surface_bg(transparent);
    let with_bar_bg = |s: Style| {
        if transparent {
            s
        } else {
            s.bg(surface)
        }
    };

    frame.render_widget(Block::default().style(bar_bg), area);

    frame.render_widget(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(app.theme.color_buff()))
            .style(bar_bg),
        area,
    );

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let top = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };
    let bottom = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };

    let prev_label = "  ";
    let play_label = if app.playback_state == PlaybackRuntimeState::Playing {
        "  "
    } else {
        "  "
    };
    let next_label = "  ";
    let mode_symbol = playback_repeat_symbol(app);

    let vol_pct = (app.volume * 100.0).round() as i32;
    let is_muted = vol_pct == 0;
    let vol_icon = if is_muted {
        "󰝟"
    } else if vol_pct < 33 {
        "󰕿"
    } else if vol_pct < 66 {
        "󰖀"
    } else {
        "󰕾"
    };
    let vol_label = format!("[{vol_icon} {vol_pct}%]");

    let prev_w = display_width(prev_label) as u16;
    let play_w = display_width(play_label) as u16;
    let next_w = display_width(next_label) as u16;
    let mode_w = display_width(mode_symbol) as u16;
    let vol_w = display_width(&vol_label) as u16;
    let controls_w = prev_w + 1 + play_w + 1 + next_w + 2 + mode_w + 2 + vol_w;

    let spectrum =
        if app.now_playing.is_some() && app.playback_state != PlaybackRuntimeState::Stopped {
            app.main_spectrum_braille()
        } else {
            " ".repeat(10)
        };

    let controls_col_w = controls_w.saturating_add(2).min(top.width);
    let controls_x = top.x + top.width.saturating_sub(controls_col_w) / 2;
    let left_col_w = controls_x.saturating_sub(top.x);

    let left_rect = Rect {
        x: top.x,
        y: top.y,
        width: left_col_w,
        height: 1,
    };
    let controls_rect = Rect {
        x: controls_x,
        y: top.y,
        width: controls_col_w,
        height: 1,
    };
    let spectrum_x = controls_x + controls_col_w;
    let spectrum_col_w = (top.x + top.width).saturating_sub(spectrum_x);
    let spectrum_rect = Rect {
        x: spectrum_x,
        y: top.y,
        width: spectrum_col_w,
        height: 1,
    };

    // Track title + artist with clearer hierarchy.
    if let Some(track) = app.now_playing.as_ref() {
        let title = track.title.trim();
        let artist = app.now_playing_artist_text();
        let heart = if app.now_playing_liked { "" } else { "" };
        let heart_style = with_bar_bg(if app.now_playing_liked {
            Style::default().fg(app.theme.color_accent3())
        } else {
            Style::default().fg(app.theme.color_subtext())
        });
        let title_style = with_bar_bg(
            Style::default()
                .fg(app.theme.color_text())
                .add_modifier(Modifier::BOLD),
        );
        let sep_style = with_bar_bg(Style::default().fg(app.theme.color_buff()));
        let artist_style = with_bar_bg(Style::default().fg(app.theme.color_subtext()));

        let heart_w = display_width(heart);
        let max_for_text = left_rect.width as usize;
        let text_budget = max_for_text.saturating_sub(heart_w + 1);

        let mut spans = Vec::new();
        if !title.is_empty() {
            if artist.trim().is_empty() {
                let clipped = clip_to_display_width(title, text_budget);
                spans.push(Span::styled(clipped, title_style));
            } else {
                let sep = " · ";
                let sep_w = display_width(sep);
                let title_budget = (text_budget * 6 / 10).max(4);
                let clipped_title = clip_to_display_width(title, title_budget);
                let used = display_width(&clipped_title) + sep_w;
                let artist_budget = text_budget.saturating_sub(used);
                let clipped_artist = clip_to_display_width(artist.trim(), artist_budget);
                spans.push(Span::styled(clipped_title, title_style));
                if !clipped_artist.is_empty() {
                    spans.push(Span::styled(sep, sep_style));
                    spans.push(Span::styled(clipped_artist, artist_style));
                }
            }
        }

        let used_w: usize = spans
            .iter()
            .map(|s| display_width(s.content.as_ref()))
            .sum();
        let pad = max_for_text.saturating_sub(used_w + heart_w);
        spans.push(Span::styled(" ".repeat(pad), with_bar_bg(Style::default())));
        spans.push(Span::styled(heart, heart_style));

        frame.render_widget(Paragraph::new(Line::from(spans)), left_rect);
    } else {
        let idle = match app.config.language {
            crate::data::config::Language::Zh => "未在播放",
            crate::data::config::Language::En => "Not playing",
        };
        frame.render_widget(
            Paragraph::new(idle).style(with_bar_bg(Style::default().fg(app.theme.color_subtext()))),
            left_rect,
        );
    }

    let prev_span = Span::styled(
        prev_label,
        with_bar_bg(Style::default().fg(app.theme.color_text())),
    );
    // Playing: green pill; paused/stopped: blue pill.
    let play_span = if app.playback_state == PlaybackRuntimeState::Playing {
        Span::styled(
            play_label,
            Style::default()
                .fg(app.theme.color_base())
                .bg(app.theme.color_accent2())
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            play_label,
            Style::default()
                .fg(app.theme.color_base())
                .bg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        )
    };
    let next_span = Span::styled(
        next_label,
        with_bar_bg(Style::default().fg(app.theme.color_text())),
    );
    let mode_span = Span::styled(
        mode_symbol,
        with_bar_bg(Style::default().fg(app.theme.color_subtext())),
    );
    let is_vol_modal_open = matches!(app.overlay, Some(crate::app::Overlay::VolumeModal));
    let vol_spans = if is_vol_modal_open {
        vec![Span::styled(
            vol_label,
            Style::default()
                .fg(app.theme.color_base())
                .bg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        )]
    } else {
        vec![
            Span::styled("[", with_bar_bg(Style::default().fg(app.theme.color_subtext()))),
            Span::styled(
                vol_icon,
                with_bar_bg(Style::default().fg(if is_muted {
                    app.theme.color_subtext()
                } else {
                    app.theme.color_accent()
                })),
            ),
            Span::styled(
                format!(" {vol_pct}%]"),
                with_bar_bg(Style::default().fg(app.theme.color_text())),
            ),
        ]
    };
    let gap = Span::styled(" ", with_bar_bg(Style::default()));
    let gap2 = Span::styled("  ", with_bar_bg(Style::default()));

    let mut controls_spans = vec![
        gap.clone(),
        prev_span,
        gap.clone(),
        play_span,
        gap,
        next_span,
        gap2.clone(),
        mode_span,
        gap2,
    ];
    controls_spans.extend(vol_spans);

    frame.render_widget(
        Paragraph::new(Line::from(controls_spans))
            .alignment(Alignment::Center),
        controls_rect,
    );

    frame.render_widget(
        Paragraph::new(spectrum)
            .style(with_bar_bg(Style::default().fg(app.theme.color_accent2())))
            .alignment(Alignment::Right),
        spectrum_rect,
    );

    let position = app.playback_position();
    let duration = app.playback_duration();
    let time_text = format!("{}/{}", format_mmss(position), format_mmss(duration));
    let time_w = display_width(&time_text) as u16;

    let progress_w = bottom.width.saturating_sub(time_w.saturating_add(1));
    let progress_rect = Rect {
        x: bottom.x,
        y: bottom.y,
        width: progress_w,
        height: 1,
    };
    let time_rect = Rect {
        x: bottom.x + progress_w,
        y: bottom.y,
        width: bottom.width.saturating_sub(progress_w),
        height: 1,
    };

    let mut hits = PlayerBarHitTargets::default();

    let line_w = 1 + prev_w + 1 + play_w + 1 + next_w + 2 + mode_w + 2 + vol_w;
    let line_start_x = controls_rect.x + controls_rect.width.saturating_sub(line_w) / 2;

    let prev_x = line_start_x.saturating_add(1);
    hits.prev = Some(HitRect {
        x: prev_x,
        y: top.y,
        width: prev_w,
        height: 1,
    });

    let play_x = prev_x.saturating_add(prev_w).saturating_add(1);
    hits.play_pause = Some(HitRect {
        x: play_x,
        y: top.y,
        width: play_w,
        height: 1,
    });

    let next_x = play_x.saturating_add(play_w).saturating_add(1);
    hits.next = Some(HitRect {
        x: next_x,
        y: top.y,
        width: next_w,
        height: 1,
    });

    let mode_x = next_x.saturating_add(next_w).saturating_add(2);
    hits.repeat_mode = Some(HitRect {
        x: mode_x.saturating_sub(1),
        y: top.y,
        width: mode_w.saturating_add(2),
        height: 1,
    });

    let vol_x = mode_x.saturating_add(mode_w).saturating_add(2);
    hits.volume = Some(HitRect {
        x: vol_x,
        y: top.y,
        width: vol_w,
        height: 1,
    });

    if app.now_playing.is_some() {
        let heart = if app.now_playing_liked { "" } else { "" };
        let heart_w = display_width(heart) as u16;
        let heart_x = left_rect.x + left_rect.width.saturating_sub(heart_w);
        hits.heart = Some(HitRect {
            x: heart_x,
            y: top.y,
            width: heart_w,
            height: 1,
        });
    }

    if progress_w > 0 {
        let ratio = progress_ratio(position, duration);
        let filled = ((ratio * progress_w as f32).round() as u16).min(progress_w);

        let buffer_ratio = app.buffer_progress().and_then(|(downloaded, total)| {
            if total > 0 {
                Some((downloaded as f32 / total as f32).min(1.0))
            } else {
                None
            }
        });
        let buffer_filled = buffer_ratio
            .map(|r| ((r * progress_w as f32).round() as u16).min(progress_w))
            .unwrap_or(0);

        let mut spans = Vec::new();
        let played_style = with_bar_bg(
            Style::default()
                .fg(app.theme.color_accent2())
                .add_modifier(Modifier::BOLD),
        );
        let buffer_style = with_bar_bg(Style::default().fg(app.theme.color_accent()));
        let empty_track = with_bar_bg(Style::default().fg(app.theme.color_buff()));

        if buffer_filled == 0 {
            if filled > 0 {
                let played_len = filled.saturating_sub(1);
                if played_len > 0 {
                    spans.push(Span::styled("━".repeat(played_len as usize), played_style));
                }
                spans.push(Span::styled("●", played_style));
            }
            let remaining = progress_w.saturating_sub(filled);
            if remaining > 0 {
                spans.push(Span::styled("─".repeat(remaining as usize), empty_track));
            }
        } else {
            let played_len = filled.saturating_sub(1);
            if played_len > 0 {
                spans.push(Span::styled("━".repeat(played_len as usize), played_style));
            }
            if filled > 0 {
                spans.push(Span::styled("●", played_style));
            }

            let buffered_not_played = buffer_filled.saturating_sub(filled);
            if buffered_not_played > 0 {
                spans.push(Span::styled(
                    "─".repeat(buffered_not_played as usize),
                    buffer_style,
                ));
            }

            let unbuffered = progress_w.saturating_sub(buffer_filled);
            if unbuffered > 0 {
                spans.push(Span::styled("·".repeat(unbuffered as usize), empty_track));
            }
        }

        frame.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Left),
            progress_rect,
        );

        hits.progress = Some(HitRect {
            x: progress_rect.x,
            y: progress_rect.y,
            width: progress_rect.width,
            height: 1,
        });
    }

    frame.render_widget(
        Paragraph::new(time_text)
            .style(with_bar_bg(Style::default().fg(app.theme.color_subtext())))
            .alignment(Alignment::Right),
        time_rect,
    );

    app.set_player_bar_hits(hits);
}

fn progress_ratio(position: Duration, duration: Duration) -> f32 {
    if duration.as_millis() == 0 {
        return 0.0;
    }

    (position.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0)
}

fn format_mmss(value: Duration) -> String {
    let secs = value.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn clip_to_display_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > max_width {
            break;
        }
        out.push(ch);
        used += w;
    }
    out
}

fn playback_repeat_symbol(app: &App) -> &'static str {
    match app.playback_repeat_mode {
        crate::app::PlaybackRepeatMode::Sequence => "",
        crate::app::PlaybackRepeatMode::Shuffle => "",
        crate::app::PlaybackRepeatMode::LoopAll => "",
        crate::app::PlaybackRepeatMode::LoopOne => "",
    }
}

pub fn draw_volume_modal_overlay(frame: &mut Frame, app: &App) {
    let rect = app.volume_popover_rect();
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    let frame_area = frame.area();
    let area = Rect {
        x: rect.x.min(frame_area.width.saturating_sub(rect.width)),
        y: rect.y.min(frame_area.height.saturating_sub(rect.height)),
        width: rect.width.min(frame_area.width),
        height: rect.height.min(frame_area.height),
    };

    frame.render_widget(Clear, area);

    let vol = app.volume.clamp(0.0, 1.0);
    let vol_pct = (vol * 100.0).round() as i32;
    let is_muted = vol_pct == 0;
    let vol_icon = if is_muted {
        "󰝟"
    } else if vol_pct < 33 {
        "󰕿"
    } else if vol_pct < 66 {
        "󰖀"
    } else {
        "󰕾"
    };

    let bg = app.theme.color_surface();
    let accent_style = Style::default()
        .fg(app.theme.color_accent())
        .add_modifier(Modifier::BOLD)
        .bg(bg);
    let subtext_style = Style::default().fg(app.theme.color_subtext()).bg(bg);
    let text_style = Style::default()
        .fg(app.theme.color_text())
        .add_modifier(Modifier::BOLD)
        .bg(bg);

    let filled = (vol * 10.0).round() as usize;
    let filled_s = "█".repeat(filled.min(10));
    let empty_s = "░".repeat(10usize.saturating_sub(filled.min(10)));

    let line = Line::from(vec![
        Span::styled("[ ", subtext_style),
        Span::styled(
            vol_icon,
            if is_muted {
                subtext_style
            } else {
                accent_style
            },
        ),
        Span::styled(" ", Style::default().bg(bg)),
        Span::styled(filled_s, accent_style),
        Span::styled(empty_s, subtext_style),
        Span::styled(" ", Style::default().bg(bg)),
        Span::styled(format!("{vol_pct:>3}%"), text_style),
        Span::styled(" ]", subtext_style),
    ]);

    frame.render_widget(Paragraph::new(line).style(Style::default().bg(bg)), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_bar_hits_volume() {
        let mut hits = PlayerBarHitTargets::default();
        assert!(hits.volume.is_none());
        hits.volume = Some(HitRect {
            x: 50,
            y: 20,
            width: 9,
            height: 1,
        });
        let rect = hits.volume.unwrap();
        assert!(rect.contains(50, 20));
        assert!(rect.contains(58, 20));
        assert!(!rect.contains(59, 20));
        assert!(!rect.contains(49, 20));
        assert!(!rect.contains(50, 19));
    }

    #[test]
    fn test_volume_popover_geometry() {
        let btn = HitRect {
            x: 50,
            y: 20,
            width: 9,
            height: 1,
        };
        let popup_w: u16 = 21;
        let popup_h: u16 = 1;
        let center_x = btn.x + btn.width / 2;
        let x = center_x.saturating_sub(popup_w / 2).max(1);
        let y = btn.y.saturating_sub(popup_h + 1);

        assert_eq!(center_x, 54);
        assert_eq!(x, 44);
        assert_eq!(y, 18);
        assert_eq!(popup_w, 21);
        assert_eq!(popup_h, 1);
    }
}
