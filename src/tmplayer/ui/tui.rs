use crate::data::config::Language;
use crate::tmplayer::app::state::{AppState, Overlay};
use crate::tmplayer::render::graphics_overlay::GraphicsOverlay;
use crate::tmplayer::ui::components::control_buttons;
use crate::tmplayer::ui::panels::{info_panel, playlist_panel, visual_panel};
use crate::tmplayer::utils::input::Action;
use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, terminal};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::io::{self, Stdout};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default, Clone, Copy)]
pub struct UiLayout {
    pub full: Rect,
    pub left: Rect,
    pub right: Rect,
    pub left_width: u16,

    pub info_progress: Rect,
    pub info_volume: Rect,
    pub info_controls: Rect,
    pub info_heart: Rect,

    pub info_cover_image: Rect,

    pub playlist_rect: Rect,
    pub playlist_inner: Rect,
    pub playlist_list_inner: Rect,

    pub playlist_cover_image: Rect,

    pub spectrum_rect: Rect,
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    pub should_quit: bool,
    graphics_overlay: GraphicsOverlay,
}

impl Tui {
    pub fn new(app: &AppState) -> Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            should_quit: false,
            graphics_overlay: GraphicsOverlay::new(app.config.graphics_protocol),
        })
    }

    pub fn enter(&mut self, mouse_support: bool) -> Result<()> {
        if mouse_support {
            execute!(
                io::stdout(),
                EnterAlternateScreen,
                event::EnableMouseCapture
            )?;
        } else {
            execute!(io::stdout(), EnterAlternateScreen)?;
        }
        terminal::enable_raw_mode()?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            io::stdout(),
            event::DisableMouseCapture,
            LeaveAlternateScreen
        )?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.terminal.clear()?;
        Ok(())
    }

    /// Terminal resize can clear/lose kitty graphic placements. Mark placements dirty so
    /// the next draw will re-place images.
    pub fn on_resize(&mut self) {
        let _ = self.terminal.clear();
    }

    pub fn draw(&mut self, app: &mut AppState) -> Result<UiLayout> {
        if app.toast.as_ref().map(|(m, _)| m.as_str()) == Some("Bye") {
            self.should_quit = true;
        }

        let mut layout_out = UiLayout::default();

        self.terminal.draw(|f| {
            let size = f.area();
            layout_out.full = size;

            // small terminal: keep stable, hide secondary panels
            if size.width < 50 || size.height < 12 {
                f.render_widget(ratatui::widgets::Clear, size);

                let mut base_style = Style::default().fg(app.theme.color_text());
                if !app.config.transparent_background {
                    base_style = base_style.bg(app.theme.color_base());
                }
                f.render_widget(ratatui::widgets::Block::default().style(base_style), size);
                f.render_widget(
                    ratatui::widgets::Paragraph::new(lang_text(
                        app,
                        "终端窗口过小",
                        "Terminal too small",
                    ))
                    .style(Style::default().fg(app.theme.color_subtext())),
                    size,
                );
                return;
            }

            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
                .split(size);
            layout_out.left = cols[0];
            layout_out.right = cols[1];
            layout_out.left_width = cols[0].width;

            // right: lyrics (10%) + spectrum (rest)
            let lyric_h = ((cols[1].height as f32) * 0.10).round() as u16;
            let lyric_h = lyric_h.clamp(3, cols[1].height.saturating_sub(6));
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(lyric_h), Constraint::Min(1)])
                .split(cols[1]);

            // Mirror visual panel inner layout for auto bar count.
            let outer = Rect {
                x: rows[0].x,
                y: rows[0].y,
                width: rows[0].width,
                height: rows[0].height.saturating_add(rows[1].height),
            };
            let inner = outer.inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            });
            let lyric_h_inner = rows[0].height.saturating_sub(2).min(inner.height);
            layout_out.spectrum_rect = Rect {
                x: inner.x,
                y: inner.y + lyric_h_inner,
                width: inner.width,
                height: inner.height.saturating_sub(lyric_h_inner),
            };

            let info_l = info_panel::layout(cols[0]);
            layout_out.info_progress = info_l.progress;
            layout_out.info_controls = info_l.controls;
            layout_out.info_volume = control_buttons::volume_button_rect(info_l.controls, app);
            layout_out.info_heart = info_l.heart;

            // For kitty graphics, we draw into the inner area (optional border).
            layout_out.info_cover_image = info_l.cover.inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            });

            // base styling
            f.render_widget(ratatui::widgets::Clear, size);

            let mut base_style = Style::default().fg(app.theme.color_text());
            if !app.config.transparent_background {
                base_style = base_style.bg(app.theme.color_base());
            }
            f.render_widget(ratatui::widgets::Block::default().style(base_style), size);

            info_panel::render(f, cols[0], app);
            visual_panel::render(f, rows[0], rows[1], app);

            // playlist overlay slides in/out over left
            if app.overlay == Overlay::Playlist
                || app.playlist_slide_x != app.playlist_slide_target_x
            {
                let collapsing = app.overlay != Overlay::Playlist
                    && app.playlist_slide_x > app.playlist_slide_target_x;

                // advance animation
                let step: i16 = 4;
                if app.playlist_slide_x < app.playlist_slide_target_x {
                    app.playlist_slide_x =
                        (app.playlist_slide_x + step).min(app.playlist_slide_target_x);
                } else if app.playlist_slide_x > app.playlist_slide_target_x {
                    app.playlist_slide_x =
                        (app.playlist_slide_x - step).max(app.playlist_slide_target_x);
                }

                // Slide effect via visible width growth/shrink (x stays at left edge)
                let full_w = cols[0].width as i16;
                let visible_w = (full_w + app.playlist_slide_x).clamp(0, full_w) as u16;
                if visible_w > 0 {
                    let r = Rect {
                        x: cols[0].x,
                        y: cols[0].y,
                        width: visible_w,
                        height: cols[0].height,
                    };
                    layout_out.playlist_rect = r;

                    if collapsing {
                        // Closing animation only needs the panel shell; skip expensive list/cover rendering.
                        f.render_widget(ratatui::widgets::Clear, r);
                        f.render_widget(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_set(crate::tmplayer::ui::borders::SOLID_BORDER)
                                .style(
                                    Style::default()
                                        .fg(app.theme.color_subtext())
                                        .bg(app.theme.color_surface()),
                                ),
                            r,
                        );
                    } else {
                        let pl_layout = playlist_panel::compute_layout(r, app);
                        layout_out.playlist_inner = pl_layout.inner;
                        layout_out.playlist_list_inner = pl_layout.list_inner;
                        layout_out.playlist_cover_image = pl_layout.cover_rect;
                        playlist_panel::render(f, r, app);
                    }
                }
            }

            // folder input overlay (simple one-line prompt)
            if app.overlay == Overlay::FolderInput {
                let prompt = format!(
                    "{}: {}",
                    lang_text(app, "文件夹", "Folder"),
                    app.folder_input.buf
                );
                let area = Rect {
                    x: size.x,
                    y: size.y + size.height.saturating_sub(2),
                    width: size.width,
                    height: 1,
                };
                f.render_widget(
                    ratatui::widgets::Paragraph::new(prompt).style(
                        Style::default()
                            .fg(app.theme.color_text())
                            .bg(app.theme.color_surface()),
                    ),
                    area,
                );
            }

            // toast
            if let Some((msg, _)) = &app.toast {
                let area = Rect {
                    x: size.x,
                    y: size.y,
                    width: size.width,
                    height: 1,
                };
                f.render_widget(
                    ratatui::widgets::Paragraph::new(msg.as_str())
                        .style(Style::default().fg(app.theme.color_accent3())),
                    area,
                );
            }



            // Paint kitty images on top of ratatui widgets.
            Self::paint_kitty_images(&mut self.graphics_overlay, f, app, &layout_out);

            // modals (top-most)
            match app.overlay {
                Overlay::SettingsModal => render_settings_modal(f, size, app),
                Overlay::TransparencySettingsModal => {
                    render_transparency_settings_modal(f, size, app)
                }
                Overlay::BarSettingsModal => render_bar_settings_modal(f, size, app),
                Overlay::DesktopLyricsSettingsModal => {
                    render_desktop_lyrics_settings_modal(f, size, app)
                }
                Overlay::LocalAudioSettingsModal => render_local_audio_settings_modal(f, size, app),
                Overlay::AboutModal => render_about_modal(f, size, app),
                Overlay::AcoustIdModal => render_acoustid_modal(f, size, app),
                Overlay::HelpModal => render_help_modal(f, size, app),
                Overlay::EqModal => render_eq_modal(f, size, app),
                Overlay::VolumeModal => render_volume_modal(f, size, app),
                _ => {}
            }
        })?;

        Ok(layout_out)
    }

    fn paint_kitty_images(
        graphics_overlay: &mut GraphicsOverlay,
        f: &mut ratatui::Frame<'_>,
        app: &mut AppState,
        layout: &UiLayout,
    ) {
        let playlist_overlay_visible = app.overlay == Overlay::Playlist
            || app.playlist_slide_x != app.playlist_slide_target_x
            || layout.playlist_rect.width > 0;

        let info_cover_bytes = if playlist_overlay_visible {
            None
        } else {
            app.player.track.cover.as_deref()
        };

        let playlist_fully_expanded = app.overlay == Overlay::Playlist
            && app.playlist_slide_x == 0
            && app.playlist_slide_target_x == 0;

        let playlist_cover_bytes = if playlist_fully_expanded {
            app.local_view_album_cover.as_deref()
        } else {
            None
        };

        let info_rect = if layout.info_cover_image.width > 0 && layout.info_cover_image.height > 0 {
            Some(layout.info_cover_image)
        } else {
            None
        };

        let playlist_rect =
            if layout.playlist_cover_image.width > 0 && layout.playlist_cover_image.height > 0 {
                Some(layout.playlist_cover_image)
            } else {
                None
            };

        graphics_overlay.paint(
            app,
            f,
            info_cover_bytes,
            playlist_cover_bytes,
            info_rect,
            playlist_rect,
        );
    }
}

