//! The launcher views: toolbar (language + categories + search + sort +
//! log), the module grid, tool details, the live log, and the two in-app
//! dialogs. All text — English and Persian — flows through the bidi
//! engine ([`crate::bidi`]) so mixed Persian/Latin lines keep their
//! correct visual order, and every language direction gets the right
//! alignment.

use crate::app::{Confirm, Lang, PolarisApp};
use crate::bidi::{self, Align, Styled};
use crate::data::{self, Block, Run, Style};
use crate::readmes::{README_EN, README_FA};
use crate::theme::{self, Tokens};
use crate::widgets;
use egui::text::TextFormat;
use egui::{vec2, Color32, CornerRadius, FontId, Frame, Pos2, Rect, ScrollArea, Sense, Stroke, Ui};

/// Is the pointer currently over `rect`?
fn ptr_over(ui: &Ui, rect: Rect) -> bool {
    ui.ctx()
        .input(|in_| in_.pointer.interact_pos())
        .map(|pt| rect.contains(pt))
        .unwrap_or(false)
}

/// Blend `b` into `a` by `k` (0..1) — opaque hover washes.
fn mix(a: Color32, b: Color32, k: f32) -> Color32 {
    let f = |x: u8, y: u8| ((x as f32 * (1.0 - k) + y as f32 * k).round()) as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

// readme palette (the rendered-readme look)
const RD_H1: Color32 = Color32::from_rgb(0x5b, 0x96, 0xff);
const RD_H2: Color32 = Color32::from_rgb(0x4a, 0xde, 0x80);
const RD_H3: Color32 = Color32::from_rgb(0x7d, 0xd3, 0xfc);
const RD_CODE_INK: Color32 = Color32::from_rgb(0xfb, 0xbf, 0x24);
const RD_CODE_BG: Color32 = Color32::from_rgb(0x1d, 0x2b, 0x47);
const RD_BOLD: Color32 = Color32::from_rgb(0xee, 0xf6, 0xff);
const RD_LINK: Color32 = Color32::from_rgb(0x22, 0xd3, 0xee);
const RD_BLOCK_INK: Color32 = Color32::from_rgb(0x9c, 0xc4, 0xff);
const RD_BLOCK_BG: Color32 = Color32::from_rgb(0x0a, 0x15, 0x26);
const RD_QUOTE: Color32 = Color32::from_rgb(0x9c, 0xc4, 0xff);
const RD_TH_INK: Color32 = Color32::from_rgb(0x67, 0xe8, 0xf9);
const RD_TH_BG: Color32 = Color32::from_rgb(0x13, 0x27, 0x41);
const RD_TD_INK: Color32 = Color32::from_rgb(0xc9, 0xd6, 0xea);
const RD_TD_BG: Color32 = Color32::from_rgb(0x0e, 0x1c, 0x31);
const RD_TD_BG2: Color32 = Color32::from_rgb(0x11, 0x22, 0x3a);
const RD_HR: Color32 = Color32::from_rgb(0x2a, 0x3b, 0x55);
const RD_INK: Color32 = Color32::from_rgb(0xc9, 0xd6, 0xea);

fn rd_lang(lang: Lang, mid: &str) -> &'static [Block] {
    let table = if lang == Lang::Fa {
        &README_FA
    } else {
        &README_EN
    };
    table
        .iter()
        .find(|(m, _)| *m == mid)
        .map(|(_, b)| *b)
        .unwrap_or(&[])
}

// ============================================================== toolbar
/// One row: [categories…] …… [GRID|LIST] [SEARCH] [SORT].
///
/// LTR (EN): categories start at the left edge; the control cluster sits at
/// the right edge, ordered [view] [search] [sort] — the search bar occupies
/// the slot previously held by the sort control, and the sort buttons moved
/// to the right-hand side adjacent to the search.
/// RTL (FA): the whole row mirrors — categories flow right-to-left from the
/// right edge and the cluster anchors the left edge with the mirrored
/// order, so both languages get a perfectly symmetric professional layout.
/// The language switcher and the LOG button live in the header bar now.
pub fn toolbar(ui: &mut Ui, app: &mut PolarisApp, ctx: &egui::Context) {
    let t = Tokens::get();
    let fa = app.lang.fa();
    ui.horizontal(|ui| {
        ui.set_min_height(theme::TOOLBAR_H);
        if fa {
            // cluster anchors the LEFT edge, categories flow from the RIGHT
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                // mirrored visual order: [sort] [search] [view]
                ui.add_space(12.0);
                sort_toggle(ui, app, ctx);
                ui.add_space(8.0);
                search_box(ui, app);
                ui.add_space(8.0);
                view_toggle(ui, app, ctx);
                ui.add_space(12.0);
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(12.0);
                chips_row(ui, app, ctx);
            });
        } else {
            ui.add_space(12.0);
            chips_row(ui, app, ctx);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(12.0);
                view_toggle(ui, app, ctx);
                ui.add_space(8.0);
                search_box(ui, app);
                ui.add_space(8.0);
                sort_toggle(ui, app, ctx);
                ui.add_space(12.0);
            });
        }
    });
    // the separator under the whole toolbar
    let rect = ui.max_rect();
    ui.painter()
        .hline(rect.x_range(), rect.bottom() - 1.0, t.hairline());
}

/// The nine category pills. They start at the leading edge (left in EN,
/// right in FA) and flow in the reading direction — ALL first, SYS last.
fn chips_row(ui: &mut Ui, app: &mut PolarisApp, ctx: &egui::Context) {
    let t = Tokens::get();
    let fa = app.lang.fa();
    // static width of the control cluster (view 128 + search 220 + sort 118
    // + gaps/margins)
    const CLUSTER_W: f32 = 12.0 + 128.0 + 8.0 + 220.0 + 8.0 + 118.0 + 12.0;
    let avail = (ui.available_width() - CLUSTER_W - 12.0).max(320.0);
    let gap = 6.0;
    let cell = (((avail - gap * 8.0) / data::TABS.len() as f32).floor()).clamp(56.0, 96.0);
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (id, en, fa_label) in data::TABS {
            let label = if fa { fa_label } else { en };
            let on = app.filter == id;
            let (rect, resp) = ui.allocate_exact_size(vec2(cell, 30.0), Sense::click());
            let hovered = resp.hovered();
            let p = ui.painter();
            let (fill, ink, stroke) = if on {
                (t.accent, Color32::WHITE, Stroke::NONE)
            } else if hovered {
                (t.card, t.ink_strong, t.hairline_strong())
            } else {
                (t.card, t.muted, t.hairline())
            };
            p.rect_filled(rect, theme::R_CHIP, fill);
            if stroke != Stroke::NONE {
                p.rect_stroke(rect, theme::R_CHIP, stroke, egui::StrokeKind::Inside);
            }
            let font = if fa {
                FontId::new(10.5, theme::vazir_bold())
            } else {
                FontId::new(9.5, theme::mono_bold())
            };
            widgets::text_centered(ui, p, label, font, ink, rect.center());
            if resp
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                app.cat_changed(id, ctx);
            }
        }
    });
}

/// The search field: soft-cornered well, magnifier icon, vertically
/// centered text and caret, focus ring, and a language-aware placeholder.
fn search_box(ui: &mut Ui, app: &mut PolarisApp) {
    let t = Tokens::get();
    const W: f32 = 220.0;
    const H: f32 = 32.0;
    let fa = app.lang.fa();
    let (rect, resp) = ui.allocate_exact_size(vec2(W, H), Sense::click());
    let p = ui.painter().clone();
    let focused = app.search_focus;
    p.rect_filled(rect, CornerRadius::same(9), t.sunken);
    let stroke = if focused {
        Stroke::new(1.4, t.accent)
    } else if resp.hovered() {
        t.hairline_strong()
    } else {
        t.hairline()
    };
    p.rect_stroke(
        rect,
        CornerRadius::same(9),
        stroke,
        egui::StrokeKind::Inside,
    );
    // magnifier icon: left in EN, right in FA
    let c = egui::pos2(
        if fa {
            rect.right() - 15.0
        } else {
            rect.left() + 15.0
        },
        rect.center().y,
    );
    p.circle_stroke(
        egui::pos2(c.x - 1.5, c.y - 1.5),
        4.0,
        Stroke::new(1.4, t.faint),
    );
    p.line_segment(
        [
            egui::pos2(c.x + 1.5, c.y + 1.5),
            egui::pos2(c.x + 4.5, c.y + 4.5),
        ],
        Stroke::new(1.4, t.faint),
    );
    // the edit itself, vertically centered so the caret sits mid-box
    let edit = if fa {
        Rect::from_min_max(
            egui::pos2(rect.left() + 10.0, rect.top() + 2.0),
            egui::pos2(c.x - 12.0, rect.bottom() - 2.0),
        )
    } else {
        Rect::from_min_max(
            egui::pos2(c.x + 12.0, rect.top() + 2.0),
            egui::pos2(rect.right() - 10.0, rect.bottom() - 2.0),
        )
    };
    let edit_resp = ui.put(
        edit,
        egui::TextEdit::singleline(&mut app.search)
            .frame(Frame::NONE)
            .font(FontId::new(11.5, theme::medium()))
            .desired_width(f32::INFINITY)
            .margin(egui::Margin::same(0))
            .cursor_at_end(false),
    );
    app.search_focus = edit_resp.has_focus();
    // clicking the icon / padding area focuses the edit
    if resp.clicked() {
        ui.ctx().memory_mut(|m| m.request_focus(edit_resp.id));
    }
    // placeholder
    if app.search.is_empty() && !focused {
        let ph = app.lang.s("search_ph");
        let anchor = if fa {
            Pos2::new(edit.right(), edit.center().y)
        } else {
            Pos2::new(edit.left(), edit.center().y)
        };
        widgets::text_at(
            ui,
            &p,
            ph,
            FontId::new(11.0, theme::medium()),
            t.faint,
            anchor,
            fa,
        );
    }
}

