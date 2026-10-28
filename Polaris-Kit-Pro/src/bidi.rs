//! A minimal Unicode-bidi layout engine for egui 0.36.
//!
//! egui 0.36 shapes each font-run with harfbuzz (which joins Arabic letters
//! and reverses RTL runs correctly) but it has NO bidi algorithm: runs are
//! painted in logical order, left-to-right. That means:
//!
//! * a pure-Persian run renders correctly (harfbuzz reverses it), but
//! * a Persian sentence whose spaces resolve to a different font splits
//!   into word-runs and comes out with the word order reversed, and
//! * Latin words embedded in a Persian run are reversed (`NLA` → `ALN`).
//!
//! This engine fixes that in three steps:
//!
//! 1. every character gets a direction — strong characters keep theirs,
//!    neutrals are resolved against their surrounding strongs (UAX #9
//!    N1/N2), so brackets and commas land on the correct side;
//! 2. the text is cut into word tokens, each carrying the direction of
//!    its characters and the format of its run;
//! 3. tokens are grouped into same-direction PHRASES (so "Windows
//!    Spotlight" keeps its word order) and the phrases are laid out
//!    visually — reversed for RTL lines, in order for LTR lines — with
//!    per-line word wrap done here (egui must not re-wrap).
//!
//! Each emitted section is a single-direction harfbuzz run, so shaping is
//! always correct, including bracket mirroring (harfbuzz mirrors brackets
//! in RTL runs). Rows of an RTL paragraph are right-aligned inside the
//! galley via `LayoutSection::leading_space`, so the caller can anchor
//! the galley by either edge. Persian digits run LTR: harfbuzz guesses
//! the run direction from the first character's script (Arabic → RTL),
//! so digit-led tokens are pre-reversed to cancel that guess out.

use egui::text::{ByteIndex, LayoutJob, LayoutSection, TextFormat};
use egui::{Color32, FontId, Galley, Pos2, Rect, Sense, Ui};
use std::sync::Arc;

// ----------------------------------------------------------------- classify

/// Directional class of a character (a tiny subset of UAX #9).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    /// Strong right-to-left (Arabic script, Hebrew, RTL marks).
    R,
    /// Strong left-to-right (Latin letters, ASCII digits).
    L,
    /// Neutral (spaces, punctuation, symbols, emoji).
    N,
}

fn class_of(c: char) -> Class {
    match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' => Class::L,
        '\u{00C0}'..='\u{024F}' => Class::L,
        // Arabic comma / semicolon are COMMON SEPARATORS (bidi class CS),
        // not strong RTL — classifying them neutral keeps Latin lists
        // like "Desktop، Downloads، Music" as one LTR phrase.
        '\u{060C}' | '\u{061B}' => Class::N,
        // Arabic-Indic and Extended (Persian) digits run LEFT-TO-RIGHT even
        // inside RTL text — classify them with the LTR side so they are
        // never reversed.
        '\u{0660}'..='\u{0669}' | '\u{06F0}'..='\u{06F9}' => Class::L,
        // Hebrew, Arabic, Syriac, Thaana and the Arabic extensions,
        // plus the presentation-form blocks (defensive) and RLM.
        '\u{0590}'..='\u{08FF}'
        | '\u{FB1D}'..='\u{FDFF}'
        | '\u{FE70}'..='\u{FEFF}'
        | '\u{200F}' => Class::R,
        _ => Class::N,
    }
}

/// Arabic-Indic and Extended (Persian) digits.
fn is_arabic_digit(c: char) -> bool {
    matches!(c, '\u{0660}'..='\u{0669}' | '\u{06F0}'..='\u{06F9}')
}

/// Base direction of a paragraph: the first strong character wins,
/// defaulting to LTR (UAX #9 P3 with an LTR default).
pub fn base_rtl(text: &str) -> bool {
    for c in text.chars() {
        match class_of(c) {
            Class::R => return true,
            Class::L => return false,
            Class::N => {}
        }
    }
    false
}

// ------------------------------------------------------------------ styled

