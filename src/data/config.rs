use crate::data::assets;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_EQ_BANDS_DB: [f32; crate::tmplayer::app::state::EQ_BANDS] =
    [0.0; crate::tmplayer::app::state::EQ_BANDS];
const LEGACY_STARTUP_FOLDER_KEY: &str = concat!("default", "_opening", "_folder");
const LEGACY_STARTUP_FOLDER_KEY_KEBAB: &str = concat!("default", "-opening", "-folder");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GraphicsProtocol {
    Off,
    #[default]
    #[serde(alias = "auto")]
    #[serde(alias = "sixel")]
    #[serde(alias = "kitty")]
    #[serde(alias = "iterm2")]
    Halfblocks,
}

impl GraphicsProtocol {
    const ALL: [Self; 2] = [Self::Off, Self::Halfblocks];

    pub fn to_ratatui_protocol(self) -> Option<ratatui_image::picker::ProtocolType> {
        match self {
            GraphicsProtocol::Off => None,
            GraphicsProtocol::Halfblocks => Some(ratatui_image::picker::ProtocolType::Halfblocks),
        }
    }

    pub fn cycle(self, delta: i32) -> Self {
        if delta == 0 {
            return self;
        }

        let current = match self {
            GraphicsProtocol::Off => 0,
            GraphicsProtocol::Halfblocks => 1,
        };
        let next = (current + delta).rem_euclid(Self::ALL.len() as i32) as usize;
        Self::ALL[next]
    }

