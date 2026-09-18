use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    TrueColor,
    Ansi256,
    NoColor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeName {
    System,
    Hyprland,
    Latte,
    Frappe,
    Macchiato,
    Mocha,
    Custom(String),
}

impl ThemeName {
    pub fn from_str_or_system(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "hyprland" => Self::Hyprland,
            "latte" => Self::Latte,
            "frappe" => Self::Frappe,
            "macchiato" => Self::Macchiato,
            "mocha" => Self::Mocha,
            "system" => Self::System,
            other if !other.is_empty() => Self::Custom(s.to_string()),
            _ => Self::System,
        }
    }

    #[allow(dead_code)]
    pub fn as_label(&self) -> &str {
        match self {
            ThemeName::System => "System",
            ThemeName::Hyprland => "Hyprland",
            ThemeName::Latte => "Latte",
            ThemeName::Frappe => "Frappe",
            ThemeName::Macchiato => "Macchiato",
            ThemeName::Mocha => "Mocha",
            ThemeName::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    pub text: (u8, u8, u8),
    pub subtext: (u8, u8, u8),
    pub base: (u8, u8, u8),
    pub surface: (u8, u8, u8),
    #[allow(dead_code)]
    pub buff: (u8, u8, u8),
    pub accent: (u8, u8, u8),
    pub accent2: (u8, u8, u8),
    pub accent3: (u8, u8, u8),
}

impl ThemePalette {
    pub fn apply_overrides(&mut self, overrides: &crate::data::config::CustomPaletteConfig) {
        if let Some(ref hex) = overrides.text {
            self.text = parse_hex_color(hex);
        }
        if let Some(ref hex) = overrides.subtext {
            self.subtext = parse_hex_color(hex);
        }
        if let Some(ref hex) = overrides.base {
            self.base = parse_hex_color(hex);
        }
        if let Some(ref hex) = overrides.surface {
            self.surface = parse_hex_color(hex);
        }
        if let Some(ref hex) = overrides.buff {
            self.buff = parse_hex_color(hex);
        }
        if let Some(ref hex) = overrides.accent {
            self.accent = parse_hex_color(hex);
        }
        if let Some(ref hex) = overrides.accent2 {
            self.accent2 = parse_hex_color(hex);
        }
        if let Some(ref hex) = overrides.accent3 {
            self.accent3 = parse_hex_color(hex);
        }
    }
}

pub fn parse_hex_color(raw: &str) -> (u8, u8, u8) {
    let hex = raw.trim().trim_start_matches('#');
    if !hex.is_ascii() || hex.len() != 6 {
        return (255, 255, 255);
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    (r, g, b)
}

#[derive(Debug, Clone)]
pub struct Theme {
    #[allow(dead_code)]
    pub name: ThemeName,
    pub palette: ThemePalette,
    pub capability: ColorCapability,
}

impl Theme {
    pub fn color_text(&self) -> Color {
        map_color(self.capability, self.palette.text)
    }
    pub fn color_subtext(&self) -> Color {
        map_color(self.capability, self.palette.subtext)
    }
    pub fn color_base(&self) -> Color {
        map_color(self.capability, self.palette.base)
    }
    pub fn color_surface(&self) -> Color {
        map_color(self.capability, self.palette.surface)
    }
    #[allow(dead_code)]
    pub fn color_buff(&self) -> Color {
        map_color(self.capability, self.palette.buff)
    }
    pub fn color_accent(&self) -> Color {
        map_color(self.capability, self.palette.accent)
    }
    pub fn color_accent2(&self) -> Color {
        map_color(self.capability, self.palette.accent2)
    }
    pub fn color_accent3(&self) -> Color {
        map_color(self.capability, self.palette.accent3)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: ThemeName::System,
            capability: ColorCapability::TrueColor,
            palette: ThemePalette {
                text: (242, 244, 248),
                subtext: (148, 156, 187),
                base: (17, 17, 27),
                surface: (24, 24, 37),
                buff: (42, 43, 61),
                accent: (51, 204, 255),
                accent2: (0, 255, 153),
                accent3: (203, 166, 247),
            },
        }
    }
}

pub fn detect_color_capability() -> ColorCapability {
    let colorterm = std::env::var("COLORTERM")
        .unwrap_or_default()
        .to_lowercase();
    if colorterm.contains("truecolor") || colorterm.contains("24bit") {
        return ColorCapability::TrueColor;
    }

    let term = std::env::var("TERM").unwrap_or_default().to_lowercase();
    if term.contains("256color") {
        return ColorCapability::Ansi256;
    }

    ColorCapability::NoColor
}

fn map_color(cap: ColorCapability, t: (u8, u8, u8)) -> Color {
    match cap {
        ColorCapability::TrueColor => Color::Rgb(t.0, t.1, t.2),
        ColorCapability::Ansi256 => Color::Indexed(rgb_to_ansi256(t.0, t.1, t.2)),
        ColorCapability::NoColor => Color::Reset,
    }
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // 6x6x6 color cube, 16..231
    let r6 = (r as u16 * 5 / 255) as u8;
    let g6 = (g as u16 * 5 / 255) as u8;
    let b6 = (b as u16 * 5 / 255) as u8;
    16 + 36 * r6 + 6 * g6 + b6
}
