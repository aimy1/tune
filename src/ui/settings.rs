use crate::app::{App, Overlay};
use crate::data::config::{AudioQuality, BarChannels, BarNumber, Language, VisualizeMode};

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use unicode_width::UnicodeWidthStr;

pub fn draw_settings_modal(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    if matches!(app.overlay, Some(Overlay::SettingsAbout)) {
        crate::ui::about::draw_about_modal(frame, app, size);
        return;
    }

    let area = centered_rect(70, 20, size);

    frame.render_widget(Clear, area);

    let title = match app.overlay {
        Some(Overlay::Settings) => l(app, " 设置 ", " Settings "),
        Some(Overlay::SettingsTransparency) => l(app, " 透明设置 ", " Transparency Settings "),
        Some(Overlay::SettingsPlayback) => l(app, " 播放设置 ", " Playback Settings "),
        Some(Overlay::SettingsDesktopLyrics) => l(app, " 桌面歌词设置 ", " Desktop Lyrics Settings "),
        Some(Overlay::SettingsKeybinds) => l(app, " 按键绑定 ", " Keybinds "),
        Some(Overlay::SettingsAbout) => " about ",
        _ => l(app, " 设置 ", " Settings "),
    };

    let close_btn = if matches!(app.overlay, Some(Overlay::Settings)) {
        l(app, " [关闭 ×] ", " [Close ×] ")
    } else {
        l(app, " [返回 ‹] ", " [Back ‹] ")
    };

    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(title)
            .title(
                Line::from(Span::styled(
                    close_btn,
                    Style::default().fg(app.theme.color_subtext()),
                ))
                .alignment(Alignment::Right),
            )
            .border_style(
                Style::default()
                    .fg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD),
            )
            .style(base_bg_style(app)),
        area,
    );

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    match app.overlay {
        Some(Overlay::SettingsTransparency) => draw_transparency_settings(frame, app, area, inner),
        Some(Overlay::SettingsPlayback) => draw_playback_settings(frame, app, area, inner),
        Some(Overlay::SettingsDesktopLyrics) => draw_desktop_lyrics_settings(frame, app, area, inner),
        Some(Overlay::SettingsKeybinds) => draw_keybind_settings(frame, app, area, inner),
        _ => draw_root_settings(frame, app, area, inner),
    }
}

fn draw_root_settings(frame: &mut Frame, app: &mut App, area: Rect, inner: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    app.settings_modal_hits = crate::app::SettingsModalHits {
        modal_area: Some(crate::app::HitRect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        }),
        list_area: Some(crate::app::HitRect {
            x: rows[1].x,
            y: rows[1].y,
            width: rows[1].width,
            height: rows[1].height,
        }),
        footer_area: Some(crate::app::HitRect {
            x: rows[2].x,
            y: rows[2].y,
            width: rows[2].width,
            height: rows[2].height,
        }),
        list_scroll: 0,
        list_count: crate::app::SETTINGS_ROOT_ITEMS,
    };
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(app.theme.color_surface())),
        rows[0],
    );

    let raw_items = vec![
        (l(app, "主题", "Theme"), app.config.theme.clone()),
        (
            l(app, "透明设置", "Transparency Settings"),
            "...".to_string(),
        ),
        (
            l(app, "语言", "Language"),
            match app.config.language {
                Language::Zh => l(app, "中文", "Chinese").to_string(),
                Language::En => "English".to_string(),
            },
        ),
        (
            l(app, "图像协议", "Image Protocol"),
            app.config.graphics_protocol.display_name().to_string(),
        ),
        (
            l(app, "播放设置", "Playback Settings"),
            "...".to_string(),
        ),
        (
            l(app, "桌面歌词设置", "Desktop Lyrics Settings"),
            "...".to_string(),
        ),
        (
            l(app, "按键绑定", "Keybinds"),
            "...".to_string(),
        ),
        (
            l(app, "鼠标支持", "Mouse Support"),
            on_off(app, app.config.mouse_support).to_string(),
        ),
        (
            l(app, "显示提示", "Show Hints"),
            on_off(app, app.config.show_hints).to_string(),
        ),
        (
            l(app, "主页更多推荐", "More Home Recommendations"),
            on_off(app, app.config.home_more_recommend).to_string(),
        ),
        (
            l(app, "退出登录", "Logout"),
            "".to_string(),
        ),
        (
            l(app, "关于", "About"),
            "".to_string(),
        ),
    ];

    let lines: Vec<Line> = raw_items
        .iter()
        .enumerate()
        .map(|(idx, (key, val))| {
            let selected = idx == app.settings_selected;
            let prefix = if selected { "› " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(app.theme.color_base())
                    .bg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(app.theme.color_text())
                    .bg(app.theme.color_surface())
            };
            let line_str = format_setting_line(prefix, key, val, inner.width);
            Line::from(Span::styled(line_str, style))
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(app.theme.color_surface())),
        rows[1],
    );

    let footer_text = l(
        app,
        "  ↑/k ↓/j: 导航  ←/h →/l: 调节/进入  Enter: 确认  Esc/t: 关闭",
        "  ↑/k ↓/j: Navigate  ←/h →/l: Adjust/Enter  Enter: Confirm  Esc/t: Close",
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(app.theme.color_subtext()),
        )))
        .style(Style::default().bg(app.theme.color_surface())),
        rows[2],
    );
}

