# Changelog

All notable changes to Polaris-Kit-Pro are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project adheres to [Semantic Versioning](https://semver.org/).

## [3.0.0] — Vazirmatn One-Family Refresh

A complete typographic and bidirectional-text overhaul of the NATIVE-1080
interface, plus a redesigned toolbar and window chrome.

### Added
- **In-app Unicode-bidi engine** (`src/bidi.rs`). egui 0.36 has no bidi
  algorithm, so mixed Persian/Latin lines used to come out with reversed
  words (`NLA` -> `ALN`) and Persian sentences with reversed word order.
  The engine resolves per-character directions (UAX #9 N1/N2), groups
  tokens into same-direction phrases, lays phrases out visually and wraps
  lines itself — Persian digits, brackets, quoted Latin phrases and
  comma-separated Latin lists all render correctly now, and RTL paragraphs
  are properly right-aligned with per-line padding.
- **One type family: Vazirmatn.** Every stack — English and Persian, body
  text and log console — is now Vazirmatn (Regular/Medium/SemiBold/Bold).
  Inter, Archivo and Geist Mono are retired, which also shrinks the EXE.
- **Titlebar lion mark.** The lion icon returns to the title bar at a
  correct 30px size with clean rounded corners, decoded once into a GPU
  texture.
- **Category section bar.** A labelled separator above the grid
  ("SYS — 1 TOOL" / «سیستم — ۱ ابزار») frames every category — including
  the single-tool ones such as SYS and AUDIT.
- **ACTIONS / TARGET / README section headers** with hairline separators
  on every tool page, giving single-action modules the same framed
  structure as the rest.

### Changed
- **Toolbar redesign.** One row: language switch at the far left, the nine
  category pills starting directly under the wordmark and flowing
  left-to-right in both languages, then search, the A–Z / 01–12 segmented
  sort switch and a compact LOG button at the far right.
- **Search field.** Soft-cornered well with a magnifier glyph, a focus
  ring, a language-aware placeholder and a vertically centered caret (the
  old caret floated at the top of the box).
- **Cards.** Larger 4-column cards with everything centered: accent badge,
  title, blurb, GitHub link and the OPEN button.
- **Window controls.** Smaller, rounded, painter-drawn glyphs (a crisp
  minus, square and cross) instead of font glyphs; the close button gets
  its classic red hover.
- **Log console.** Rows now start at the left edge as a proper LTR
  terminal (they used to render centered), with ellipsized long lines.
- **README_FA data.** The embedded Persian documentation was stored
  pre-shaped in visual order (a leftover from the Python era) and rendered
  mirrored by the new shaping stack; it is converted back to logical
  order with restored ZWNJ half-spaces.
- **Subtitle.** Now reads "Built for Windows 10 & 11 — best experienced on
  Windows 10 and 11" / «برای ویندوز ۱۰ و ۱۱ ساخته شده — بهترین تجربه در
  ویندوز ۱۰ و ۱۱».

### Fixed
- RTL header rows inside the detail scroll area consumed the full
  (unbounded) scroll height and pushed the page content off-screen; they
  now allocate an explicit row height.
- Persian digits ran reversed inside RTL runs (harfbuzz guesses the run
  direction from the first character's script); digit-led tokens are
  pre-reversed to cancel the guess.

## [3.0.0] — NATIVE-1080

The first GitHub-source release of the PRO edition, shipped with a full
interface refresh targeting a 1920×1080 desktop.

### Fixed
- **Window controls.** Minimize / Maximize / Close were pushed off-screen by an
  over-sized drag band; the band now reserves exactly the width the controls
  need, so all three buttons are always visible at the top-right edge.
- **Category tabs.** The nine category tabs rendered stacked vertically in the
  middle of the page; they are now a single horizontal row pinned to the far
  top-right corner, with search and sort on the same line.
- **Broken in-app images.** All in-app bitmap images (header mark, module
  icons) failed to load in release builds and rendered as warning triangles;
  they are removed entirely — the UI is now pure vector text and hairlines.
- **White corner fringe.** The Windows 11 1px system border drew a light rim
  around the rounded corners (most visible at the top); the border color is
  now set to `DWMWA_COLOR_NONE`, leaving clean rounded corners.
- **Lion mark keyline.** The window/taskbar lion no longer carries the white
  keyline ring around its rounded square.
- **RTL text alignment.** Persian card titles, blurbs and table rows are
  properly right-aligned instead of painted left.

### Changed
- **One language switch for everything.** The single green `FA/EN` button now
  translates the entire interface — tabs, cards, actions, dialogs, boot banner,
  vault listings and hints — except the product name. Persian titles were added
  for all 12 modules.
- **Cleaner header.** The in-app lion logo, the yellow `PRO` chip and the boxed
  brand frame are removed; the wordmark sits top-left with the GitHub link
  beside it and the "Made for Windows 10 and 11" tag inline.
- **Toolbar at the top-right.** Category tabs, search and the suite-order/A–Z
  sort share one 44px row; tab cells adapt their width so nothing overflows.
- **Footer removed.** The version/name repetition, VAULT/ELEVATED/1080P badges
  and the UI-scale chip are gone; content reaches the bottom edge. UI zoom
  stays available via `Ctrl + +/−/0` (persisted).
- **Module cards are icon-free** and text-align consistently; the OPEN button
  keeps its full-width anchored placement.
- **Bilingual vault inspector.** File listings, sizes and folder scans follow
  the active language (including byte-size units in Persian).

### Security
- Staging directories keep unpredictable per-run names and are sha256-verified
  before execution; stale directories are purged on boot (24h cutoff).
- Every tool runs inside a kill-on-close job object; STOP and window close
  terminate the entire process tree.
- `unsafe` remains denied crate-wide except in the single audited Win32 FFI
  module; release builds use LTO and stripped symbols.