fn centered_rect(size: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(size.width.saturating_sub(4)).max(10);
    let h = height.min(size.height.saturating_sub(4)).max(6);
    Rect {
        x: size.x + (size.width.saturating_sub(w)) / 2,
        y: size.y + (size.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
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

fn base_bg_style(app: &AppState) -> Style {
    Style::default()
        .fg(app.theme.color_subtext())
        .bg(app.theme.color_surface())
}

fn render_settings_modal(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    let area = centered_rect(size, 70, 20);
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(lang_text(app, " 设置 ", " Settings "))
        .title(
            Line::from(Span::styled(
                lang_text(app, " [关闭 ×] ", " [Close ×] "),
                Style::default().fg(app.theme.color_subtext()),
            ))
            .alignment(ratatui::layout::Alignment::Right),
        )
        .border_style(
            Style::default()
                .fg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(base_bg_style(app));
    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(app.theme.color_surface())),
        rows[0],
    );

    let language_label = match app.language {
        crate::data::config::Language::Zh => lang_text(app, "中文", "Chinese"),
        crate::data::config::Language::En => "English",
    };

    let raw_items = vec![
        (
            lang_text(app, "主题", "Theme"),
            app.config.theme.clone(),
        ),
        (
            lang_text(app, "透明设置", "Transparency Settings"),
            "...".to_string(),
        ),
        (
            lang_text(app, "语言", "Language"),
            language_label.to_string(),
        ),
        (
            lang_text(app, "图像协议", "Image Protocol"),
            app.config.graphics_protocol.display_name().to_string(),
        ),
        (
            lang_text(app, "播放设置", "Playback Settings"),
            "...".to_string(),
        ),
        (
            lang_text(app, "桌面歌词设置", "Desktop Lyrics Settings"),
            "...".to_string(),
        ),
        (
            lang_text(app, "按键绑定", "Keybinds"),
            "...".to_string(),
        ),
        (
            lang_text(app, "鼠标支持", "Mouse Support"),
            lang_on_off(app, app.config.mouse_support).to_string(),
        ),
        (
            lang_text(app, "显示提示", "Show Hints"),
            lang_on_off(app, app.config.show_hints).to_string(),
        ),
        (
            lang_text(app, "主页更多推荐", "More Home Recommendations"),
            lang_on_off(app, app.config.home_more_recommend).to_string(),
        ),
        (
            lang_text(app, "退出登录", "Logout"),
            "".to_string(),
        ),
        (
            lang_text(app, "关于", "About"),
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

    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(app.theme.color_surface())),
        rows[1],
    );

    let footer_text = lang_text(
        app,
        "  ↑/k ↓/j: 导航  ←/h →/l: 调节/进入  Enter: 确认  Esc/t: 关闭",
        "  ↑/k ↓/j: Navigate  ←/h →/l: Adjust/Enter  Enter: Confirm  Esc/t: Close",
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(app.theme.color_subtext()),
        )))
        .style(Style::default().bg(app.theme.color_surface())),
        rows[2],
    );
}

fn render_transparency_settings_modal(
    f: &mut ratatui::Frame,
    size: Rect,
    app: &mut AppState,
) {
    let area = centered_rect(size, 70, 20);
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(lang_text(app, " 透明设置 ", " Transparency Settings "))
        .title(
            Line::from(Span::styled(
                lang_text(app, " [返回 ‹] ", " [Back ‹] "),
                Style::default().fg(app.theme.color_subtext()),
            ))
            .alignment(ratatui::layout::Alignment::Right),
        )
        .border_style(
            Style::default()
                .fg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(base_bg_style(app));
    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(app.theme.color_surface())),
        rows[0],
    );

    let raw_items = vec![
        (
            lang_text(app, "背景透明", "Transparent Background"),
            lang_on_off(app, app.config.transparent_background).to_string(),
        ),
        (
            lang_text(app, "个人中心透明", "Personal Center Transparent"),
            lang_on_off(app, app.config.transparent_sidebar).to_string(),
        ),
        (
            lang_text(app, "桌面歌词背景样式", "Desktop Lyrics Background Style"),
            app.config
                .desktop_lyrics_bg
                .display_name(app.language)
                .to_string(),
        ),
        (
            lang_text(app, "桌面歌词背景透明度", "Desktop Lyrics Background Opacity"),
            format!("{}%", app.config.desktop_lyrics_opacity),
        ),
    ];

    let lines: Vec<Line> = raw_items
        .iter()
        .enumerate()
        .map(|(idx, (key, val))| {
            let selected = idx == app.transparency_settings_selected;
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

    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(app.theme.color_surface())),
        rows[1],
    );

    let footer_text = lang_text(
        app,
        "  ↑/k ↓/j: 导航  ←/h →/l: 调节  Enter: 切换  Esc/t: 返回上一级",
        "  ↑/k ↓/j: Navigate  ←/h →/l: Adjust  Enter: Toggle  Esc/t: Back",
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(app.theme.color_subtext()),
        )))
        .style(Style::default().bg(app.theme.color_surface())),
        rows[2],
    );
}