fn draw_transparency_settings(frame: &mut Frame, app: &mut App, area: Rect, inner: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    app.settings_modal_hits = crate::app::SettingsModalHits {
        modal_area: Some(crate::app::HitRect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        }),
        list_area: Some(crate::app::HitRect {
            x: rows[1].x,
            y: rows[1].y,
            width: rows[1].width,
            height: rows[1].height,
        }),
        footer_area: Some(crate::app::HitRect {
            x: rows[2].x,
            y: rows[2].y,
            width: rows[2].width,
            height: rows[2].height,
        }),
        list_scroll: 0,
        list_count: crate::app::SETTINGS_TRANSPARENCY_ITEMS,
    };
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(app.theme.color_surface())),
        rows[0],
    );

    let raw_items = vec![
        (
            l(app, "背景透明", "Transparent Background"),
            on_off(app, app.config.transparent_background).to_string(),
        ),
        (
            l(app, "个人中心透明", "Personal Center Transparent"),
            on_off(app, app.config.transparent_sidebar).to_string(),
        ),
        (
            l(app, "桌面歌词背景样式", "Desktop Lyrics Background Style"),
            app.config
                .desktop_lyrics_bg
                .display_name(app.config.language)
                .to_string(),
        ),
        (
            l(app, "桌面歌词背景透明度", "Desktop Lyrics Background Opacity"),
            format!("{}%", app.config.desktop_lyrics_opacity),
        ),
    ];

    let lines: Vec<Line> = raw_items
        .iter()
        .enumerate()
        .map(|(idx, (key, val))| {
            let selected = idx == app.settings_transparency_selected;
            let prefix = if selected { "› " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(app.theme.color_base())
                    .bg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(app.theme.color_text())
                    .bg(app.theme.color_surface())
            };
            let line_str = format_setting_line(prefix, key, val, inner.width);
            Line::from(Span::styled(line_str, style))
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(app.theme.color_surface())),
        rows[1],
    );

    let footer_text = l(
        app,
        "  ↑/k ↓/j: 导航  ←/h →/l: 调节  Enter: 切换  Esc/t: 返回上一级",
        "  ↑/k ↓/j: Navigate  ←/h →/l: Adjust  Enter: Toggle  Esc/t: Back",
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(app.theme.color_subtext()),
        )))
        .style(Style::default().bg(app.theme.color_surface())),
        rows[2],
    );
}