/// The segmented view switch: [GRID | LIST] (FA: [شبکه | فهرست]).
/// The active mode is filled; the choice persists in settings.ini.
fn view_toggle(ui: &mut Ui, app: &mut PolarisApp, ctx: &egui::Context) {
    let t = Tokens::get();
    const W: f32 = 64.0;
    const H: f32 = 32.0;
    let fa = app.lang.fa();
    let (rect_outer, _) = ui.allocate_exact_size(vec2(W * 2.0, H), Sense::hover());
    let p = ui.painter();
    p.rect_stroke(
        rect_outer,
        theme::R_BTN,
        t.hairline_strong(),
        egui::StrokeKind::Inside,
    );
    // LTR: grid left, list right. RTL: mirrored.
    let left_cell = Rect::from_min_size(rect_outer.min, vec2(W, H));
    let right_cell = Rect::from_min_size(
        egui::pos2(rect_outer.min.x + W, rect_outer.min.y),
        vec2(W, H),
    );
    let (grid_rect, list_rect) = if fa {
        (right_cell, left_cell)
    } else {
        (left_cell, right_cell)
    };
    let grid_outer_left = !fa;
    for (mode, rect, outer_left, label) in [
        (
            crate::app::ViewMode::Grid,
            grid_rect,
            grid_outer_left,
            app.lang.s("view_grid"),
        ),
        (
            crate::app::ViewMode::List,
            list_rect,
            !grid_outer_left,
            app.lang.s("view_list"),
        ),
    ] {
        let radius = CornerRadius {
            nw: if outer_left { 7 } else { 0 },
            ne: if outer_left { 0 } else { 7 },
            sw: if outer_left { 7 } else { 0 },
            se: if outer_left { 0 } else { 7 },
        };
        let on = app.view == mode;
        let resp = ui.interact(rect, ui.id().with(("view", label)), Sense::click());
        let hovered = resp.hovered();
        if on {
            p.rect_filled(rect, radius, t.accent);
        } else if hovered {
            p.rect_filled(rect, radius, t.card);
        }
        let ink = if on {
            Color32::WHITE
        } else if hovered {
            t.ink_strong
        } else {
            t.muted
        };
        let font = if fa {
            FontId::new(10.0, theme::vazir_bold())
        } else {
            FontId::new(9.5, theme::mono_bold())
        };
        widgets::text_centered(ui, p, label, font, ink, rect.center());
        if resp
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
            && app.view != mode
        {
            app.set_view(mode, ctx);
        }
    }
}

/// The segmented sort switch: [A – Z | 01 – 12] (FA: [الفبا | ترتیب]).
fn sort_toggle(ui: &mut Ui, app: &mut PolarisApp, ctx: &egui::Context) {
    let t = Tokens::get();
    const W: f32 = 59.0;
    const H: f32 = 32.0;
    let (rect_outer, _) = ui.allocate_exact_size(vec2(W * 2.0, H), Sense::hover());
    let p = ui.painter();
    p.rect_stroke(
        rect_outer,
        theme::R_BTN,
        t.hairline_strong(),
        egui::StrokeKind::Inside,
    );
    let left = Rect::from_min_size(rect_outer.min, vec2(W, H));
    let right = Rect::from_min_size(
        egui::pos2(rect_outer.min.x + W, rect_outer.min.y),
        vec2(W, H),
    );
    let fa = app.lang.fa();
    // RTL mirrors the segmented control: the first option sits on the right.
    let (az_rect, order_rect) = if fa { (right, left) } else { (left, right) };
    let az_outer_left = !fa;
    for (mode, rect, outer_left) in [
        (true, az_rect, az_outer_left),
        (false, order_rect, !az_outer_left),
    ] {
        let radius = CornerRadius {
            nw: if outer_left { 7 } else { 0 },
            ne: if outer_left { 0 } else { 7 },
            sw: if outer_left { 7 } else { 0 },
            se: if outer_left { 0 } else { 7 },
        };
        let on = app.sort_az == mode;
        let resp = ui.interact(rect, ui.id().with(("sort", mode)), Sense::click());
        let hovered = resp.hovered();
        if on {
            p.rect_filled(rect, radius, t.accent);
        } else if hovered {
            p.rect_filled(rect, radius, t.card);
        }
        let label = if mode {
            app.lang.s("sort_az")
        } else {
            app.lang.s("sort_order")
        };
        let ink = if on {
            Color32::WHITE
        } else if hovered {
            t.ink_strong
        } else {
            t.muted
        };
        let font = if fa && mode {
            FontId::new(10.5, theme::vazir_bold())
        } else {
            FontId::new(9.5, theme::mono_bold())
        };
        widgets::text_centered(ui, p, label, font, ink, rect.center());
        if resp
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
            && app.sort_az != mode
        {
            app.sort_toggle(ctx);
        }
    }
}

// ============================================================== section
/// The thin bar above the grid: "SYS — 1 TOOL" (or «سیستم — ۱ ابزار»)
/// with a hairline under it. Gives every category — including the
/// single-tool ones like SYS and AUDIT — a proper framed section.
pub fn section_bar(ui: &mut Ui, app: &mut PolarisApp) {
    let t = Tokens::get();
    let fa = app.lang.fa();
    let count = app.visible_modules().len();
    let (cat_id, _, cat_fa) = *data::TABS
        .iter()
        .find(|tab| tab.0 == app.filter)
        .unwrap_or(&data::TABS[0]);
    let text = if fa {
        format!("{} — {} {}", cat_fa, fa_num(count), app.lang.s("tools"))
    } else {
        format!("{} — {} {}", cat_id, count, app.lang.s("tools"))
    };
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
    let galley = bidi::galley_text(
        ui,
        &text,
        fa,
        FontId::new(10.0, theme::mono()),
        t.faint,
        0.0,
        if fa { Align::Right } else { Align::Left },
    );
    let pos = if fa {
        Pos2::new(
            rect.right() - galley.size().x,
            rect.center().y - galley.size().y / 2.0,
        )
    } else {
        Pos2::new(rect.left() + 2.0, rect.center().y - galley.size().y / 2.0)
    };
    ui.painter().galley(pos, galley, Color32::WHITE);
    ui.painter()
        .hline(rect.x_range(), rect.bottom() - 1.0, t.hairline());
}

/// Dashboard body: the responsive card deck (grid) or the compact
/// structured list. The preference is saved locally and both views share
/// the same selection behaviour.
pub fn body(ui: &mut Ui, app: &mut PolarisApp, ctx: &egui::Context) {
    match app.view {
        crate::app::ViewMode::Grid => grid_view(ui, app, ctx),
        crate::app::ViewMode::List => list_view(ui, app, ctx),
    }
}

/// Seconds elapsed since the current run started (progress readout).
fn self_elapsed(app: &PolarisApp) -> u64 {
    app.busy_started.elapsed().as_secs()
}