/// A styled piece of text plus an optional link target (for hit-testing).
#[derive(Clone)]
pub struct Styled<'a> {
    pub text: &'a str,
    pub format: TextFormat,
    pub link: Option<&'a str>,
}

impl<'a> Styled<'a> {
    pub fn new(text: &'a str, format: TextFormat) -> Self {
        Self {
            text,
            format,
            link: None,
        }
    }

    pub fn linked(text: &'a str, format: TextFormat, url: &'a str) -> Self {
        Self {
            text,
            format,
            link: Some(url),
        }
    }
}

// ------------------------------------------------------------------ tokens

/// A wrappable word: the word plus its trailing spaces.
#[derive(Clone)]
struct Token {
    text: String,
    rtl: bool,
    format: TextFormat,
    /// link target, if the word came from a link run
    link: Option<String>,
    /// width including trailing spaces
    w: f32,
    /// width without trailing spaces
    w_trim: f32,
}

/// One flattened character with the direction it resolved to.
struct FlatChar {
    ch: char,
    rtl: bool,
    format: TextFormat,
    link: Option<String>,
}

/// Flatten runs to characters and resolve per-character directions:
/// strongs keep theirs; neutrals take the surrounding strongs' class
/// when both sides agree, otherwise the base direction (UAX #9 N1/N2).
fn flatten(runs: &[Styled<'_>], base: bool) -> Vec<FlatChar> {
    let mut chars: Vec<(char, usize, Option<&str>)> = Vec::new(); // (ch, run idx, link)
    let mut formats: Vec<TextFormat> = Vec::new();
    for (ri, r) in runs.iter().enumerate() {
        let _ = ri;
        formats.push(r.format.clone());
        for ch in r.text.chars() {
            chars.push((ch, formats.len() - 1, r.link));
        }
    }
    let n = chars.len();
    let mut dirs: Vec<Option<bool>> = Vec::with_capacity(n);
    for (ch, _, _) in &chars {
        dirs.push(match class_of(*ch) {
            Class::R => Some(true),
            Class::L => Some(false),
            Class::N => None,
        });
    }
    for i in 0..n {
        if dirs[i].is_some() {
            continue;
        }
        let prev = dirs[..i].iter().rev().flatten().next().copied();
        let next = dirs[i + 1..].iter().flatten().next().copied();
        dirs[i] = match (prev, next) {
            (Some(p), Some(q)) if p == q => Some(p),
            _ => Some(base),
        };
    }
    chars
        .into_iter()
        .zip(dirs)
        .map(|((ch, fi, link), dir)| FlatChar {
            ch,
            rtl: dir.unwrap_or(base),
            format: formats[fi].clone(),
            link: link.map(str::to_string),
        })
        .collect()
}

/// Cut the flattened characters into word tokens. A new token starts at a
/// direction change, a format change, or a new word after a space; spaces
/// always ride with the word before them.
fn tokenize(
    runs: &[Styled<'_>],
    base: bool,
    measure: &mut dyn FnMut(&str, &FontId) -> f32,
    default_font: &FontId,
) -> Vec<Token> {
    let flat = flatten(runs, base);
    let space_font = runs
        .first()
        .map(|r| r.format.font_id.clone())
        .unwrap_or_else(|| default_font.clone());
    let space_w = measure(" ", &space_font);

    let mut tokens: Vec<Token> = Vec::new();
    let mut cur_text = String::new();
    let mut cur_rtl = false;
    let mut cur_fmt: Option<TextFormat> = None;
    let mut cur_link: Option<String> = None;
    let mut cur_has_strong = false;

    macro_rules! flush {
        () => {
            if !cur_text.is_empty() {
                let fmt = cur_fmt
                    .clone()
                    .unwrap_or_else(|| TextFormat::simple(default_font.clone(), Color32::WHITE));
                let trimmed = cur_text.trim_end_matches(' ');
                let w_trim = measure(trimmed, &fmt.font_id);
                let spaces = (cur_text.len() - trimmed.len()) as f32;
                tokens.push(Token {
                    text: std::mem::take(&mut cur_text),
                    rtl: cur_rtl,
                    format: fmt,
                    link: cur_link.take(),
                    w: w_trim + spaces * space_w,
                    w_trim,
                });
                cur_has_strong = false;
            }
        };
    }

    for fc in &flat {
        let ch = fc.ch;
        if ch == ' ' {
            if !cur_text.is_empty() {
                cur_text.push(' ');
            } else {
                // leading spaces: attach to the previous token if any
                match tokens.last_mut() {
                    Some(t) => {
                        t.text.push(' ');
                        t.w += space_w;
                    }
                    None => cur_text.push(' '),
                }
            }
            continue;
        }
        let boundary = !cur_text.is_empty()
            && (cur_text.ends_with(' ')
                || cur_rtl != fc.rtl
                || cur_fmt.as_ref().is_none_or(|f| f != &fc.format));
        if boundary {
            flush!();
        }
        if cur_text.is_empty() {
            cur_rtl = fc.rtl;
            cur_fmt = Some(fc.format.clone());
            cur_link = fc.link.clone();
        }
        cur_text.push(ch);
        if class_of(ch) != Class::N {
            cur_has_strong = true;
        }
    }
    flush!();
    let _ = cur_has_strong;
    tokens
}

// --------------------------------------------------------------------- job

/// Build a laid-out job for a mixed-direction text with a single format.
/// `max_width` <= 0 means "do not wrap".
#[allow(clippy::too_many_arguments)]
pub fn job_text(
    text: &str,
    rtl_base: bool,
    font: FontId,
    color: Color32,
    max_width: f32,
    letter_spacing: f32,
    align: Align,
    measure: &mut dyn FnMut(&str, &FontId) -> f32,
) -> LayoutJob {
    let mut fmt = TextFormat::simple(font, color);
    fmt.extra_letter_spacing = letter_spacing;
    job_styled(
        &[Styled {
            text,
            format: fmt,
            link: None,
        }],
        rtl_base,
        max_width,
        align,
        measure,
    )
    .0
}

/// Build a laid-out job from styled runs (logical order). Returns the job
/// plus a map of `section index -> link url` for hit-testing.
pub fn job_styled(
    runs: &[Styled<'_>],
    rtl_base: bool,
    max_width: f32,
    align: Align,
    measure: &mut dyn FnMut(&str, &FontId) -> f32,
) -> (LayoutJob, Vec<(usize, String)>) {
    let font = runs
        .first()
        .map(|r| r.format.font_id.clone())
        .unwrap_or_else(|| FontId::proportional(12.0));
    let mut job = LayoutJob::default();
    // We do our own wrapping with hard newlines; egui must not re-wrap.
    job.wrap.max_width = f32::INFINITY;
    job.wrap.break_anywhere = false;
    // Sections are pushed MANUALLY: `LayoutJob::append` merges adjacent
    // same-format sections, which would re-join the carefully separated
    // direction runs into one harfbuzz run (and re-break Latin words).
    let push = |job: &mut LayoutJob, text: &str, lead: f32, fmt: TextFormat| {
        if text.is_empty() {
            return;
        }
        let start = job.text.len();
        job.text.push_str(text);
        job.sections.push(LayoutSection {
            leading_space: lead,
            byte_range: ByteIndex(start)..ByteIndex(job.text.len()),
            format: fmt,
        });
    };
    let mut links: Vec<(usize, String)> = Vec::new();
    let mut section_index: usize = 0;

    let mut first_line = true;
    for para in split_paragraphs(runs) {
        let tokens = tokenize(&para, rtl_base, measure, &font);
        // ---- greedy line filling (logical order); widths include every
        // trailing space for now — the visual-edge hang is removed once
        // the visual order is known.
        let mut lines: Vec<Vec<usize>> = Vec::new();
        let mut line_w: Vec<f32> = Vec::new();
        let mut cur: Vec<usize> = Vec::new();
        let mut cur_w = 0.0f32;
        for (i, t) in tokens.iter().enumerate() {
            if cur.is_empty() {
                cur.push(i);
                cur_w = t.w;
            } else if cur_w + t.w_trim <= max_width {
                cur.push(i);
                cur_w += t.w;
            } else {
                line_w.push(cur_w);
                lines.push(std::mem::take(&mut cur));
                cur.push(i);
                cur_w = t.w;
            }
        }
        if !cur.is_empty() {
            line_w.push(cur_w);
            lines.push(cur);
        }
        // ---- visual order per line (phrase-aware for RTL), and drop the
        // hanging trailing spaces of whichever token lands at the visual
        // end of its line.
        let mut orders: Vec<Vec<usize>> = Vec::with_capacity(lines.len());
        for line in &lines {
            let order = visual_order(line, &tokens, rtl_base);
            if let Some(&vl) = order.last() {
                let hang = tokens[vl].w - tokens[vl].w_trim;
                if hang > 0.0 {
                    if let Some(w) = line_w.last_mut() {
                        *w -= hang;
                    }
                }
            }
            orders.push(order);
        }
        let max_line_w = line_w.iter().cloned().fold(0.0f32, f32::max);

        // ---- emit sections per line, in visual order
        for (li, line) in lines.iter().enumerate() {
            if !first_line {
                push(
                    &mut job,
                    "\n",
                    0.0,
                    TextFormat::simple(font.clone(), Color32::TRANSPARENT),
                );
                section_index += 1;
            }
            first_line = false;
            // Row alignment inside the galley.
            let pad = match align {
                Align::Left => 0.0,
                Align::Right => (max_line_w - line_w[li]).max(0.0),
                Align::Center => ((max_line_w - line_w[li]) / 2.0).max(0.0),
            };
            let order = &orders[li];
            let visual_last = order.last().copied();
            // On RTL lines, a Latin token whose logical NEXT token is RTL
            // borders it on the visual LEFT — its trailing spaces must
            // render at the LEFT edge of its Latin phrase (a trailing
            // space inside an LTR section would render on the right).
            let mut lead_extra = vec![0.0f32; order.len()];
            let mut body_override: Vec<Option<String>> = vec![None; order.len()];
            if rtl_base {
                for (pos, &ti) in order.iter().enumerate() {
                    let t = &tokens[ti];
                    if t.rtl {
                        continue;
                    }
                    let Some(lpos) = line.iter().position(|&x| x == ti) else {
                        continue;
                    };
                    let next_rtl = line.get(lpos + 1).is_some_and(|&nx| tokens[nx].rtl);
                    if next_rtl {
                        let hang = t.w - t.w_trim;
                        if hang > 0.0 {
                            body_override[pos] = Some(t.text.trim_end_matches(' ').to_owned());
                            // the LEFTMOST section of this Latin phrase
                            let mut p0 = pos;
                            while p0 > 0 && !tokens[order[p0 - 1]].rtl {
                                p0 -= 1;
                            }
                            lead_extra[p0] += hang;
                        }
                    }
                }
            }
            for (k, ti) in order.iter().enumerate() {
                let t = &tokens[*ti];
                let mut fmt = t.format.clone();
                if t.rtl {
                    // letter spacing would tear joined Arabic apart.
                    fmt.extra_letter_spacing = 0.0;
                }
                let trimmed: &str = if let Some(o) = &body_override[k] {
                    o
                } else if Some(*ti) == visual_last {
                    t.text.trim_end_matches(' ')
                } else {
                    t.text.as_str()
                };
                // Persian/Arabic digits run LTR, but harfbuzz guesses the
                // direction from the first character's script — an Arabic
                // digit means an RTL guess, which would reverse the
                // number. Pre-reverse such tokens so the RTL shaping
                // restores the correct LTR digit order.
                let reversed: String;
                let body: &str = if !t.rtl && trimmed.chars().next().is_some_and(is_arabic_digit) {
                    reversed = trimmed.chars().rev().collect();
                    &reversed
                } else {
                    trimmed
                };
                let lead = if k == 0 {
                    pad + lead_extra[0]
                } else {
                    lead_extra[k]
                };
                push(&mut job, body, lead, fmt);
                if let Some(url) = &t.link {
                    links.push((section_index, url.clone()));
                }
                section_index += 1;
            }
        }
    }
    (job, links)
}

/// Visual emission order for one line of tokens.
///
/// Adjacent same-direction tokens form a phrase: Latin phrases
/// ("Windows Spotlight") keep their word order on RTL lines, and RTL
/// phrases reverse; the PHRASE order itself is reversed for RTL lines.
fn visual_order(line: &[usize], tokens: &[Token], rtl_base: bool) -> Vec<usize> {
    if !rtl_base {
        return line.to_vec();
    }
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for &ti in line.iter() {
        let rtl = tokens[ti].rtl;
        match groups.last_mut() {
            Some(g) if tokens[g[0]].rtl == rtl => g.push(ti),
            _ => groups.push(vec![ti]),
        }
    }
    let mut out = Vec::new();
    for g in groups.iter().rev() {
        if tokens[g[0]].rtl {
            out.extend(g.iter().rev().copied());
        } else {
            out.extend(g.iter().copied());
        }
    }
    out
}

/// Split runs into paragraphs on '\n' (a newline ends the paragraph; the
/// text after it continues with the same run format).
fn split_paragraphs<'a>(runs: &[Styled<'a>]) -> Vec<Vec<Styled<'a>>> {
    let mut out: Vec<Vec<Styled<'_>>> = vec![Vec::new()];
    for r in runs {
        let mut first = true;
        for piece in r.text.split('\n') {
            if !first {
                out.push(Vec::new());
            }
            first = false;
            if !piece.is_empty() {
                out.last_mut().unwrap().push(Styled {
                    text: piece,
                    format: r.format.clone(),
                    link: r.link,
                });
            }
        }
    }
    out
}

