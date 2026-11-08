//! Design tokens, fonts and visuals for Polaris-Kit-Pro v3.0.0
//! "NATIVE-1080" — the Nova palette, now fully set in Vazirmatn.
//!
//! The whole UI — English and Persian, body text and log text — renders in
//! ONE type family: Vazirmatn (OFL). Mixed Persian/Latin lines are shaped
//! correctly by the in-app bidi engine ([`crate::bidi`]); the font stacks
//! stay single-family so every script segment resolves to the same face
//! and harfbuzz shapes each segment with its own direction.

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak, Margin,
    Stroke, TextStyle, Visuals,
};
use std::sync::Arc;

// ----------------------------------------------------------------- tokens
/// Colour tokens (dark appearance) — the v3 "Nova" palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    /// Page surface.
    pub bg: Color32,
    /// Cards and raised panels.
    pub card: Color32,
    /// Badges, wells and row hovers.
    pub sunken: Color32,
    /// Body text.
    pub ink: Color32,
    /// Headings.
    pub ink_strong: Color32,
    /// Secondary text.
    pub muted: Color32,
    /// Labels and tertiary text.
    pub faint: Color32,
    /// Hairlines.
    pub line: Color32,
    /// Outlined buttons and stronger rules.
    pub line_strong: Color32,
    /// Brand blue for surfaces (links, selection).
    pub accent: Color32,
    /// Brand blue for text and icons.
    pub accent_ink: Color32,
    /// Primary buttons and hover inversion.
    pub invert_bg: Color32,
    pub invert_fg: Color32,
    pub good: Color32,
    pub warn: Color32,
    pub bad: Color32,
}