fn render_acoustid_modal(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    let area = centered_rect(size, 60, 8);
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(crate::tmplayer::ui::borders::SOLID_BORDER)
        .title(lang_text(app, "AcoustID API 密钥", "AcoustID API Key"))
        .style(
            Style::default()
                .fg(app.theme.color_subtext())
                .bg(app.theme.color_surface()),
        );
    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::styled(
        "",
        Style::default()
            .fg(app.theme.color_subtext())
            .bg(app.theme.color_surface()),
    ));
    lines.push(Line::styled(
        "",
        Style::default().bg(app.theme.color_surface()),
    ));
    lines.push(Line::styled(
        format!(
            "{}: {}",
            lang_text(app, "API 密钥", "API Key"),
            app.acoustid_input
        ),
        Style::default()
            .fg(app.theme.color_text())
            .bg(app.theme.color_surface()),
    ));

    let p = Paragraph::new(lines)
        .style(Style::default().bg(app.theme.color_surface()))
        .wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn render_bar_settings_modal(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    let area = centered_rect(size, 70, 20);
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(lang_text(app, " 播放设置 ", " Playback Settings "))
        .title(
            Line::from(Span::styled(
                lang_text(app, " [返回 ‹] ", " [Back ‹] "),
                Style::default().fg(app.theme.color_subtext()),
            ))
            .alignment(ratatui::layout::Alignment::Right),
        )
        .border_style(
            Style::default()
                .fg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(base_bg_style(app));
    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(app.theme.color_surface())),
        rows[0],
    );

    let bar_number_label = match app.config.bar_number {
        crate::tmplayer::data::config::BarNumber::Auto => lang_text(app, "自动", "Auto"),
        crate::tmplayer::data::config::BarNumber::N16 => "16",
        crate::tmplayer::data::config::BarNumber::N32 => "32",
        crate::tmplayer::data::config::BarNumber::N48 => "48",
        crate::tmplayer::data::config::BarNumber::N64 => "64",
        crate::tmplayer::data::config::BarNumber::N80 => "80",
        crate::tmplayer::data::config::BarNumber::N96 => "96",
    };
    let channels_label = match app.config.bar_channels {
        crate::tmplayer::data::config::BarChannels::Mono => "Mono",
        crate::tmplayer::data::config::BarChannels::Stereo => "Stereo",
    };

    let raw_items = vec![
        (
            lang_text(app, "可视化", "Visualization"),
            match app.config.visualize {
                crate::tmplayer::data::config::VisualizeMode::Off => {
                    lang_text(app, "关闭", "Off").to_string()
                }
                crate::tmplayer::data::config::VisualizeMode::Bars => {
                    lang_text(app, "柱状频谱", "Bars").to_string()
                }
                crate::tmplayer::data::config::VisualizeMode::Oscilloscope => {
                    lang_text(app, "示波波形", "Oscilloscope").to_string()
                }
                crate::tmplayer::data::config::VisualizeMode::Circle => {
                    lang_text(app, "环形频谱", "Circle Spectrum").to_string()
                }
                crate::tmplayer::data::config::VisualizeMode::Particles => {
                    lang_text(app, "电光粒子", "Particle Wave").to_string()
                }
                crate::tmplayer::data::config::VisualizeMode::Mirror => {
                    lang_text(app, "双向镜像", "Symmetric Mirror").to_string()
                }
            },
        ),
        (
            lang_text(app, "超级流畅", "Super Smooth"),
            lang_on_off(app, app.config.super_smooth_bar).to_string(),
        ),
        (
            lang_text(app, "频谱间隔", "Bars Gap"),
            lang_on_off(app, app.config.bars_gap).to_string(),
        ),
        (
            lang_text(app, "频谱数", "Bars Count"),
            bar_number_label.to_string(),
        ),
        (
            lang_text(app, "声道", "Channels"),
            channels_label.to_string(),
        ),
        (
            lang_text(app, "封面边框", "Cover Border"),
            lang_on_off(app, app.config.album_border).to_string(),
        ),
        (
            lang_text(app, "页面歌词", "Page Lyrics"),
            lang_on_off(app, app.config.page_lyrics).to_string(),
        ),
        (
            lang_text(app, "音质", "Audio Quality"),
            match app.config.audio_quality {
                crate::tmplayer::data::config::AudioQuality::Standard => {
                    lang_text(app, "标准", "Standard").to_string()
                }
                crate::tmplayer::data::config::AudioQuality::Higher => {
                    lang_text(app, "较高", "Higher").to_string()
                }
                crate::tmplayer::data::config::AudioQuality::Exhigh => {
                    lang_text(app, "极高", "Exhigh").to_string()
                }
                crate::tmplayer::data::config::AudioQuality::Lossless => {
                    lang_text(app, "无损", "Lossless").to_string()
                }
                crate::tmplayer::data::config::AudioQuality::Hires => "Hi-Res".to_string(),
                crate::tmplayer::data::config::AudioQuality::Jyeffect => {
                    lang_text(app, "高清环绕声", "JYEffect").to_string()
                }
                crate::tmplayer::data::config::AudioQuality::Sky => {
                    lang_text(app, "沉浸环绕声", "Sky").to_string()
                }
                crate::tmplayer::data::config::AudioQuality::Dolby => {
                    lang_text(app, "杜比全景声", "Dolby").to_string()
                }
                crate::tmplayer::data::config::AudioQuality::Jymaster => {
                    lang_text(app, "超清母带", "JYMaster").to_string()
                }
            },
        ),
        (
            lang_text(app, "播放记忆", "Playback Memory"),
            lang_on_off(app, app.config.playback_memory).to_string(),
        ),
    ];

    let lines: Vec<Line> = raw_items
        .iter()
        .enumerate()
        .map(|(idx, (key, val))| {
            let selected = idx == app.bar_settings_selected;
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

    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(app.theme.color_surface())),
        rows[1],
    );

    let footer_text = lang_text(
        app,
        "  ↑/k ↓/j: 导航  ←/h →/l: 调节  Enter: 切换/进入  Esc: 返回上一级",
        "  ↑/k ↓/j: Navigate  ←/h →/l: Adjust  Enter: Toggle/Enter  Esc: Back",
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(app.theme.color_subtext()),
        )))
        .style(Style::default().bg(app.theme.color_surface())),
        rows[2],
    );
}

fn render_desktop_lyrics_settings_modal(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    let area = centered_rect(size, 70, 20);
    f.render_widget(ratatui::widgets::Clear, area);

    let title = lang_text(app, " 桌面歌词设置 ", " Desktop Lyrics Settings ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(title)
        .title(
            Line::from(Span::styled(
                lang_text(app, " [返回 ‹] ", " [Back ‹] "),
                Style::default().fg(app.theme.color_subtext()),
            ))
            .alignment(ratatui::layout::Alignment::Right),
        )
        .border_style(
            Style::default()
                .fg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(base_bg_style(app));

    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let locked_str = if app.config.desktop_lyrics_locked {
        lang_text(app, "锁定 (不可拖动)", "Locked (No Drag)")
    } else {
        lang_text(app, "解锁 (可拖动)", "Unlocked (Draggable)")
    };

    let dual_line_str = if app.config.desktop_lyrics_dual_line {
        lang_text(app, "双行歌词", "Dual Lines")
    } else {
        lang_text(app, "单行歌词", "Single Line")
    };

    let align_str = app.config.desktop_lyrics_align.display_name(app.language);
    let bg_str = app.config.desktop_lyrics_bg.display_name(app.language);
    let width_str = app.config.desktop_lyrics_width.display_name(app.language);
    let opacity_str = format!("{}%", app.config.desktop_lyrics_opacity);

    let pos_str = if let (Some(x), Some(y)) = (
        app.config.desktop_lyrics_pos_x,
        app.config.desktop_lyrics_pos_y,
    ) {
        format!("{}, {}", x, y)
    } else {
        lang_text(app, "底部居中 (默认)", "Bottom Center (Default)").to_string()
    };

    let raw_items = vec![
        (
            lang_text(app, "桌面歌词开关", "Desktop Lyrics"),
            lang_on_off(app, app.config.desktop_lyrics).to_string(),
        ),
        (
            lang_text(app, "锁定位置", "Lock Position"),
            locked_str.to_string(),
        ),
        (
            lang_text(app, "字体大小", "Font Size"),
            format!("{}px", app.config.desktop_lyrics_font_size),
        ),
        (
            lang_text(app, "窗口宽度", "Window Width"),
            width_str.to_string(),
        ),
        (
            lang_text(app, "歌词行数", "Lyrics Lines"),
            dual_line_str.to_string(),
        ),
        (
            lang_text(app, "歌词对齐", "Lyrics Alignment"),
            align_str.to_string(),
        ),
        (
            lang_text(app, "背景样式", "Background Style"),
            bg_str.to_string(),
        ),
        (
            lang_text(app, "背景透明度", "Background Opacity"),
            opacity_str,
        ),
        (
            lang_text(app, "恢复默认位置", "Reset Default Position"),
            pos_str,
        ),
    ];

    let lines: Vec<Line> = raw_items
        .iter()
        .enumerate()
        .map(|(idx, (key, val))| {
            let selected = idx == app.desktop_lyrics_settings_selected;
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

    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(app.theme.color_surface())),
        rows[1],
    );

    let footer_text = lang_text(
        app,
        "  ↑/k ↓/j: 导航  ←/h →/l: 调节  Enter: 确认/重置  Esc: 返回上一级",
        "  ↑/k ↓/j: Navigate  ←/h →/l: Adjust  Enter: Confirm/Reset  Esc: Back",
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(app.theme.color_subtext()),
        )))
        .style(Style::default().bg(app.theme.color_surface())),
        rows[2],
    );
}

