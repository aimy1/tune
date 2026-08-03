use crate::app::{App, Overlay, Page};
use ratatui::Frame;
use ratatui::layout::{Rect, Alignment, Layout, Direction, Constraint};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub mod about;
pub mod author;
pub mod home;
pub mod loading;
pub mod login;
pub mod page_lyrics;
pub mod player_bar;
pub mod playlist;
pub mod search;
pub mod search_box;
pub mod settings;
pub mod theme;

pub fn draw_header_bar(frame: &mut Frame, app: &App, area: Rect) {
    if area.width < 10 || area.height == 0 {
        return;
    }

    let transparent = app.config.transparent_background;
    let bar_bg = app.theme.style_surface_bg(transparent);
    let with_bar_bg = |s: Style| {
        if transparent {
            s
        } else {
            s.bg(app.theme.color_surface())
        }
    };

    // Soft surface strip for the header.
    frame.render_widget(Block::default().style(bar_bg), area);

    frame.render_widget(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(app.theme.color_buff())),
        area,
    );

    let inner = Rect {
        x: area.x.saturating_add(1),
        y: area.y,
        width: area.width.saturating_sub(2),
        height: 1,
    };

    let logo_style = Style::default()
        .fg(app.theme.color_base())
        .bg(app.theme.color_accent())
        .add_modifier(Modifier::BOLD);

    let logo_text = match app.config.language {
        crate::data::config::Language::Zh => " 󰎆 网易云 ",
        crate::data::config::Language::En => " 󰎆 NetEase ",
    };

    let show_full_header = inner.width >= 80;

    let left_spans = vec![
        Span::styled(logo_text, logo_style),
    ];

    let user_name = if app.home_sidebar.user_name.trim().is_empty() {
        match app.config.language {
            crate::data::config::Language::Zh => "游客模式",
            crate::data::config::Language::En => "Guest Mode",
        }
    } else {
        &app.home_sidebar.user_name
    };

    let user_style = with_bar_bg(
        Style::default()
            .fg(app.theme.color_accent2())
            .add_modifier(Modifier::BOLD),
    );
    let icon_style = with_bar_bg(Style::default().fg(app.theme.color_subtext()));

    let right_spans = vec![
        Span::styled("󰀄 ", icon_style),
        Span::styled(user_name, user_style),
        Span::styled(" ", with_bar_bg(Style::default())),
    ];

    let header_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if show_full_header {
            [
                Constraint::Length(15),
                Constraint::Min(20),
                Constraint::Length(15),
            ]
        } else {
            [
                Constraint::Min(10),
                Constraint::Length(0),
                Constraint::Length(15),
            ]
        })
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(left_spans)).alignment(Alignment::Left),
        header_cols[0],
    );

    if show_full_header {
        let active_idx = match app.page {
            Page::Home => 0,
            Page::Playlist => 1,
            Page::Search => 2,
            _ => 99,
        };

        let tabs = match app.config.language {
            crate::data::config::Language::Zh => vec![
                (0, " 󰎆 发现 "),
                (1, " 󰓏 歌单 "),
                (2, " 🔍 搜索 "),
            ],
            crate::data::config::Language::En => vec![
                (0, " 󰎆 Discover "),
                (1, " 󰓏 Playlist "),
                (2, " 🔍 Search "),
            ],
        };

        let mut tab_spans = Vec::new();
        for (idx, label) in tabs {
            if idx > 0 {
                tab_spans.push(Span::styled("   ", with_bar_bg(Style::default())));
            }
            if idx == active_idx {
                tab_spans.push(Span::styled(
                    label,
                    Style::default()
                        .fg(app.theme.color_base())
                        .bg(app.theme.color_accent())
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                tab_spans.push(Span::styled(
                    label,
                    with_bar_bg(Style::default().fg(app.theme.color_subtext())),
                ));
            }
        }

        frame.render_widget(
            Paragraph::new(Line::from(tab_spans)).alignment(Alignment::Center),
            header_cols[1],
        );
    }

    frame.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
        header_cols[2],
    );
}

pub fn draw_settings(frame: &mut Frame, app: &mut App) {
    if matches!(
        app.overlay,
        Some(Overlay::Settings)
            | Some(Overlay::SettingsPlayback)
            | Some(Overlay::SettingsKeybinds)
            | Some(Overlay::SettingsAbout)
    ) {
        settings::draw_settings_modal(frame, app);
    }
    if matches!(app.overlay, Some(Overlay::SearchBox)) {
        search_box::draw_search_box_overlay(frame, app);
    }
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    match app.page {
        Page::Login => login::draw_login(frame, app),
        Page::Loading => loading::draw_loading(frame, app),
        Page::Home => home::draw_home(frame, app),
        Page::Playlist => playlist::draw_playlist(frame, app),
        Page::Author => author::draw_author(frame, app),
        Page::Search => search::draw_search(frame, app),
    }
}
