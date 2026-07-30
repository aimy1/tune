use ratatui::style::Color;
use ratatui::style::Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    TrueColor,
    Ansi256,
    NoColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    System,
    Hyprland,
    Latte,
    Frappe,
    Macchiato,
    Mocha,
}

impl ThemeName {
    pub fn from_str_or_system(raw: &str) -> Self {
        match raw.to_lowercase().as_str() {
            "hyprland" => Self::Hyprland,
            "latte" => Self::Latte,
            "frappe" => Self::Frappe,
            "macchiato" => Self::Macchiato,
            "mocha" => Self::Mocha,
            _ => Self::System,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub text: (u8, u8, u8),
    pub subtext: (u8, u8, u8),
    pub base: (u8, u8, u8),
    pub surface: (u8, u8, u8),
    pub buff: (u8, u8, u8),
    pub accent: (u8, u8, u8),
    pub accent2: (u8, u8, u8),
    pub accent3: (u8, u8, u8),
}

#[derive(Debug, Clone, Copy)]
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

    pub fn style_surface_bg(&self, transparent: bool) -> Style {
        if transparent {
            Style::default()
        } else {
            Style::default().bg(self.color_surface())
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: ThemeName::Hyprland,
            capability: detect_color_capability(),
            palette: ThemePalette {
                text: (242, 244, 248),
                subtext: (148, 156, 187),
                base: (17, 17, 27),
                surface: (24, 24, 37),
                buff: (42, 43, 61),
                accent: (51, 204, 255),  // Hyprland Cyan #33CCFF
                accent2: (0, 255, 153),  // Hyprland Emerald #00FF99
                accent3: (203, 166, 247), // Hyprland / Noctalia Purple #CBA6F7
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

fn map_color(cap: ColorCapability, rgb: (u8, u8, u8)) -> Color {
    match cap {
        ColorCapability::TrueColor => Color::Rgb(rgb.0, rgb.1, rgb.2),
        ColorCapability::Ansi256 => Color::Indexed(rgb_to_ansi256(rgb.0, rgb.1, rgb.2)),
        ColorCapability::NoColor => Color::Reset,
    }
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let r6 = (r as u16 * 5 / 255) as u8;
    let g6 = (g as u16 * 5 / 255) as u8;
    let b6 = (b as u16 * 5 / 255) as u8;
    16 + 36 * r6 + 6 * g6 + b6
}