    pub fn display_name(self) -> &'static str {
        match self {
            GraphicsProtocol::Off => "off",
            GraphicsProtocol::Halfblocks => "Halfblocks",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub ui_fps: u32,
    pub spectrum_hz: u32,
    pub mpris_poll_ms: u64,

    #[serde(default = "default_visualize")]
    pub visualize: VisualizeMode,

    #[serde(default = "default_eq_bands_db")]
    pub eq_bands_db: [f32; crate::tmplayer::app::state::EQ_BANDS],

    #[serde(default)]
    pub transparent_background: bool,

    #[serde(default = "default_album_border")]
    pub album_border: bool,

    #[serde(default)]
    pub graphics_protocol: GraphicsProtocol,

    #[serde(default = "default_kitty_cover_scale_percent")]
    pub kitty_cover_scale_percent: u8,

    #[serde(default)]
    pub super_smooth_bar: bool,

    #[serde(default)]
    pub bars_gap: bool,

    #[serde(default = "default_bar_number")]
    pub bar_number: BarNumber,

    #[serde(default = "default_bar_channels")]
    pub bar_channels: BarChannels,

    #[serde(default)]
    pub bar_channel_reverse: bool,

    #[serde(default)]
    pub lyrics_cover_fetch: bool,

    #[serde(default)]
    pub lyrics_cover_download: bool,

    #[serde(default)]
    pub audio_fingerprint: bool,

    #[serde(default)]
    pub acoustid_api_key: String,

    #[serde(default)]
    pub resume_last_position: bool,

    #[serde(default)]
    pub default_opening_title: String,

    #[serde(default = "default_language")]
    pub language: Language,

    #[serde(default = "default_page_lyrics")]
    pub page_lyrics: bool,

    #[serde(default)]
    pub desktop_lyrics: bool,

    #[serde(default = "default_desktop_lyrics_locked")]
    pub desktop_lyrics_locked: bool,

    #[serde(default = "default_desktop_lyrics_font_size")]
    pub desktop_lyrics_font_size: u16,

    #[serde(default = "default_desktop_lyrics_dual_line")]
    pub desktop_lyrics_dual_line: bool,

    #[serde(default = "default_desktop_lyrics_align")]
    pub desktop_lyrics_align: DesktopLyricsAlign,

    #[serde(default = "default_desktop_lyrics_bg")]
    pub desktop_lyrics_bg: DesktopLyricsBg,

    #[serde(default = "default_desktop_lyrics_width")]
    pub desktop_lyrics_width: DesktopLyricsWidth,

    #[serde(default = "default_desktop_lyrics_opacity")]
    pub desktop_lyrics_opacity: u8,

    #[serde(default)]
    pub desktop_lyrics_pos_x: Option<i32>,

    #[serde(default)]
    pub desktop_lyrics_pos_y: Option<i32>,

    #[serde(default = "default_audio_quality")]
    pub audio_quality: AudioQuality,

    #[serde(default)]
    pub playback_memory: bool,

    #[serde(default)]
    pub transparent_sidebar: bool,

    #[serde(default = "default_show_hints")]
    pub show_hints: bool,

    #[serde(default = "default_volume")]
    pub volume: f32,

    #[serde(default)]
    pub home_more_recommend: bool,

    #[serde(default)]
    pub cache: CacheConfig,

    #[serde(default = "default_keybind_search_box")]
    pub keybind_search_box: String,

    #[serde(default = "default_keybind_fullscreen")]
    pub keybind_fullscreen: String,

    #[serde(default = "default_keybind_settings")]
    pub keybind_settings: String,

    #[serde(default = "default_keybind_sidebar")]
    pub keybind_sidebar: String,

    #[serde(default = "default_keybind_quit")]
    pub keybind_quit: String,

    #[serde(default = "default_keybind_prev")]
    pub keybind_prev: String,

    #[serde(default = "default_keybind_next")]
    pub keybind_next: String,

    #[serde(default = "default_keybind_toggle_play_pause")]
    pub keybind_toggle_play_pause: String,

    #[serde(default = "default_keybind_toggle_mode")]
    pub keybind_toggle_mode: String,

    #[serde(default = "default_keybind_fullscreen_prev")]
    pub keybind_fullscreen_prev: String,

    #[serde(default = "default_keybind_fullscreen_next")]
    pub keybind_fullscreen_next: String,

    #[serde(default = "default_keybind_fullscreen_toggle_play_pause")]
    pub keybind_fullscreen_toggle_play_pause: String,

    #[serde(default = "default_keybind_fullscreen_toggle_mode")]
    pub keybind_fullscreen_toggle_mode: String,

    #[serde(default = "default_keybind_fullscreen_eq")]
    pub keybind_fullscreen_eq: String,

    #[serde(default = "default_keybind_fullscreen_eq_reset")]
    pub keybind_fullscreen_eq_reset: String,

    #[serde(default = "default_keybind_toggle_like_fullscreen")]
    pub keybind_toggle_like_fullscreen: String,

    #[serde(default = "default_keybind_toggle_like_collapsed")]
    pub keybind_toggle_like_collapsed: String,

    #[serde(default = "default_keybind_personal_center")]
    pub keybind_personal_center: String,

    #[serde(default = "default_keybind_home")]
    pub keybind_home: String,

    #[serde(default = "default_keybind_desktop_lyrics")]
    pub keybind_desktop_lyrics: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum CacheCleanStrategy {
    Size,
    Age,
    #[default]
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub path: Option<String>,

    #[serde(default)]
    pub clean_strategy: CacheCleanStrategy,

    #[serde(default = "default_cache_max_size_mb")]
    pub max_size_mb: u64,

    #[serde(default = "default_cache_max_age_days")]
    pub max_age_days: u64,

    #[serde(default = "default_cache_clean_on_startup")]
    pub clean_on_startup: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            path: None,
            clean_strategy: CacheCleanStrategy::default(),
            max_size_mb: default_cache_max_size_mb(),
            max_age_days: default_cache_max_age_days(),
            clean_on_startup: default_cache_clean_on_startup(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisualizeMode {
    Off,
    Bars,
    Oscilloscope,
    Circle,
    Particles,
    Mirror,
}

impl VisualizeMode {
    pub fn cycle(self, delta: i32) -> Self {
        const MODES: [VisualizeMode; 6] = [
            VisualizeMode::Off,
            VisualizeMode::Bars,
            VisualizeMode::Oscilloscope,
            VisualizeMode::Circle,
            VisualizeMode::Particles,
            VisualizeMode::Mirror,
        ];

        let index = MODES.iter().position(|mode| *mode == self).unwrap_or(1) as i32;
        let next = (index + delta).rem_euclid(MODES.len() as i32) as usize;
        MODES[next]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarChannels {
    Stereo,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarNumber {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "16")]
    N16,
    #[serde(rename = "32")]
    N32,
    #[serde(rename = "48")]
    N48,
    #[serde(rename = "64")]
    N64,
    #[serde(rename = "80")]
    N80,
    #[serde(rename = "96")]
    N96,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Zh,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioQuality {
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "higher")]
    Higher,
    #[serde(rename = "exhigh")]
    Exhigh,
    #[serde(rename = "lossless")]
    Lossless,
    #[serde(rename = "hires")]
    Hires,
    #[serde(rename = "jyeffect")]
    Jyeffect,
    #[serde(rename = "sky")]
    Sky,
    #[serde(rename = "dolby")]
    Dolby,
    #[serde(rename = "jymaster")]
    Jymaster,
}

impl AudioQuality {
    pub const FREE_LEVELS: [Self; 3] = [Self::Standard, Self::Higher, Self::Exhigh];
    pub const ALL_LEVELS: [Self; 9] = [
        Self::Standard,
        Self::Higher,
        Self::Exhigh,
        Self::Lossless,
        Self::Hires,
        Self::Jyeffect,
        Self::Sky,
        Self::Dolby,
        Self::Jymaster,
    ];

    pub fn as_api_level(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Higher => "higher",
            Self::Exhigh => "exhigh",
            Self::Lossless => "lossless",
            Self::Hires => "hires",
            Self::Jyeffect => "jyeffect",
            Self::Sky => "sky",
            Self::Dolby => "dolby",
            Self::Jymaster => "jymaster",
        }
    }

    pub fn clamp_for_vip(self, vip_unlocked: bool) -> Self {
        if vip_unlocked {
            self
        } else {
            match self {
                Self::Standard | Self::Higher | Self::Exhigh => self,
                _ => Self::Exhigh,
            }
        }
    }

    pub fn cycle(self, delta: i32, vip_unlocked: bool) -> Self {
        let options: &[Self] = if vip_unlocked {
            &Self::ALL_LEVELS
        } else {
            &Self::FREE_LEVELS
        };

        let current = self.clamp_for_vip(vip_unlocked);
        let index = options
            .iter()
            .position(|item| *item == current)
            .unwrap_or(0) as i32;
        let next = (index + delta).rem_euclid(options.len() as i32) as usize;
        options[next]
    }
}

fn default_visualize() -> VisualizeMode {
    if crate::tmplayer::audio::cava::is_available() {
        VisualizeMode::Bars
    } else {
        VisualizeMode::Off
    }
}

fn default_eq_bands_db() -> [f32; crate::tmplayer::app::state::EQ_BANDS] {
    DEFAULT_EQ_BANDS_DB
}

fn default_album_border() -> bool {
    true
}

fn default_kitty_cover_scale_percent() -> u8 {
    100
}

fn default_bar_number() -> BarNumber {
    BarNumber::Auto
}

fn default_bar_channels() -> BarChannels {
    BarChannels::Mono
}

fn default_language() -> Language {
    Language::Zh
}

fn default_page_lyrics() -> bool {
    false
}

fn default_audio_quality() -> AudioQuality {
    AudioQuality::Exhigh
}

fn default_show_hints() -> bool {
    true
}

fn default_volume() -> f32 {
    1.0
}

fn default_cache_max_size_mb() -> u64 {
    500
}

fn default_cache_max_age_days() -> u64 {
    7
}

fn default_cache_clean_on_startup() -> bool {
    true
}

fn default_keybind_search_box() -> String {
    "S".to_string()
}

fn default_keybind_fullscreen() -> String {
    "F".to_string()
}

fn default_keybind_settings() -> String {
    ",".to_string()
}

fn default_keybind_sidebar() -> String {
    "B".to_string()
}

fn is_legacy_sidebar_default(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase().replace(' ', "");
    normalized == "alt+b"
}

fn default_keybind_quit() -> String {
    "Q".to_string()
}

fn default_keybind_prev() -> String {
    "[".to_string()
}

fn default_keybind_next() -> String {
    "]".to_string()
}

fn default_keybind_toggle_play_pause() -> String {
    "Space".to_string()
}

fn default_keybind_toggle_mode() -> String {
    "M".to_string()
}

fn default_keybind_fullscreen_prev() -> String {
    "Left".to_string()
}

fn default_keybind_fullscreen_next() -> String {
    "Right".to_string()
}

fn default_keybind_fullscreen_toggle_play_pause() -> String {
    "Space".to_string()
}

fn default_keybind_fullscreen_toggle_mode() -> String {
    "M".to_string()
}

fn default_keybind_fullscreen_eq() -> String {
    "E".to_string()
}

fn default_keybind_fullscreen_eq_reset() -> String {
    "Alt+R".to_string()
}

fn default_keybind_toggle_like_fullscreen() -> String {
    "L".to_string()
}

fn default_keybind_toggle_like_collapsed() -> String {
    "L".to_string()
}

fn default_keybind_personal_center() -> String {
    "Z".to_string()
}

fn default_keybind_home() -> String {
    "X".to_string()
}

fn default_keybind_desktop_lyrics() -> String {
    crate::app::keybinds::DEFAULT_KEYBIND_DESKTOP_LYRICS.to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DesktopLyricsAlign {
    #[default]
    Center,
    Left,
    Right,
}

impl DesktopLyricsAlign {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    pub fn cycle(self, delta: i32) -> Self {
        let items = [Self::Center, Self::Left, Self::Right];
        let idx = match self {
            Self::Center => 0,
            Self::Left => 1,
            Self::Right => 2,
        };
        let next = (idx as i32 + delta).rem_euclid(items.len() as i32) as usize;
        items[next]
    }

    pub fn display_name(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::Center, Language::Zh) => "居中",
            (Self::Center, Language::En) => "Center",
            (Self::Left, Language::Zh) => "居左",
            (Self::Left, Language::En) => "Left",
            (Self::Right, Language::Zh) => "居右",
            (Self::Right, Language::En) => "Right",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DesktopLyricsBg {
    #[default]
    Translucent,
    Dark,
    Light,
    Transparent,
}

impl DesktopLyricsBg {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Translucent => "translucent",
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Transparent => "transparent",
        }
    }

    pub fn cycle(self, delta: i32) -> Self {
        let items = [Self::Translucent, Self::Dark, Self::Light, Self::Transparent];
        let idx = match self {
            Self::Translucent => 0,
            Self::Dark => 1,
            Self::Light => 2,
            Self::Transparent => 3,
        };
        let next = (idx as i32 + delta).rem_euclid(items.len() as i32) as usize;
        items[next]
    }

    pub fn display_name(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::Translucent, Language::Zh) => "经典半透明",
            (Self::Translucent, Language::En) => "Translucent",
            (Self::Dark, Language::Zh) => "深色磨砂",
            (Self::Dark, Language::En) => "Dark Blur",
            (Self::Light, Language::Zh) => "轻薄高透",
            (Self::Light, Language::En) => "Light Glass",
            (Self::Transparent, Language::Zh) => "纯净无底色",
            (Self::Transparent, Language::En) => "Pure Text",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DesktopLyricsWidth {
    Compact,
    #[default]
    Standard,
    Wide,
    UltraWide,
}

impl DesktopLyricsWidth {
    pub fn pixel_width(&self) -> u16 {
        match self {
            Self::Compact => 560,
            Self::Standard => 760,
            Self::Wide => 960,
            Self::UltraWide => 1200,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Standard => "standard",
            Self::Wide => "wide",
            Self::UltraWide => "ultrawide",
        }
    }

    pub fn cycle(self, delta: i32) -> Self {
        let items = [Self::Compact, Self::Standard, Self::Wide, Self::UltraWide];
        let idx = match self {
            Self::Compact => 0,
            Self::Standard => 1,
            Self::Wide => 2,
            Self::UltraWide => 3,
        };
        let next = (idx as i32 + delta).rem_euclid(items.len() as i32) as usize;
        items[next]
    }

    pub fn display_name(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::Compact, Language::Zh) => "紧凑 (560px)",
            (Self::Compact, Language::En) => "Compact (560px)",
            (Self::Standard, Language::Zh) => "标准 (760px)",
            (Self::Standard, Language::En) => "Standard (760px)",
            (Self::Wide, Language::Zh) => "宽屏 (960px)",
            (Self::Wide, Language::En) => "Wide (960px)",
            (Self::UltraWide, Language::Zh) => "超宽 (1200px)",
            (Self::UltraWide, Language::En) => "Ultra-Wide (1200px)",
        }
    }
}

pub const DESKTOP_LYRICS_OPACITIES: [u8; 5] = [100, 85, 70, 50, 30];

pub fn cycle_desktop_lyrics_opacity(current: u8, delta: i32) -> u8 {
    let list = DESKTOP_LYRICS_OPACITIES;
    let idx = list.iter().position(|&o| o == current).unwrap_or(1);
    let next = (idx as i32 + delta).rem_euclid(list.len() as i32) as usize;
    list[next]
}

pub const DESKTOP_LYRICS_FONT_SIZES: [u16; 6] = [16, 18, 20, 24, 28, 32];

pub fn cycle_desktop_lyrics_font_size(current: u16, delta: i32) -> u16 {
    let sizes = DESKTOP_LYRICS_FONT_SIZES;
    let idx = sizes.iter().position(|&s| s == current).unwrap_or(2);
    let next = (idx as i32 + delta).rem_euclid(sizes.len() as i32) as usize;
    sizes[next]
}

fn default_desktop_lyrics_locked() -> bool {
    true
}

fn default_desktop_lyrics_font_size() -> u16 {
    20
}

fn default_desktop_lyrics_dual_line() -> bool {
    true
}

fn default_desktop_lyrics_align() -> DesktopLyricsAlign {
    DesktopLyricsAlign::Center
}

fn default_desktop_lyrics_bg() -> DesktopLyricsBg {
    DesktopLyricsBg::Translucent
}

fn default_desktop_lyrics_width() -> DesktopLyricsWidth {
    DesktopLyricsWidth::Standard
}

fn default_desktop_lyrics_opacity() -> u8 {
    85
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "frappe".to_string(),
            ui_fps: 30,
            spectrum_hz: 60,
            mpris_poll_ms: 100,
            visualize: default_visualize(),
            eq_bands_db: default_eq_bands_db(),
            transparent_background: true,
            album_border: default_album_border(),
            graphics_protocol: GraphicsProtocol::default(),
            kitty_cover_scale_percent: default_kitty_cover_scale_percent(),
            super_smooth_bar: false,
            bars_gap: false,
            bar_number: default_bar_number(),
            bar_channels: default_bar_channels(),
            bar_channel_reverse: false,
            lyrics_cover_fetch: false,
            lyrics_cover_download: false,
            audio_fingerprint: false,
            acoustid_api_key: String::new(),
            resume_last_position: false,
            default_opening_title: String::new(),
            language: default_language(),
            page_lyrics: default_page_lyrics(),
            desktop_lyrics: false,
            desktop_lyrics_locked: default_desktop_lyrics_locked(),
            desktop_lyrics_font_size: default_desktop_lyrics_font_size(),
            desktop_lyrics_dual_line: default_desktop_lyrics_dual_line(),
            desktop_lyrics_align: default_desktop_lyrics_align(),
            desktop_lyrics_bg: default_desktop_lyrics_bg(),
            desktop_lyrics_width: default_desktop_lyrics_width(),
            desktop_lyrics_opacity: default_desktop_lyrics_opacity(),
            desktop_lyrics_pos_x: None,
            desktop_lyrics_pos_y: None,
            audio_quality: default_audio_quality(),
            playback_memory: false,
            transparent_sidebar: false,
            show_hints: default_show_hints(),
            volume: default_volume(),
            home_more_recommend: false,
            cache: CacheConfig::default(),
            keybind_search_box: default_keybind_search_box(),
            keybind_fullscreen: default_keybind_fullscreen(),
            keybind_settings: default_keybind_settings(),
            keybind_sidebar: default_keybind_sidebar(),
            keybind_quit: default_keybind_quit(),
            keybind_prev: default_keybind_prev(),
            keybind_next: default_keybind_next(),
            keybind_toggle_play_pause: default_keybind_toggle_play_pause(),
            keybind_toggle_mode: default_keybind_toggle_mode(),
            keybind_fullscreen_prev: default_keybind_fullscreen_prev(),
            keybind_fullscreen_next: default_keybind_fullscreen_next(),
            keybind_fullscreen_toggle_play_pause: default_keybind_fullscreen_toggle_play_pause(),
            keybind_fullscreen_toggle_mode: default_keybind_fullscreen_toggle_mode(),
            keybind_fullscreen_eq: default_keybind_fullscreen_eq(),
            keybind_fullscreen_eq_reset: default_keybind_fullscreen_eq_reset(),
            keybind_toggle_like_fullscreen: default_keybind_toggle_like_fullscreen(),
            keybind_toggle_like_collapsed: default_keybind_toggle_like_collapsed(),
            keybind_personal_center: default_keybind_personal_center(),
            keybind_home: default_keybind_home(),
            keybind_desktop_lyrics: default_keybind_desktop_lyrics(),
        }
    }
}

