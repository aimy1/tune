use crate::tmplayer::app::state::{AppState, Overlay, PlaybackState, RepeatMode};
use crate::tmplayer::utils::input::Action;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

fn get_control_strings(app: &AppState) -> (&'static str, String, &'static str, String, String) {
    let play = match app.player.playback {
        PlaybackState::Playing => "[]",
        _ => "[]",
    };
    let repeat_symbol = app.player.repeat_mode.symbol();

    let vol_pct = (app.player.volume * 100.0).round() as i32;
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

    let s_prev = "[] ";
    let s_play = format!("{play} ");
    let s_next = "[]  ";
    let s_mode = format!("{repeat_symbol}  ");
    let s_vol = format!("[{vol_icon} {vol_pct}%]");

    (s_prev, s_play, s_next, s_mode, s_vol)
}

pub fn volume_button_rect(area: Rect, app: &AppState) -> Rect {
    let (s_prev, s_play, s_next, s_mode, s_vol) = get_control_strings(app);
    let label = format!("{s_prev}{s_play}{s_next}{s_mode}{s_vol}");

    let text_w = UnicodeWidthStr::width(label.as_str()) as u16;
    if text_w == 0 || area.width == 0 {
        return Rect::default();
    }

    let start_x = area.x + area.width.saturating_sub(text_w) / 2;
    let w_prev = UnicodeWidthStr::width(s_prev) as u16;
    let w_play = UnicodeWidthStr::width(s_play.as_str()) as u16;
    let w_next = UnicodeWidthStr::width(s_next) as u16;
    let w_mode = UnicodeWidthStr::width(s_mode.as_str()) as u16;
    let w_vol = UnicodeWidthStr::width(s_vol.as_str()) as u16;

    let vol_x = start_x + w_prev + w_play + w_next + w_mode;
    Rect {
        x: vol_x,
        y: area.y,
        width: w_vol,
        height: 1,
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &AppState) {
    let (s_prev, s_play, s_next, s_mode, s_vol) = get_control_strings(app);

    let play_style = match app.player.playback {
        PlaybackState::Playing => Style::default()
            .fg(app.theme.color_accent2())
            .add_modifier(Modifier::BOLD),
        _ => Style::default()
            .fg(app.theme.color_accent())
            .add_modifier(Modifier::BOLD),
    };

    let repeat_style = if app.player.repeat_mode == RepeatMode::Sequence {
        Style::default().fg(app.theme.color_subtext())
    } else {
        Style::default()
            .fg(app.theme.color_accent())
            .add_modifier(Modifier::BOLD)
    };

    let is_vol_open = app.overlay == Overlay::VolumeModal;
    let vol_pct = (app.player.volume * 100.0).round() as i32;
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

    let mut spans = vec![
        Span::styled(s_prev, Style::default().fg(app.theme.color_text())),
        Span::styled(s_play, play_style),
        Span::styled(s_next, Style::default().fg(app.theme.color_text())),
        Span::styled(s_mode, repeat_style),
    ];

    if is_vol_open {
        // Active button styling: accent background
        spans.push(Span::styled(
            s_vol,
            Style::default()
                .fg(app.theme.color_base())
                .bg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled("[", Style::default().fg(app.theme.color_subtext())));
        spans.push(Span::styled(
            format!("{vol_icon} "),
            Style::default().fg(if is_muted {
                app.theme.color_subtext()
            } else {
                app.theme.color_accent()
            }),
        ));
        spans.push(Span::styled(
            format!("{vol_pct}%"),
            Style::default().fg(app.theme.color_text()),
        ));
        spans.push(Span::styled("]", Style::default().fg(app.theme.color_subtext())));
    }

    f.render_widget(
        Paragraph::new(Line::from(spans))
            .style(Style::default())
            .alignment(ratatui::layout::Alignment::Center),
        area,
    );
}

pub fn hit_test(area: Rect, app: &AppState, col: u16, row: u16) -> Option<Action> {
    if row < area.y || row >= area.y + area.height {
        return None;
    }

    let (s_prev, s_play, s_next, s_mode, s_vol) = get_control_strings(app);
    let label = format!("{s_prev}{s_play}{s_next}{s_mode}{s_vol}");

    let text_w = UnicodeWidthStr::width(label.as_str()) as u16;
    if text_w == 0 || area.width == 0 {
        return None;
    }

    let start_x = area.x + area.width.saturating_sub(text_w) / 2;
    let mut x = start_x;

    let w_prev = UnicodeWidthStr::width(s_prev) as u16;
    if col >= x && col < x + w_prev {
        return Some(Action::Prev);
    }
    x += w_prev;

    let w_play = UnicodeWidthStr::width(s_play.as_str()) as u16;
    if col >= x && col < x + w_play {
        return Some(Action::TogglePlayPause);
    }
    x += w_play;

    let w_next = UnicodeWidthStr::width(s_next) as u16;
    if col >= x && col < x + w_next {
        return Some(Action::Next);
    }
    x += w_next;

    let w_mode = UnicodeWidthStr::width(s_mode.as_str()) as u16;
    if col >= x && col < x + w_mode {
        return Some(Action::ToggleRepeatMode);
    }
    x += w_mode;

    let w_vol = UnicodeWidthStr::width(s_vol.as_str()) as u16;
    if col >= x && col < x + w_vol {
        if app.overlay == Overlay::VolumeModal {
            return Some(Action::CloseOverlay);
        } else {
            return Some(Action::OpenVolumeModal);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_button_rect_and_hit_test() {
        let app = AppState::new(
            crate::tmplayer::data::config::Config::default(),
            crate::tmplayer::ui::theme::Theme::default(),
            crate::data::config::Language::Zh,
        );
        let area = Rect { x: 0, y: 10, width: 40, height: 1 };
        let vol_rect = volume_button_rect(area, &app);
        assert_eq!(vol_rect.y, 10);
        assert!(vol_rect.width > 0);
        assert!(vol_rect.x > 0);

        // Clicking on volume button triggers OpenVolumeModal
        let act = hit_test(area, &app, vol_rect.x + 1, 10);
        assert_eq!(act, Some(Action::OpenVolumeModal));
    }
}
