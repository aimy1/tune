use crate::app::{App, LoginMethod};
use crate::data::config::Language;
use qrcode::QrCode;
use qrcode::render::unicode;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

const DEFAULT_OPENING_TITLE: &str = "████████╗██╗   ██╗███╗   ██╗███████╗\n╚══██╔══╝██║   ██║████╗  ██║██╔════╝\n   ██║   ██║   ██║██╔██╗ ██║█████╗  \n   ██║   ██║   ██║██║╚██╗██║██╔══╝  \n   ██║   ╚██████╔╝██║ ╚████║███████╗\n   ╚═╝    ╚═════╝ ╚═╝  ╚═══╝╚══════╝";

pub fn draw_login(frame: &mut Frame, app: &App) {
    let size = frame.area();

    if size.width < 36 || size.height < 12 {
        draw_too_small(frame, app, size);
        return;
    }

    frame.render_widget(Block::default().style(base_bg_style(app)), size);

    let title_height = (size.height / 4).clamp(3, 7);
    let hint_height = 1;
    let form_height_zone = size.height.saturating_sub(title_height + hint_height);

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height),
            Constraint::Length(form_height_zone),
            Constraint::Length(hint_height),
        ])
        .split(size);

    let title_block = opening_title_block(&app.config.default_opening_title);
    let title_lines = title_block.lines().count().max(1) as u16;
    let title_area = centered_rect(areas[0].width, title_lines.min(areas[0].height), areas[0]);
    frame.render_widget(
        Paragraph::new(title_block)
            .style(
                Style::default()
                    .fg(app.theme.color_accent())
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center),
        title_area,
    );

    let card_height = form_height(app.login.method).min(areas[1].height);
    let card_width = 60.min(areas[1].width.saturating_sub(2));
    let form_area = centered_rect(card_width, card_height, areas[1]);

    let form_title = match app.login.method {
        LoginMethod::Qr => lang_text(app, " 📱 扫码登录 ", " 📱 QR Login "),
        LoginMethod::Username => lang_text(app, " 󰀄 账户登录 ", " 󰀄 Account Login "),
        LoginMethod::Phone => lang_text(app, " 󰌘 手机登录 ", " 󰌘 Phone Login "),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(form_title)
        .border_style(
            Style::default()
                .fg(app.theme.color_accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(app.theme.color_surface()));

    frame.render_widget(block, form_area);

    let inner = form_area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });

    if app.login.method == LoginMethod::Qr {
        render_qr_login_card(frame, app, inner);
    } else {
        let content = Paragraph::new(build_form_lines(app))
            .style(Style::default().fg(app.theme.color_text()).bg(app.theme.color_surface()))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(content, inner);
    }

    let hint_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(areas[2]);

    let left_hint = "F1/F2/F3 切换登录方式  Tab/↑↓ 切换焦点  Enter 确认";
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" 󰌌 ", Style::default().fg(app.theme.color_buff())),
            Span::styled(
                lang_text(
                    app,
                    left_hint,
                    "F1/F2/F3 switch login mode  Tab/Up/Down switch focus  Enter confirm",
                ),
                Style::default().fg(app.theme.color_subtext()),
            ),
        ]))
        .alignment(Alignment::Left),
        hint_cols[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                current_login_method_text(app, app.login.method),
                Style::default()
                    .fg(app.theme.color_accent2())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]))
        .alignment(Alignment::Right),
        hint_cols[1],
    );
}

fn draw_too_small(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Block::default().style(base_bg_style(app)), area);
    let msg = Paragraph::new(lang_text(app, "终端窗口过小", "Terminal too small"))
        .style(Style::default().fg(app.theme.color_subtext()))
        .alignment(Alignment::Center);
    frame.render_widget(msg, area);
}

fn opening_title_block(custom: &str) -> String {
    if custom.trim().is_empty() {
        return DEFAULT_OPENING_TITLE.to_string();
    }

    // Allow literal "\\n" in config to become line breaks.
    custom.replace("\\n", "\n")
}

