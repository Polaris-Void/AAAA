//! Reusable design-system widgets: soft-cornered buttons, the custom
//! window controls, and the modal scrim.

use crate::bidi;
use crate::theme::{self, Tokens};
use egui::{vec2, Color32, CornerRadius, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// Button appearance variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Inverted (white) primary button.
    Primary,
    /// Outlined, quiet.
    Secondary,
    /// Colored fill + supplied foreground.
    Tinted,
}

/// A compact control button. `label` may be Persian — it is routed through
/// the bidi engine so mixed text keeps its correct visual order, and RTL
/// labels never get letter-spaced (which would tear joined letters apart).
pub fn button(ui: &mut Ui, label: &str, kind: Kind, width: f32) -> Response {
    button_color(
        ui,
        label,
        kind,
        width,
        Color32::TRANSPARENT,
        Color32::TRANSPARENT,
    )
}

/// Button with explicit tint (used for LOG/GITHUB/LANG/STOP/CLEAR/EXPORT).
pub fn button_color(
    ui: &mut Ui,
    label: &str,
    kind: Kind,
    width: f32,
    fill: Color32,
    fg: Color32,
) -> Response {
    button_sized(ui, label, kind, vec2(width, theme::CONTROL_H), fill, fg)
}

/// Button with an explicit size (toolbar rows use the compact height).
pub fn button_sized(
    ui: &mut Ui,
    label: &str,
    kind: Kind,
    size: Vec2,
    fill: Color32,
    fg: Color32,
) -> Response {
    let t = Tokens::get();
    let (base_fill, base_stroke, base_fg) = match kind {
        Kind::Primary => (t.invert_bg, t.invert_bg, t.invert_fg),
        Kind::Secondary => (Color32::TRANSPARENT, t.line_strong, t.ink),
        Kind::Tinted => (
            fill,
            if fill == Color32::TRANSPARENT {
                t.line_strong
            } else {
                fill
            },
            fg,
        ),
    };
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let (paint_fill, paint_fg) = if response.is_pointer_button_down_on() {
        (blend(base_fill, t.sunken, 0.8), base_fg)
    } else if response.hovered() {
        (
            blend(base_fill, t.sunken, 0.55),
            if base_fg == t.ink {
                t.ink_strong
            } else {
                base_fg
            },
        )
    } else {
        (base_fill, base_fg)
    };

    let painter = ui.painter();
    painter.rect_filled(rect, theme::R_BTN, paint_fill);
    if base_stroke != t.bg {
        painter.rect_stroke(
            rect,
            theme::R_BTN,
            Stroke::new(1.0, base_stroke),
            egui::StrokeKind::Inside,
        );
    }
    let font = egui::FontId::new(11.0, theme::mono_bold());
    let galley = bidi::galley_text(
        ui,
        label,
        bidi::base_rtl(label),
        font,
        paint_fg,
        0.0,
        bidi::Align::Center,
    );
    painter.galley(
        egui::pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        paint_fg,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Blend a tint over a base color (alpha in 0..1).
fn blend(base: Color32, tint: Color32, amount: f32) -> Color32 {
    if base == Color32::TRANSPARENT {
        return tint;
    }
    let a = amount.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| ((x as f32 * (1.0 - a) + y as f32 * a).round()) as u8;
    Color32::from_rgb(
        mix(base.r(), tint.r()),
        mix(base.g(), tint.g()),
        mix(base.b(), tint.b()),
    )
}

/// The three custom titlebar buttons: small, rounded, painter-drawn glyphs
/// (a crisp minus / square / cross — no font glyphs, so they can never
/// fall back to a placeholder triangle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinBtn {
    Min,
    Max,
    Close,
}

pub fn window_buttons(ui: &mut Ui, maximized: bool) -> Option<WinBtn> {
    let t = Tokens::get();
    const W: f32 = 40.0;
    const H: f32 = 30.0;
    const GAP: f32 = 4.0;
    let mut pressed = None;
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        for button in [WinBtn::Min, WinBtn::Max, WinBtn::Close] {
            let (rect, response) = ui.allocate_exact_size(vec2(W, H), Sense::click());
            let hovered = response.hovered();
            let painter = ui.painter();
            let close = button == WinBtn::Close;
            if hovered {
                let fill = if close {
                    theme::CLOSE_RED
                } else {
                    Color32::from_white_alpha(14)
                };
                painter.rect_filled(rect, theme::R_BTN, fill);
            }
            let ink = if hovered && close {
                Color32::WHITE
            } else {
                t.muted
            };
            let c = rect.center();
            match button {
                WinBtn::Min => {
                    painter.line_segment(
                        [
                            egui::pos2(c.x - 5.0, c.y + 3.5),
                            egui::pos2(c.x + 5.0, c.y + 3.5),
                        ],
                        Stroke::new(1.6, ink),
                    );
                }
                WinBtn::Max => {
                    if maximized {
                        // restore: two offset squares
                        let back = Rect::from_center_size(
                            egui::pos2(c.x + 2.0, c.y - 2.0),
                            vec2(9.0, 9.0),
                        );
                        let front = Rect::from_center_size(
                            egui::pos2(c.x - 1.5, c.y + 1.5),
                            vec2(9.0, 9.0),
                        );
                        painter.rect_filled(front, CornerRadius::same(2), t.bg);
                        painter.rect_stroke(
                            front,
                            CornerRadius::same(2),
                            Stroke::new(1.3, ink),
                            egui::StrokeKind::Inside,
                        );
                        painter.rect_stroke(
                            back,
                            CornerRadius::same(2),
                            Stroke::new(1.3, ink),
                            egui::StrokeKind::Inside,
                        );
                    } else {
                        let r = Rect::from_center_size(c, vec2(10.0, 10.0));
                        painter.rect_stroke(
                            r,
                            CornerRadius::same(2),
                            Stroke::new(1.4, ink),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
                WinBtn::Close => {
                    painter.line_segment(
                        [
                            egui::pos2(c.x - 4.5, c.y - 4.5),
                            egui::pos2(c.x + 4.5, c.y + 4.5),
                        ],
                        Stroke::new(1.6, ink),
                    );
                    painter.line_segment(
                        [
                            egui::pos2(c.x - 4.5, c.y + 4.5),
                            egui::pos2(c.x + 4.5, c.y - 4.5),
                        ],
                        Stroke::new(1.6, ink),
                    );
                }
            }
            if response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                pressed = Some(button);
            }
        }
    });
    pressed
}

/// Dim the page behind a modal dialog.
pub fn scrim(ui: &mut Ui, amount: f32) {
    let rect = ui.ctx().input(|input| input.viewport_rect());
    ui.painter()
        .rect_filled(rect, theme::R_FRAME, theme::scrim(amount));
}

/// A small caps HUD label (letterspaced latin; RTL-safe for Persian).
pub fn hud_label(ui: &mut Ui, text: &str, color: Color32) -> Response {
    let galley = bidi::galley_text(
        ui,
        text,
        bidi::base_rtl(text),
        egui::FontId::new(theme::HUD_SIZE, theme::mono()),
        color,
        0.0,
        bidi::Align::Left,
    );
    let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
    ui.painter().galley(rect.min, galley, Color32::WHITE);
    response
}

/// Centered HUD text drawn directly with the painter inside `rect`.
pub fn hud_at(painter: &egui::Painter, ui: &Ui, text: &str, rect: Rect, color: Color32) {
    let galley = bidi::galley_text(
        ui,
        text,
        bidi::base_rtl(text),
        egui::FontId::new(theme::HUD_SIZE, theme::mono()),
        color,
        0.0,
        bidi::Align::Left,
    );
    painter.galley(
        egui::pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        Color32::WHITE,
    );
}

/// Left/right anchored text drawn with the painter (RTL text anchors
/// right, LTR anchors left) — used for table cells and log rows.
pub fn text_at(
    ui: &Ui,
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    color: Color32,
    anchor: Pos2,
    right: bool,
) -> egui::Vec2 {
    let galley = bidi::galley_text(
        ui,
        text,
        bidi::base_rtl(text),
        font,
        color,
        0.0,
        if right {
            bidi::Align::Right
        } else {
            bidi::Align::Left
        },
    );
    let pos = if right {
        egui::pos2(anchor.x - galley.size().x, anchor.y - galley.size().y / 2.0)
    } else {
        egui::pos2(anchor.x, anchor.y - galley.size().y / 2.0)
    };
    painter.galley(pos, galley.clone(), Color32::WHITE);
    galley.size()
}

/// Centered painter text (returns the size for callers that need it).
pub fn text_centered(
    ui: &Ui,
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    color: Color32,
    center: egui::Pos2,
) -> egui::Vec2 {
    let galley = bidi::galley_text(
        ui,
        text,
        bidi::base_rtl(text),
        font,
        color,
        0.0,
        bidi::Align::Center,
    );
    painter.galley(
        egui::pos2(
            center.x - galley.size().x / 2.0,
            center.y - galley.size().y / 2.0,
        ),
        galley.clone(),
        Color32::WHITE,
    );
    galley.size()
}