// ============================================================== list
/// Compact structured list: [ID] [TITLE] [CATEGORY] [BRIEF] [GITHUB] [OPEN].
/// Mirrors in RTL (badge on the right, quick actions on the left).
fn list_view(ui: &mut Ui, app: &mut PolarisApp, ctx: &egui::Context) {
    let t = Tokens::get();
    let fa = app.lang.fa();
    let idxs = app.visible_modules();
    if idxs.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(70.0);
            let g = bidi::galley_text(
                ui,
                app.lang.s("modules"),
                app.lang.rtl(),
                FontId::new(18.0, theme::semibold()),
                t.muted,
                0.0,
                Align::Center,
            );
            bidi::put(ui, g, false);
            ui.add_space(8.0);
            let g = bidi::galley_text(
                ui,
                app.lang.s("no_match"),
                app.lang.rtl(),
                FontId::new(12.0, theme::medium()),
                t.faint,
                0.0,
                Align::Center,
            );
            bidi::put(ui, g, false);
        });
        return;
    }

    ScrollArea::vertical().id_salt("pkp_list").show(ui, |ui| {
        ui.add_space(6.0);
        let row_h = 62.0;
        let w = ui.available_width();
        for (rank, &i) in idxs.iter().enumerate() {
            let m = &data::MODULES[i];
            let accent = Tokens::accent_of(m.accent);
            let (rect, _) = ui.allocate_exact_size(vec2(w, row_h), Sense::hover());
            let p = ui.painter();
            let hovered = ptr_over(ui, rect) && !app.busy;
            if rank % 2 == 1 || hovered {
                let fill = if hovered {
                    mix(t.card, accent, 0.07)
                } else {
                    t.card
                };
                p.rect_filled(rect, theme::R_CHIP, fill);
            }
            if hovered {
                p.rect_stroke(
                    rect,
                    theme::R_CHIP,
                    Stroke::new(1.0, mix(t.line_strong, accent, 0.5)),
                    egui::StrokeKind::Inside,
                );
            }

            // layout columns; RTL mirrors them
            let pad = 14.0;
            let open_w = 92.0;
            let gh_w = 78.0;
            let badge_w = 40.0;
            let cat_w = 78.0;
            let gap = 10.0;
            // x anchors from the leading edge
            let x0 = if fa { rect.right() - pad } else { rect.left() + pad };
            let dir: f32 = if fa { -1.0 } else { 1.0 };
            let mut x = x0;

            // ID badge
            let badge = Rect::from_min_size(
                Pos2::new(if fa { x - badge_w } else { x }, rect.center().y - 11.0),
                vec2(badge_w, 22.0),
            );
            p.rect_filled(badge, CornerRadius::same(6), accent);
            let num = if fa {
                fa_num2(i + 1)
            } else {
                format!("{:02}", i + 1)
            };
            widgets::text_centered(
                ui,
                p,
                &num,
                FontId::new(10.0, theme::mono_bold()),
                t.invert_fg,
                badge.center(),
            );
            x += dir * (badge_w + gap);

            // quick actions at the trailing edge: [GITHUB] [OPEN]
            let open_x = if fa { rect.left() + pad } else { rect.right() - pad - open_w };
            let gh_x = if fa {
                open_x + open_w + gap
            } else {
                open_x - gap - gh_w
            };

            // title (strong) + one-line brief, beside the badge.
            // `x` now sits just past the badge on the reading side; the
            // trailing columns (category, github, open) sit at the far end.
            let title = if fa { m.title_fa } else { m.title };
            let blurb = if fa { m.blurb_fa } else { m.blurb_en };
            let trailing = cat_w + gh_w + open_w + 3.0 * gap;
            let text_left = if fa {
                rect.left() + pad + trailing
            } else {
                x
            };
            let text_right = if fa {
                x
            } else {
                rect.right() - pad - trailing
            };
            let text_w = (text_right - text_left).max(120.0);
            let title_font = if fa {
                FontId::new(12.5, theme::vazir_bold())
            } else {
                FontId::new(12.5, theme::semibold())
            };
            let title_g = bidi::galley_text(
                ui,
                title,
                fa,
                title_font,
                t.ink_strong,
                0.0,
                if fa { Align::Right } else { Align::Left },
            );
            p.galley(
                Pos2::new(
                    if fa { text_right - title_g.size().x } else { text_left },
                    rect.min.y + 10.0,
                ),
                title_g,
                Color32::WHITE,
            );
            // truncate the brief to one visual line
            let blurb_line = truncate_to_width(ui, blurb, text_w, FontId::new(10.0, theme::medium()));
            let blurb_g = bidi::galley_text(
                ui,
                &blurb_line,
                fa,
                FontId::new(10.0, theme::medium()),
                t.muted,
                0.0,
                if fa { Align::Right } else { Align::Left },
            );
            p.galley(
                Pos2::new(
                    if fa {
                        text_right - blurb_g.size().x
                    } else {
                        text_left
                    },
                    rect.min.y + 33.0,
                ),
                blurb_g,
                Color32::WHITE,
            );

            // category chip
            let (_, _, cat_fa) = *data::TABS
                .iter()
                .find(|tab| tab.0 == m.cat)
                .unwrap_or(&data::TABS[0]);
            let cat_label = if fa { cat_fa } else { m.cat };
            let cat_rect = Rect::from_center_size(
                Pos2::new(
                    if fa {
                        text_left - gap - cat_w / 2.0
                    } else {
                        text_right + gap + cat_w / 2.0
                    },
                    rect.center().y,
                ),
                vec2(cat_w, 24.0),
            );
            p.rect_filled(cat_rect, CornerRadius::same(6), theme::tint(accent, 0.16));
            p.rect_stroke(
                cat_rect,
                CornerRadius::same(6),
                Stroke::new(1.0, theme::tint(accent, 0.5)),
                egui::StrokeKind::Inside,
            );
            widgets::text_centered(
                ui,
                p,
                cat_label,
                if fa {
                    FontId::new(9.5, theme::vazir_bold())
                } else {
                    FontId::new(9.0, theme::mono_bold())
                },
                accent,
                cat_rect.center(),
            );

            // GITHUB mini link
            let gh_label = "GITHUB ↗";
            let gh_rect =
                Rect::from_min_size(Pos2::new(gh_x, rect.center().y - 12.0), vec2(gh_w, 24.0));
            let gh_hover = ptr_over(ui, gh_rect);
            let gh_g = bidi::galley_text(
                ui,
                gh_label,
                false,
                FontId::new(8.5, theme::mono_bold()),
                if gh_hover { accent } else { t.faint },
                0.0,
                Align::Center,
            );
            p.galley(
                Pos2::new(
                    gh_rect.center().x - gh_g.size().x / 2.0,
                    gh_rect.center().y - gh_g.size().y / 2.0,
                ),
                gh_g,
                Color32::WHITE,
            );

            // OPEN button
            let open_rect =
                Rect::from_min_size(Pos2::new(open_x, rect.center().y - 15.0), vec2(open_w, 30.0));
            let open_hover = ptr_over(ui, open_rect) && !app.busy;
            p.rect_filled(
                open_rect,
                theme::R_BTN,
                if open_hover {
                    theme::tint(accent, 0.20)
                } else {
                    theme::tint(t.ink, 0.06)
                },
            );
            p.rect_stroke(
                open_rect,
                theme::R_BTN,
                Stroke::new(
                    1.0,
                    theme::tint(accent, if open_hover { 0.8 } else { 0.55 }),
                ),
                egui::StrokeKind::Inside,
            );
            let open_label = app.lang.s("open");
            widgets::text_centered(
                ui,
                p,
                open_label,
                if fa {
                    FontId::new(10.5, theme::vazir_bold())
                } else {
                    FontId::new(10.0, theme::mono_bold())
                },
                t.ink,
                open_rect.center(),
            );

            // interactions
            let row_resp = ui.interact(
                rect,
                ui.id().with(("lrow", i)),
                if app.busy { Sense::hover() } else { Sense::click() },
            );
            if !app.busy
                && row_resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
            {
                app.select_module(i, ctx);
            }
            let gh_resp = ui.interact(gh_rect, ui.id().with(("lrow_gh", i)), Sense::click());
            if gh_resp
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                app.open_link(m.url);
            }
            let open_resp = ui.interact(open_rect, ui.id().with(("lrow_open", i)), Sense::click());
            if !app.busy
                && open_resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
            {
                app.select_module(i, ctx);
            }
        }
        ui.add_space(10.0);
    });
}