impl Config {
    pub fn load_or_default() -> Result<Self> {
        let _ = assets::ensure_assets_ready();
        let path = Self::default_path();
        if !path.exists() {
            let cfg = Self::default();
            let _ = cfg.save();
            return Ok(cfg);
        }

        let raw = fs::read_to_string(path)?;
        let legacy_startup_folder_key_present = raw.contains(LEGACY_STARTUP_FOLDER_KEY_KEBAB)
            || raw.contains(LEGACY_STARTUP_FOLDER_KEY);
        let graphics_protocol_needs_save = graphics_protocol_needs_save(&raw);
        let mut cfg: Config = toml::from_str(&raw).unwrap_or_default();

        if cfg.ui_fps == 0 {
            cfg.ui_fps = 30;
        }
        if cfg.spectrum_hz == 0 {
            cfg.spectrum_hz = 60;
        }

        let mut forced_visualize_off = false;
        if !crate::tmplayer::audio::cava::is_available() && cfg.visualize != VisualizeMode::Off {
            cfg.visualize = VisualizeMode::Off;
            forced_visualize_off = true;
        }

        let mut migrated_legacy_sidebar = false;
        if is_legacy_sidebar_default(&cfg.keybind_sidebar) {
            cfg.keybind_sidebar = default_keybind_sidebar();
            migrated_legacy_sidebar = true;
        }

        if !raw.contains("default_opening_title")
            || !raw.contains("language")
            || !raw.contains("page_lyrics")
            || !raw.contains("desktop_lyrics")
            || !raw.contains("eq_bands_db")
            || !raw.contains("audio_quality")
            || !raw.contains("playback_memory")
            || !raw.contains("transparent_sidebar")
            || !raw.contains("show_hints")
            || !raw.contains("home_more_recommend")
            || !raw.contains("[cache]")
            || !raw.contains("bar_number")
            || !raw.contains("bar_channels")
            || !raw.contains("bar_channel_reverse")
            || graphics_protocol_needs_save
            || !raw.contains("keybind_search_box")
            || !raw.contains("keybind_fullscreen")
            || !raw.contains("keybind_settings")
            || !raw.contains("keybind_sidebar")
            || !raw.contains("keybind_quit")
            || !raw.contains("keybind_prev")
            || forced_visualize_off
            || !raw.contains("keybind_next")
            || !raw.contains("keybind_toggle_play_pause")
            || !raw.contains("keybind_toggle_mode")
            || !raw.contains("keybind_fullscreen_prev")
            || !raw.contains("keybind_fullscreen_next")
            || !raw.contains("keybind_fullscreen_toggle_play_pause")
            || !raw.contains("keybind_fullscreen_toggle_mode")
            || !raw.contains("keybind_fullscreen_eq")
            || !raw.contains("keybind_fullscreen_eq_reset")
            || !raw.contains("keybind_toggle_like_fullscreen")
            || !raw.contains("keybind_toggle_like_collapsed")
            || legacy_startup_folder_key_present
            || migrated_legacy_sidebar
        {
            let _ = cfg.save();
        }

        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let _ = assets::ensure_assets_ready();
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let raw = toml::to_string_pretty(self).unwrap_or_default();
        fs::write(path, raw)?;
        Ok(())
    }

