//! The application state machine: language, view routing, the run worker
//! lifecycle, the in-app dialogs (vault / confirm) and the boot sequence.

use crate::data;
use crate::logstore::Log;
use crate::run::{self, RunEvent, RunShared};
use crate::theme::{self, Tokens};
use crate::views;
use crate::widgets::{self, WinBtn};
use egui::{vec2, Color32, CornerRadius, RichText, Sense};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::sync::Arc;

/// Dashboard layout mode (persisted; grid is the product default).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    List,
}

/// UI language. Persian strings are stored in logical order and shaped
/// at layout time by the in-app bidi engine (egui 0.36 has none), so
/// switching languages only swaps strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Fa,
}

impl Lang {
    pub const fn fa(self) -> bool {
        matches!(self, Lang::Fa)
    }
    pub const fn rtl(self) -> bool {
        self.fa()
    }
    /// The string for `key` in this language.
    pub fn s(self, key: &str) -> &'static str {
        let v = data::t(key);
        if self.fa() {
            v.fa
        } else {
            v.en
        }
    }
}

/// A pending in-app confirm dialog.
#[derive(Clone, Copy)]
pub struct Confirm {
    pub module: usize,
    pub action: usize,
    pub key: &'static str,
}

pub struct PolarisApp {
    // ---- i18n
    pub lang: Lang,
    pub readme_lang: Lang,
    // ---- view routing
    pub detail: Option<usize>,
    pub show_log: bool,
    // ---- launcher state
    pub filter: &'static str,
    pub sort_az: bool,
    pub search: String,
    /// whether the search edit currently holds keyboard focus (for the
    /// focus ring, which is painted before the edit reports back)
    pub search_focus: bool,
    /// dashboard layout (grid deck vs compact list); persisted
    pub view: ViewMode,
    /// slide-over log panel open (accessible from every view)
    pub log_slide: f32,
    // ---- execution
    pub folder: String,
    pub busy: bool,
    pub was_stopped: bool,
    /// module/action currently running (progress + STOP live under them)
    pub busy_module: Option<usize>,
    pub busy_action: Option<usize>,
    /// when the current run started (elapsed readout on the progress bar)
    pub busy_started: std::time::Instant,
    /// frames the progress bar has stayed frozen (indeterminate watchdog)
    pub progress_stall_guard: u32,
    pub shared: Arc<RunShared>,
    pub tx: Option<SyncSender<RunEvent>>,
    pub rx: Option<Receiver<RunEvent>>,
    pub log: Log,
    // ---- in-app dialogs
    pub confirm: Option<Confirm>,
    // ---- window (custom title bar)
    pub maximized: bool,
    pub resizing: bool,
    pub title_dragging: bool,
    pub rounded_corners_applied: bool,
    /// Boot: payload vault integrity audit result (logged at start).
    pub boot_vault_ok: bool,
    /// The lion mark, lazily decoded into a GPU texture for the titlebar.
    pub logo: Option<egui::TextureHandle>,
    /// One-shot smoke-test hook (PKP_SMOKE=detail:<i>|log) — inert unless
    /// the env var is set, used by the automated GUI smoke test.
    smoke_routed: bool,
    // ---- v3 UI scale (NATIVE-1080)
    /// The OS-reported pixels-per-point captured at boot.
    pub native_ppp: f32,
    /// User scale multiplier persisted in settings.ini.
    pub ui_scale: f32,
}

impl Default for PolarisApp {
    fn default() -> Self {
        Self::new()
    }
}

impl PolarisApp {
    pub fn new() -> Self {
        Self {
            lang: Lang::En,
            readme_lang: Lang::En,
            detail: None,
            show_log: false,
            filter: "ALL",
            sort_az: false,
            search: String::new(),
            search_focus: false,
            view: ViewMode::Grid,
            log_slide: 0.0,
            folder: String::new(),
            busy: false,
            was_stopped: false,
            busy_module: None,
            busy_action: None,
            busy_started: std::time::Instant::now(),
            progress_stall_guard: 0,
            shared: Arc::new(RunShared::new()),
            tx: None,
            rx: None,
            log: Log::new(),
            confirm: None,
            maximized: false,
            resizing: false,
            title_dragging: false,
            rounded_corners_applied: false,
            boot_vault_ok: false,
            logo: None,
            smoke_routed: false,
            native_ppp: 1.0,
            ui_scale: 1.0,
        }
    }

