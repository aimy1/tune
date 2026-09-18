use crate::data::assets;
use crate::data::config::CustomPaletteConfig;
use crate::ui::theme::{Theme, ThemeName, ThemePalette, detect_color_capability};
use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ThemeLoader;

#[derive(Debug, Deserialize)]
struct ThemeToml {
    #[serde(default)]
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    subtext: Option<String>,
    #[serde(default)]
    base: Option<String>,
    #[serde(default)]
    surface: Option<String>,
    #[serde(default)]
    buff: Option<String>,
    #[serde(default)]
    accent: Option<String>,
    #[serde(default)]
    accent2: Option<String>,
    #[serde(default)]
    accent3: Option<String>,
}

impl ThemeLoader {
    #[allow(dead_code)]
    pub fn load(name: &str) -> Result<Theme> {
        Self::load_with_overrides(name, None)
    }

    pub fn load_with_overrides(
        spec: &str,
        overrides: Option<&CustomPaletteConfig>,
    ) -> Result<Theme> {
        let _ = assets::ensure_assets_ready();
        let path = resolve_theme_file(spec);
        let raw = fs::read_to_string(&path)?;
        let parsed: ThemeToml = toml::from_str(&raw)?;

        let default_base = (17, 17, 27);
        let default_surface = (24, 24, 37);
        let default_text = (242, 244, 248);
        let default_subtext = (148, 156, 187);
        let default_accent = (51, 204, 255);
        let default_accent2 = (0, 255, 153);
        let default_accent3 = (203, 166, 247);

        let surface = parsed
            .surface
            .as_deref()
            .map(parse_hex)
            .unwrap_or(default_surface);

        let buff_hex = if let Some(buff) = parsed.buff.clone() {
            buff
        } else if let Some(ref s) = parsed.surface {
            let generated = derive_buff_hex(s);
            let upgraded = inject_buff_entry(&raw, &generated);
            let _ = fs::write(&path, upgraded);
            generated
        } else {
            format!(
                "#{:02X}{:02X}{:02X}",
                surface.0.saturating_add(10),
                surface.1.saturating_add(10),
                surface.2.saturating_add(10)
            )
        };

        let mut palette = ThemePalette {
            text: parsed.text.as_deref().map(parse_hex).unwrap_or(default_text),
            subtext: parsed
                .subtext
                .as_deref()
                .map(parse_hex)
                .unwrap_or(default_subtext),
            base: parsed.base.as_deref().map(parse_hex).unwrap_or(default_base),
            surface,
            buff: parse_hex(&buff_hex),
            accent: parsed
                .accent
                .as_deref()
                .map(parse_hex)
                .unwrap_or(default_accent),
            accent2: parsed
                .accent2
                .as_deref()
                .map(parse_hex)
                .unwrap_or(default_accent2),
            accent3: parsed
                .accent3
                .as_deref()
                .map(parse_hex)
                .unwrap_or(default_accent3),
        };

        if let Some(ov) = overrides {
            palette.apply_overrides(ov);
        }

        let name = ThemeName::from_str_or_system(spec);

        Ok(Theme {
            name,
            capability: detect_color_capability(),
            palette,
        })
    }

    pub fn available_themes() -> Vec<String> {
        let _ = assets::ensure_assets_ready();
        let mut themes = vec![
            "system".to_string(),
            "hyprland".to_string(),
            "latte".to_string(),
            "frappe".to_string(),
            "macchiato".to_string(),
            "mocha".to_string(),
        ];

        let themes_dir = assets::resolve_asset_path(Path::new("themes"));
        if let Ok(entries) = fs::read_dir(&themes_dir) {
            let mut custom_themes = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path.extension().and_then(|ext| ext.to_str()) == Some("toml")
                {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let canonical = match stem {
                            "system" | "hyprland" => continue,
                            "catppuccin_latte" => continue,
                            "catppuccin_frappe" => continue,
                            "catppuccin_macchiato" => continue,
                            "catppuccin_mocha" => continue,
                            other => other.to_string(),
                        };
                        if !themes.contains(&canonical) && !custom_themes.contains(&canonical) {
                            custom_themes.push(canonical);
                        }
                    }
                }
            }
            custom_themes.sort();
            themes.extend(custom_themes);
        }

        themes
    }
}

fn resolve_theme_file(spec: &str) -> PathBuf {
    let direct = PathBuf::from(spec);
    if direct.is_file() {
        return direct;
    }
    let as_asset = assets::resolve_asset_path(&direct);
    if as_asset.is_file() {
        return as_asset;
    }

    let built_in_rel = match spec.to_lowercase().as_str() {
        "system" => Some("themes/system.toml"),
        "hyprland" => Some("themes/hyprland.toml"),
        "latte" | "catppuccin_latte" => Some("themes/catppuccin_latte.toml"),
        "frappe" | "catppuccin_frappe" => Some("themes/catppuccin_frappe.toml"),
        "macchiato" | "catppuccin_macchiato" => Some("themes/catppuccin_macchiato.toml"),
        "mocha" | "catppuccin_mocha" => Some("themes/catppuccin_mocha.toml"),
        _ => None,
    };

    if let Some(rel) = built_in_rel {
        let p = assets::resolve_asset_path(Path::new(rel));
        if p.is_file() {
            return p;
        }
    }

    let custom_file = format!("themes/{}.toml", spec);
    let custom_p = assets::resolve_asset_path(Path::new(&custom_file));
    if custom_p.is_file() {
        return custom_p;
    }

    assets::resolve_asset_path(Path::new("themes/system.toml"))
}

