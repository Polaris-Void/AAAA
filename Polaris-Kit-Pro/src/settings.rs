//! Polaris-Kit-Pro v3 user settings: UI language and UI scale, persisted
//! to a tiny hand-rolled INI so the crate keeps a zero-new-dependency
//! supply chain (std only).
//!
//! Design rules:
//! - The file lives in the user profile (`%APPDATA%\Polaris-Kit-Pro` on
//!   Windows, XDG/`~/.config` elsewhere) - never next to the exe, so a
//!   read-only install dir can never break the launcher.
//! - Every value is whitelisted and clamped: a corrupted or hand-edited
//!   file can only ever produce a valid language and a scale inside
//!   `0.75..=1.50`. Unknown keys, junk lines and wrong types are ignored.
//! - Writes are atomic (temp file + rename), so a crash mid-save can
//!   never leave a half-written settings file.

use std::path::PathBuf;

use crate::app::{Lang, ViewMode};

/// Persisted launcher preferences.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// UI language for the next launch.
    pub lang: Lang,
    /// UI scale multiplier applied over the OS-reported DPI factor.
    pub scale: f32,
    /// Dashboard layout (grid deck or compact list); defaults to grid.
    pub view: ViewMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lang: Lang::En,
            scale: 1.0,
            view: ViewMode::Grid,
        }
    }
}

impl Settings {
    /// The allowed scale range (1080p-native: 100% is the tuned default).
    pub const SCALE_MIN: f32 = 0.75;
    pub const SCALE_MAX: f32 = 1.50;
    /// The scale steps the footer "UI" chip cycles through (Ctrl+0 = 1.0).
    pub const SCALE_STEPS: [f32; 4] = [0.90, 1.00, 1.10, 1.25];

    pub fn clamped_scale(scale: f32) -> f32 {
        scale.clamp(Self::SCALE_MIN, Self::SCALE_MAX)
    }

    /// Next step in the footer cycle, wrapping after the largest zoom.
    pub fn next_scale(current: f32) -> f32 {
        // Compare with a small epsilon so float drift cannot skip a step.
        let eps = 0.001;
        let mut last = Self::SCALE_STEPS[0];
        for step in Self::SCALE_STEPS {
            if current < step - eps {
                return step;
            }
            last = step;
        }
        let _ = last;
        Self::SCALE_STEPS[0]
    }

    /// Serialize to the on-disk INI text (stable key order).
    pub fn serialize(&self) -> String {
        let lang = match self.lang {
            Lang::En => "en",
            Lang::Fa => "fa",
        };
        let view = match self.view {
            ViewMode::Grid => "grid",
            ViewMode::List => "list",
        };
        format!(
            "# Polaris-Kit-Pro v{} settings\nlang={lang}\nscale={:.2}\nview={view}\n",
            crate::VERSION,
            self.scale
        )
    }

    /// Parse INI text; unknown keys, junk lines and out-of-range values
    /// are silently ignored (whitelist parsing).
    pub fn parse(text: &str) -> Self {
        let mut settings = Self::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim().to_ascii_lowercase(), value.trim());
            match key.as_str() {
                "lang" => match value.to_ascii_lowercase().as_str() {
                    "en" => settings.lang = Lang::En,
                    "fa" => settings.lang = Lang::Fa,
                    _ => {}
                },
                "view" => match value.to_ascii_lowercase().as_str() {
                    "grid" => settings.view = ViewMode::Grid,
                    "list" => settings.view = ViewMode::List,
                    _ => {}
                },
                "scale" => {
                    if let Ok(parsed) = value.parse::<f32>() {
                        // Reject NaN/inf and clamp into the whitelist range.
                        if parsed.is_finite() {
                            settings.scale = Self::clamped_scale(parsed);
                        }
                    }
                }
                _ => {}
            }
        }
        settings
    }

    /// The settings file location (created on demand by [`load`]).
    pub fn file_path() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var_os("APPDATA")
                .map(std::path::PathBuf::from)
                .map(|base| base.join("Polaris-Kit-Pro").join("settings.ini"))
        }
        #[cfg(not(windows))]
        {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(std::path::PathBuf::from)
                .filter(|p| p.is_absolute())
                .or_else(|| {
                    std::env::var_os("HOME")
                        .map(|home| std::path::PathBuf::from(home).join(".config"))
                })
                .map(|base| base.join("Polaris-Kit-Pro").join("settings.ini"))
        }
    }

    /// Load settings from disk; any failure falls back to defaults.
    pub fn load() -> Self {
        let Some(path) = Self::file_path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::parse(&text),
            Err(_) => Self::default(),
        }
    }

    /// Persist settings atomically (temp file in the same directory, then
    /// rename over the target). Errors are swallowed by design: settings
    /// are a convenience, never a launch blocker.
    pub fn save(&self) {
        let Some(path) = Self::file_path() else {
            return;
        };
        let Some(dir) = path.parent() else { return };
        if std::fs::create_dir_all(dir).is_err() {
            return;
        }
        let text = self.serialize();
        let tmp = dir.join("Polaris-Kit-Pro.settings.tmp");
        if std::fs::write(&tmp, text.as_bytes()).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
        let _ = std::fs::remove_file(&tmp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let s = Settings {
            lang: Lang::Fa,
            scale: 1.10,
            view: ViewMode::List,
        };
        let parsed = Settings::parse(&s.serialize());
        assert_eq!(parsed.lang, Lang::Fa);
        assert!((parsed.scale - 1.10).abs() < 0.001);
        assert_eq!(parsed.view, ViewMode::List);
    }

    #[test]
    fn junk_is_ignored() {
        let text = "garbage\nlang=fa\nbroken====\nscale=9999\nlang2=de\n; comment\n# comment\n";
        let s = Settings::parse(text);
        assert_eq!(s.lang, Lang::Fa);
        assert_eq!(
            s.scale,
            Settings::SCALE_MAX,
            "out-of-range scale must clamp"
        );
    }

    #[test]
    fn nan_and_inf_scale_rejected() {
        for bad in ["nan", "inf", "-inf"] {
            let s = Settings::parse(&format!("scale={bad}\n"));
            assert_eq!(s.scale, 1.0, "scale={bad} must fall back to 1.0");
        }
    }

    #[test]
    fn scale_cycle_wraps() {
        assert_eq!(Settings::next_scale(0.90), 1.00);
        assert_eq!(Settings::next_scale(1.00), 1.10);
        assert_eq!(Settings::next_scale(1.10), 1.25);
        assert_eq!(
            Settings::next_scale(1.25),
            0.90,
            "wrap after the largest step"
        );
        assert_eq!(
            Settings::next_scale(1.33),
            0.90,
            "unknown scale restarts the cycle"
        );
    }

    #[test]
    fn missing_file_yields_defaults() {
        // parse("") must be valid and default.
        let s = Settings::parse("");
        assert_eq!(s, Settings::default());
    }
}
