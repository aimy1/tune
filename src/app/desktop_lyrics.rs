use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const DESKTOP_LYRICS_SCRIPT: &str = include_str!("../assets/desktop_lyrics.py");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopLyricPayload {
    pub title: String,
    pub artist: String,
    pub line1: String,
    pub line2: String,
    pub state: String,
    pub accent_color: String,
    pub subtext_color: String,
}

pub struct DesktopLyricsManager {
    child: Option<Child>,
    script_path: PathBuf,
    state_file: PathBuf,
    text_file: PathBuf,
    last_payload: Option<DesktopLyricPayload>,
}

impl DesktopLyricsManager {
    pub fn new(cache_root: &Path) -> Self {
        let script_dir = directories::BaseDirs::new()
            .map(|d| d.data_local_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share"))
            .join("tune");
        let _ = fs::create_dir_all(&script_dir);
        let script_path = script_dir.join("desktop_lyrics.py");
        let _ = fs::write(&script_path, DESKTOP_LYRICS_SCRIPT);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(&script_path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&script_path, perms);
            }
        }

        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| cache_root.to_path_buf())
            .join("tune");
        let _ = fs::create_dir_all(&runtime_dir);
        let state_file = runtime_dir.join("desktop_lyric.json");
        let text_file = cache_root.join("desktop_lyric.txt");

        Self {
            child: None,
            script_path,
            state_file,
            text_file,
            last_payload: None,
        }
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.child = None;
                    false
                }
                Ok(None) => true,
                Err(_) => {
                    self.child = None;
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn start(&mut self) {
        if self.is_running() {
            return;
        }

        // Spawn python desktop_lyrics.py
        let child = Command::new("python3")
            .arg(&self.script_path)
            .arg(&self.state_file)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match child {
            Ok(c) => {
                log::info!("Started desktop lyrics overlay (PID: {})", c.id());
                self.child = Some(c);
            }
            Err(e) => {
                log::warn!("Failed to start desktop lyrics script: {e}");
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            log::info!("Stopping desktop lyrics overlay (PID: {})", child.id());
            let _ = child.kill();
            let _ = child.wait();
        }
        self.last_payload = None;
        let _ = fs::remove_file(&self.state_file);
        let _ = fs::remove_file(&self.text_file);
    }

    pub fn sync(&mut self, enabled: bool, payload: DesktopLyricPayload) {
        if !enabled {
            if self.child.is_some() {
                self.stop();
            }
            return;
        }

        if !self.is_running() {
            self.start();
        }

        if self.last_payload.as_ref() == Some(&payload) {
            return;
        }

        self.write_state_files(payload);
    }

    pub fn write_state_files(&mut self, payload: DesktopLyricPayload) {
        let pid = std::process::id();
        let json_value = serde_json::json!({
            "tune_pid": pid,
            "title": payload.title,
            "artist": payload.artist,
            "line1": payload.line1,
            "line2": payload.line2,
            "state": payload.state,
            "accent_color": payload.accent_color,
            "subtext_color": payload.subtext_color,
        });

        // Write atomic state file
        if let Ok(content) = serde_json::to_string(&json_value) {
            let tmp_path = self.state_file.with_extension("tmp");
            if let Ok(mut file) = File::create(&tmp_path) {
                let _ = file.write_all(content.as_bytes());
                let _ = file.sync_all();
                let _ = fs::rename(&tmp_path, &self.state_file);
            }
        }

        // Write plain text lyric file for Waybar/polybar
        let text_line = if payload.line1.trim().is_empty() {
            if payload.state == "Playing" && !payload.title.trim().is_empty() {
                format!("♪ {}", payload.title)
            } else {
                String::new()
            }
        } else {
            payload.line1.clone()
        };
        let _ = fs::write(&self.text_file, text_line);

        self.last_payload = Some(payload);
    }
}

impl Drop for DesktopLyricsManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_lyrics_manager_sync_and_file_creation() {
        let temp_dir = std::env::temp_dir().join(format!("tune_test_dl_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let mut manager = DesktopLyricsManager::new(&temp_dir);
        let payload = DesktopLyricPayload {
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            line1: "Hello world lyric".to_string(),
            line2: "你好世界".to_string(),
            state: "Playing".to_string(),
            accent_color: "#b4befe".to_string(),
            subtext_color: "#a6adc8".to_string(),
        };

        // Write payload without starting subprocess
        manager.write_state_files(payload.clone());

        assert!(manager.state_file.exists());
        assert!(manager.text_file.exists());

        let json_str = fs::read_to_string(&manager.state_file).unwrap();
        assert!(json_str.contains("Hello world lyric"));
        assert!(json_str.contains("你好世界"));
        assert!(json_str.contains("#b4befe"));

        let txt_str = fs::read_to_string(&manager.text_file).unwrap();
        assert_eq!(txt_str, "Hello world lyric");

        // Clean up
        let _ = fs::remove_file(&manager.state_file);
        let _ = fs::remove_file(&manager.text_file);
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
