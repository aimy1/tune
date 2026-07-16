use crate::app::{App, Overlay, Page};
use ratatui::Frame;
use ratatui::layout::{Rect, Alignment};
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
    let separator_style = with_bar_bg(Style::default().fg(app.theme.color_buff()));
    let tagline_style = with_bar_bg(Style::default().fg(app.theme.color_subtext()));

    let logo_text = match app.config.language {
        crate::data::config::Language::Zh => " 󰎆 网易云 ",
        crate::data::config::Language::En => " 󰎆 NetEase ",
    };
    let tagline_text = match app.config.language {
        crate::data::config::Language::Zh => "传递音乐的力量",
        crate::data::config::Language::En => "Convey the power of music",
    };

    let left_spans = vec![
        Span::styled(logo_text, logo_style),
        Span::styled("  ", with_bar_bg(Style::default())),
        Span::styled("│", separator_style),
        Span::styled("  ", with_bar_bg(Style::default())),
        Span::styled(tagline_text, tagline_style),
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

    frame.render_widget(
        Paragraph::new(Line::from(left_spans)).alignment(Alignment::Left),
        inner,
    );

    frame.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
        inner,
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