    /// Construct and run the boot sequence (banner + vault audit + stage purge).
    pub fn booted() -> Self {
        Self::booted_with(Lang::En)
    }

    /// Boot with persisted preferences (v3: settings.ini; v3.1 adds view).
    pub fn booted_with(lang: Lang) -> Self {
        Self::booted_from(crate::settings::Settings {
            lang,
            scale: 1.0,
            view: ViewMode::Grid,
        })
    }

    /// Boot from a full settings snapshot (main.rs merges the file first).
    pub fn booted_from(settings: crate::settings::Settings) -> Self {
        let mut app = Self::new();
        app.lang = settings.lang;
        app.readme_lang = settings.lang;
        app.view = settings.view;
        app.boot();
        app
    }

    /// Apply the persisted UI scale over the OS-reported DPI factor and
    /// remember the native factor for later adjustments.
    pub fn set_ui_scale(&mut self, ctx: &egui::Context, scale: f32) {
        self.ui_scale = crate::settings::Settings::clamped_scale(scale);
        self.native_ppp = ctx.pixels_per_point();
        ctx.set_pixels_per_point(self.native_ppp * self.ui_scale);
        ctx.request_repaint();
    }

    /// Cycle to the next UI scale step and persist the choice.
    pub fn cycle_ui_scale(&mut self, ctx: &egui::Context) {
        let next = crate::settings::Settings::next_scale(self.ui_scale);
        self.set_ui_scale(ctx, next);
        self.log.push(&format!(
            "[SYS] ui scale :: {}%",
            (self.ui_scale * 100.0).round() as i32
        ));
        self.persist_settings();
    }

    /// Persist language + scale (called on every preference change).
    pub fn persist_settings(&self) {
        crate::settings::Settings {
            lang: self.lang,
            scale: self.ui_scale,
            view: self.view,
        }
        .save();
    }

    /// Switch the dashboard layout (grid <-> list) and persist the choice.
    pub fn set_view(&mut self, view: ViewMode, ctx: &egui::Context) {
        let changed = self.view != view;
        self.view = view;
        if changed {
            self.log.push(&format!(
                "[NAV] view :: {}",
                match view {
                    ViewMode::Grid => "grid",
                    ViewMode::List => "list",
                }
            ));
            ctx.request_repaint();
            self.persist_settings();
        }
    }

