use crate::data::config::CustomPaletteConfig;
use crate::tmplayer::data::assets;
use crate::tmplayer::ui::theme::{Theme, ThemeName, ThemePalette, detect_color_capability};
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
        let t: ThemeToml = toml::from_str(&raw)?;

        let default_base = (17, 17, 27);
        let default_surface = (24, 24, 37);
        let default_text = (242, 244, 248);
        let default_subtext = (148, 156, 187);
        let default_accent = (51, 204, 255);
        let default_accent2 = (0, 255, 153);
        let default_accent3 = (203, 166, 247);

        let surface = t
            .surface
            .as_deref()
            .map(parse_hex)
            .unwrap_or(default_surface);

        let buff_hex = if let Some(buff) = t.buff.clone() {
            buff
        } else if let Some(ref s) = t.surface {
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
            text: t.text.as_deref().map(parse_hex).unwrap_or(default_text),
            subtext: t
                .subtext
                .as_deref()
                .map(parse_hex)
                .unwrap_or(default_subtext),
            base: t.base.as_deref().map(parse_hex).unwrap_or(default_base),
            surface,
            buff: parse_hex(&buff_hex),
            accent: t
                .accent
                .as_deref()
                .map(parse_hex)
                .unwrap_or(default_accent),
            accent2: t
                .accent2
                .as_deref()
                .map(parse_hex)
                .unwrap_or(default_accent2),
            accent3: t
                .accent3
                .as_deref()
                .map(parse_hex)
                .unwrap_or(default_accent3),
        };

        if let Some(ov) = overrides {
            palette.apply_overrides(ov);
        }

        let name = ThemeName::from_str_or_system(spec);
        let capability = detect_color_capability();

        Ok(Theme {
            name,
            palette,
            capability,
        })
    }

    pub fn resolve_theme_path(spec: &str) -> PathBuf {
        let _ = assets::ensure_assets_ready();
        resolve_theme_file(spec)
    }

    pub fn available_themes() -> Vec<String> {
        let _ = assets::ensure_assets_ready();
        let mut themes = vec![
            "noctalia".to_string(),
            "hyprland".to_string(),
            "mocha".to_string(),
            "macchiato".to_string(),
            "frappe".to_string(),
            "latte".to_string(),
            "system".to_string(),
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
                            "system" | "hyprland" | "noctalia" => continue,
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
        "noctalia" => Some("themes/noctalia.toml"),
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
        if spec.eq_ignore_ascii_case("noctalia") {
            try_generate_or_fallback_noctalia(&p);
            if p.is_file() {
                return p;
            }
        }
    }

    let custom_file = format!("themes/{}.toml", spec);
    let custom_p = assets::resolve_asset_path(Path::new(&custom_file));
    if custom_p.is_file() {
        return custom_p;
    }

    assets::resolve_asset_path(Path::new("themes/system.toml"))
}

fn try_generate_or_fallback_noctalia(target_path: &Path) {
    if let Some(dirs) = directories::BaseDirs::new() {
        let config_dir = dirs.config_dir();

        let mut text = None;
        let subtext = None;
        let mut base = None;
        let surface = None;
        let buff = None;
        let mut accent = None;
        let mut accent2 = None;
        let mut accent3 = None;

        // 1. Try alacritty noctalia.toml
        let alacritty = config_dir.join("alacritty/themes/noctalia.toml");
        if let Ok(raw) = fs::read_to_string(&alacritty) {
            for line in raw.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once('=') {
                    let k = k.trim();
                    let val = v.trim().trim_matches('\'').trim_matches('"');
                    if val.starts_with('#') {
                        match k {
                            "background" if base.is_none() => base = Some(val.to_string()),
                            "foreground" if text.is_none() => text = Some(val.to_string()),
                            "magenta" if accent.is_none() => accent = Some(val.to_string()),
                            "yellow" if accent2.is_none() => accent2 = Some(val.to_string()),
                            "blue" if accent3.is_none() => accent3 = Some(val.to_string()),
                            _ => {}
                        }
                    }
                }
            }
        }

        // 2. Try kitty noctalia.conf if missing
        let kitty = config_dir.join("kitty/themes/noctalia.conf");
        if let Ok(raw) = fs::read_to_string(&kitty) {
            for line in raw.lines() {
                let mut parts = line.split_whitespace();
                if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                    let val = v.trim_matches('\'').trim_matches('"');
                    if val.starts_with('#') {
                        match k {
                            "background" if base.is_none() => base = Some(val.to_string()),
                            "foreground" if text.is_none() => text = Some(val.to_string()),
                            "active_border_color" if accent.is_none() => accent = Some(val.to_string()),
                            "color3" if accent2.is_none() => accent2 = Some(val.to_string()),
                            "color4" if accent3.is_none() => accent3 = Some(val.to_string()),
                            _ => {}
                        }
                    }
                }
            }
        }

        if let (Some(b), Some(t), Some(a)) = (base.as_deref(), text.as_deref(), accent.as_deref()) {
            let def_base = parse_hex(b);
            let s_val = surface.unwrap_or_else(|| {
                format!(
                    "#{:02X}{:02X}{:02X}",
                    def_base.0.saturating_add(13),
                    def_base.1.saturating_add(13),
                    def_base.2.saturating_add(13)
                )
            });
            let b_val = buff.unwrap_or_else(|| {
                format!(
                    "#{:02X}{:02X}{:02X}",
                    def_base.0.saturating_add(24),
                    def_base.1.saturating_add(24),
                    def_base.2.saturating_add(24)
                )
            });
            let sub_val = subtext.unwrap_or_else(|| "#d3c2c9".to_string());
            let a2_val = accent2.unwrap_or_else(|| "#debece".to_string());
            let a3_val = accent3.unwrap_or_else(|| "#f4ba9f".to_string());

            let toml_content = format!(
                "# Noctalia theme for Tune\n# Generated from system Noctalia configuration\nname = \"noctalia\"\n\ntext = \"{}\"\nsubtext = \"{}\"\nbase = \"{}\"\nsurface = \"{}\"\nbuff = \"{}\"\naccent = \"{}\"\naccent2 = \"{}\"\naccent3 = \"{}\"\n",
                t, sub_val, b, s_val, b_val, a, a2_val, a3_val
            );
            if let Some(parent) = target_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(target_path, toml_content);
        }
    }
}

fn parse_hex(s: &str) -> (u8, u8, u8) {
    let s = s.trim().trim_start_matches('#');
    if !s.is_ascii() || s.len() != 6 {
        return (255, 255, 255);
    }

    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
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