    fn default_path() -> PathBuf {
        assets::resolve_config_path()
    }
}

fn graphics_protocol_needs_save(raw: &str) -> bool {
    let Some(value) = raw.lines().map(str::trim).find_map(|line| {
        if line.starts_with('#') || !line.starts_with("graphics_protocol") {
            return None;
        }

        let (_, value) = line.split_once('=')?;
        let value = value.split('#').next()?.trim().trim_matches('"');
        Some(value)
    }) else {
        return true;
    };

    matches!(value, "auto" | "sixel" | "kitty" | "iterm2")
}

#[cfg(test)]
mod tests {
    use super::GraphicsProtocol;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct GraphicsProtocolWrapper {
        protocol: GraphicsProtocol,
    }

    #[test]
    fn graphics_protocol_keeps_legacy_values_loadable() {
        let cases = [
            ("off", GraphicsProtocol::Off),
            ("halfblocks", GraphicsProtocol::Halfblocks),
            ("auto", GraphicsProtocol::Halfblocks),
            ("sixel", GraphicsProtocol::Halfblocks),
            ("kitty", GraphicsProtocol::Halfblocks),
            ("iterm2", GraphicsProtocol::Halfblocks),
        ];

        for (raw, expected) in cases {
            let parsed: GraphicsProtocolWrapper =
                toml::from_str(&format!("protocol = \"{}\"", raw)).unwrap();
            assert_eq!(parsed.protocol, expected);
        }
    }