// ------------------------------------------------------------------ paint

/// Horizontal alignment of wrapped rows inside a galley.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Right,
    Center,
}

/// Layout `text` through the bidi engine and return the galley.
/// `max_width` <= 0 means "do not wrap".
pub fn galley_text(
    ui: &Ui,
    text: &str,
    rtl_base: bool,
    font: FontId,
    color: Color32,
    max_width: f32,
    align: Align,
) -> Arc<Galley> {
    let painter = ui.painter().clone();
    let mut measure = |t: &str, f: &FontId| {
        painter
            .layout_no_wrap(t.to_owned(), f.clone(), Color32::WHITE)
            .size()
            .x
    };
    let max_w = if max_width <= 0.0 {
        f32::INFINITY
    } else {
        max_width
    };
    let job = job_text(text, rtl_base, font, color, max_w, 0.0, align, &mut measure);
    painter.layout_job(job)
}

/// Layout styled runs through the bidi engine. Returns the galley plus
/// the `section -> url` link map (for hit-testing after placement).
pub fn galley_styled(
    ui: &Ui,
    runs: &[Styled<'_>],
    rtl_base: bool,
    max_width: f32,
    align: Align,
) -> (Arc<Galley>, Vec<(usize, String)>) {
    let painter = ui.painter().clone();
    let mut measure = |t: &str, f: &FontId| {
        painter
            .layout_no_wrap(t.to_owned(), f.clone(), Color32::WHITE)
            .size()
            .x
    };
    let max_w = if max_width <= 0.0 {
        f32::INFINITY
    } else {
        max_width
    };
    let (job, links) = job_styled(runs, rtl_base, max_w, align, &mut measure);
    (painter.layout_job(job), links)
}

/// Add a galley as a widget. `right` anchors the text block to the right
/// edge of the allocated rect (RTL paragraphs).
pub fn put(ui: &mut Ui, galley: Arc<Galley>, right: bool) -> egui::Response {
    let size = galley.size();
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let pos = if right {
        Pos2::new(rect.right() - size.x, rect.min.y)
    } else {
        rect.min
    };
    ui.painter().galley(pos, galley, Color32::WHITE);
    response
}

/// Rect of a byte range inside a laid-out galley (for link hit-testing).
/// Uses the galley's cursor API: the union of the carets at both ends of
/// the section text spans the section's glyphs on its row(s).
pub fn section_rect(
    galley: &Galley,
    byte_start: usize,
    byte_end: usize,
    origin: Pos2,
) -> Option<Rect> {
    let text = &galley.job.text;
    let cs = text[..byte_start.min(text.len())].chars().count();
    let ce = text[..byte_end.min(text.len())].chars().count();
    let r1 = galley.pos_from_cursor(egui::text::CCursor::new(cs));
    let r2 = galley.pos_from_cursor(egui::text::CCursor::new(ce));
    if r1.is_negative() || r2.is_negative() {
        return None;
    }
    Some(r1.union(r2).translate(origin.to_vec2()))
}

// ------------------------------------------------------------------ tests
#[cfg(test)]
mod tests {
    use super::*;

    fn fake_measure(t: &str, _f: &FontId) -> f32 {
        t.chars().count() as f32 * 6.0
    }

    fn fmt() -> TextFormat {
        TextFormat::simple(FontId::proportional(12.0), Color32::WHITE)
    }

    fn section_texts(job: &LayoutJob) -> Vec<&str> {
        job.sections
            .iter()
            .map(|s| &job.text[s.byte_range.start.0..s.byte_range.end.0])
            .collect()
    }

    #[test]
    fn base_direction_from_first_strong() {
        assert!(base_rtl("سلام world"));
        assert!(!base_rtl("world سلام"));
        assert!(!base_rtl("12345"));
    }

    #[test]
    fn visual_order_reversed_for_rtl() {
        // "سلام NLA" in RTL: NLA must be emitted first (leftmost).
        let job = job_text(
            "سلام NLA",
            true,
            FontId::proportional(12.0),
            Color32::WHITE,
            f32::INFINITY,
            0.0,
            Align::Right,
            &mut fake_measure,
        );
        let texts = section_texts(&job);
        assert_eq!(texts, vec!["NLA", "سلام"]);
    }

    #[test]
    fn visual_order_kept_for_ltr() {
        let job = job_text(
            "سلام NLA",
            false,
            FontId::proportional(12.0),
            Color32::WHITE,
            f32::INFINITY,
            0.0,
            Align::Left,
            &mut fake_measure,
        );
        let texts = section_texts(&job);
        assert_eq!(texts, vec!["سلام ", "NLA"]);
    }

    #[test]
    fn brackets_land_on_the_rtl_side() {
        // Logical: کاربر (Desktop و Videos) را
        // The parens resolve RTL (mixed neighbours -> base) and must be
        // laid out with the RTL flow, NOT inside the Latin phrase.
        let job = job_text(
            "کاربر (Desktop و Videos) را",
            true,
            FontId::proportional(12.0),
            Color32::WHITE,
            f32::INFINITY,
            0.0,
            Align::Right,
            &mut fake_measure,
        );
        let texts = section_texts(&job);
        // visual (L->R): [را][ logical-) ][Videos][ و ][Desktop][ logical-( ][کاربر]
        // (the parens render mirrored by harfbuzz: ")" glyph left, "(" glyph right)
        assert_eq!(
            texts,
            vec!["را", ") ", "Videos", "و ", "Desktop", "(", "کاربر"]
        );
    }

    #[test]
    fn latin_phrases_keep_word_order_on_rtl_lines() {
        // Logical: قابلیت Windows Spotlight تصاویر
        // Visual:  [تصاویر] [Windows Spotlight] [قابلیت]
        let job = job_text(
            "قابلیت Windows Spotlight تصاویر",
            true,
            FontId::proportional(12.0),
            Color32::WHITE,
            f32::INFINITY,
            0.0,
            Align::Right,
            &mut fake_measure,
        );
        let texts = section_texts(&job);
        // Windows keeps its inner-phrase space; Spotlight's space moved to
        // the phrase's leading_space (it borders the RTL قابلیت).
        assert_eq!(texts, vec!["تصاویر", "Windows ", "Spotlight", "قابلیت"]);
    }

    #[test]
    fn persian_digits_are_ltr_tokens() {
        // Logical: ویندوز ۱۰ و ۱۱ — the numbers are pre-reversed in the
        // section text so harfbuzz's RTL guess restores them.
        let job = job_text(
            "ویندوز ۱۰ و ۱۱",
            true,
            FontId::proportional(12.0),
            Color32::WHITE,
            f32::INFINITY,
            0.0,
            Align::Right,
            &mut fake_measure,
        );
        let texts = section_texts(&job);
        assert_eq!(texts, vec!["۱۱", "و ", "۰۱", "ویندوز"]);
    }

    #[test]
    fn tokenizing_keeps_spaces_with_words() {
        let tokens = tokenize(
            &[Styled::new("سلام NLA و دنیا", fmt())],
            true,
            &mut fake_measure,
            &FontId::proportional(12.0),
        );
        let texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["سلام ", "NLA ", "و ", "دنیا"]);
        assert!(tokens[0].rtl && tokens[2].rtl && tokens[3].rtl);
        assert!(!tokens[1].rtl, "NLA must classify as LTR");
    }

    #[test]
    fn format_changes_split_tokens() {
        let tokens = tokenize(
            &[
                Styled::new("normal ", fmt()),
                Styled::new(
                    "bold",
                    TextFormat::simple(FontId::proportional(12.0), Color32::RED),
                ),
            ],
            false,
            &mut fake_measure,
            &FontId::proportional(12.0),
        );
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "normal ");
        assert_eq!(tokens[1].text, "bold");
    }

    #[test]
    fn wrapping_breaks_lines() {
        // 6px/char, width 60 => 10 chars per line.
        let job = job_text(
            "one two three four five",
            false,
            FontId::proportional(12.0),
            Color32::WHITE,
            60.0,
            0.0,
            Align::Left,
            &mut fake_measure,
        );
        let rows = job.text.matches('\n').count() + 1;
        assert!(rows >= 2, "expected wrapping, text: {:?}", job.text);
    }

    #[test]
    fn rtl_rows_get_right_align_padding() {
        let job = job_text(
            "سلام این متن طولانی است کلمه بیشتر\nکوتاه",
            true,
            FontId::proportional(12.0),
            Color32::WHITE,
            66.0,
            0.0,
            Align::Right,
            &mut fake_measure,
        );
        assert!(job.sections.iter().any(|s| s.leading_space > 0.0));
    }

    #[test]
    fn letter_spacing_zeroed_on_rtl_tokens() {
        let job = job_text(
            "سلام",
            true,
            FontId::proportional(12.0),
            Color32::WHITE,
            f32::INFINITY,
            1.5,
            Align::Right,
            &mut fake_measure,
        );
        assert_eq!(job.sections[0].format.extra_letter_spacing, 0.0);
        let job_l = job_text(
            "OPEN",
            false,
            FontId::proportional(12.0),
            Color32::WHITE,
            f32::INFINITY,
            1.5,
            Align::Left,
            &mut fake_measure,
        );
        assert_eq!(job_l.sections[0].format.extra_letter_spacing, 1.5);
    }

    #[test]
    fn empty_and_newline_inputs_survive() {
        let job = job_text(
            "",
            true,
            FontId::proportional(12.0),
            Color32::WHITE,
            100.0,
            0.0,
            Align::Right,
            &mut fake_measure,
        );
        assert!(job.sections.is_empty());
        let job2 = job_text(
            "یک\nدو",
            true,
            FontId::proportional(12.0),
            Color32::WHITE,
            100.0,
            0.0,
            Align::Right,
            &mut fake_measure,
        );
        assert!(job2.text.contains('\n'));
        assert_eq!(job2.text.matches('\n').count(), 1, "two rows");
    }

    #[test]
    fn links_are_reported_per_section() {
        let runs = [
            Styled::new("see ", fmt()),
            Styled::linked("docs", fmt(), "https://example.com"),
        ];
        let (job, links) = job_styled(&runs, false, f32::INFINITY, Align::Left, &mut fake_measure);
        assert_eq!(links.len(), 1);
        let (idx, url) = &links[0];
        assert_eq!(url, "https://example.com");
        let section = &job.sections[*idx];
        assert_eq!(
            &job.text[section.byte_range.start.0..section.byte_range.end.0],
            "docs"
        );
    }
}