fn parse_hex(raw: &str) -> (u8, u8, u8) {
    let hex = raw.trim().trim_start_matches('#');
    if !hex.is_ascii() || hex.len() != 6 {
        return (255, 255, 255);
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    (r, g, b)
}

fn derive_buff_hex(surface_hex: &str) -> String {
    let (r, g, b) = parse_hex(surface_hex);
    format!(
        "#{:02X}{:02X}{:02X}",
        r.saturating_add(10),
        g.saturating_add(10),
        b.saturating_add(10)
    )
}

fn inject_buff_entry(raw: &str, buff_hex: &str) -> String {
    if raw.lines().any(|line| is_toml_key(line, "buff")) {
        return raw.to_string();
    }

    let mut out = String::with_capacity(raw.len() + 24);
    let mut inserted = false;

    for line in raw.lines() {
        out.push_str(line);
        out.push('\n');
        if !inserted && is_toml_key(line, "surface") {
            out.push_str(&format!("buff = \"{}\"\n", buff_hex));
            inserted = true;
        }
    }

    if !inserted {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!("buff = \"{}\"\n", buff_hex));
    }

    out
}

fn is_toml_key(line: &str, key: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with(key) {
        return false;
    }
    trimmed[key.len()..].trim_start().starts_with('=')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_ascii() {
        assert_eq!(parse_hex("#123456"), (0x12, 0x34, 0x56));
        assert_eq!(parse_hex("ffffff"), (255, 255, 255));
    }

    #[test]
    fn test_parse_hex_non_ascii_no_panic() {
        // "你好" is 6 bytes in UTF-8, previously would panic on &hex[0..2]
        assert_eq!(parse_hex("#你好"), (255, 255, 255));
        assert_eq!(parse_hex("你好"), (255, 255, 255));
        assert_eq!(parse_hex(""), (255, 255, 255));
        assert_eq!(parse_hex("123"), (255, 255, 255));
    }

    #[test]
    fn test_load_with_palette_overrides() {
        let overrides = CustomPaletteConfig {
            accent: Some("#FF007F".to_string()),
            text: Some("#AABBCC".to_string()),
            ..Default::default()
        };

        let theme = ThemeLoader::load_with_overrides("hyprland", Some(&overrides)).unwrap();
        assert_eq!(theme.palette.accent, (0xFF, 0x00, 0x7F));
        assert_eq!(theme.palette.text, (0xAA, 0xBB, 0xCC));
        // Un-overridden colors stay from hyprland
        assert_eq!(theme.palette.accent2, (0x00, 0xFF, 0x99));
    }

    #[test]
    fn test_available_themes_contains_builtins() {
        let themes = ThemeLoader::available_themes();
        assert!(themes.contains(&"system".to_string()));
        assert!(themes.contains(&"hyprland".to_string()));
        assert!(themes.contains(&"latte".to_string()));
        assert!(themes.contains(&"frappe".to_string()));
        assert!(themes.contains(&"macchiato".to_string()));
        assert!(themes.contains(&"mocha".to_string()));
    }

    #[test]
    fn test_load_custom_theme_file() {
        let temp_dir = std::env::temp_dir();
        let custom_file = temp_dir.join("tune_test_custom_theme.toml");
        let content = r##"
name = "my_custom"
text = "#112233"
subtext = "#445566"
base = "#010203"
surface = "#102030"
accent = "#AABB00"
accent2 = "#CCDDEE"
accent3 = "#FF1122"
"##;
        std::fs::write(&custom_file, content).unwrap();

        let loaded = ThemeLoader::load(custom_file.to_str().unwrap()).unwrap();
        assert_eq!(loaded.palette.text, (0x11, 0x22, 0x33));
        assert_eq!(loaded.palette.subtext, (0x44, 0x55, 0x66));
        assert_eq!(loaded.palette.base, (0x01, 0x02, 0x03));
        assert_eq!(loaded.palette.surface, (0x10, 0x20, 0x30));
        assert_eq!(loaded.palette.accent, (0xAA, 0xBB, 0x00));
        assert_eq!(loaded.palette.accent2, (0xCC, 0xDD, 0xEE));
        assert_eq!(loaded.palette.accent3, (0xFF, 0x11, 0x22));

        let _ = std::fs::remove_file(custom_file);
    }
}