    #[test]
    fn test_desktop_lyrics_cycling_and_serde() {
        use super::*;

        // Font sizes
        assert_eq!(cycle_desktop_lyrics_font_size(20, 1), 24);
        assert_eq!(cycle_desktop_lyrics_font_size(32, 1), 16);
        assert_eq!(cycle_desktop_lyrics_font_size(16, -1), 32);

        // Align
        assert_eq!(DesktopLyricsAlign::Center.cycle(1), DesktopLyricsAlign::Left);
        assert_eq!(DesktopLyricsAlign::Left.cycle(1), DesktopLyricsAlign::Right);
        assert_eq!(DesktopLyricsAlign::Right.cycle(1), DesktopLyricsAlign::Center);
        assert_eq!(DesktopLyricsAlign::Center.as_str(), "center");

        // Bg
        assert_eq!(DesktopLyricsBg::Translucent.cycle(1), DesktopLyricsBg::Dark);
        assert_eq!(DesktopLyricsBg::Transparent.cycle(1), DesktopLyricsBg::Translucent);
        assert_eq!(DesktopLyricsBg::Dark.as_str(), "dark");

        // Width
        assert_eq!(DesktopLyricsWidth::Standard.cycle(1), DesktopLyricsWidth::Wide);
        assert_eq!(DesktopLyricsWidth::UltraWide.cycle(1), DesktopLyricsWidth::Compact);
        assert_eq!(DesktopLyricsWidth::Compact.cycle(-1), DesktopLyricsWidth::UltraWide);
        assert_eq!(DesktopLyricsWidth::Wide.pixel_width(), 960);

        // Opacity
        assert_eq!(cycle_desktop_lyrics_opacity(85, 1), 70);
        assert_eq!(cycle_desktop_lyrics_opacity(30, 1), 100);
        assert_eq!(cycle_desktop_lyrics_opacity(100, -1), 30);

        // Serde roundtrip
        let mut cfg = Config::default();
        cfg.desktop_lyrics = true;
        cfg.desktop_lyrics_locked = false;
        cfg.desktop_lyrics_font_size = 28;
        cfg.desktop_lyrics_dual_line = false;
        cfg.desktop_lyrics_align = DesktopLyricsAlign::Right;
        cfg.desktop_lyrics_bg = DesktopLyricsBg::Dark;
        cfg.desktop_lyrics_width = DesktopLyricsWidth::Wide;
        cfg.desktop_lyrics_opacity = 70;
        cfg.desktop_lyrics_pos_x = Some(100);
        cfg.desktop_lyrics_pos_y = Some(200);

        let serialized = toml::to_string(&cfg).unwrap();
        let parsed: Config = toml::from_str(&serialized).unwrap();
        assert!(parsed.desktop_lyrics);
        assert!(!parsed.desktop_lyrics_locked);
        assert_eq!(parsed.desktop_lyrics_font_size, 28);
        assert!(!parsed.desktop_lyrics_dual_line);
        assert_eq!(parsed.desktop_lyrics_align, DesktopLyricsAlign::Right);
        assert_eq!(parsed.desktop_lyrics_bg, DesktopLyricsBg::Dark);
        assert_eq!(parsed.desktop_lyrics_width, DesktopLyricsWidth::Wide);
        assert_eq!(parsed.desktop_lyrics_opacity, 70);
        assert_eq!(parsed.desktop_lyrics_pos_x, Some(100));
        assert_eq!(parsed.desktop_lyrics_pos_y, Some(200));

        // Legacy config backwards compatibility
        let default_serialized = toml::to_string(&Config::default()).unwrap();
        let legacy: Config = toml::from_str(&default_serialized).unwrap();
        assert!(!legacy.desktop_lyrics);
        assert!(legacy.desktop_lyrics_locked);
        assert_eq!(legacy.desktop_lyrics_font_size, 20);
        assert!(legacy.desktop_lyrics_dual_line);
        assert_eq!(legacy.desktop_lyrics_align, DesktopLyricsAlign::Center);
        assert_eq!(legacy.desktop_lyrics_bg, DesktopLyricsBg::Translucent);
        assert_eq!(legacy.desktop_lyrics_width, DesktopLyricsWidth::Standard);
        assert_eq!(legacy.desktop_lyrics_opacity, 85);
        assert_eq!(legacy.desktop_lyrics_pos_x, None);
        assert_eq!(legacy.desktop_lyrics_pos_y, None);
    }
}