/// One-line truncation with an ellipsis for list rows.
fn truncate_to_width(ui: &Ui, text: &str, max_w: f32, font: FontId) -> String {
    let painter = ui.painter().clone();
    let width = |s: &str| {
        painter
            .layout_no_wrap(s.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
    };
    if width(text) <= max_w {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut lo = 0usize;
    let mut hi = chars.len();
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let candidate: String = chars[..mid].iter().collect::<String>() + "…";
        if width(&candidate) <= max_w {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let mut out: String = chars[..lo].iter().collect();
    out.push('…');
    out
}

// ============================================================== grid
fn grid_view(ui: &mut Ui, app: &mut PolarisApp, ctx: &egui::Context) {
    let t = Tokens::get();
    let idxs = app.visible_modules();
    if idxs.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(70.0);
            let g = bidi::galley_text(
                ui,
                app.lang.s("modules"),
                app.lang.rtl(),
                FontId::new(18.0, theme::semibold()),
                t.muted,
                0.0,
                Align::Center,
            );
            bidi::put(ui, g, false);
            ui.add_space(8.0);
            let g = bidi::galley_text(
                ui,
                app.lang.s("no_match"),
                app.lang.rtl(),
                FontId::new(12.0, theme::medium()),
                t.faint,
                0.0,
                Align::Center,
            );
            bidi::put(ui, g, false);
        });
        return;
    }
    let width = ui.available_width().max(1.0);
    let gap = 10.0;
    let cols = ((width / theme::CARD_MIN_W).floor() as usize).clamp(1, 4);
    let card_w = ((width - (cols - 1) as f32 * gap) / cols as f32).min(440.0);

    ScrollArea::vertical().id_salt("pkp_grid").show(ui, |ui| {
        ui.add_space(8.0);
        for chunk in idxs.chunks(cols) {
            let mut row_h: f32 = 0.0;
            for &i in chunk {
                row_h = row_h.max(card_height(ui, app, i, card_w));
            }
            let n = chunk.len() as f32;
            let total_w = card_w * n + gap * (n - 1.0);
            let x0 = ui.cursor().left() + ((width - total_w) / 2.0).max(0.0);
            let (row_rect, _) = ui.allocate_exact_size(vec2(width, row_h), Sense::hover());
            for (column, &i) in chunk.iter().enumerate() {
                let x = x0 + column as f32 * (card_w + gap);
                let card_rect =
                    Rect::from_min_size(Pos2::new(x, row_rect.min.y), vec2(card_w, row_h));
                draw_card(ui, app, ctx, card_rect, i);
            }
            ui.add_space(gap);
        }
        ui.add_space(10.0);
    });
}

/// Card height from its (measured) content.
fn card_height(ui: &mut Ui, app: &PolarisApp, i: usize, card_w: f32) -> f32 {
    let m = &data::MODULES[i];
    let fa = app.lang.fa();
    let pad = 14.0;
    let text_w = card_w - 2.0 * pad;
    let title = if fa { m.title_fa } else { m.title };
    let blurb = if fa { m.blurb_fa } else { m.blurb_en };
    let title_font = if fa {
        FontId::new(13.0, theme::vazir_bold())
    } else {
        FontId::new(13.0, theme::semibold())
    };
    let title_g = bidi::galley_text(
        ui,
        title,
        fa,
        title_font,
        Color32::WHITE,
        text_w,
        Align::Center,
    );
    let blurb_g = bidi::galley_text(
        ui,
        blurb,
        fa,
        FontId::new(10.5, theme::medium()),
        Color32::WHITE,
        text_w,
        Align::Center,
    );
    let mid = title_g.size().y + 6.0 + blurb_g.size().y;
    pad + 20.0 + 10.0 + mid + 10.0 + 16.0 + 10.0 + 32.0 + pad
}

/// One module card — everything centered: badge, title, blurb, GitHub
/// link and the OPEN button.
fn draw_card(ui: &mut Ui, app: &mut PolarisApp, ctx: &egui::Context, crect: Rect, i: usize) {
    let t = Tokens::get();
    let m = &data::MODULES[i];
    let accent = Tokens::accent_of(m.accent);
    let fa = app.lang.fa();
    let p = ui.painter();
    let hovered = ptr_over(ui, crect) && !app.busy;

    // surface + border
    let fill = if hovered {
        mix(t.card, accent, 0.06)
    } else {
        t.card
    };
    p.rect_filled(crect, theme::R_CARD, fill);
    p.rect_stroke(
        crect,
        theme::R_CARD,
        if hovered {
            Stroke::new(1.0, mix(t.line_strong, accent, 0.6))
        } else {
            t.hairline()
        },
        egui::StrokeKind::Inside,
    );

    let pad = 14.0;
    // number badge (centered)
    let badge = Rect::from_center_size(
        Pos2::new(crect.center().x, crect.min.y + pad + 10.0),
        vec2(38.0, 20.0),
    );
    p.rect_filled(badge, CornerRadius::same(6), accent);
    let num = if fa {
        fa_num2(i + 1)
    } else {
        format!("{:02}", i + 1)
    };
    widgets::text_centered(
        ui,
        p,
        &num,
        FontId::new(10.0, theme::mono_bold()),
        t.invert_fg,
        badge.center(),
    );

    // title + blurb (centered, wrapped)
    let text_w = crect.width() - 2.0 * pad;
    let title = if fa { m.title_fa } else { m.title };
    let blurb = if fa { m.blurb_fa } else { m.blurb_en };
    let title_font = if fa {
        FontId::new(13.0, theme::vazir_bold())
    } else {
        FontId::new(13.0, theme::semibold())
    };
    let title_g = bidi::galley_text(
        ui,
        title,
        fa,
        title_font,
        t.ink_strong,
        text_w,
        Align::Center,
    );
    let blurb_g = bidi::galley_text(
        ui,
        blurb,
        fa,
        FontId::new(10.5, theme::medium()),
        t.muted,
        text_w,
        Align::Center,
    );
    let mut y = badge.bottom() + 10.0;
    p.galley(
        Pos2::new(crect.center().x - title_g.size().x / 2.0, y),
        title_g.clone(),
        Color32::WHITE,
    );
    y += title_g.size().y + 6.0;
    p.galley(
        Pos2::new(crect.center().x - blurb_g.size().x / 2.0, y),
        blurb_g.clone(),
        Color32::WHITE,
    );
    y += blurb_g.size().y + 10.0;

    // GitHub link (centered, small)
    let gh_label = "GITHUB ↗";
    let gh_g = bidi::galley_text(
        ui,
        gh_label,
        false,
        FontId::new(8.5, theme::mono_bold()),
        t.faint,
        0.0,
        Align::Center,
    );
    let gh_rect = Rect::from_center_size(
        Pos2::new(crect.center().x, y + 8.0),
        vec2(gh_g.size().x + 14.0, 16.0),
    );
    let gh_hover = ptr_over(ui, gh_rect);
    let gh_ink = if gh_hover { accent } else { t.faint };
    let gh_show = if gh_hover {
        bidi::galley_text(
            ui,
            gh_label,
            false,
            FontId::new(8.5, theme::mono_bold()),
            gh_ink,
            0.0,
            Align::Center,
        )
    } else {
        gh_g
    };
    p.galley(
        Pos2::new(
            gh_rect.center().x - gh_show.size().x / 2.0,
            gh_rect.center().y - gh_show.size().y / 2.0,
        ),
        gh_show,
        Color32::WHITE,
    );

    // OPEN button (full width, anchored to the card bottom)
    let open_h = 32.0;
    let open_rect = Rect::from_min_size(
        Pos2::new(crect.min.x + pad, crect.max.y - pad - open_h),
        vec2(crect.width() - 2.0 * pad, open_h),
    );
    let open_hover = ptr_over(ui, open_rect) && !app.busy;
    let open_fill = if open_hover {
        theme::tint(accent, 0.20)
    } else {
        theme::tint(t.ink, 0.06)
    };
    p.rect_filled(open_rect, theme::R_BTN, open_fill);
    p.rect_stroke(
        open_rect,
        theme::R_BTN,
        Stroke::new(
            1.0,
            theme::tint(accent, if open_hover { 0.8 } else { 0.55 }),
        ),
        egui::StrokeKind::Inside,
    );
    let open_label = app.lang.s("open");
    let open_font = if fa {
        FontId::new(11.0, theme::vazir_bold())
    } else {
        FontId::new(10.5, theme::mono_bold())
    };
    widgets::text_centered(ui, p, open_label, open_font, t.ink, open_rect.center());

    // interactions (added after the visuals so they win hit-testing)
    let card_resp = ui.interact(
        crect,
        ui.id().with(("card", i)),
        if app.busy {
            Sense::hover()
        } else {
            Sense::click()
        },
    );
    if !app.busy
        && card_resp
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
    {
        app.select_module(i, ctx);
    }
    let gh_resp = ui.interact(gh_rect, ui.id().with(("card_gh", i)), Sense::click());
    if gh_resp
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        app.open_link(m.url);
    }
    let open_resp = ui.interact(open_rect, ui.id().with(("card_open", i)), Sense::click());
    if !app.busy
        && open_resp
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
    {
        app.select_module(i, ctx);
    }
}

/// FA digits 0-9.
fn fa_num(n: usize) -> String {
    n.to_string()
        .chars()
        .map(|c| match c {
            '0' => '۰',
            '1' => '۱',
            '2' => '۲',
            '3' => '۳',
            '4' => '۴',
            '5' => '۵',
            '6' => '۶',
            '7' => '۷',
            '8' => '۸',
            '9' => '۹',
            c => c,
        })
        .collect()
}

/// FA digits, zero-padded to two places (۰۱..۱۲).
fn fa_num2(n: usize) -> String {
    if n < 10 {
        format!("۰{}", fa_num(n))
    } else {
        fa_num(n)
    }
}

