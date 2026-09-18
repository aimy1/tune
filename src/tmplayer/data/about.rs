use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct BrailleImage {
    pub width: usize,
    pub height: usize,
    pub art: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AboutInfo {
    #[allow(dead_code)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub license: String,
    #[serde(default)]
    pub links: BTreeMap<String, String>,
    #[serde(default)]
    pub braille_images: Vec<BrailleImage>,
}

pub fn about_info() -> &'static AboutInfo {
    static INFO: OnceLock<AboutInfo> = OnceLock::new();
    INFO.get_or_init(|| {
        let raw = include_str!("../../../about/about.toml");
        let mut info = toml::from_str(raw).unwrap_or_else(|_| AboutInfo {
            description: String::new(),
            version: String::new(),
            author: String::new(),
            license: String::new(),
            links: BTreeMap::new(),
            braille_images: Vec::new(),
        });
        info.version = env!("CARGO_PKG_VERSION").to_string();
        info
    })
}

pub fn select_logo_art<'a>(
    width: usize,
    height: usize,
    arts: &'a [BrailleImage],
) -> Option<&'a BrailleImage> {
    let mut best_fit: Option<(&'a BrailleImage, u128)> = None;
    for art in arts {
        if art.width == 0 || art.height == 0 {
            continue;
        }
        if art.width <= width && art.height <= height {
            let score = (art.width as u128) * (art.height as u128);
            let should_replace = best_fit
                .as_ref()
                .map(|(_, best_score)| score > *best_score)
                .unwrap_or(true);
            if should_replace {
                best_fit = Some((art, score));
            }
        }
    }

    if let Some((art, _)) = best_fit {
        return Some(art);
    }

    arts.iter()
        .filter(|art| art.width > 0 && art.height > 0)
        .min_by_key(|art| {
            let dw = art.width.saturating_sub(width);
            let dh = art.height.saturating_sub(height);
            dw * dw + dh * dh
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_about_info_loads_and_contains_expected_metadata() {
        let info = about_info();
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(!info.description.is_empty());
        assert_eq!(info.author, "Asniya (@aimy1)");
        assert_eq!(info.license, "GNU AGPL-3.0");
        assert_eq!(
            info.links.get("github_url").map(|s| s.as_str()),
            Some("https://github.com/aimy1/tune")
        );
        assert_eq!(
            info.links.get("issues_url").map(|s| s.as_str()),
            Some("https://github.com/aimy1/tune/issues")
        );
        assert!(!info.braille_images.is_empty());
    }
}