fn draw_playback_settings(frame: &mut Frame, app: &mut App, area: Rect, inner: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    app.settings_modal_hits = crate::app::SettingsModalHits {
        modal_area: Some(crate::app::HitRect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        }),
        list_area: Some(crate::app::HitRect {
            x: rows[1].x,
            y: rows[1].y,
            width: rows[1].width,
            height: rows[1].height,
        }),
        footer_area: Some(crate::app::HitRect {
            x: rows[2].x,
            y: rows[2].y,
            width: rows[2].width,
            height: rows[2].height,
        }),
        list_scroll: 0,
        list_count: crate::app::SETTINGS_PLAYBACK_ITEMS,
    };
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(app.theme.color_surface())),
        rows[0],
    );

    let bar_number = match app.config.bar_number {
        BarNumber::Auto => l(app, "自动", "Auto"),
        BarNumber::N16 => "16",
        BarNumber::N32 => "32",
        BarNumber::N48 => "48",
        BarNumber::N64 => "64",
        BarNumber::N80 => "80",
        BarNumber::N96 => "96",
    };

    let channels = match app.config.bar_channels {
        BarChannels::Mono => "Mono",
        BarChannels::Stereo => "Stereo",
    };

    let raw_items = vec![
        (
            l(app, "可视化", "Visualization"),
            match app.config.visualize {
                VisualizeMode::Off => l(app, "关闭", "Off").to_string(),
                VisualizeMode::Bars => l(app, "柱状频谱", "Bars").to_string(),
                VisualizeMode::Oscilloscope => l(app, "示波波形", "Oscilloscope").to_string(),
                VisualizeMode::Circle => l(app, "环形频谱", "Circle Spectrum").to_string(),
                VisualizeMode::Particles => l(app, "电光粒子", "Particle Wave").to_string(),
                VisualizeMode::Mirror => l(app, "双向镜像", "Symmetric Mirror").to_string(),
            }
        ),
        (
            l(app, "超级流畅", "Super Smooth"),
            on_off(app, app.config.super_smooth_bar).to_string()
        ),
        (
            l(app, "频谱间隔", "Bars Gap"),
            on_off(app, app.config.bars_gap).to_string()
        ),
        (l(app, "频谱数", "Bars Count"), bar_number.to_string()),
        (l(app, "声道", "Channels"), channels.to_string()),
        (
            l(app, "封面边框", "Cover Border"),
            on_off(app, app.config.album_border).to_string()
        ),
        (
            l(app, "页面歌词", "Page Lyrics"),
            on_off(app, app.config.page_lyrics).to_string()
        ),
        (
            l(app, "音质", "Audio Quality"),
            audio_quality_label(app, app.config.audio_quality).to_string()
        ),
        (
            l(app, "播放记忆", "Playback Memory"),
            on_off(app, app.config.playback_memory).to_string()
        ),
    ];

    let lines: Vec<Line> = raw_items
        .iter()
        .enumerate()
        .map(|(idx, (key, val))| {
            let selected = idx == app.settings_playback_selected;
            let disabled = idx == 0 && !crate::tmplayer::audio::cava::is_available();
            let prefix = if selected { "› " } else { "  " };
            let style = if disabled {
                Style::default()
                    .fg(app.theme.color_subtext())
                    .bg(app.theme.color_surface())
            } else if selected {
                Style::default()
                    .fg(app.theme.color_base())
                    .bg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(app.theme.color_text())
                    .bg(app.theme.color_surface())
            };
            let line_str = format_setting_line(prefix, key, val, inner.width);
            Line::from(Span::styled(line_str, style))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(app.theme.color_surface())),
        rows[1],
    );

    let footer_text = l(
        app,
        "  ↑/k ↓/j: 导航  ←/h →/l: 调节  Enter: 切换/进入  Esc: 返回上一级",
        "  ↑/k ↓/j: Navigate  ←/h →/l: Adjust  Enter: Toggle/Enter  Esc: Back",
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(app.theme.color_subtext()),
        )))
        .style(Style::default().bg(app.theme.color_surface())),
        rows[2],
    );
}

