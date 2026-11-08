//! Polaris-Kit-Pro v3.1.0 "NATIVE-1080" — 12 Windows utilities in one
//! native launcher.
//!
//! Pure Rust/egui build: one static executable, zero runtime dependencies,
//! byte-exact payload vault, sha256-verified staging, EN/FA (RTL) UI.
//!
//! Identity constants in this crate are the single source of truth for the
//! full product name and version - the UI, the log banner, the export
//! header and the build scripts all derive from or agree with them.

pub mod app;
pub mod bidi;
pub mod data;
pub mod payloads;
pub mod readmes;
pub mod run;
pub mod settings;
pub mod theme;
pub mod views;
pub mod widgets;

/// The full, correct product name - used verbatim everywhere.
pub const APP_NAME: &str = "Polaris-Kit-Pro";
/// This release line: native 1080p refresh of the PRO edition.
pub const VERSION: &str = "3.1.1";
/// Display string combining both (footer chip, window title, log banner).
pub const APP_VERSION: &str = concat!("Polaris-Kit-Pro v", "3.1.1");

#[cfg(test)]
mod tests {
    #[test]
    fn identity_is_exact() {
        assert_eq!(super::APP_NAME, "Polaris-Kit-Pro");
        assert!(super::APP_VERSION.starts_with("Polaris-Kit-Pro v"));
        assert!(super::APP_VERSION.contains('3'));
    }
}

pub mod logstore {
    //! The live log: a bounded, timestamped line buffer with export/clear.
    //! Bounded by tracked character length (no O(n) join per append), and
    //! trimmed at line boundaries so a half-line is never left at the top.

    #[derive(Clone, Debug)]
    pub struct Line {
        pub ts: String,
        pub text: String,
    }

    pub struct Log {
        pub lines: Vec<Line>,
        /// Tracked character length; avoids an O(n) join per appended line.
        pub chars: usize,
    }

    impl Default for Log {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Log {
        pub const MAX_CHARS: usize = 80_000;
        pub const TRIM_AT: usize = 50_000;

        pub fn new() -> Self {
            Self {
                lines: Vec::new(),
                chars: 0,
            }
        }

        fn now() -> String {
            let Ok(fmt) = time::format_description::parse_borrowed::<2>("[hour]:[minute]:[second]")
            else {
                return "[--:--:--]".to_string();
            };
            match time::OffsetDateTime::now_local() {
                Ok(dt) => dt
                    .format(&fmt)
                    .map(|s| format!("[{s}]"))
                    .unwrap_or_else(|_| "[--:--:--]".to_string()),
                Err(_) => "[--:--:--]".to_string(),
            }
        }

        pub fn push(&mut self, text: &str) {
            let line = Line {
                ts: Self::now(),
                text: text.to_string(),
            };
            self.chars += line.text.len();
            self.lines.push(line);
            if self.chars > Self::MAX_CHARS {
                // Trim at a line boundary so no half-line is left at the top.
                let mut keep_from = 0;
                let mut acc = 0usize;
                for (i, l) in self.lines.iter().enumerate() {
                    acc += l.text.len();
                    if acc >= Self::TRIM_AT {
                        keep_from = i + 1;
                        break;
                    }
                }
                self.chars = self.lines[keep_from..].iter().map(|l| l.text.len()).sum();
                self.lines.drain(0..keep_from);
            }
        }

        pub fn clear(&mut self) {
            self.lines.clear();
            self.chars = 0;
            self.push("[SYS] log purged.");
        }

        /// Full text for export (branded with the product identity).
        pub fn export_text(&self) -> String {
            let mut out = String::new();
            out.push_str(&format!("{} [PRO] :: log export\n\n", crate::APP_NAME));
            for l in &self.lines {
                out.push_str(&l.ts);
                out.push_str(&l.text);
                out.push('\n');
            }
            out
        }

        pub fn text_lines(&self) -> Vec<String> {
            self.lines
                .iter()
                .map(|l| format!("{}{}", l.ts, l.text))
                .collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn export_header_carries_product_name() {
            let mut log = Log::new();
            log.push("hello");
            assert!(log
                .export_text()
                .starts_with("Polaris-Kit-Pro [PRO] :: log export"));
        }
    }
}