fn render_local_audio_settings_modal(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    let area = centered_rect(size, 60, 12);
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(crate::tmplayer::ui::borders::SOLID_BORDER)
        .title(lang_text(app, "本地音频", "Local Audio"))
        .style(
            Style::default()
                .fg(app.theme.color_subtext())
                .bg(app.theme.color_surface()),
        );
    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::styled(
        "",
        Style::default()
            .fg(app.theme.color_subtext())
            .bg(app.theme.color_surface()),
    ));
    lines.push(Line::styled(
        "",
        Style::default().bg(app.theme.color_surface()),
    ));

    let lyrics_fetch_label = format!(
        "{}: {}",
        lang_text(app, "歌词/封面获取", "Lyrics/Cover Fetch"),
        if app.config.lyrics_cover_fetch {
            lang_text(app, "开", "On")
        } else {
            lang_text(app, "关", "Off")
        }
    );
    let lyrics_download_label = format!(
        "{}: {}",
        lang_text(app, "歌词/封面下载", "Lyrics/Cover Download"),
        if app.config.lyrics_cover_download {
            lang_text(app, "开", "On")
        } else {
            lang_text(app, "关", "Off")
        }
    );
    let fingerprint_label = if app.config.acoustid_api_key.trim().is_empty() {
        format!(
            "{}: {} ({})",
            lang_text(app, "音频指纹", "Audio Fingerprint"),
            lang_text(app, "关", "Off"),
            lang_text(app, "需要 API 密钥", "API key required")
        )
    } else {
        format!(
            "{}: {}",
            lang_text(app, "音频指纹", "Audio Fingerprint"),
            if app.config.audio_fingerprint {
                lang_text(app, "开", "On")
            } else {
                lang_text(app, "关", "Off")
            }
        )
    };
    let acoustid_label = format!(
        "{}: {}",
        lang_text(app, "AcoustID API", "AcoustID API"),
        if app.config.acoustid_api_key.trim().is_empty() {
            lang_text(app, "未设置", "Not set")
        } else {
            lang_text(app, "已设置", "Set")
        }
    );
    let resume_label = format!(
        "{}: {}",
        lang_text(app, "记住上次进度", "Resume Last Position"),
        if app.config.resume_last_position {
            lang_text(app, "开", "On")
        } else {
            lang_text(app, "关", "Off")
        }
    );

    let items = [
        lyrics_fetch_label,
        lyrics_download_label,
        fingerprint_label,
        acoustid_label,
        resume_label,
    ];

    for (idx, text) in items.iter().enumerate() {
        let disabled = match idx {
            2 => app.config.acoustid_api_key.trim().is_empty(),
            _ => false,
        };

        let style = if idx == app.local_audio_settings_selected {
            if disabled {
                Style::default()
                    .fg(app.theme.color_subtext())
                    .bg(app.theme.color_surface())
            } else {
                Style::default()
                    .fg(app.theme.color_accent2())
                    .add_modifier(Modifier::BOLD)
            }
        } else if disabled {
            Style::default()
                .fg(app.theme.color_subtext())
                .bg(app.theme.color_surface())
        } else {
            Style::default()
                .fg(app.theme.color_text())
                .bg(app.theme.color_surface())
        };
        lines.push(Line::styled(format!("  {}", text), style));
    }

    let p = Paragraph::new(lines)
        .style(Style::default().bg(app.theme.color_surface()))
        .wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn render_about_modal(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    if size.width < 32 || size.height < 10 {
        render_about_compact(f, size, app);
        return;
    }

    let area = about_modal_area(size);
    f.render_widget(ratatui::widgets::Clear, area);

    // Clean border with NO text on borders
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD),
            )
            .style(
                Style::default()
                    .fg(app.theme.color_subtext())
                    .bg(app.theme.color_surface()),
            ),
        area,
    );

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });
    render_about_content(f, inner, app);
}

fn render_about_compact(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    let area = about_compact_area(size);
    f.render_widget(ratatui::widgets::Clear, area);

    // Clean border with NO text on borders
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD),
            )
            .style(
                Style::default()
                    .fg(app.theme.color_subtext())
                    .bg(app.theme.color_surface()),
            ),
        area,
    );

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    render_about_content(f, inner, app);
}

fn render_about_content(f: &mut ratatui::Frame, area: Rect, app: &AppState) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let info = crate::tmplayer::data::about::about_info();
    let mut lines: Vec<Line<'static>> = Vec::new();

    // 1. Logo Art (if height allows)
    let show_art = area.height >= 9 && area.width >= 26;
    if show_art {
        let max_logo_h = if area.height >= 12 && area.width >= 48 {
            6
        } else if area.height >= 10 && area.width >= 38 {
            5
        } else {
            4
        };
        let logo_lines = about_logo_lines(app, area.width as usize, max_logo_h);
        lines.extend(logo_lines);
        lines.push(blank_about_line(app));
    }

    // 2. Version
    let badge_style = Style::default()
        .fg(app.theme.color_base())
        .bg(app.theme.color_accent2())
        .add_modifier(Modifier::BOLD);
    lines.push(Line::from(vec![
        Span::styled(format!(" 󰎆 v{} ", info.version), badge_style),
    ]));

    // 3. Tagline & Author
    let author = if info.author.is_empty() {
        "Asniya (@aimy1)"
    } else {
        info.author.as_str()
    };
    let tagline = match app.language {
        Language::Zh => format!("终端网易云音乐 · By {author}"),
        Language::En => format!("NetEase Cloud Music TUI Player · By {author}"),
    };
    lines.push(Line::from(Span::styled(
        tagline,
        Style::default()
            .fg(app.theme.color_text())
            .add_modifier(Modifier::BOLD),
    )));

    // 4. Repo Link
    let repo_url = info
        .links
        .get("github_url")
        .map(|s| s.as_str())
        .unwrap_or("https://github.com/aimy1/tune");
    lines.push(Line::from(Span::styled(
        repo_url.to_string(),
        Style::default()
            .fg(app.theme.color_accent())
            .add_modifier(Modifier::UNDERLINED),
    )));

    // 5. Exit Hint
    if area.height as usize > lines.len() + 1 {
        lines.push(blank_about_line(app));
    }
    let exit_hint = match app.language {
        Language::Zh => "Esc / q  返回",
        Language::En => "Esc / q  Back",
    };
    lines.push(Line::from(Span::styled(
        exit_hint,
        Style::default().fg(app.theme.color_subtext()),
    )));

    f.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(app.theme.color_surface()))
            .alignment(Alignment::Center),
        area,
    );
}

fn blank_about_line(app: &AppState) -> Line<'static> {
    Line::from(Span::styled(
        " ",
        Style::default().bg(app.theme.color_surface()),
    ))
}

fn about_modal_area(size: Rect) -> Rect {
    let want_w = 58u16;
    let want_h = 16u16;

    let max_w = size.width.saturating_sub(2);
    let max_h = size.height.saturating_sub(1);
    let w = want_w.min(max_w).max(32.min(max_w));
    let h = want_h.min(max_h).max(10.min(max_h));
    centered_rect(size, w, h)
}