fn draw_desktop_lyrics_settings(frame: &mut Frame, app: &mut App, area: Rect, inner: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    app.settings_modal_hits = crate::app::SettingsModalHits {
        modal_area: Some(crate::app::HitRect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        }),
        list_area: Some(crate::app::HitRect {
            x: rows[1].x,
            y: rows[1].y,
            width: rows[1].width,
            height: rows[1].height,
        }),
        footer_area: Some(crate::app::HitRect {
            x: rows[2].x,
            y: rows[2].y,
            width: rows[2].width,
            height: rows[2].height,
        }),
        list_scroll: 0,
        list_count: crate::app::SETTINGS_DESKTOP_LYRICS_ITEMS,
    };
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(app.theme.color_surface())),
        rows[0],
    );

    let locked_str = if app.config.desktop_lyrics_locked {
        l(app, "锁定 (不可拖动)", "Locked (No Drag)")
    } else {
        l(app, "解锁 (可拖动)", "Unlocked (Draggable)")
    };

    let dual_line_str = if app.config.desktop_lyrics_dual_line {
        l(app, "双行歌词", "Dual Lines")
    } else {
        l(app, "单行歌词", "Single Line")
    };

    let align_str = app.config.desktop_lyrics_align.display_name(app.config.language);
    let bg_str = app.config.desktop_lyrics_bg.display_name(app.config.language);

    let pos_str = if let (Some(x), Some(y)) = (
        app.config.desktop_lyrics_pos_x,
        app.config.desktop_lyrics_pos_y,
    ) {
        format!("{}, {}", x, y)
    } else {
        l(app, "底部居中 (默认)", "Bottom Center (Default)").to_string()
    };

    let width_str = app.config.desktop_lyrics_width.display_name(app.config.language);
    let opacity_str = format!("{}%", app.config.desktop_lyrics_opacity);

    let raw_items = vec![
        (
            l(app, "桌面歌词开关", "Desktop Lyrics"),
            on_off(app, app.config.desktop_lyrics).to_string(),
        ),
        (
            l(app, "锁定位置", "Lock Position"),
            locked_str.to_string(),
        ),
        (
            l(app, "字体大小", "Font Size"),
            format!("{}px", app.config.desktop_lyrics_font_size),
        ),
        (
            l(app, "窗口宽度", "Window Width"),
            width_str.to_string(),
        ),
        (
            l(app, "歌词行数", "Lyrics Lines"),
            dual_line_str.to_string(),
        ),
        (
            l(app, "歌词对齐", "Lyrics Alignment"),
            align_str.to_string(),
        ),
        (
            l(app, "背景样式", "Background Style"),
            bg_str.to_string(),
        ),
        (
            l(app, "背景透明度", "Background Opacity"),
            opacity_str,
        ),
        (
            l(app, "恢复默认位置", "Reset Position"),
            pos_str,
        ),
    ];

    let lines: Vec<Line> = raw_items
        .iter()
        .enumerate()
        .map(|(idx, (key, val))| {
            let selected = idx == app.settings_desktop_lyrics_selected;
            let prefix = if selected { "› " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(app.theme.color_base())
                    .bg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(app.theme.color_text())
                    .bg(app.theme.color_surface())
            };
            let line_str = format_setting_line(prefix, key, val, inner.width);
            Line::from(Span::styled(line_str, style))
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(app.theme.color_surface())),
        rows[1],
    );

    let footer_text = l(
        app,
        "  ↑/k ↓/j: 导航  ←/h →/l: 调节  Enter: 确认/重置  Esc: 返回上一级",
        "  ↑/k ↓/j: Navigate  ←/h →/l: Adjust  Enter: Confirm/Reset  Esc: Back",
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(app.theme.color_subtext()),
        )))
        .style(Style::default().bg(app.theme.color_surface())),
        rows[2],
    );
}

