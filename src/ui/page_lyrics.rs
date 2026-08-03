use crate::app::App;
use crate::data::config::Language;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

const PAGE_LYRICS_PANEL_HEIGHT: u16 = 4;

pub fn overlay_panel_area(content_area: Rect) -> Rect {
    if content_area.width < 18 || content_area.height < PAGE_LYRICS_PANEL_HEIGHT {
        return Rect::default();
    }

    let mut width = ((content_area.width as f32) * 0.33).round() as u16;
    width = width.clamp(18, content_area.width.min(72));

    Rect {
        x: content_area.x + content_area.width.saturating_sub(width),
        y: content_area.y + content_area.height.saturating_sub(PAGE_LYRICS_PANEL_HEIGHT),
        width,
        height: PAGE_LYRICS_PANEL_HEIGHT,
    }
}

pub fn draw_page_lyrics_panel(frame: &mut Frame, app: &App, area: Rect) {
    if area.width < 8 || area.height < PAGE_LYRICS_PANEL_HEIGHT {
        return;
    }

    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(match app.config.language {
                Language::Zh => " 󰎈 歌词 ",
                Language::En => " 󰎈 Lyrics ",
            })
            .border_style(
                Style::default()
                    .fg(app.theme.color_accent2())
                    .add_modifier(Modifier::BOLD),
            )
            .style(panel_bg_style(app)),
        area,
    );

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    if inner.width == 0 || inner.height < 2 {
        return;
    }

    let (line1, line2) = app.current_page_lyric_lines();

    if line2.trim().is_empty() {
        let y_off = if inner.height >= 2 { (inner.height - 1) / 2 } else { 0 };
        let r = Rect {
            x: inner.x,
            y: inner.y + y_off,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                line1,
                Style::default()
                    .fg(app.theme.color_accent2())
                    .add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
            r,
        );
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                line1,
                Style::default()
                    .fg(app.theme.color_accent2())
                    .add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
            rows[0],
        );

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                line2,
                Style::default().fg(app.theme.color_subtext()),
            )))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
            rows[1],
        );
    }
}

fn panel_bg_style(app: &App) -> Style {
    if app.config.transparent_background {
        Style::default()
    } else {
        Style::default().bg(app.theme.color_surface())
    }
}