// ============================================================== detail
pub fn detail(ui: &mut Ui, app: &mut PolarisApp, i: usize, ctx: &egui::Context) {
    let t = Tokens::get();
    let m = &data::MODULES[i];
    let accent = Tokens::accent_of(m.accent);
    let fa = app.lang.fa();

    // fixed 3px accent strip
    let (strip, _) = ui.allocate_exact_size(vec2(ui.available_width(), 3.0), Sense::hover());
    ui.painter().rect_filled(strip, CornerRadius::ZERO, accent);

    ScrollArea::vertical()
        .id_salt(format!("pkp_detail_{}", m.mid))
        .show(ui, |ui| {
            // ---- header row (mirrored in FA): [back][badge][title] … [hud][GITHUB][lang]
            let title = if fa { m.title_fa } else { m.title };
            let title_g = bidi::galley_text(
                ui,
                title,
                fa,
                if fa {
                    FontId::new(16.0, theme::vazir_bold())
                } else {
                    FontId::new(16.0, theme::semibold())
                },
                t.ink_strong,
                0.0,
                Align::Left,
            );
            let num = if fa {
                fa_num2(i + 1)
            } else {
                format!("{:02}", i + 1)
            };
            let header = |ui: &mut Ui| {
                // back button
                let (brect, bresp) = ui.allocate_exact_size(vec2(130.0, 32.0), Sense::click());
                let p = ui.painter();
                if bresp.hovered() {
                    p.rect_filled(brect, theme::R_BTN, theme::tint(accent, 0.08));
                }
                p.rect_stroke(
                    brect,
                    theme::R_BTN,
                    Stroke::new(1.0, theme::tint(accent, 0.65)),
                    egui::StrokeKind::Inside,
                );
                widgets::text_centered(
                    ui,
                    p,
                    app.lang.s("back_all"),
                    if fa {
                        FontId::new(10.5, theme::vazir_bold())
                    } else {
                        FontId::new(9.5, theme::mono_bold())
                    },
                    accent,
                    brect.center(),
                );
                if bresp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    app.show_grid(ctx);
                }
                ui.add_space(10.0);
                // number badge
                let (badge, _) = ui.allocate_exact_size(vec2(34.0, 22.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(badge, CornerRadius::same(6), accent);
                widgets::text_centered(
                    ui,
                    p,
                    &num,
                    FontId::new(11.0, theme::mono_bold()),
                    t.invert_fg,
                    badge.center(),
                );
                ui.add_space(10.0);
                // title (already measured; anchored to the badge side)
                let (trect, _) =
                    ui.allocate_exact_size(title_g.size() + vec2(4.0, 0.0), Sense::hover());
                let tpos = if fa {
                    Pos2::new(
                        trect.right() - title_g.size().x,
                        trect.center().y - title_g.size().y / 2.0,
                    )
                } else {
                    Pos2::new(trect.left(), trect.center().y - title_g.size().y / 2.0)
                };
                ui.painter().galley(tpos, title_g.clone(), Color32::WHITE);
                // trailing cluster at the far edge
                if fa {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        detail_cluster(ui, app, m.url);
                    });
                } else {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        detail_cluster(ui, app, m.url);
                    });
                }
            };
            if fa {
                // NOTE: `with_layout` inside a ScrollArea would consume the
                // full (unbounded) height; allocate an explicit row height.
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), 36.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    header,
                );
            } else {
                ui.horizontal(header);
            }
            ui.add_space(4.0);

            // ---- description (native blurb in FA, official desc in EN)
            let desc = if fa { m.blurb_fa } else { m.desc_official };
            let w = ui.available_width();
            let g = bidi::galley_text(
                ui,
                desc,
                fa,
                FontId::new(
                    12.5,
                    if fa {
                        theme::vazir()
                    } else {
                        egui::FontFamily::Proportional
                    },
                ),
                t.muted,
                w - 4.0,
                if fa { Align::Right } else { Align::Left },
            );
            bidi::put(ui, g, fa);
            ui.add_space(6.0);
            // meta: category · tech
            let cat_fa_label = data::TABS
                .iter()
                .find(|tab| tab.0 == m.cat)
                .map(|tab| tab.2)
                .unwrap_or(m.cat);
            let meta = if fa {
                format!("{} · {}", cat_fa_label, m.tech_fa)
            } else {
                format!("{} · {}", m.cat, m.tech)
            };
            let g = bidi::galley_text(
                ui,
                &meta,
                fa,
                FontId::new(10.0, theme::mono()),
                accent,
                w - 4.0,
                if fa { Align::Right } else { Align::Left },
            );
            bidi::put(ui, g, fa);

            // ---- ACTIONS
            section_hr(ui, app, "actions");

            // Action buttons keep their ENGLISH labels in both languages
            // (product rule). Layout: the two primary contrasting actions
            // sit side-by-side in a 2-column grid; every further action
            // gets its own full-width row beneath - every row shares the
            // same margins, identical heights and symmetric spacing.
            let acts_all = m.acts;
            let busy_here = app.busy && app.busy_module == Some(i);
            let pair_count = acts_all.len().min(2);
            if pair_count == 2 {
                let gap = 8.0;
                let button_w = ((ui.available_width() - gap) / 2.0).max(96.0);
                ui.horizontal(|ui| {
                    for (k, action) in acts_all.iter().take(2).enumerate() {
                        let dim = app.busy && !busy_here;
                        let (fill, fg) = if dim {
                            (
                                mix(t.invert_bg, t.bg, 0.45),
                                mix(t.invert_fg, t.bg, 0.45),
                            )
                        } else {
                            (Color32::TRANSPARENT, Color32::TRANSPARENT)
                        };
                        let resp = widgets::button_sized(
                            ui,
                            action.label,
                            widgets::Kind::Secondary,
                            vec2(button_w, 42.0),
                            fill,
                            fg,
                        );
                        if resp.clicked() && !app.busy {
                            app.request_run(i, k, ctx);
                        }
                        if k == 0 {
                            ui.add_space(gap);
                        }
                    }
                });
            } else if pair_count == 1 {
                let action = &acts_all[0];
                let (prect, presp) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 42.0), Sense::click());
                let p = ui.painter();
                let dim = app.busy && !busy_here;
                let p_fill = if dim {
                    mix(t.invert_bg, t.bg, 0.45)
                } else if presp.hovered() && !app.busy {
                    mix(t.invert_bg, t.sunken, 0.12)
                } else {
                    t.invert_bg
                };
                p.rect_filled(prect, theme::R_BTN, p_fill);
                let lab_font = if fa {
                    FontId::new(13.0, theme::vazir_bold())
                } else {
                    FontId::new(12.0, theme::mono_bold())
                };
                widgets::text_centered(ui, p, action.label, lab_font, t.invert_fg, prect.center());
                if presp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                    && !app.busy
                {
                    app.request_run(i, 0, ctx);
                }
            }
            // remaining actions: one centered row each, symmetric margins,
            // identical heights to the pair grid
            for (k, action) in acts_all.iter().enumerate().skip(2) {
                let action = *action;
                ui.add_space(8.0);
                let avail_w = ui.available_width();
                let button_w = (avail_w * 0.5).max(200.0);
                let (strip, _) =
                    ui.allocate_exact_size(vec2(avail_w, 42.0), Sense::hover());
                let rect = Rect::from_min_size(
                    Pos2::new(strip.left() + (avail_w - button_w) / 2.0, strip.min.y),
                    vec2(button_w, 42.0),
                );
                let resp = ui.interact(rect, ui.id().with(("act", k)), Sense::click());
                let p = ui.painter();
                let dim = app.busy && !busy_here;
                let fill = if dim {
                    t.sunken
                } else if resp.hovered() && !app.busy {
                    mix(t.card, accent, 0.10)
                } else {
                    t.card
                };
                p.rect_filled(rect, theme::R_BTN, fill);
                p.rect_stroke(
                    rect,
                    theme::R_BTN,
                    Stroke::new(1.0, mix(t.line_strong, accent, 0.35)),
                    egui::StrokeKind::Inside,
                );
                let lab_font = if fa {
                    FontId::new(12.0, theme::vazir_bold())
                } else {
                    FontId::new(11.0, theme::mono_bold())
                };
                widgets::text_centered(ui, p, action.label, lab_font, t.ink, rect.center());
                if resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                    && !app.busy
                {
                    app.request_run(i, k, ctx);
                }
            }

            // ---- live progress + STOP, mounted directly underneath the
            // clicked action while the task runs
            if busy_here {
                ui.add_space(10.0);
                let fraction = app.shared.progress_fraction();
                widgets::linear_progress(
                    ui,
                    ui.available_width(),
                    6.0,
                    fraction,
                    accent,
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let elapsed = self_elapsed(app);
                    let status = match fraction {
                        Some(f) => {
                            let pct = (f * 100.0).round() as i32;
                            format!("{}%  ·  {}s", pct, elapsed)
                        }
                        None => format!("{}  ·  {}s", app.lang.s("indeterminate"), elapsed),
                    };
                    let g = bidi::galley_text(
                        ui,
                        &status,
                        false,
                        FontId::new(10.5, theme::mono()),
                        theme::AMBER,
                        0.0,
                        if fa { Align::Right } else { Align::Left },
                    );
                    bidi::put(ui, g, fa);
                    // STOP / CANCEL at the trailing edge
                    let stop_w = 150.0;
                    let rect = Rect::from_min_size(
                        Pos2::new(
                            if fa {
                                ui.cursor().left()
                            } else {
                                ui.cursor().left() + ui.available_width() - stop_w
                            },
                            ui.cursor().min.y,
                        ),
                        vec2(stop_w, 30.0),
                    );
                    let resp = ui.interact(rect, ui.id().with("stop_inline"), Sense::click());
                    let p = ui.painter();
                    p.rect_filled(
                        rect,
                        theme::R_BTN,
                        if resp.hovered() {
                            theme::RED
                        } else {
                            theme::tint(theme::RED, 0.30)
                        },
                    );
                    p.rect_stroke(
                        rect,
                        theme::R_BTN,
                        Stroke::new(1.0, theme::RED),
                        egui::StrokeKind::Inside,
                    );
                    widgets::text_centered(
                        ui,
                        p,
                        app.lang.s("stop_action"),
                        if fa {
                            FontId::new(11.0, theme::vazir_bold())
                        } else {
                            FontId::new(10.0, theme::mono_bold())
                        },
                        Color32::WHITE,
                        rect.center(),
                    );
                    if resp
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        app.stop_current();
                    }
                });
            }
            ui.add_space(4.0);

            // ---- TARGET (only modules that need one)
            if m.acts.iter().any(|a| a.need_folder) {
                section_hr(ui, app, "target");
                let row = |ui: &mut Ui| {
                    widgets::hud_label(ui, app.lang.s("target"), t.muted);
                    ui.add_space(8.0);
                    let path = if app.folder.is_empty() {
                        app.lang.s("notarget").to_string()
                    } else {
                        app.folder.clone()
                    };
                    let g = bidi::galley_text(
                        ui,
                        &path,
                        false,
                        FontId::new(11.0, theme::mono()),
                        if app.folder.is_empty() {
                            t.faint
                        } else {
                            theme::GREEN
                        },
                        0.0,
                        Align::Left,
                    );
                    let (prect, _) = ui.allocate_exact_size(
                        vec2((ui.available_width() * 0.5).max(200.0), 30.0),
                        Sense::hover(),
                    );
                    let p = ui.painter();
                    p.rect_filled(prect, theme::R_BTN, t.bg);
                    p.rect_stroke(prect, theme::R_BTN, t.hairline(), egui::StrokeKind::Inside);
                    p.galley(
                        Pos2::new(prect.left() + 10.0, prect.center().y - g.size().y / 2.0),
                        g,
                        Color32::WHITE,
                    );
                    ui.add_space(8.0);
                    if widgets::button_sized(
                        ui,
                        app.lang.s("browse"),
                        widgets::Kind::Secondary,
                        vec2(110.0, 30.0),
                        accent,
                        accent,
                    )
                    .clicked()
                        && !app.busy
                    {
                        app.pick_folder();
                    }
                };
                if fa {
                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width(), 30.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        row,
                    );
                } else {
                    ui.horizontal(row);
                }
            }

            // ---- README
            section_hr(ui, app, "readme");
            Frame::new()
                .fill(theme::CONBG)
                .stroke(t.hairline())
                .corner_radius(theme::R_FRAME)
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    let blocks = rd_lang(app.readme_lang, m.mid);
                    for b in blocks {
                        render_block(ui, b, app, ctx);
                    }
                });
            ui.add_space(14.0);
        });
}