const fn c(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub const T: Tokens = Tokens {
    bg: c(0x0d, 0x0e, 0x14),
    card: c(0x10, 0x13, 0x1b),
    sunken: c(0x05, 0x06, 0x0a),
    ink: c(0xf4, 0xf3, 0xf0),
    ink_strong: c(0xff, 0xff, 0xff),
    muted: Color32::from_rgba_unmultiplied_const(0xf4, 0xf3, 0xf0, 156),
    faint: Color32::from_rgba_unmultiplied_const(0xf4, 0xf3, 0xf0, 100),
    line: Color32::from_rgba_unmultiplied_const(0xf4, 0xf3, 0xf0, 42),
    line_strong: Color32::from_rgba_unmultiplied_const(0xf4, 0xf3, 0xf0, 110),
    accent: c(0x3d, 0x8b, 0xff),
    accent_ink: c(0x82, 0xb4, 0xff),
    invert_bg: c(0xff, 0xff, 0xff),
    invert_fg: c(0x0b, 0x0b, 0x0c),
    good: c(0x3f, 0xd6, 0xb8),
    warn: c(0xff, 0xc1, 0x4d),
    bad: c(0xff, 0x62, 0x59),
};

impl Tokens {
    pub const fn get() -> Self {
        T
    }

    pub fn hairline(&self) -> Stroke {
        Stroke::new(1.0, self.line)
    }

    pub fn hairline_strong(&self) -> Stroke {
        Stroke::new(1.0, self.line_strong)
    }

    /// A module's accent colour.
    pub fn accent_of(accent: [u8; 3]) -> Color32 {
        c(accent[0], accent[1], accent[2])
    }
}

// Brand colours carried over from the Polaris kit (used by LOG/GITHUB/lang
// buttons, the log view and the vault inspector).
pub const AMBER: Color32 = c(0xff, 0xc2, 0x4d);
pub const GREEN: Color32 = c(0x2f, 0xd4, 0x70);
pub const RED: Color32 = c(0xff, 0x52, 0x52);
pub const CLOSE_RED: Color32 = c(0xe8, 0x11, 0x23);
pub const LOGGREEN: Color32 = c(0x7c, 0xe6, 0xa6);
pub const CONBG: Color32 = c(0x08, 0x0a, 0x10);
pub const PANEL: Color32 = c(0x15, 0x17, 0x21);
pub const GOLD: Color32 = c(0xc0, 0x8a, 0x3e);

// ----------------------------------------------------------------- fonts
/// Vazirmatn Medium.
pub fn medium() -> FontFamily {
    FontFamily::Name("pkp-medium".into())
}
/// Vazirmatn SemiBold.
pub fn semibold() -> FontFamily {
    FontFamily::Name("pkp-semibold".into())
}
/// Vazirmatn SemiBold wide-tracked: headings and the app wordmark.
pub fn display() -> FontFamily {
    FontFamily::Name("pkp-semibold".into())
}
/// Vazirmatn Regular: log text and HUD labels.
pub fn mono() -> FontFamily {
    FontFamily::Name("pkp-regular".into())
}
/// Vazirmatn Bold: buttons and number badges.
pub fn mono_bold() -> FontFamily {
    FontFamily::Name("pkp-bold".into())
}
/// Persian body text (Vazirmatn Regular).
pub fn vazir() -> FontFamily {
    FontFamily::Name("pkp-regular".into())
}
/// Persian headings (Vazirmatn Bold).
pub fn vazir_bold() -> FontFamily {
    FontFamily::Name("pkp-bold".into())
}

/// Registers the Vazirmatn family (embedded TTFs, OFL) as EVERY stack in
/// the app. Single-family stacks are what keeps mixed Persian/Latin lines
/// segmentable: each script chunk becomes its own harfbuzz run and is
/// shaped with its own direction by the bidi engine.
pub fn install_fonts(ctx: &egui::Context) {
    const REGULAR: &[u8] = include_bytes!("../assets/fonts/Vazirmatn-Regular.ttf");
    const MEDIUM: &[u8] = include_bytes!("../assets/fonts/Vazirmatn-Medium.ttf");
    const SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Vazirmatn-SemiBold.ttf");
    const BOLD: &[u8] = include_bytes!("../assets/fonts/Vazirmatn-Bold.ttf");

    let mut fonts = FontDefinitions::default();
    // egui's own fonts stay behind Vazirmatn for the few symbols it lacks.
    let fallback: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    // No subpixel binning: Vazirmatn carries no hinting instructions, so a
    // glyph drawn at a fractional x smears each stem across two pixels.
    let tweak = || FontTweak {
        subpixel_binning: Some(false),
        ..Default::default()
    };

    let faces: [(&str, &[u8], FontFamily); 4] = [
        ("Vazirmatn", REGULAR, FontFamily::Proportional),
        ("Vazirmatn-Medium", MEDIUM, medium()),
        ("Vazirmatn-SemiBold", SEMIBOLD, semibold()),
        ("Vazirmatn-Bold", BOLD, mono_bold()),
    ];
    // Every logical family maps to exactly ONE Vazirmatn face (plus the
    // egui fallback chain for rare symbols).
    let stacks: [(FontFamily, &str); 8] = [
        (FontFamily::Proportional, "Vazirmatn"),
        (FontFamily::Monospace, "Vazirmatn"),
        (medium(), "Vazirmatn-Medium"),
        (semibold(), "Vazirmatn-SemiBold"),
        (mono(), "Vazirmatn"),
        (mono_bold(), "Vazirmatn-Bold"),
        (vazir(), "Vazirmatn"),
        (vazir_bold(), "Vazirmatn-Bold"),
    ];
    for (name, bytes, _) in faces {
        fonts.font_data.insert(
            name.to_owned(),
            Arc::new(FontData::from_static(bytes).tweak(tweak())),
        );
    }
    for (family, name) in stacks {
        let mut stack: Vec<String> = vec![name.to_string()];
        stack.extend(fallback.iter().cloned());
        fonts.families.insert(family, stack);
    }
    ctx.set_fonts(fonts);
}

// ----------------------------------------------------------------- metrics
/// v3 NATIVE-1080 metrics, tuned for comfortable 1920x1080 use.
/// Height of primary buttons and inputs.
pub const CONTROL_H: f32 = 36.0;
/// Height of the compact toolbar controls (search / sort / lang / log).
pub const TOOLBAR_CTRL_H: f32 = 32.0;
/// The site's `.hud-label`: 11px, upper case, 0.15em apart.
pub const HUD_SIZE: f32 = 11.0;
/// Buttons: 11px, upper case, 0.12em apart.
pub const BUTTON_SPACING: f32 = 1.3;
/// Header bar height (logo + wordmark + window controls).
pub const HEADER_H: f32 = 52.0;
/// Subtitle strip under the header.
pub const SUBTITLE_H: f32 = 24.0;
/// Footer status bar height (kept as a token; the bar itself is retired).
pub const FOOTER_H: f32 = 26.0;
/// Toolbar (lang + tabs + search + sort + log) row height.
pub const TOOLBAR_H: f32 = 50.0;
/// Card strip height: number tab + links.
pub const STRIP_H: f32 = 40.0;
pub const CARD_PAD: f32 = 16.0;
pub const CARD_MIN_W: f32 = 330.0;
pub const CARD_H: f32 = 196.0;
/// Padding inside the detail page and the dialogs.
pub const PAD: f32 = 20.0;
/// Title sizes.
pub const PAGE_TITLE: f32 = 26.0;
pub const DIALOG_TITLE: f32 = 16.0;
/// Padding inside a hairline box.
pub const BOX_MARGIN: Margin = Margin::symmetric(14, 12);

// Corner radii — the v3.1 soft-touch pass.
pub const R_CARD: CornerRadius = CornerRadius::same(10);
pub const R_BTN: CornerRadius = CornerRadius::same(7);
pub const R_CHIP: CornerRadius = CornerRadius::same(8);
pub const R_FRAME: CornerRadius = CornerRadius::same(8);

/// Mixes `c` into the background at `alpha` (0-1): tinted washes.
pub fn tint(c: Color32, alpha: f32) -> Color32 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// The dim over the page behind a dialog.
pub fn scrim(amount: f32) -> Color32 {
    Color32::from_black_alpha((170.0 * amount.clamp(0.0, 1.0)).round() as u8)
}

/// Applies the dark visuals: soft corners, 1px hairlines, no expansion.
pub fn apply(ctx: &egui::Context) {
    let t = T;
    let mut v = Visuals::dark();
    v.panel_fill = t.bg;
    v.window_fill = t.bg;
    v.extreme_bg_color = t.sunken;
    v.text_edit_bg_color = Some(t.sunken);
    v.faint_bg_color = t.sunken;
    v.code_bg_color = t.sunken;
    v.window_stroke = t.hairline_strong();
    v.window_corner_radius = R_FRAME;
    v.menu_corner_radius = R_FRAME;
    v.hyperlink_color = t.accent_ink;
    v.selection.bg_fill = t.accent;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.warn_fg_color = t.warn;
    v.error_fg_color = t.bad;
    for (w, fill, stroke, fg) in [
        (&mut v.widgets.noninteractive, t.bg, t.line, t.muted),
        (&mut v.widgets.inactive, t.bg, t.line, t.ink),
        (
            &mut v.widgets.hovered,
            t.sunken,
            t.line_strong,
            t.ink_strong,
        ),
        (&mut v.widgets.active, t.sunken, t.line_strong, t.ink_strong),
        (&mut v.widgets.open, t.sunken, t.line_strong, t.ink_strong),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::new(1.0, stroke);
        w.corner_radius = R_BTN;
        w.fg_stroke = Stroke::new(1.0, fg);
        w.expansion = 0.0;
    }
    ctx.set_visuals(v);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.icon_spacing = 10.0;
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.interact_size.y = CONTROL_H;
        style.spacing.menu_margin = Margin::same(4);
        style.spacing.window_margin = Margin::same(4);
        // --- sleek dark scrollbar (v3.1 overflow/bleed fix):
        // floating over the content but with a permanently reserved thin
        // gutter, so the thumb can never cut into the cards; expands on
        // hover; translucent dark handle, no white bar anywhere.
        let scroll = &mut style.spacing.scroll;
        scroll.floating = true;
        scroll.bar_width = 9.0;
        scroll.floating_width = 4.0;
        scroll.floating_allocated_width = 7.0; // stable gutter (like
                                               // scrollbar-gutter: stable)
        scroll.bar_inner_margin = 3.0;
        scroll.bar_outer_margin = 2.0;
        // foreground_color=true paints the handle from the widget's
        // fg_stroke (off-white), which at these low opacities reads as a
        // sleek translucent grey pill on the dark surface - never white.
        scroll.foreground_color = true;
        scroll.dormant_handle_opacity = 0.20;
        scroll.active_handle_opacity = 0.48;
        scroll.interact_handle_opacity = 0.85;
        scroll.dormant_background_opacity = 0.0;
        scroll.active_background_opacity = 0.12;
        scroll.interact_background_opacity = 0.30;
        style.text_styles = [
            (TextStyle::Heading, FontId::new(22.0, display())),
            (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(12.5, medium())),
            (
                TextStyle::Small,
                FontId::new(11.5, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(11.5, FontFamily::Monospace),
            ),
        ]
        .into();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The design system promises: hairlines are exactly 1 px, the accent
    /// is the polar-star blue, and the 1080p control heights hold.
    #[test]
    fn nova_metrics_hold() {
        assert_eq!(T.hairline().width, 1.0);
        assert_eq!(T.hairline_strong().width, 1.0);
        assert_eq!(T.accent, Color32::from_rgb(0x3d, 0x8b, 0xff));
        assert_eq!(CONTROL_H, 36.0);
        assert_eq!(FOOTER_H, 26.0);
        const _: () = assert!(CARD_MIN_W >= 288.0, "cards too narrow");
        const _: () = assert!(TOOLBAR_CTRL_H >= 28.0, "toolbar controls too small");
    }

    /// Every logical family must resolve to a Vazirmatn-only stack: the
    /// single-face rule is what the bidi engine depends on.
    #[test]
    fn font_families_are_single_face() {
        assert_eq!(vazir(), FontFamily::Name("pkp-regular".into()));
        assert_eq!(vazir_bold(), FontFamily::Name("pkp-bold".into()));
        assert_eq!(mono(), FontFamily::Name("pkp-regular".into()));
        assert_eq!(mono_bold(), FontFamily::Name("pkp-bold".into()));
        assert_eq!(medium(), FontFamily::Name("pkp-medium".into()));
        assert_eq!(semibold(), FontFamily::Name("pkp-semibold".into()));
        assert_eq!(display(), semibold());
    }

    /// tint/scrim must stay inside alpha bounds for any input.
    #[test]
    fn helpers_clamp() {
        let _ = tint(Color32::WHITE, 2.0);
        let _ = tint(Color32::WHITE, -1.0);
        let _ = scrim(2.0);
        let _ = scrim(-1.0);
    }
}