fn about_compact_area(size: Rect) -> Rect {
    let w = size.width.saturating_sub(2).max(20);
    let h = size.height.saturating_sub(2).max(8);
    centered_rect(size, w, h)
}

/// Center the logo in the panel; if the panel is smaller, crop from the center.
fn about_logo_lines(app: &AppState, width: usize, height: usize) -> Vec<Line<'static>> {
    let blank = " ".repeat(width);
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let info = crate::tmplayer::data::about::about_info();
    let Some(selected) = crate::tmplayer::data::about::select_logo_art(width, height, &info.braille_images) else {
        return (0..height).map(|_| Line::from(blank.clone())).collect();
    };

    let mut rows: Vec<String> = selected
        .art
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();

    let mut start = 0usize;
    let mut end = rows.len();
    while start < end && rows[start].trim().is_empty() {
        start += 1;
    }
    while end > start && rows[end - 1].trim().is_empty() {
        end -= 1;
    }
    rows = rows[start..end].to_vec();

    let art_h = rows.len();
    let art_w = rows
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    let (src_y0, dst_y0, copy_h) = if art_h <= height {
        (0, (height - art_h) / 2, art_h)
    } else {
        ((art_h - height) / 2, 0, height)
    };
    let (src_x0, dst_x0, copy_w) = if art_w <= width {
        (0, (width - art_w) / 2, art_w)
    } else {
        ((art_w - width) / 2, 0, width)
    };

    let mut grid: Vec<Vec<char>> = vec![vec![' '; width]; height];
    for row_i in 0..copy_h {
        let src_row = &rows[src_y0 + row_i];
        let src_chars: Vec<char> = src_row.chars().collect();
        let gy = dst_y0 + row_i;
        for col_i in 0..copy_w {
            let sx = src_x0 + col_i;
            let gx = dst_x0 + col_i;
            if sx < src_chars.len() {
                grid[gy][gx] = src_chars[sx];
            }
        }
    }

    let total_rows = grid.len();
    let c1 = app.theme.palette.accent;
    let c2 = app.theme.palette.accent2;

    grid.into_iter()
        .enumerate()
        .map(|(r_idx, row)| {
            let color = match app.theme.capability {
                crate::tmplayer::ui::theme::ColorCapability::TrueColor => {
                    let t = if total_rows <= 1 {
                        0.0
                    } else {
                        r_idx as f32 / (total_rows - 1) as f32
                    };
                    let r = (c1.0 as f32 * (1.0 - t) + c2.0 as f32 * t).round() as u8;
                    let g = (c1.1 as f32 * (1.0 - t) + c2.1 as f32 * t).round() as u8;
                    let b = (c1.2 as f32 * (1.0 - t) + c2.2 as f32 * t).round() as u8;
                    ratatui::style::Color::Rgb(r, g, b)
                }
                _ => {
                    if r_idx < total_rows / 2 {
                        app.theme.color_accent()
                    } else {
                        app.theme.color_accent2()
                    }
                }
            };
            let text: String = row.into_iter().collect();
            Line::from(Span::styled(
                text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ))
        })
        .collect()
}

fn render_help_modal(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    let area = centered_rect(size, 70, 20);
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(lang_text(app, " 按键绑定 ", " Keybinds "))
        .border_style(
            Style::default()
                .fg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(base_bg_style(app));
    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    });

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(app.theme.color_surface())),
        rows[0],
    );

    let items = [
        (
            lang_text(app, "搜索框", "Search Box"),
            app.config.keybind_search_box.as_str(),
        ),
        (
            lang_text(app, "全屏播放页", "Fullscreen"),
            app.config.keybind_fullscreen.as_str(),
        ),
        (
            lang_text(app, "设置弹窗", "Settings Modal"),
            app.config.keybind_settings.as_str(),
        ),
        (
            lang_text(app, "侧边栏", "Sidebar"),
            app.config.keybind_sidebar.as_str(),
        ),
        (
            lang_text(app, "退出应用", "Quit"),
            app.config.keybind_quit.as_str(),
        ),
        (
            lang_text(app, "上一首", "Previous"),
            app.config.keybind_fullscreen_prev.as_str(),
        ),
        (
            lang_text(app, "下一首", "Next"),
            app.config.keybind_fullscreen_next.as_str(),
        ),
        (
            lang_text(app, "播放/暂停", "Play/Pause"),
            app.config.keybind_fullscreen_toggle_play_pause.as_str(),
        ),
        (
            lang_text(app, "全屏模式切换", "Fullscreen Mode Switch"),
            app.config.keybind_fullscreen_toggle_mode.as_str(),
        ),
        (
            lang_text(app, "EQ均衡器", "EQ Equalizer"),
            app.config.keybind_fullscreen_eq.as_str(),
        ),
        (
            lang_text(app, "EQ重置", "EQ Reset"),
            app.config.keybind_fullscreen_eq_reset.as_str(),
        ),
        (
            lang_text(app, "收藏/取消收藏", "Like/Unlike"),
            app.config.keybind_toggle_like_fullscreen.as_str(),
        ),
        (
            lang_text(app, "个人中心", "Personal Center"),
            app.config.keybind_personal_center.as_str(),
        ),
        (
            lang_text(app, "主页", "Home"),
            app.config.keybind_home.as_str(),
        ),
        (
            lang_text(app, "侧边栏歌单区切换", "Sidebar Playlist Section Switch"),
            "Ctrl+Up/Down",
        ),
        (
            lang_text(app, "桌面歌词", "Desktop Lyrics"),
            app.config.keybind_desktop_lyrics.as_str(),
        ),
        (
            lang_text(app, "桌面歌词锁定/拖动", "Desktop Lyrics Lock/Drag"),
            app.config.keybind_desktop_lyrics_lock.as_str(),
        ),
        (lang_text(app, "按键绑定", "Keybinds"), "Ctrl+K"),
    ];

    let visible_rows = rows[1].height as usize;
    let total_rows = items.len();
    let selected = app.help_keybind_selected.min(total_rows.saturating_sub(1));
    let max_scroll = total_rows.saturating_sub(visible_rows);
    let scroll = if visible_rows == 0 || selected < visible_rows {
        0
    } else {
        (selected + 1 - visible_rows).min(max_scroll)
    };

    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_rows)
        .map(|(idx, (label, key))| {
            let is_sel = idx == selected;
            let prefix = if is_sel { "› " } else { "  " };
            let style = if is_sel {
                Style::default()
                    .fg(app.theme.color_base())
                    .bg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(app.theme.color_text())
                    .bg(app.theme.color_surface())
            };
            let line_str = format_setting_line(prefix, label, key, inner.width);
            Line::from(Span::styled(line_str, style))
        })
        .collect();

    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(app.theme.color_surface())),
        rows[1],
    );

    let footer_text = lang_text(
        app,
        "  ↑/k ↓/j: 导航  Esc/t: 返回上一级",
        "  ↑/k ↓/j: Navigate  Esc/t: Back",
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer_text,
            Style::default().fg(app.theme.color_subtext()),
        )))
        .style(Style::default().bg(app.theme.color_surface())),
        rows[2],
    );
}