fn form_height(method: LoginMethod) -> u16 {
    match method {
        LoginMethod::Qr => 26,
        LoginMethod::Username => 9,
        LoginMethod::Phone => 10,
    }
}

fn get_qr_url(app: &App) -> String {
    let url = app.login.qr_url.trim();
    if !url.is_empty() {
        return url.to_string();
    }
    let key = app.login.qr_key.trim();
    if !key.is_empty() {
        if key.starts_with("http://") || key.starts_with("https://") {
            return key.to_string();
        } else {
            return format!("https://music.163.com/login?codekey={}", key);
        }
    }
    String::new()
}

fn render_qr_login_card(frame: &mut Frame, app: &App, area: Rect) {
    let url = get_qr_url(app);
    let payload = if !url.is_empty() {
        url.as_str()
    } else {
        app.login.qr_key.trim()
    };

    let has_url = !url.is_empty();
    let url_height = if has_url { 2 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(14),
            Constraint::Length(1),
            Constraint::Length(url_height),
            Constraint::Length(3),
        ])
        .split(area);

    if payload.is_empty() {
        let msg = match app.config.language {
            Language::Zh => "\n󰐑 正在获取二维码...\n(按 F1 刷新)",
            Language::En => "\n󰐑 Generating QR...\n(Press F1 to refresh)",
        };
        frame.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(app.theme.color_subtext()))
                .alignment(Alignment::Center),
            chunks[0],
        );
    } else if let Ok(code) = QrCode::new(payload.as_bytes()) {
        let image = code
            .render::<unicode::Dense1x2>()
            .quiet_zone(false)
            .dark_color(unicode::Dense1x2::Dark)
            .light_color(unicode::Dense1x2::Light)
            .build();

        frame.render_widget(
            Paragraph::new(image)
                .style(Style::default().fg(app.theme.color_text()))
                .alignment(Alignment::Center),
            chunks[0],
        );
    }

    let hint = match app.config.language {
        Language::Zh => "请使用 网易云音乐 App 扫描二维码",
        Language::En => "Scan with NetEase Music App",
    };
    frame.render_widget(
        Paragraph::new(hint)
            .style(
                Style::default()
                    .fg(app.theme.color_accent2())
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center),
        chunks[1],
    );

    if has_url {
        let url_text = format!("🔗 链接: {}", url);
        frame.render_widget(
            Paragraph::new(url_text)
                .style(Style::default().fg(app.theme.color_accent()))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            chunks[2],
        );
    }

    let mut button_lines = Vec::new();
    push_action_line(
        &mut button_lines,
        app,
        0,
        format!("󰐑 {}", lang_text(app, "刷新二维码", "Refresh QR")),
        app.login.focus_index == 0,
    );
    push_action_line(
        &mut button_lines,
        app,
        1,
        format!(
            "󰦏 {}",
            lang_text(app, "已扫码，确认登录", "Scanned, Confirm Login")
        ),
        app.login.focus_index == 1,
    );

    let btn_chunk_idx = if has_url { 3 } else { 2 };
    frame.render_widget(
        Paragraph::new(button_lines)
            .style(Style::default().fg(app.theme.color_text()))
            .alignment(Alignment::Center),
        chunks[btn_chunk_idx],
    );
}