/// The far-edge cluster of the detail sub-header. Exactly ONE tool-specific
/// GitHub button lives on this page (right side); the EN/FA readme toggle
/// sits on its left. The caller wraps it in the correct layout direction.
fn detail_cluster(ui: &mut Ui, app: &mut PolarisApp, url: &'static str) {
    ui.add_space(14.0);
    // Emitted in this order so the GITHUB button lands on the OUTER edge in
    // both directions (right in EN, left in FA) and the EN/FA readme toggle
    // sits on its inner side, next to the title.
    if widgets::button_sized(
        ui,
        "GITHUB ↗",
        widgets::Kind::Secondary,
        vec2(104.0, 30.0),
        Color32::TRANSPARENT,
        Color32::TRANSPARENT,
    )
    .clicked()
    {
        app.open_link(url);
    }
    ui.add_space(8.0);
    let rl = app.readme_lang;
    let rl_label = if rl == Lang::Fa { "EN" } else { "فارسی" };
    if widgets::button_sized(
        ui,
        rl_label,
        widgets::Kind::Secondary,
        vec2(72.0, 30.0),
        Color32::TRANSPARENT,
        Color32::TRANSPARENT,
    )
    .clicked()
    {
        app.toggle_readme();
    }
}

/// A section header: small caps label + hairline separator (gives every
/// module — including SYS and AUDIT — the same framed structure).
fn section_hr(ui: &mut Ui, app: &PolarisApp, key: &str) {
    let t = Tokens::get();
    let fa = app.lang.fa();
    ui.add_space(16.0);
    let galley = bidi::galley_text(
        ui,
        app.lang.s(key),
        fa,
        FontId::new(10.0, theme::mono()),
        t.faint,
        0.0,
        Align::Left,
    );
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
    let pos = if fa {
        Pos2::new(rect.right() - galley.size().x, rect.min.y)
    } else {
        Pos2::new(rect.left(), rect.min.y)
    };
    ui.painter().galley(pos, galley.clone(), Color32::WHITE);
    ui.painter()
        .hline(rect.x_range(), rect.bottom() - 1.0, t.hairline());
    ui.add_space(10.0);
}

// ============================================================== readme
/// Renders one pre-parsed readme block through the bidi engine.
fn render_block(ui: &mut Ui, b: &Block, app: &mut PolarisApp, _ctx: &egui::Context) {
    let fa = app.readme_lang.fa();
    match b {
        Block::H1(runs) => heading(ui, runs, RD_H1, 15.5, fa),
        Block::H2(runs) => heading(ui, runs, RD_H2, 13.5, fa),
        Block::H3(runs) => heading(ui, runs, RD_H3, 12.0, fa),
        Block::P(runs) => paragraph(ui, app, runs, fa, None, RD_INK),
        Block::Quote(runs) => {
            ui.add_space(4.0);
            paragraph(ui, app, runs, fa, Some("\" \" "), RD_QUOTE);
            ui.add_space(2.0);
        }
        Block::Ul(items) => {
            for it in *items {
                paragraph(ui, app, it, fa, Some("•  "), RD_INK);
            }
            ui.add_space(2.0);
        }
        Block::Ol(items) => {
            for (n, it) in items.iter().enumerate() {
                let prefix = if fa {
                    format!("{}.  ", fa_num(n + 1))
                } else {
                    format!("{}.  ", n + 1)
                };
                paragraph(ui, app, it, fa, Some(&prefix), RD_INK);
            }
            ui.add_space(2.0);
        }
        Block::Code(code) => {
            ui.add_space(4.0);
            Frame::new()
                .fill(RD_BLOCK_BG)
                .corner_radius(theme::R_FRAME)
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    let g = bidi::galley_text(
                        ui,
                        code,
                        false,
                        FontId::new(11.0, theme::mono()),
                        RD_BLOCK_INK,
                        0.0,
                        Align::Left,
                    );
                    bidi::put(ui, g, false);
                });
            ui.add_space(4.0);
        }
        Block::Hr => {
            ui.add_space(6.0);
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
            ui.painter()
                .hline(rect.x_range(), rect.center().y, Stroke::new(1.0, RD_HR));
            ui.add_space(6.0);
        }
        Block::Table(rows) => render_table(ui, rows, fa),
    }
}

fn heading(ui: &mut Ui, runs: &[Run], color: Color32, size: f32, fa: bool) {
    ui.add_space(10.0);
    let text: String = runs.iter().map(|r| r.0).collect();
    let font = if fa {
        FontId::new(size, theme::vazir_bold())
    } else {
        FontId::new(size, theme::semibold())
    };
    let g = bidi::galley_text(
        ui,
        &text,
        fa,
        font,
        color,
        0.0,
        if fa { Align::Right } else { Align::Left },
    );
    bidi::put(ui, g, fa);
}