fn render_eq_modal(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    // 需求：柱状条宽 2 格，高度 +12/-12（含 0 行共 25）
    // 额外预留：顶部提示 1 行 + 底部频率/数值 2 行
    let area = centered_rect(size, 44, 31);
    f.render_widget(ratatui::widgets::Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(crate::tmplayer::ui::borders::SOLID_BORDER)
        .title(lang_text(app, " 均衡器 ", " Equalizer "))
        .border_style(
            Style::default()
                .fg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(
            Style::default()
                .fg(app.theme.color_subtext())
                .bg(app.theme.color_surface()),
        );
    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });

    let bg = Style::default().bg(app.theme.color_surface());
    let sub = Style::default()
        .fg(app.theme.color_subtext())
        .bg(app.theme.color_surface());
    let text = Style::default()
        .fg(app.theme.color_text())
        .bg(app.theme.color_surface());
    let selected_bg = Style::default()
        .fg(app.theme.color_base())
        .bg(app.theme.color_accent())
        .add_modifier(Modifier::BOLD);

    // layout inside modal
    if inner.height < 3 {
        return;
    }
    let hint_rect = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };
    let freq_label_rect = Rect {
        x: inner.x,
        y: inner.y + inner.height - 2,
        width: inner.width,
        height: 1,
    };
    let gain_label_rect = Rect {
        x: inner.x,
        y: inner.y + inner.height - 1,
        width: inner.width,
        height: 1,
    };
    let bars_rect = Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: inner.height.saturating_sub(3),
    };

    f.render_widget(
        Paragraph::new("").style(sub).wrap(Wrap { trim: true }),
        hint_rect,
    );

    // compute band geometry
    const BANDS: usize = crate::tmplayer::app::state::EQ_BANDS;
    const BAR_W: u16 = 2;
    const GAP: u16 = 1;

    fn fmt_db2(v: f32) -> String {
        let i = v.clamp(-12.0, 12.0).round() as i32;
        format!("{:+03}", i)
    }

    fn fmt_freq(freq_hz: f32) -> String {
        let f = freq_hz.round() as i32;
        if f >= 1000 {
            format!("{}k", f / 1000)
        } else {
            format!("{f}")
        }
    }

    let gains = app.eq.bands_db;
    let freq_labels: Vec<String> = crate::tmplayer::app::state::EQ_FREQS_HZ
        .iter()
        .map(|&f| fmt_freq(f))
        .collect();
    let gain_labels: Vec<String> = gains.iter().map(|&g| fmt_db2(g)).collect();

    // Fit columns to available width (10 bands should still render on typical terminals).
    let gaps_w = GAP.saturating_mul((BANDS as u16).saturating_sub(1));
    let mut cw = if bars_rect.width > gaps_w {
        (bars_rect.width - gaps_w) / (BANDS as u16)
    } else {
        BAR_W
    };
    cw = cw.clamp(BAR_W, 10);
    let total_w: u16 = cw.saturating_mul(BANDS as u16) + gaps_w;
    let x0 = bars_rect.x + (bars_rect.width.saturating_sub(total_w)) / 2;
    let gap = GAP;

    // fixed height: 25 rows => +12..0..-12
    let want_h: u16 = 25;
    let bars_h = if bars_rect.height >= want_h {
        want_h
    } else {
        bars_rect.height.max(3)
    };
    let y0 = bars_rect.y + (bars_rect.height.saturating_sub(bars_h)) / 2;

    // helper: map row index to db
    let row_to_db = |r: i32| -> i32 {
        if bars_h == want_h {
            // r: 0..24 => +12..-12
            12 - r
        } else {
            // fallback scale to +/-12
            let mid = (bars_h as i32) / 2;
            if r == mid {
                0
            } else if r < mid {
                let level = (mid - r) as f32;
                let max = mid.max(1) as f32;
                ((12.0 * (level / max)).round() as i32).clamp(0, 12)
            } else {
                let level = (r - mid) as f32;
                let max = (bars_h as i32 - 1 - mid).max(1) as f32;
                (-(12.0 * (level / max)).round() as i32).clamp(-12, 0)
            }
        }
    };

    let mut lines: Vec<Line> = Vec::with_capacity(bars_h as usize);
    for r in 0..bars_h {
        let rr = r as i32;
        let db_row = row_to_db(rr);

        let mut spans: Vec<ratatui::text::Span> = Vec::new();

        // left padding
        if x0 > bars_rect.x {
            spans.push(ratatui::text::Span::styled(
                " ".repeat((x0 - bars_rect.x) as usize),
                bg,
            ));
        }

        for b in 0..BANDS {
            let gain = gains[b].clamp(-12.0, 12.0).round() as i32;
            let filled = if db_row == 0 {
                false
            } else if db_row > 0 {
                // +1..+12: fill when row <= gain (e.g. gain=3 fills +1..+3)
                gain > 0 && db_row <= gain
            } else {
                // -1..-12: fill when row >= gain (e.g. gain=-5 fills -1..-5)
                gain < 0 && db_row >= gain
            };

            // Each column: center the 2-cell bar within fixed column width.
            let left_pad = cw.saturating_sub(BAR_W) / 2;
            let right_pad = cw.saturating_sub(BAR_W) - left_pad;
            let mut cell = String::new();
            cell.push_str(&" ".repeat(left_pad as usize));
            // 需求：零点(0dB)使用“▓▓”标识。
            if db_row == 0 {
                cell.push_str("▓▓");
            } else {
                cell.push_str(if filled { "██" } else { "░░" });
            }
            cell.push_str(&" ".repeat(right_pad as usize));
            if b + 1 < BANDS {
                cell.push_str(&" ".repeat(gap as usize));
            }

            // 需求：仅去除柱的选中效果（柱体不高亮）
            spans.push(ratatui::text::Span::styled(cell, text));
        }

        // right padding
        let drawn = (cw.saturating_mul(BANDS as u16)
            + gap.saturating_mul((BANDS as u16).saturating_sub(1)))
            + (x0 - bars_rect.x);
        if drawn < bars_rect.width {
            spans.push(ratatui::text::Span::styled(
                " ".repeat((bars_rect.width - drawn) as usize),
                bg,
            ));
        }

        lines.push(Line::from(spans));
    }

    let draw_rect = Rect {
        x: bars_rect.x,
        y: y0,
        width: bars_rect.width,
        height: bars_h,
    };
    f.render_widget(
        Paragraph::new(lines).style(bg).wrap(Wrap { trim: false }),
        draw_rect,
    );

    // bottom labels (two lines): keep frequency + always show numeric gain.
    let mut freq_spans: Vec<ratatui::text::Span> = Vec::new();
    let mut gain_spans: Vec<ratatui::text::Span> = Vec::new();
    if x0 > bars_rect.x {
        let pad = " ".repeat((x0 - bars_rect.x) as usize);
        freq_spans.push(ratatui::text::Span::styled(pad.clone(), bg));
        gain_spans.push(ratatui::text::Span::styled(pad, bg));
    }
    for b in 0..BANDS {
        let style = if b == app.eq_selected {
            selected_bg
        } else {
            sub
        };

        let mut ftxt = freq_labels[b].clone();
        if unicode_width::UnicodeWidthStr::width(ftxt.as_str()) as u16 > cw {
            ftxt = ftxt.chars().take(cw as usize).collect();
        }
        let fpad = cw.saturating_sub(unicode_width::UnicodeWidthStr::width(ftxt.as_str()) as u16);
        let fleft = fpad / 2;
        let fright = fpad - fleft;
        let mut fcell = format!(
            "{}{}{}",
            " ".repeat(fleft as usize),
            ftxt,
            " ".repeat(fright as usize)
        );
        if b + 1 < BANDS {
            fcell.push_str(&" ".repeat(gap as usize));
        }
        freq_spans.push(ratatui::text::Span::styled(fcell, style));

        let mut gtxt = gain_labels[b].clone();
        if unicode_width::UnicodeWidthStr::width(gtxt.as_str()) as u16 > cw {
            gtxt = gtxt.chars().take(cw as usize).collect();
        }
        let gpad = cw.saturating_sub(unicode_width::UnicodeWidthStr::width(gtxt.as_str()) as u16);
        let gleft = gpad / 2;
        let gright = gpad - gleft;
        let mut gcell = format!(
            "{}{}{}",
            " ".repeat(gleft as usize),
            gtxt,
            " ".repeat(gright as usize)
        );
        if b + 1 < BANDS {
            gcell.push_str(&" ".repeat(gap as usize));
        }
        gain_spans.push(ratatui::text::Span::styled(gcell, style));
    }
    f.render_widget(
        Paragraph::new(Line::from(freq_spans)).style(bg),
        freq_label_rect,
    );
    f.render_widget(
        Paragraph::new(Line::from(gain_spans)).style(bg),
        gain_label_rect,
    );
}