    /// The startup banner (mirrors the Python boot_log), plus a real
    /// payload audit: every on-disk file in `payloads\` next to the
    /// executable must match its embedded sha256 pin.
    pub fn boot(&mut self) {
        run::clean_old_stages();
        // integrity audit: the payloads folder must be complete and
        // byte-exact (tamper evidence without any temp copies)
        let mut total = 0usize;
        let mut ok = true;
        let pdir = run::payload_dir();
        match &pdir {
            Ok(dir) => {
                for p in crate::payloads::PAYLOADS {
                    let path = dir.join(p.file);
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            total += bytes.len();
                            if run::sha256_hex(&bytes) != p.sha256_str() {
                                ok = false;
                                self.log.push(&format!(
                                    "[ERR ] payload audit :: {} hash mismatch",
                                    p.resid
                                ));
                            }
                        }
                        Err(error) => {
                            ok = false;
                            self.log.push(&format!(
                                "[ERR ] payload audit :: {} unreadable ({error})",
                                p.resid
                            ));
                        }
                    }
                }
            }
            Err(reason) => {
                ok = false;
                self.log.push(&format!(
                    "[WARN] payloads folder unavailable :: {reason}"
                ));
                self.log
                    .push("[TIP ] keep the payloads folder next to Polaris-Kit-Pro.exe.");
            }
        }
        self.boot_vault_ok = ok;

        // The LOG is English-only by product rule - even when the UI runs
        // in Persian. Timestamps, markers and banner stay parseable.
        let line = "=".repeat(64);
        self.log.push(&line);
        self.log
            .push(&format!("{} [PRO EDITION :: NATIVE-1080]", crate::APP_VERSION));
        let payload_note = match &pdir {
            Ok(_) => format!(
                "{} modules loaded :: {} payloads on disk ({} KB, sha256-pinned)",
                data::MODULES.len(),
                crate::payloads::PAYLOADS.len(),
                total / 1024
            ),
            Err(_) => format!(
                "{} modules loaded :: payload folder NOT FOUND",
                data::MODULES.len()
            ),
        };
        self.log.push(&payload_note);
        self.log
            .push("execution policy :: on-disk dispatch only - no %TEMP% staging, no encoded commands");
        let session = if run::is_admin() {
            "ELEVATED ADMIN"
        } else {
            "STANDARD (dev)"
        };
        self.log.push(&format!(
            "session: {session} :: popups: ZERO (all output streams here)"
        ));
        self.log
            .push("font: Vazirmatn (one family, EN+FA) :: layout: in-app bidi engine");
        let payload_path = pdir
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "unavailable".to_string());
        self.log.push(&format!(
            "payload bay :: {payload_path} (executed in place, hash-verified, zero temp writes)"
        ));
        self.log.push(&line);
        if ok {
            self.log.push(&format!(
                "[ OK ] payload audit :: {} files sha256-verified",
                crate::payloads::PAYLOADS.len()
            ));
        }
        self.log
            .push("[TIP] click a card to open its full GitHub README + actions.");
        self.log
            .push("[TIP] the LOG button (top bar) opens the log from every view.");
        self.log
            .push("[TIP] progress is live under the running action; STOP cancels it.");
        self.log
            .push("[TIP] UI zoom: Ctrl +/- and Ctrl + 0 to reset.");
    }

    // ------------------------------------------------------------ language
    pub fn toggle_lang(&mut self, ctx: &egui::Context) {
        self.lang = if self.lang.fa() { Lang::En } else { Lang::Fa };
        self.apply_lang(ctx);
        self.persist_settings();
    }

    /// Switching language also syncs the in-app readme language (the pass-7
    /// fix: readme_lang must follow the UI language or FA mode shows EN).
    pub fn apply_lang(&mut self, ctx: &egui::Context) {
        self.readme_lang = self.lang;
        if self.detail.is_some() {
            self.render_readme_now();
        }
        ctx.request_repaint();
    }

    pub fn toggle_readme(&mut self) {
        self.readme_lang = if self.readme_lang.fa() {
            Lang::En
        } else {
            Lang::Fa
        };
        self.render_readme_now();
    }

    fn render_readme_now(&mut self) {
        // The readme is rendered from the pre-parsed blocks; nothing to
        // regenerate - the view picks the right array each frame.
    }

    // ------------------------------------------------------------ navigation
    pub fn select_module(&mut self, i: usize, ctx: &egui::Context) {
        if i < data::MODULES.len() {
            self.detail = Some(i);
            self.log
                .push(&format!("[NAV] module :: {}", data::MODULES[i].mid));
            ctx.request_repaint();
        }
    }

    pub fn show_grid(&mut self, ctx: &egui::Context) {
        self.detail = None;
        ctx.request_repaint();
    }

    /// Open the slide-over log (non-blocking; the current view stays live).
    pub fn show_log_view(&mut self, ctx: &egui::Context) {
        self.show_log = true;
        ctx.request_repaint();
    }

    pub fn toggle_log_view(&mut self, ctx: &egui::Context) {
        self.show_log = !self.show_log;
        ctx.request_repaint();
    }

    pub fn cat_changed(&mut self, c: &'static str, ctx: &egui::Context) {
        let changed = self.filter != c;
        self.filter = c;
        if changed {
            self.log.push(&format!("[NAV] filter :: {c}"));
        }
        // a tab click ALWAYS returns to the grid (the slide-over stays)
        self.detail = None;
        ctx.request_repaint();
    }

    pub fn sort_toggle(&mut self, ctx: &egui::Context) {
        self.sort_az = !self.sort_az;
        self.detail = None;
        ctx.request_repaint();
    }

    /// Modules passing the current category + search, in the current order.
    pub fn visible_modules(&self) -> Vec<usize> {
        let q = self.search.trim().to_lowercase();
        let mut idxs = Vec::new();
        for (i, m) in data::MODULES.iter().enumerate() {
            if self.filter != "ALL" && m.cat != self.filter {
                continue;
            }
            if !q.is_empty() {
                let hay = format!(
                    "{} {} {} {} {}",
                    m.title, m.mid, m.desc_official, m.blurb_fa, m.cat
                )
                .to_lowercase();
                if !hay.contains(&q) {
                    continue;
                }
            }
            idxs.push(i);
        }
        if self.sort_az {
            idxs.sort_by(|a, b| {
                data::MODULES[*a]
                    .title
                    .to_ascii_lowercase()
                    .cmp(&data::MODULES[*b].title.to_ascii_lowercase())
            });
        }
        idxs
    }

    // ------------------------------------------------------------ links
    pub fn open_link(&mut self, url: &str) {
        match open::that(url) {
            Ok(()) => {
                self.log.push(&format!("[ OK ] browser launched :: {url}"));
            }
            Err(e) => {
                self.log
                    .push(&format!("[ERR ] cannot open browser ({e}) :: {url}"));
            }
        }
    }

    // ------------------------------------------------------------ folder
    pub fn pick_folder(&mut self) {
        // The only OS dialog in the whole app (a native folder picker, like
        // the Python filedialog). Everything else stays in-app.
        let Some(folder) = rfd::FileDialog::new()
            .set_title("Select TARGET folder (USB root for immunizer)")
            .pick_folder()
        else {
            self.log.push("[SKIP] picker cancelled.");
            return;
        };
        let s = folder.to_string_lossy().into_owned();
        match run::check_folder(&s) {
            Ok(p) => {
                self.folder = p.to_string_lossy().into_owned();
                self.log
                    .push(&format!("[ OK ] target locked :: {}", self.folder));
            }
            Err(why) => {
                self.log.push(&format!("[ABORT] rejected target :: {why}"));
            }
        }
    }

    // ------------------------------------------------------------ log ops
    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    pub fn export_log(&mut self) {
        let base = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(std::env::temp_dir);
        let timestamp = time::OffsetDateTime::now_local()
            .ok()
            .and_then(|dt| {
                let format = time::format_description::parse_borrowed::<2>(
                    "[year][month][day]-[hour][minute][second]",
                )
                .ok()?;
                dt.format(&format).ok()
            })
            .unwrap_or_else(|| "unknown-time".to_string());
        let text = self.log.export_text();
        let result = (0..100).find_map(|suffix| {
            let name = if suffix == 0 {
                format!("Polaris-Kit-Pro-log-{timestamp}.txt")
            } else {
                format!("Polaris-Kit-Pro-log-{timestamp}-{suffix}.txt")
            };
            let path = base.join(name);
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    use std::io::Write as _;
                    let write_result = file
                        .write_all(text.as_bytes())
                        .and_then(|()| file.sync_all());
                    Some((path, write_result))
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
                Err(error) => Some((path, Err(error))),
            }
        });
        match result {
            Some((path, Ok(()))) => {
                self.log.push(&format!(
                    "[ OK ] log exported :: {} ({} bytes)",
                    path.display(),
                    text.len()
                ));
            }
            Some((path, Err(error))) => {
                let _ = std::fs::remove_file(&path);
                self.log.push(&format!("[ERR ] export failed ({error})"));
            }
            None => self
                .log
                .push("[ERR ] export failed (unable to reserve a unique file name)"),
        }
    }

    pub fn stop_current(&mut self) {
        if !self.busy {
            return;
        }
        self.shared
            .stop
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.was_stopped = true;
        if self
            .shared
            .child_pid
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
        {
            self.log.push(self.lang.s("stop_log_line"));
        } else {
            self.log
                .push("[WARN] stop requested - aborting before dispatch...");
        }
        run::stop_tree(&self.shared);
    }

    /// Closing the window must not leave an elevated tool running.
    pub fn on_close(&mut self) {
        if self.busy {
            self.shared
                .stop
                .store(true, std::sync::atomic::Ordering::Relaxed);
            run::stop_tree(&self.shared);
        }
    }

    // ------------------------------------------------------------ confirm
    pub fn request_run(&mut self, module: usize, action: usize, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        let Some(m) = data::MODULES.get(module) else {
            return;
        };
        let Some(a) = m.acts.get(action) else {
            return;
        };
        if a.need_folder && self.folder.is_empty() {
            self.log
                .push("[WARN] target folder required - opening picker...");
            self.pick_folder();
            if self.folder.is_empty() {
                self.log.push("[ABORT] no target selected.");
                return;
            }
        }
        if let Some(key) = a.confirm {
            self.log
                .push("[THINK] action needs operator consent - asking in-app...");
            self.confirm = Some(Confirm {
                module,
                action,
                key,
            });
            ctx.request_repaint();
            return;
        }
        self.start_run(module, action, None, ctx);
    }

    /// Confirm dialog answered: start (or cancel) the run.
    pub fn confirm_answer(&mut self, yes: bool, ctx: &egui::Context) {
        let Some(c) = self.confirm.take() else {
            return;
        };
        if yes {
            self.log
                .push("[ OK ] consent granted :: piping YES to the tool prompt.");
            self.start_run(c.module, c.action, Some("Y\r\n"), ctx);
        } else {
            self.log.push("[ABORT] cancelled by operator.");
        }
        ctx.request_repaint();
    }

    fn start_run(
        &mut self,
        module: usize,
        action: usize,
        answer: Option<&str>,
        ctx: &egui::Context,
    ) {
        let Some(m) = data::MODULES.get(module) else {
            self.log.push("[ABORT] invalid module selection.");
            return;
        };
        let Some(a) = m.acts.get(action) else {
            self.log.push("[ABORT] invalid action selection.");
            return;
        };
        if self.busy {
            return;
        }

        let (tx, rx) = sync_channel(1024);
        self.tx = Some(tx.clone());
        self.rx = Some(rx);
        self.shared
            .stop
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.shared
            .child_exited
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.was_stopped = false;
        self.busy = true;
        let folder = self.folder.clone();
        if let Err(error) = run::spawn_worker(
            m,
            a,
            folder,
            answer.map(str::to_string),
            tx,
            self.shared.clone(),
        ) {
            self.busy = false;
            self.tx = None;
            self.rx = None;
            self.log
                .push(&format!("[ERR ] unable to start worker thread ({error})"));
        }
        ctx.request_repaint();
    }

    /// Drain run-worker events (called every frame, non-blocking).
    pub fn pump(&mut self, ctx: &egui::Context) {
        let mut received = false;
        let mut finished = false;
        let mut processed = 0usize;
        if let Some(rx) = self.rx.as_ref() {
            while processed < 512 {
                match rx.try_recv() {
                    Ok(RunEvent::Log(message)) => {
                        processed += 1;
                        received = true;
                        self.log.push(&message);
                    }
                    Ok(RunEvent::State(_)) => {
                        processed += 1;
                        received = true;
                    }
                    Ok(RunEvent::Done) => {
                        received = true;
                        finished = true;
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        finished = true;
                        break;
                    }
                }
            }
        }
        if finished {
            self.busy = false;
            self.busy_module = None;
            self.busy_action = None;
            self.tx = None;
            self.rx = None;
        }

        if received || finished {
            ctx.request_repaint();
        }
        if self.busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        }
    }
}