/// Map data runs to bidi styled runs (readme palette).
fn styled_runs(runs: &[Run], fa: bool, base: Color32) -> Vec<Styled<'_>> {
    runs.iter()
        .map(|r| {
            let (font, color, size) = match r.1 {
                Style::Norm => (
                    if fa {
                        theme::vazir()
                    } else {
                        egui::FontFamily::Proportional
                    },
                    base,
                    11.5,
                ),
                Style::Bold => (
                    if fa {
                        theme::vazir_bold()
                    } else {
                        theme::semibold()
                    },
                    RD_BOLD,
                    11.5,
                ),
                Style::Code => (theme::mono(), RD_CODE_INK, 10.5),
                Style::Link => (theme::mono(), RD_LINK, 10.5),
            };
            let mut fmt = TextFormat::simple(FontId::new(size, font), color);
            if r.1 == Style::Code {
                fmt.background = RD_CODE_BG;
            }
            if r.1 == Style::Link {
                fmt.underline = Stroke::new(1.0, RD_LINK);
            }
            if r.1 == Style::Link && !r.2.is_empty() {
                Styled::linked(r.0, fmt, r.2)
            } else {
                Styled::new(r.0, fmt)
            }
        })
        .collect()
}

/// A wrapped paragraph with working inline links. `prefix` (bullet,
/// number) is folded into the first run so RTL orderings stay correct.
fn paragraph(
    ui: &mut Ui,
    app: &mut PolarisApp,
    runs: &[Run],
    fa: bool,
    prefix: Option<&str>,
    base: Color32,
) {
    let mut srs: Vec<Styled<'_>> = Vec::new();
    if let Some(pre) = prefix {
        let font = if fa {
            theme::vazir()
        } else {
            egui::FontFamily::Proportional
        };
        srs.push(Styled::new(
            pre,
            TextFormat::simple(FontId::new(11.5, font), base),
        ));
    }
    srs.extend(styled_runs(runs, fa, base));
    let w = ui.available_width();
    let (galley, links) =
        bidi::galley_styled(ui, &srs, fa, w, if fa { Align::Right } else { Align::Left });
    let size = galley.size();
    let (rect, _) = ui.allocate_exact_size(size + vec2(0.0, 3.0), Sense::hover());
    let pos = if fa {
        Pos2::new(rect.right() - size.x, rect.min.y)
    } else {
        rect.min
    };
    ui.painter().galley(pos, galley.clone(), Color32::WHITE);
    if !links.is_empty() {
        for (section, url) in &links {
            if let Some(sec) = galley.job.sections.get(*section) {
                let (b0, b1) = (sec.byte_range.start.0, sec.byte_range.end.0);
                if let Some(r) = bidi::section_rect(&galley, b0, b1, pos) {
                    let resp = ui.interact(
                        r,
                        ui.id().with(("rdlink", section, url.as_str())),
                        Sense::click(),
                    );
                    if resp
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        app.open_link(url);
                    }
                }
            }
        }
    }
}

/// The readme file table. In FA the column order mirrors (first column
/// at the right) and cells right-align.
fn render_table(ui: &mut Ui, rows: &[&[&[Run]]], fa: bool) {
    let Some(first) = rows.first() else {
        return;
    };
    let ncols = first.len();
    let w = ui.available_width();
    let col_w = w / ncols as f32;
    ui.add_space(6.0);
    for (ri, row) in rows.iter().enumerate() {
        // measure cells first (wrapped) to get the row height
        let mut galleys = Vec::with_capacity(row.len());
        let mut row_h: f32 = if ri == 0 { 26.0 } else { 30.0 };
        for cell in row.iter() {
            let text: String = cell.iter().map(|r| r.0).collect();
            let font = if ri == 0 {
                FontId::new(10.0, theme::mono_bold())
            } else {
                FontId::new(10.0, theme::mono())
            };
            let ink = if ri == 0 { RD_TH_INK } else { RD_TD_INK };
            let g = bidi::galley_text(
                ui,
                &text,
                fa,
                font,
                ink,
                (col_w - 18.0).max(40.0),
                if fa { Align::Right } else { Align::Left },
            );
            row_h = row_h.max(g.size().y + 12.0);
            galleys.push(g);
        }
        let (rrect, _) = ui.allocate_exact_size(vec2(w, row_h), Sense::hover());
        let p = ui.painter();
        for (ci, g) in galleys.iter().enumerate() {
            let visual_ci = if fa { ncols - 1 - ci } else { ci };
            let cell_rect = Rect::from_min_size(
                Pos2::new(
                    rrect.min.x + visual_ci as f32 * col_w + 1.0,
                    rrect.min.y + 1.0,
                ),
                vec2(col_w - 2.0, row_h - 2.0),
            );
            let bg = if ri == 0 {
                RD_TH_BG
            } else if ri % 2 == 0 {
                RD_TD_BG
            } else {
                RD_TD_BG2
            };
            p.rect_filled(cell_rect, CornerRadius::same(5), bg);
            let pos = if fa {
                Pos2::new(
                    cell_rect.right() - 9.0 - g.size().x,
                    cell_rect.center().y - g.size().y / 2.0,
                )
            } else {
                Pos2::new(
                    cell_rect.left() + 9.0,
                    cell_rect.center().y - g.size().y / 2.0,
                )
            };
            p.galley(pos, g.clone(), Color32::WHITE);
        }
    }
    ui.add_space(6.0);
}

// ============================================================== log
/// The universal LOG slide-over panel. Globally accessible: the header LOG
/// button opens it from EVERY view and sub-page (dashboard, any tool page,
/// while a task is running). It slides in from the trailing edge (right in
/// EN, left in FA), is non-blocking - the page beneath stays visible - and
/// the log content itself is ALWAYS English (product rule), LTR rows.
pub fn log_panel(ui: &mut Ui, app: &mut PolarisApp, ctx: &egui::Context) {
    if !app.show_log && app.log_slide <= 0.01 {
        return;
    }
    let t = Tokens::get();
    let fa = app.lang.fa();
    let screen = ui.ctx().input(|i| i.viewport_rect());
    let width = (screen.width() * 0.34).clamp(360.0, 480.0);
    let header_h = 52.0;

    // smooth slide animation (one animated value; cheap)
    let target = if app.show_log { 1.0 } else { 0.0 };
    let slide = ctx.animate_value_with_time(egui::Id::new("pkp_log_slide"), target, 0.16);
    app.log_slide = slide;
    if slide <= 0.01 {
        return;
    }
    let ease = slide * slide * (3.0 - 2.0 * slide);

    // panel rect: anchored to the trailing edge (right in EN, left in FA)
    let panel_w = width + 14.0;
    let y0 = screen.top() + header_h;
    let y1 = screen.bottom();
    let rect = if fa {
        Rect::from_min_max(
            Pos2::new(screen.left() - panel_w * (1.0 - ease), y0),
            Pos2::new(screen.left() + width - panel_w * (1.0 - ease), y1),
        )
    } else {
        Rect::from_min_max(
            Pos2::new(screen.right() - width + panel_w * (1.0 - ease), y0),
            Pos2::new(screen.right() + panel_w * (1.0 - ease), y1),
        )
    };

    let p = ui.painter_at(rect.expand(2.0));
    // shadow + surface
    let shadow_dir = if fa { 6.0 } else { -6.0 };
    p.rect_filled(
        rect.translate(vec2(shadow_dir, 0.0)),
        CornerRadius { nw: if fa { 10 } else { 0 }, ne: if fa { 0 } else { 10 }, sw: if fa { 10 } else { 0 }, se: if fa { 0 } else { 10 } },
        Color32::from_black_alpha(90),
    );
    p.rect_filled(
        rect,
        CornerRadius { nw: if fa { 10 } else { 0 }, ne: if fa { 0 } else { 10 }, sw: if fa { 10 } else { 0 }, se: if fa { 0 } else { 10 } },
        t.bg,
    );
    p.rect_stroke(
        rect,
        CornerRadius { nw: if fa { 10 } else { 0 }, ne: if fa { 0 } else { 10 }, sw: if fa { 10 } else { 0 }, se: if fa { 0 } else { 10 } },
        t.hairline_strong(),
        egui::StrokeKind::Inside,
    );

    // panel content: an absolute child ui anchored to the panel rect
    let mut panel_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    panel_ui.vertical(|ui| {
            // ---- header row: [LOG title] ... [close X]
            ui.horizontal(|ui| {
                ui.set_min_height(34.0);
                let g = bidi::galley_text(
                    ui,
                    app.lang.s("log_title"),
                    false,
                    FontId::new(13.0, theme::mono_bold()),
                    theme::LOGGREEN,
                    0.0,
                    Align::Left,
                );
                let (trect, _) =
                    ui.allocate_exact_size(g.size() + vec2(4.0, 0.0), Sense::hover());
                ui.painter().galley(
                    Pos2::new(trect.left(), trect.center().y - g.size().y / 2.0),
                    g,
                    Color32::WHITE,
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (xrect, xresp) =
                        ui.allocate_exact_size(vec2(28.0, 26.0), Sense::click());
                    let p = ui.painter();
                    if xresp.hovered() {
                        p.rect_filled(xrect, theme::R_BTN, Color32::from_white_alpha(14));
                    }
                    let c = xrect.center();
                    p.line_segment(
                        [egui::pos2(c.x - 4.0, c.y - 4.0), egui::pos2(c.x + 4.0, c.y + 4.0)],
                        Stroke::new(1.5, t.muted),
                    );
                    p.line_segment(
                        [egui::pos2(c.x - 4.0, c.y + 4.0), egui::pos2(c.x + 4.0, c.y - 4.0)],
                        Stroke::new(1.5, t.muted),
                    );
                    if xresp
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        app.toggle_log_view(ctx);
                    }
                });
            });
            ui.add_space(4.0);

            // ---- controls row: STOP (when busy) CLEAR EXPORT
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let export = widgets::button_sized(
                        ui,
                        app.lang.s("export"),
                        widgets::Kind::Tinted,
                        vec2(96.0, 30.0),
                        theme::GREEN,
                        Color32::from_rgb(0x03, 0x13, 0x0a),
                    );
                    if export.clicked() {
                        app.export_log();
                    }
                    ui.add_space(6.0);
                    let clear = widgets::button_sized(
                        ui,
                        app.lang.s("clear"),
                        widgets::Kind::Tinted,
                        vec2(84.0, 30.0),
                        theme::RED,
                        Color32::WHITE,
                    );
                    if clear.clicked() {
                        app.clear_log();
                    }
                    if app.busy {
                        ui.add_space(6.0);
                        let stop = widgets::button_sized(
                            ui,
                            app.lang.s("stop"),
                            widgets::Kind::Tinted,
                            vec2(84.0, 30.0),
                            theme::RED,
                            Color32::WHITE,
                        );
                        if stop.clicked() {
                            app.stop_current();
                        }
                    }
                });
            });
            ui.add_space(8.0);

            // ---- console frame with the English-only stream
            Frame::new()
                .fill(theme::CONBG)
                .stroke(t.hairline())
                .corner_radius(theme::R_FRAME)
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    log_stream(ui, app);
                });
    });
}