fn get_volume_popover_rect(layout_full: Rect, left_rect: Rect, btn_rect: Rect) -> (Rect, bool) {
    let popup_w: u16 = 21;
    let popup_h: u16 = 1;

    let center_x = if btn_rect.width > 0 {
        btn_rect.x + btn_rect.width / 2
    } else {
        left_rect.x + left_rect.width / 2
    };

    let min_x = left_rect.x.saturating_add(1);
    let max_x = (left_rect.x + left_rect.width).saturating_sub(popup_w + 1);
    let popup_x = center_x.saturating_sub(popup_w / 2).clamp(min_x, max_x.max(min_x));

    // Place popover below the button by default; fallback to above if not enough space at bottom
    let below_y = btn_rect.y.saturating_add(btn_rect.height.max(1));
    let max_allowed_y = (left_rect.y + left_rect.height).saturating_sub(popup_h + 1);
    let (popup_y, arrow_down) = if below_y <= max_allowed_y {
        (below_y, false)
    } else if btn_rect.y >= popup_h {
        (btn_rect.y.saturating_sub(popup_h), true)
    } else {
        (below_y.min(layout_full.height.saturating_sub(popup_h)), false)
    };

    (
        Rect {
            x: popup_x,
            y: popup_y,
            width: popup_w.min(layout_full.width),
            height: popup_h,
        },
        arrow_down,
    )
}

fn render_volume_modal(f: &mut ratatui::Frame, size: Rect, app: &mut AppState) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(size);
    let info_l = info_panel::layout(cols[0]);
    let btn_rect = control_buttons::volume_button_rect(info_l.controls, app);
    let (area, _) = get_volume_popover_rect(size, cols[0], btn_rect);

    if area.width < 10 || area.height < 1 {
        return;
    }

    f.render_widget(ratatui::widgets::Clear, area);

    let vol = app.player.volume.clamp(0.0, 1.0);
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
    let accent_style = Style::default().fg(app.theme.color_accent()).add_modifier(Modifier::BOLD).bg(bg);
    let subtext_style = Style::default().fg(app.theme.color_subtext()).bg(bg);
    let text_style = Style::default().fg(app.theme.color_text()).add_modifier(Modifier::BOLD).bg(bg);

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

    f.render_widget(Paragraph::new(line).style(Style::default().bg(bg)), area);
}

pub fn hit_test(layout: &UiLayout, app: &AppState, col: u16, row: u16) -> Option<Action> {
    // Volume modal popover consumes clicks first
    if app.overlay == Overlay::VolumeModal {
        let (area, _) = get_volume_popover_rect(layout.full, layout.left, layout.info_volume);

        if !contains(area, col, row) {
            return Some(Action::CloseOverlay);
        }

        let x = area.x;

        // Left bracket: 0%
        if col <= x + 1 {
            return Some(Action::SetVolume(0.0));
        }

        // Icon area: toggle mute
        if col <= x + 3 {
            return Some(Action::ToggleMute);
        }

        // 10-block slider
        if col >= x + 4 && col <= x + 13 {
            let block = col - (x + 4);
            let ratio = ((block as f32 + 0.5) / 10.0).clamp(0.0, 1.0);
            return Some(Action::SetVolume(ratio));
        }

        // Right bracket: 100%
        if col >= x + 19 {
            return Some(Action::SetVolume(1.0));
        }

        // Percentage text: toggle mute
        return Some(Action::ToggleMute);
    }

    // Eq modal consumes clicks second
    if app.overlay == Overlay::EqModal {
        let area = centered_rect(layout.full, 44, 31);
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });
        if inner.height >= 3 {
            let bars_rect = Rect {
                x: inner.x,
                y: inner.y + 1,
                width: inner.width,
                height: inner.height.saturating_sub(3),
            };

            if contains(bars_rect, col, row) {
                const BANDS: usize = crate::tmplayer::app::state::EQ_BANDS;
                const BAR_W: u16 = 2;
                const GAP: u16 = 1;

                let gaps_w = GAP.saturating_mul((BANDS as u16).saturating_sub(1));
                let mut cw = if bars_rect.width > gaps_w {
                    (bars_rect.width - gaps_w) / (BANDS as u16)
                } else {
                    BAR_W
                };
                cw = cw.clamp(BAR_W, 10);
                let total_w: u16 = cw.saturating_mul(BANDS as u16) + gaps_w;
                let x0 = bars_rect.x + (bars_rect.width.saturating_sub(total_w)) / 2;
                if col < x0 || col >= x0 + total_w {
                    return None;
                }

                // Find band by fixed widths; then require click within the centered BAR_W region.
                let mut band: Option<usize> = None;
                for b in 0..BANDS {
                    let col_start = x0 + (b as u16) * (cw + GAP);
                    let col_end = col_start + cw;
                    if col >= col_start && col < col_end {
                        let left_pad = cw.saturating_sub(BAR_W) / 2;
                        let bar_start = col_start + left_pad;
                        let bar_end = bar_start + BAR_W;
                        if col < bar_start || col >= bar_end {
                            return None;
                        }
                        band = Some(b);
                        break;
                    }
                }

                let band = band?;

                // fixed height mapping: prefer 25 rows (12..0..-12)
                let want_h: u16 = 25;
                let bars_h = if bars_rect.height >= want_h {
                    want_h
                } else {
                    bars_rect.height.max(3)
                };
                let y0 = bars_rect.y + (bars_rect.height.saturating_sub(bars_h)) / 2;
                if row < y0 || row >= y0 + bars_h {
                    return None;
                }
                let rr = (row - y0) as i32;

                let db_i = if bars_h == want_h {
                    (12 - rr).clamp(-12, 12)
                } else {
                    let mid = (bars_h as i32) / 2;
                    if rr == mid {
                        0
                    } else if rr < mid {
                        let level = (mid - rr) as f32;
                        let max = mid.max(1) as f32;
                        ((12.0 * (level / max)).round() as i32).clamp(0, 12)
                    } else {
                        let level = (rr - mid) as f32;
                        let max = (bars_h as i32 - 1 - mid).max(1) as f32;
                        (-(12.0 * (level / max)).round() as i32).clamp(-12, 0)
                    }
                };

                return Some(Action::EqSetBandDb {
                    band,
                    db: db_i as f32,
                });
            }
        }
    }

    if app.overlay == Overlay::SettingsModal {
        let area = centered_rect(layout.full, 70, 20);
        if !contains(area, col, row) || row == area.y {
            return Some(Action::ExitSettings);
        }
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 1,
        });
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);
        if contains(rows[2], col, row) {
            return Some(Action::ExitSettings);
        }
        if contains(rows[1], col, row) {
            let idx = (row.saturating_sub(rows[1].y)) as usize;
            if idx < 12 {
                return Some(Action::SettingsClickItem {
                    index: idx,
                    is_right: col >= inner.x + inner.width / 2,
                });
            }
        }
        return None;
    }

    if app.overlay == Overlay::TransparencySettingsModal {
        let area = centered_rect(layout.full, 70, 20);
        if !contains(area, col, row) || row == area.y {
            return Some(Action::CloseOverlay);
        }
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 1,
        });
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);
        if contains(rows[2], col, row) {
            return Some(Action::CloseOverlay);
        }
        if contains(rows[1], col, row) {
            let idx = (row.saturating_sub(rows[1].y)) as usize;
            if idx < 4 {
                return Some(Action::TransparencySettingsClickItem {
                    index: idx,
                    is_right: col >= inner.x + inner.width / 2,
                });
            }
        }
        return None;
    }

    if app.overlay == Overlay::BarSettingsModal {
        let area = centered_rect(layout.full, 70, 20);
        if !contains(area, col, row) || row == area.y {
            return Some(Action::CloseOverlay);
        }
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 1,
        });
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);
        if contains(rows[2], col, row) {
            return Some(Action::CloseOverlay);
        }
        if contains(rows[1], col, row) {
            let idx = (row.saturating_sub(rows[1].y)) as usize;
            if idx < 9 {
                return Some(Action::BarSettingsClickItem {
                    index: idx,
                    is_right: col >= inner.x + inner.width / 2,
                });
            }
        }
        return None;
    }

    if app.overlay == Overlay::DesktopLyricsSettingsModal {
        let area = centered_rect(layout.full, 70, 20);
        if !contains(area, col, row) || row == area.y {
            return Some(Action::CloseOverlay);
        }
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 1,
        });
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);
        if contains(rows[2], col, row) {
            return Some(Action::CloseOverlay);
        }
        if contains(rows[1], col, row) {
            let idx = (row.saturating_sub(rows[1].y)) as usize;
            if idx < 9 {
                return Some(Action::DesktopLyricsSettingsClickItem {
                    index: idx,
                    is_right: col >= inner.x + inner.width / 2,
                });
            }
        }
        return None;
    }

    if app.overlay == Overlay::AboutModal {
        return Some(Action::CloseOverlay);
    }

    if app.overlay == Overlay::HelpModal {
        let area = centered_rect(layout.full, 70, 20);
        if !contains(area, col, row) || row == area.y {
            return Some(Action::CloseOverlay);
        }
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 2,
            vertical: 1,
        });
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(inner);
        if contains(rows[2], col, row) {
            return Some(Action::CloseOverlay);
        }
        if contains(rows[1], col, row) {
            let idx = (row.saturating_sub(rows[1].y)) as usize;
            if idx < 17 {
                return Some(Action::HelpSelect(idx));
            }
        }
        return None;
    }

    if contains(layout.info_controls, col, row) {
        return control_buttons::hit_test(layout.info_controls, app, col, row);
    }

    if contains(layout.info_volume, col, row) {
        return Some(Action::OpenVolumeModal);
    }

    if contains(layout.info_heart, col, row) {
        return Some(Action::ToggleFavorite);
    }

    if contains(layout.info_progress, col, row) {
        return Some(Action::SeekToFraction(ratio_in_track(
            layout.info_progress,
            col,
        )));
    }

    if contains(layout.playlist_list_inner, col, row) {
        let idx = row.saturating_sub(layout.playlist_list_inner.y) as usize;
        return Some(Action::PlaylistSelect(idx));
    }

    None
}