fn draw_keybind_settings(frame: &mut Frame, app: &mut App, area: Rect, inner: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(inner);
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(app.theme.color_surface())),
        rows[0],
    );

    let mut lines: Vec<Line> = (0..crate::app::SETTINGS_KEYBIND_ITEMS)
        .map(|idx| {
            let is_rebinding = app.settings_keybind_rebinding == Some(idx);
            let selected = idx == app.settings_keybind_selected;
            let prefix = if is_rebinding || selected {
                "› "
            } else {
                "  "
            };
            let style = if is_rebinding {
                Style::default()
                    .fg(app.theme.color_base())
                    .bg(app.theme.color_accent3())
                    .add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default()
                    .fg(app.theme.color_base())
                    .bg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(app.theme.color_text())
                    .bg(app.theme.color_surface())
            };

            let full_label = app.keybind_label_for_index(idx);
            let (key, val) = if let Some(pos) = full_label.find(": ") {
                (full_label[..pos].to_string(), full_label[pos + 2..].to_string())
            } else {
                (full_label, "".to_string())
            };

            let mut val_str = val;
            if is_rebinding {
                val_str.push_str(l(app, "  [等待输入]", "  [Waiting Input]"));
            }

            let line_str = format_setting_line(prefix, &key, &val_str, inner.width);
            Line::from(Span::styled(line_str, style))
        })
        .collect();

    lines.push(Line::from(Span::styled(
        format!(
            "  {}",
            l(
                app,
                "个人中心分区切换（Left/Right 或 滚动到边界）",
                "Personal Center Section Switch (Left/Right or Scroll Boundary)",
            )
        ),
        Style::default().fg(app.theme.color_subtext()),
    )));
    lines.push(Line::from(Span::styled(
        format!(
            "  {}",
            l(app, "按键绑定弹窗（Ctrl+K）", "Open Keybinds (Ctrl+K)")
        ),
        Style::default().fg(app.theme.color_subtext()),
    )));
    lines.push(Line::from(Span::styled(
        format!(
            "  {}",
            l(
                app,
                "重置快捷键（Ctrl+Alt+R）",
                "Reset Keybinds (Ctrl+Alt+R)"
            )
        ),
        Style::default().fg(app.theme.color_subtext()),
    )));

    let focus_index = app
        .settings_keybind_rebinding
        .unwrap_or(app.settings_keybind_selected);
    let visible_rows = rows[1].height as usize;
    let total_rows = lines.len();
    let max_scroll = total_rows.saturating_sub(visible_rows);
    let scroll = if visible_rows == 0 || focus_index < visible_rows {
        0
    } else {
        (focus_index + 1 - visible_rows).min(max_scroll)
    };

    app.settings_modal_hits = crate::app::SettingsModalHits {
        modal_area: Some(crate::app::HitRect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        }),
        list_area: Some(crate::app::HitRect {
            x: rows[1].x,
            y: rows[1].y,
            width: rows[1].width,
            height: rows[1].height,
        }),
        footer_area: Some(crate::app::HitRect {
            x: rows[2].x,
            y: rows[2].y,
            width: rows[2].width,
            height: rows[2].height,
        }),
        list_scroll: scroll,
        list_count: total_rows,
    };

    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(app.theme.color_surface()))
            .scroll((scroll as u16, 0)),
        rows[1],
    );

    let hint = if let Some(index) = app.settings_keybind_rebinding {
        format!(
            "{}: {}  {}",
            l(app, "正在重绑", "Rebinding"),
            app.keybind_label_for_index(index),
            l(
                app,
                "按下新快捷键，Esc 取消",
                "Press a new shortcut, Esc to cancel"
            )
        )
    } else {
        l(
            app,
            "Enter 重绑  Ctrl+Alt+R 重置  Esc 返回",
            "Enter rebind  Ctrl+Alt+R reset  Esc back",
        )
        .to_string()
    };

    frame.render_widget(
        Paragraph::new(hint)
            .style(
                Style::default()
                    .fg(app.theme.color_subtext())
                    .bg(app.theme.color_surface()),
            )
            .wrap(ratatui::widgets::Wrap { trim: true }),
        rows[2],
    );
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width.saturating_sub(2)).max(12);
    let h = height.min(area.height.saturating_sub(2)).max(5);
    Rect {
        x: area.x + area.width.saturating_sub(w) / 2,
        y: area.y + area.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    }
}

fn base_bg_style(app: &App) -> Style {
    Style::default()
        .fg(app.theme.color_subtext())
        .bg(app.theme.color_surface())
}

fn audio_quality_label(app: &App, quality: AudioQuality) -> &'static str {
    match app.config.language {
        Language::Zh => match quality {
            AudioQuality::Standard => "标准",
            AudioQuality::Higher => "较高",
            AudioQuality::Exhigh => "极高",
            AudioQuality::Lossless => "无损",
            AudioQuality::Hires => "Hi-Res",
            AudioQuality::Jyeffect => "高清环绕声",
            AudioQuality::Sky => "沉浸环绕声",
            AudioQuality::Dolby => "杜比全景声",
            AudioQuality::Jymaster => "超清母带",
        },
        Language::En => match quality {
            AudioQuality::Standard => "Standard",
            AudioQuality::Higher => "Higher",
            AudioQuality::Exhigh => "Exhigh",
            AudioQuality::Lossless => "Lossless",
            AudioQuality::Hires => "Hi-Res",
            AudioQuality::Jyeffect => "JYEffect",
            AudioQuality::Sky => "Sky",
            AudioQuality::Dolby => "Dolby",
            AudioQuality::Jymaster => "JYMaster",
        },
    }
}

fn l<'a>(app: &App, zh: &'a str, en: &'a str) -> &'a str {
    match app.config.language {
        Language::Zh => zh,
        Language::En => en,
    }
}

fn on_off(app: &App, enabled: bool) -> &'static str {
    match app.config.language {
        Language::Zh => {
            if enabled {
                "开"
            } else {
                "关"
            }
        }
        Language::En => {
            if enabled {
                "On"
            } else {
                "Off"
            }
        }
    }
}

fn format_setting_line(prefix: &str, key: &str, val: &str, width: u16) -> String {
    let prefix_w = UnicodeWidthStr::width(prefix);
    let key_w = UnicodeWidthStr::width(key);
    let val_w = UnicodeWidthStr::width(val);
    
    let padding_budget = (width as usize).saturating_sub(prefix_w + key_w + val_w + 2);
    let pad = " ".repeat(padding_budget);
    format!(" {prefix}{key}{pad}{val} ")
}