// ----------------------------------------------------------------- update
impl eframe::App for PolarisApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.pump(&ctx);
        if !self.rounded_corners_applied {
            self.rounded_corners_applied = run::apply_window_rounding(frame);
        }

        // --- custom title bar: drag + double-click maximize + edge resize
        let screen = ctx.input(|i| i.viewport_rect());
        let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        if maximized != self.maximized {
            self.maximized = maximized;
        }

        if !self.maximized {
            // edge resize zones (6px)
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                const EDGE: f32 = 6.0;
                let l = pos.x < screen.left() + EDGE;
                let r = pos.x > screen.right() - EDGE;
                let t = pos.y < screen.top() + EDGE;
                let b = pos.y > screen.bottom() - EDGE;
                let dir = if t && l {
                    Some(egui::ResizeDirection::NorthWest)
                } else if t && r {
                    Some(egui::ResizeDirection::NorthEast)
                } else if b && l {
                    Some(egui::ResizeDirection::SouthWest)
                } else if b && r {
                    Some(egui::ResizeDirection::SouthEast)
                } else if t {
                    Some(egui::ResizeDirection::North)
                } else if b {
                    Some(egui::ResizeDirection::South)
                } else if l {
                    Some(egui::ResizeDirection::West)
                } else if r {
                    Some(egui::ResizeDirection::East)
                } else {
                    None
                };
                if let Some(d) = dir {
                    let cursor = match d {
                        egui::ResizeDirection::North | egui::ResizeDirection::South => {
                            egui::CursorIcon::ResizeVertical
                        }
                        egui::ResizeDirection::East | egui::ResizeDirection::West => {
                            egui::CursorIcon::ResizeHorizontal
                        }
                        egui::ResizeDirection::NorthEast | egui::ResizeDirection::SouthWest => {
                            egui::CursorIcon::ResizeNwSe
                        }
                        _ => egui::CursorIcon::ResizeNeSw,
                    };
                    ctx.set_cursor_icon(cursor);
                    if ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
                        && !self.resizing
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(d));
                        self.resizing = true;
                    }
                }
            }
            if self.resizing
                && !ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
            {
                self.resizing = false;
            }
        } else {
            self.resizing = false;
        }

        // ------------------------------------------- logo texture (once)
        if self.logo.is_none() {
            self.logo = load_logo(&ctx);
        }

        // smoke-test routing (one shot, env-gated). Comma-separated:
        // "fa" -> Persian UI, "list" -> list view, "log" -> open the
        // slide-over log, "detail:<i>" -> open tool i. Inert unless the
        // PKP_SMOKE env var is set (used by the automated GUI tests).
        if !self.smoke_routed {
            self.smoke_routed = true;
            if let Ok(v) = std::env::var("PKP_SMOKE") {
                for part in v.split(',') {
                    let part = part.trim();
                    if let Some(idx) = part.strip_prefix("detail:") {
                        if let Ok(i) = idx.parse::<usize>() {
                            self.detail = Some(i.min(data::MODULES.len() - 1));
                        }
                    } else if part == "log" {
                        self.show_log = true;
                    } else if part == "fa" {
                        self.lang = Lang::Fa;
                        self.readme_lang = Lang::Fa;
                    } else if part == "list" {
                        self.view = ViewMode::List;
                    }
                }
            }
        }

        // ------------------------------------------------ header / title bar
        // One 52px row: [logo] [Polaris-Kit-Pro] [GITHUB ↗] [subtitle]
        // …… drag band …… [–] [□] [✕]. The wordmark and window controls
        // are always LTR (the product name never translates); only the
        // subtitle text follows the language.
        egui::Panel::top("pkp_header")
            .exact_size(theme::HEADER_H)
            .show(ui, |ui| {
                let t = Tokens::get();
                ui.horizontal(|ui| {
                    ui.set_min_height(theme::HEADER_H);
                    ui.add_space(10.0);
                    // the lion mark (same icon as the exe; clean rounded
                    // corners, no keyline)
                    if let Some(tex) = self.logo.as_ref() {
                        ui.add(
                            egui::Image::from_texture(tex)
                                .fit_to_exact_size(vec2(30.0, 30.0))
                                .corner_radius(7),
                        );
                    } else {
                        let (rect, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(7), t.sunken);
                    }
                    ui.add_space(10.0);
                    // brand (never translated, per the product identity rule)
                    ui.label(
                        RichText::new(crate::APP_NAME)
                            .family(theme::display())
                            .size(15.0)
                            .extra_letter_spacing(1.1)
                            .color(t.ink_strong),
                    );
                    ui.add_space(10.0);
                    // GitHub link right next to the wordmark
                    if widgets::button_sized(
                        ui,
                        "GITHUB ↗",
                        widgets::Kind::Secondary,
                        vec2(100.0, 30.0),
                        Color32::TRANSPARENT,
                        Color32::TRANSPARENT,
                    )
                    .clicked()
                    {
                        self.open_link(data::GITHUB);
                    }
                    ui.add_space(12.0);
                    // "Built for Windows 10 & 11 — best experienced on 10/11"
                    let sub = crate::bidi::galley_text(
                        ui,
                        self.lang.s("subtitle"),
                        self.lang.rtl(),
                        egui::FontId::new(
                            10.5,
                            if self.lang.fa() {
                                theme::vazir()
                            } else {
                                egui::FontFamily::Proportional
                            },
                        ),
                        t.faint,
                        0.0,
                        crate::bidi::Align::Left,
                    );
                    let (srect, _) = ui.allocate_exact_size(sub.size(), Sense::hover());
                    ui.painter().galley(
                        egui::pos2(srect.left(), srect.center().y - sub.size().y / 2.0),
                        sub,
                        Color32::WHITE,
                    );

                    // draggable band: exactly the width left of the
                    // [LOG] [LANG] [–][□][✕] cluster, so nothing gets
                    // pushed off-screen.
                    const GAP_LOG_LANG: f32 = 8.0;
                    const BTNS_W: f32 = 8.0 + widgets::LOG_BTN_W + GAP_LOG_LANG
                        + widgets::LANG_BTN_W
                        + 8.0 + 3.0 * 40.0 + 2.0 * 4.0 + widgets::WINCTRL_PAD_RIGHT;
                    let band_w = (ui.available_width() - BTNS_W).max(40.0);
                    let (rect, drag) =
                        ui.allocate_exact_size(vec2(band_w, theme::HEADER_H), Sense::drag());
                    if drag.dragged() && !self.title_dragging {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        self.title_dragging = true;
                    }
                    if !ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) {
                        self.title_dragging = false;
                    }
                    if drag.double_clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                    }
                    let _ = rect;

                    // ---- global LOG button: immediately left of the
                    // language switcher, reachable from EVERY view and
                    // sub-page (opens the slide-over log).
                    ui.add_space(8.0);
                    let log_fill = if self.show_log {
                        theme::LOGGREEN
                    } else if self.busy {
                        theme::AMBER
                    } else {
                        Color32::TRANSPARENT
                    };
                    let log_fg = if self.show_log {
                        Color32::from_rgb(0x03, 0x13, 0x0a)
                    } else {
                        t.ink
                    };
                    let log_kind = if self.show_log {
                        widgets::Kind::Tinted
                    } else {
                        widgets::Kind::Secondary
                    };
                    if widgets::button_sized(
                        ui,
                        self.lang.s("log"),
                        log_kind,
                        vec2(widgets::LOG_BTN_W, 30.0),
                        log_fill,
                        log_fg,
                    )
                    .clicked()
                    {
                        self.toggle_log_view(&ctx);
                    }
                    ui.add_space(GAP_LOG_LANG);

                    // ---- language switcher (header, black button, next to
                    // minimize; shows the OTHER language's name)
                    let lang_label = if self.lang.fa() { "EN" } else { "فارسی" };
                    if widgets::button_sized(
                        ui,
                        lang_label,
                        widgets::Kind::Tinted,
                        vec2(widgets::LANG_BTN_W, 30.0),
                        widgets::BLACK_BTN,
                        Color32::WHITE,
                    )
                    .clicked()
                    {
                        self.toggle_lang(&ctx);
                    }
                    ui.add_space(8.0);

                    // window controls at the far edge (small, rounded,
                    // painter-drawn glyphs, Fluent-style edge padding so the
                    // close button never jams against the window border)
                    if let Some(btn) = widgets::window_buttons(ui, self.maximized) {
                        match btn {
                            WinBtn::Min => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true))
                            }
                            WinBtn::Max => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized))
                            }
                            WinBtn::Close => {
                                self.on_close();
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    }
                });
                ui.painter().hline(
                    ui.max_rect().x_range(),
                    ui.max_rect().bottom() - 1.0,
                    t.hairline(),
                );
            });

        // ------------------------------------------------ v3.1 hotkeys: log
        // Ctrl+L toggles the slide-over log from anywhere; Esc closes it.
        let (log_toggle, log_close) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::L),
                i.key_pressed(egui::Key::Escape),
            )
        });
        if log_toggle {
            self.toggle_log_view(&ctx);
        } else if log_close && self.show_log {
            self.show_log = false;
        }

        // ------------------------------------------------ v3 hotkeys: UI scale
        // Ctrl + '=' zoom in, Ctrl + '-' zoom out, Ctrl + '0' reset to 100%.
        let (zoom_in, zoom_out, zoom_reset) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::Equals),
                i.modifiers.command && i.key_pressed(egui::Key::Minus),
                i.modifiers.command && i.key_pressed(egui::Key::Num0),
            )
        });
        if zoom_in || zoom_out || zoom_reset {
            if zoom_reset {
                self.set_ui_scale(&ctx, 1.0);
                self.persist_settings();
            } else {
                let step = if zoom_in { 0.05 } else { -0.05 };
                self.set_ui_scale(&ctx, self.ui_scale + step);
                self.persist_settings();
            }
        }

        // ------------------------------------------------ main area + overlays
        // (the v3 footer status bar was removed: the version/name repetition,
        // the vault/elevated/1080p badges and the UI-scale chip are gone -
        // the content area now reaches the bottom edge. Ctrl +/- still
        // adjusts the UI scale via the hotkeys above.)
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(i) = self.detail {
                views::detail(ui, self, i, &ctx);
            } else {
                views::toolbar(ui, self, &ctx);
                views::section_bar(ui, self);
                views::body(ui, self, &ctx);
            }

            // in-app overlays (drawn last, so they sit on top). Dialog data
            // is extracted first so it cannot alias the `&mut self` below.
            let confirm = self.confirm;
            if let Some(c) = confirm {
                views::confirm_dialog(ui, &ctx, &c, self);
            }
            // universal slide-over LOG: available from every view & sub-page
            views::log_panel(ui, self, &ctx);
        });
    }
}

/// Decode the embedded lion mark into a 128px GPU texture for the
/// titlebar logo (downscaled once with Lanczos so the small render stays
/// crisp; no keyline, clean transparent rounded corners).
fn load_logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    const LION: &[u8] = include_bytes!("../assets/lion-mark.png");
    let img = image::load_from_memory(LION).ok()?.to_rgba8();
    let small = image::imageops::resize(&img, 128, 128, image::imageops::FilterType::Lanczos3);
    let size = [small.width() as usize, small.height() as usize];
    Some(ctx.load_texture(
        "pkp-lion-mark",
        egui::ColorImage::from_rgba_unmultiplied(size, &small),
        egui::TextureOptions::LINEAR,
    ))
}