fn contains(r: Rect, col: u16, row: u16) -> bool {
    col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
}

#[allow(dead_code)]
fn ratio_in_bar(r: Rect, col: u16) -> f32 {
    if r.width <= 2 {
        return 0.0;
    }
    let inner = (r.width - 2) as f32;
    let x = col.saturating_sub(r.x + 1) as f32;
    (x / inner).clamp(0.0, 1.0)
}

fn ratio_in_track(r: Rect, col: u16) -> f32 {
    if r.width <= 1 {
        return 0.0;
    }
    let denom = (r.width - 1) as f32;
    let x = col.saturating_sub(r.x) as f32;
    (x / denom).clamp(0.0, 1.0)
}

pub(crate) fn lang_text<'a>(app: &AppState, zh: &'a str, en: &'a str) -> &'a str {
    match app.language {
        crate::data::config::Language::Zh => zh,
        crate::data::config::Language::En => en,
    }
}

fn lang_on_off(app: &AppState, enabled: bool) -> &'static str {
    match app.language {
        crate::data::config::Language::Zh => {
            if enabled {
                "开"
            } else {
                "关"
            }
        }
        crate::data::config::Language::En => {
            if enabled {
                "On"
            } else {
                "Off"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_setting_line_alignment() {
        let line = format_setting_line("› ", "主题", "catppuccin_mocha", 66);
        assert!(line.starts_with(" › 主题"));
        assert!(line.ends_with("catppuccin_mocha "));
        let width = UnicodeWidthStr::width(line.as_str());
        assert_eq!(width, 66);
    }

    #[test]
    fn test_volume_modal_hit_test() {
        let mut app = AppState::new(
            crate::tmplayer::data::config::Config::default(),
            crate::tmplayer::ui::theme::Theme::default(),
            crate::data::config::Language::Zh,
        );
        app.player.volume = 0.5;

        let mut layout = UiLayout::default();
        layout.full = Rect { x: 0, y: 0, width: 100, height: 30 };
        layout.left = Rect { x: 0, y: 0, width: 33, height: 30 };
        layout.info_volume = Rect { x: 15, y: 20, width: 8, height: 1 };

        // 1. In normal mode, clicking info_volume opens volume modal
        app.overlay = Overlay::None;
        let act = hit_test(&layout, &app, 16, 20);
        assert_eq!(act, Some(Action::OpenVolumeModal));

        // 2. In VolumeModal mode, clicking outside the modal closes it
        app.overlay = Overlay::VolumeModal;
        let act_outside = hit_test(&layout, &app, 2, 2);
        assert_eq!(act_outside, Some(Action::CloseOverlay));

        // Capsule is at x = 9, y = 21, width = 21, height = 1
        // Col 10 (x + 1): 0%
        let act_zero = hit_test(&layout, &app, 10, 21);
        assert_eq!(act_zero, Some(Action::SetVolume(0.0)));

        // Col 11 (x + 2): speaker icon click toggles mute
        let act_mute = hit_test(&layout, &app, 11, 21);
        assert_eq!(act_mute, Some(Action::ToggleMute));

        // Col 28 (x + 19): 100%
        let act_full = hit_test(&layout, &app, 28, 21);
        assert_eq!(act_full, Some(Action::SetVolume(1.0)));

        // Col 18 (x + 9): block 5 -> ~55%
        let act_bar = hit_test(&layout, &app, 18, 21);
        if let Some(Action::SetVolume(v)) = act_bar {
            assert!((v - 0.55).abs() < 0.05);
        } else {
            panic!("Expected SetVolume action, got {:?}", act_bar);
        }
    }

    #[test]
    fn test_volume_modal_input_mapping() {
        use crate::tmplayer::utils::input::map_key;
        let config = crate::tmplayer::data::config::Config::default();

        let left = crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Left);
        assert_eq!(map_key(left, Overlay::VolumeModal, &config), Action::VolumeDown);

        let right = crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Right);
        assert_eq!(map_key(right, Overlay::VolumeModal, &config), Action::VolumeUp);

        let space = crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char(' '));
        assert_eq!(map_key(space, Overlay::VolumeModal, &config), Action::ToggleMute);

        let esc = crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Esc);
        assert_eq!(map_key(esc, Overlay::VolumeModal, &config), Action::CloseOverlay);

        let v_key = crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('v'));
        assert_eq!(map_key(v_key, Overlay::None, &config), Action::OpenVolumeModal);
    }

    #[test]
    fn test_heart_hit_test() {
        let app = AppState::new(
            crate::tmplayer::data::config::Config::default(),
            crate::tmplayer::ui::theme::Theme::default(),
            crate::data::config::Language::Zh,
        );

        let mut layout = UiLayout::default();
        layout.info_heart = Rect { x: 30, y: 5, width: 3, height: 1 };

        let act = hit_test(&layout, &app, 31, 5);
        assert_eq!(act, Some(Action::ToggleFavorite));

        let act_miss = hit_test(&layout, &app, 29, 5);
        assert_eq!(act_miss, None);
    }

    #[test]
    fn test_tmplayer_about_modal_area() {
        let terminal = Rect::new(0, 0, 80, 24);
        let area = about_modal_area(terminal);
        assert!(area.width <= terminal.width);
        assert!(area.height <= terminal.height);
        assert_eq!(area.width, 58);
        assert_eq!(area.height, 16);

        let small_terminal = Rect::new(0, 0, 50, 14);
        let small_area = about_modal_area(small_terminal);
        assert!(small_area.width <= small_terminal.width);
        assert!(small_area.height <= small_terminal.height);
    }
}