fn current_login_method_text(app: &App, method: LoginMethod) -> &'static str {
    match method {
        LoginMethod::Qr => lang_text(app, "当前方式：二维码", "Current: QR Code"),
        LoginMethod::Username => lang_text(app, "当前方式：账户", "Current: Username"),
        LoginMethod::Phone => lang_text(app, "当前方式：手机号", "Current: Phone"),
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn build_form_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    match app.login.method {
        LoginMethod::Qr => {
            push_action_line(
                &mut lines,
                app,
                0,
                format!("󰐑 {}", lang_text(app, "刷新二维码", "Refresh QR")),
                app.login.focus_index == 0,
            );
            push_action_line(
                &mut lines,
                app,
                1,
                format!(
                    "󰦏 {}",
                    lang_text(app, "已扫码，确认登录", "Scanned, Confirm Login")
                ),
                app.login.focus_index == 1,
            );
            if !app.login.qr_url.trim().is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("{}: {}", lang_text(app, "二维码", "QR"), app.login.qr_url),
                    Style::default().fg(app.theme.color_subtext()),
                )));
            }
        }
        LoginMethod::Username => {
            push_input_line(
                &mut lines,
                app,
                0,
                &format!("󰀄 {}", lang_text(app, "用户名", "Username")),
                &app.login.username,
                false,
                app.login.focus_index == 0,
            );
            push_input_line(
                &mut lines,
                app,
                1,
                &format!("󰍛 {}", lang_text(app, "密码", "Password")),
                &app.login.password,
                true,
                app.login.focus_index == 1,
            );
            push_action_line(
                &mut lines,
                app,
                2,
                format!("󰌋 {}", lang_text(app, "登录", "Login")),
                app.login.focus_index == 2,
            );
        }
        LoginMethod::Phone => {
            push_input_line(
                &mut lines,
                app,
                0,
                &format!("󰌘 {}", lang_text(app, "手机号", "Phone")),
                &app.login.phone,
                false,
                app.login.focus_index == 0,
            );
            push_input_line(
                &mut lines,
                app,
                1,
                &format!("󰣖 {}", lang_text(app, "验证码", "Captcha")),
                &app.login.captcha,
                false,
                app.login.focus_index == 1,
            );
            push_action_line(
                &mut lines,
                app,
                2,
                format!("󰯈 {}", lang_text(app, "发送验证码", "Send Captcha")),
                app.login.focus_index == 2,
            );
            push_action_line(
                &mut lines,
                app,
                3,
                format!("󰌋 {}", lang_text(app, "登录", "Login")),
                app.login.focus_index == 3,
            );
        }
    }

    lines
}



fn push_action_line(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    _index: usize,
    label: String,
    focused: bool,
) {
    let prefix = if focused { "▌ " } else { "  " };
    let style = if focused {
        Style::default()
            .fg(app.theme.color_base())
            .bg(app.theme.color_accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(app.theme.color_text())
            .bg(app.theme.color_surface())
    };

    lines.push(Line::from(Span::styled(format!("{prefix}{label}"), style)));
}

fn push_input_line(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    _index: usize,
    label: &str,
    value: &str,
    is_password: bool,
    focused: bool,
) {
    let shown = if value.is_empty() {
        "…".to_string()
    } else if is_password {
        "•".repeat(value.chars().count())
    } else {
        value.to_string()
    };

    let cursor = if focused { "▌" } else { "" };
    let row_style = if focused {
        Style::default().bg(app.theme.color_buff())
    } else {
        Style::default().bg(app.theme.color_surface())
    };
    let value_style = if value.is_empty() {
        Style::default().fg(app.theme.color_subtext())
    } else {
        Style::default().fg(app.theme.color_text())
    };
    let prefix = if focused { "▌ " } else { "  " };

    lines.push(Line::from(vec![
        Span::styled(
            format!("{prefix}{label}: "),
            row_style.fg(app.theme.color_accent2()),
        ),
        Span::styled(format!("{}{}", shown, cursor), row_style.patch(value_style)),
    ]));
}

fn base_bg_style(app: &App) -> Style {
    if app.config.transparent_background {
        Style::default()
    } else {
        Style::default().bg(app.theme.color_base())
    }
}

fn lang_text<'a>(app: &App, zh: &'a str, en: &'a str) -> &'a str {
    match app.config.language {
        Language::Zh => zh,
        Language::En => en,
    }
}