/// The bounded log stream renderer (English rows, LTR, colored markers).
fn log_stream(ui: &mut Ui, app: &mut PolarisApp) {
    let t = Tokens::get();
    if app.log.lines.is_empty() {
        let g = bidi::galley_text(
            ui,
            app.lang.s("empty"),
            false,
            FontId::new(11.0, theme::mono()),
            t.faint,
            0.0,
            Align::Left,
        );
        bidi::put(ui, g, false);
        return;
    }
    const LOG_ROW_HEIGHT: f32 = 18.0;
    let avail = ui.available_width();
    ScrollArea::vertical()
        .id_salt("pkp_log")
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show_rows(ui, LOG_ROW_HEIGHT, app.log.lines.len(), |ui, visible_rows| {
            for index in visible_rows {
                let entry = &app.log.lines[index];
                let line = format!("{} {}", entry.ts, entry.text);
                let color = log_line_color(&entry.text);
                let galley = log_line_galley(ui, &line, color, avail - 4.0);
                let (rect, _) =
                    ui.allocate_exact_size(vec2(avail, LOG_ROW_HEIGHT), Sense::hover());
                ui.painter().galley(
                    Pos2::new(rect.left(), rect.center().y - galley.size().y / 2.0),
                    galley,
                    Color32::WHITE,
                );
            }
        });
}

/// A log row galley, truncated with an ellipsis when it overflows.
fn log_line_galley(
    ui: &Ui,
    line: &str,
    color: Color32,
    avail: f32,
) -> std::sync::Arc<egui::Galley> {
    let font = FontId::new(11.0, theme::mono());
    let full = bidi::galley_text(ui, line, false, font.clone(), color, 0.0, Align::Left);
    if full.size().x <= avail {
        return full;
    }
    // trim ~10% at a time until it fits
    let mut chars: Vec<char> = line.chars().collect();
    loop {
        let cut = (chars.len() as f32 * 0.9) as usize;
        if cut < 12 {
            break;
        }
        chars.truncate(cut);
        let mut text: String = chars.iter().collect();
        text.push('…');
        let g = bidi::galley_text(ui, &text, false, font.clone(), color, 0.0, Align::Left);
        if g.size().x <= avail {
            return g;
        }
    }
    full
}

fn log_line_color(line: &str) -> Color32 {
    if line.starts_with("[ERR") {
        theme::RED
    } else if line.starts_with("[WARN") || line.starts_with("[ABORT") {
        theme::AMBER
    } else if line.starts_with("[ OK ") {
        theme::GREEN
    } else {
        theme::LOGGREEN
    }
}

// ============================================================== dialogs
/// The in-app confirm dialog (modal overlay).
pub fn confirm_dialog(ui: &mut Ui, ctx: &egui::Context, c: &Confirm, app: &mut PolarisApp) {
    widgets::scrim(ui, 1.0);
    let t = Tokens::get();
    let fa = app.lang.fa();
    let size = vec2(520.0, 240.0);
    let screen = ui.ctx().input(|i| i.viewport_rect());
    let origin = Pos2::new(
        screen.center().x - size.x / 2.0,
        screen.center().y - size.y / 2.0,
    );
    let rect = Rect::from_min_size(origin, size);
    let p = ui.painter();
    // soft shadow + card
    p.rect_filled(
        rect.translate(vec2(0.0, 3.0)),
        theme::R_FRAME,
        Color32::from_black_alpha(120),
    );
    p.rect_filled(rect, theme::R_FRAME, t.bg);
    p.rect_stroke(
        rect,
        theme::R_FRAME,
        t.hairline_strong(),
        egui::StrokeKind::Inside,
    );

    ui.put(rect, ConfirmContent { app, c, ctx, fa });
}

struct ConfirmContent<'a> {
    app: &'a mut PolarisApp,
    c: &'a Confirm,
    ctx: &'a egui::Context,
    fa: bool,
}

impl egui::Widget for ConfirmContent<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let t = Tokens::get();
        let fa = self.fa;
        ui.vertical(|ui| {
            ui.add_space(22.0);
            // centered title
            let g = bidi::galley_text(
                ui,
                self.app.lang.s("confirm_title"),
                fa,
                FontId::new(
                    theme::DIALOG_TITLE,
                    if fa {
                        theme::vazir_bold()
                    } else {
                        theme::semibold()
                    },
                ),
                t.ink_strong,
                0.0,
                Align::Center,
            );
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), g.size().y), Sense::hover());
            ui.painter().galley(
                Pos2::new(rect.center().x - g.size().x / 2.0, rect.min.y),
                g,
                Color32::WHITE,
            );
            ui.add_space(12.0);
            // message
            let msg = confirm_msg(self.c.key, self.app.lang);
            let g = bidi::galley_text(
                ui,
                msg,
                fa,
                FontId::new(
                    12.5,
                    if fa {
                        theme::vazir()
                    } else {
                        egui::FontFamily::Proportional
                    },
                ),
                t.muted,
                ui.available_width() - 44.0,
                if fa { Align::Right } else { Align::Left },
            );
            let size = g.size();
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), size.y), Sense::hover());
            let pos = if fa {
                Pos2::new(rect.right() - 22.0 - size.x, rect.min.y)
            } else {
                Pos2::new(rect.left() + 22.0, rect.min.y)
            };
            ui.painter().galley(pos, g, Color32::WHITE);
        });
        // buttons anchored to the bottom
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(20.0);
            let row = |ui: &mut Ui| {
                if fa {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::button(
                            ui,
                            self.app.lang.s("yes"),
                            widgets::Kind::Primary,
                            160.0,
                        )
                        .clicked()
                        {
                            self.app.confirm_answer(true, self.ctx);
                        }
                        ui.add_space(10.0);
                        if widgets::button(
                            ui,
                            self.app.lang.s("no"),
                            widgets::Kind::Secondary,
                            130.0,
                        )
                        .clicked()
                        {
                            self.app.confirm_answer(false, self.ctx);
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        if widgets::button(
                            ui,
                            self.app.lang.s("no"),
                            widgets::Kind::Secondary,
                            130.0,
                        )
                        .clicked()
                        {
                            self.app.confirm_answer(false, self.ctx);
                        }
                        ui.add_space(10.0);
                        if widgets::button(
                            ui,
                            self.app.lang.s("yes"),
                            widgets::Kind::Primary,
                            160.0,
                        )
                        .clicked()
                        {
                            self.app.confirm_answer(true, self.ctx);
                        }
                    });
                }
            };
            ui.horizontal(row);
        });
        ui.ctx().request_repaint();
        ui.interact(ui.max_rect(), ui.id().with("confirm_bg"), Sense::click())
    }
}

fn confirm_msg(key: &str, lang: Lang) -> &'static str {
    let (_k, en, fa) = data::CONFIRM
        .iter()
        .find(|(k, _, _)| *k == key)
        .unwrap_or(&("restore", "", ""));
    if lang == Lang::Fa {
        fa
    } else {
        en
    }
}
