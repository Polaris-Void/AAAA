//! Polaris-Kit-Pro v3.0.0 "NATIVE-1080" - the 12-tool Windows suite,
//! one native launcher.
//!
//! Launch sequence:
//!   1. Load persisted settings (language + UI scale) from the user
//!      profile; a missing or corrupt file silently falls back to defaults.
//!   2. If not elevated: relaunch elevated via UAC and exit this process.
//!   3. Set the AppUserModelID (taskbar grouping + icon identity).
//!   4. Purge stale per-run staging dirs (24h cutoff).
//!   5. Open the single window (custom title bar, no OS decorations),
//!      sized for a 1920x1080 desktop.

// A pure GUI app: no console window may flash next to the launcher in
// release builds. Debug builds keep the console so prints stay visible.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use polaris_kit_pro::app::PolarisApp;
use polaris_kit_pro::settings::Settings;
use polaris_kit_pro::{theme, APP_VERSION};

fn main() -> eframe::Result<()> {
    // 1. settings first: the elevated relaunch must not drop them.
    let settings = Settings::load();

    #[cfg(windows)]
    {
        if !polaris_kit_pro::run::is_admin() {
            polaris_kit_pro::run::elevate_self();
        }
        polaris_kit_pro::run::set_app_id();
    }
    polaris_kit_pro::run::clean_old_stages();

    // NATIVE-1080: the default geometry targets a 1920x1080 desktop;
    // the minimum still fits small laptop panels.
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(APP_VERSION)
        .with_inner_size([1440.0, 900.0])
        .with_min_inner_size([1080.0, 640.0])
        .with_decorations(false);

    // the frozen lion mark as the window/taskbar icon (256px RGBA)
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let boot_lang = settings.lang;
    let boot_scale = settings.scale;

    eframe::run_native(
        APP_VERSION,
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(move |cc| {
            theme::install_fonts(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx);
            let mut app = PolarisApp::booted_with(boot_lang);
            app.set_ui_scale(&cc.egui_ctx, boot_scale);
            Ok(Box::new(app))
        }),
    )
}

/// Decode the embedded lion mark into a 256x256 window icon.
fn window_icon() -> Option<egui::IconData> {
    const LION: &[u8] = include_bytes!("../assets/lion-mark.png");
    let img = image::load_from_memory(LION).ok()?.to_rgba8();
    let icon = image::imageops::resize(&img, 256, 256, image::imageops::FilterType::Lanczos3);
    Some(egui::IconData {
        rgba: icon.into_raw(),
        width: 256,
        height: 256,
    })
}
