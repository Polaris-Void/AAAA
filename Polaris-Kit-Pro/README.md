<div align="center">

# Polaris-Kit-Pro

**12 native Windows utilities — one launcher, one file, zero installs.**

*Version 3.1.1 — zero-temp Defender-clean execution, live progress, grid & list views.*

[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078D6?logo=windows&logoColor=white)](#)
[![Language](https://img.shields.io/badge/Language-Rust%20%7C%20egui-DEA584?logo=rust&logoColor=black)](#)
[![Architecture](https://img.shields.io/badge/Architecture-x64-555555)](#)
[![Portability](https://img.shields.io/badge/Portability-Single--File%20EXE-brightgreen)](#)
[![License](https://img.shields.io/badge/License-Apache--2.0-blue)](#)

Built for Windows 10 & 11 — best experienced on Windows 10 and 11.

**فارسی** | [English](README.md)

</div>

---

## What is this?

**Polaris-Kit-Pro** is a native Rust launcher that gathers the twelve
Windows utilities of the Polaris-Void suite into a single portable
executable. It bridges the gap between low-level system tooling (Batch,
PowerShell, registry patches) and a proper desktop app: no flashing
console windows, a live streaming log, one UAC prompt for the whole
session, in-app documentation for every tool, and a full English/Persian
interface rendered with a built-in bidirectional text engine.

The launcher executes every payload straight from the auditable
[`payloads/`](payloads/) folder that ships next to the executable —
scripts, registry patches, icon — each byte-verified against an
embedded SHA-256 pin before it runs. Nothing is written to `%TEMP%`,
nothing is installed, nothing is left behind.

---

## Highlights

- **Portable EXE + payloads folder** — no installer, no runtime
  dependencies, no .NET, no Python. Copy the folder anywhere and run
  it; the scripts stay readable and auditable next to the binary.
- **One elevation, zero popups** — UAC is requested once at launch; every
  tool then runs hidden (`CREATE_NO_WINDOW`) with its output streamed
  line-by-line into the in-app log.
- **Hash-pinned payload folder** — all 20 script/registry payloads
  live on disk and are SHA-256-verified at boot and again before
  every run; a modified or missing file refuses to execute.
- **Defender-clean execution** (v3.1.1) — payloads contain *no*
  self-elevation (no `Start-Process -Verb RunAs`, no VBS droppers), no
  `-EncodedCommand`/base64 anywhere; PowerShell runs strictly as
  `powershell.exe -NoProfile -ExecutionPolicy Bypass -File <payloads\...>`,
  and **nothing is ever written to %TEMP%** — the
  write-a-script-then-execute-it dropper pattern antivirus heuristics
  flag cannot occur by construction. The pin table is regenerated with
  `tools/gen_payloads.py`.
- **English + فارسی in one click** — the whole interface (except the
  product name) switches language instantly, powered by an in-app
  Unicode-bidi engine so mixed Persian/Latin lines always render in the
  correct order. Everything is set in **Vazirmatn** — one type family
  for both languages, body and log text alike.
- **Live log console — everywhere** — the LOG button lives in the header
  bar (next to the language switcher) and opens a non-blocking slide-over
  panel from *every* view and sub-page: timestamps, color-coded
  severities, autoscroll, one-click export, <kbd>Ctrl</kbd>+<kbd>L</kbd>
  to toggle, <kbd>Esc</kbd> to close. The log text is always English so
  markers stay greppable.
- **Live progress bars** — every action mounts a sleek 0–100 % linear
  progress bar right underneath the button you clicked, fed by the real
  percentages DISM/Robocopy print (with a polished indeterminate glide
  when a tool is silent), plus a **STOP / CANCEL** button that kills the
  whole process tree (`taskkill /PID /T /F` + Job Object) instantly.
- **Grid & list views** — the dashboard's responsive card deck or a
  compact structured list (ID, title, category, brief, quick actions);
  the preference is saved locally.
- **In-app README renderer** — each tool's full documentation (headings,
  tables, code blocks, clickable links) in both languages, rendered
  inside the app.
- **NATIVE-1080 design** — a 1440×900 dark interface tuned for 1920×1080
  panels, with a custom title bar, soft-cornered cards, category tabs,
  live search and A–Z / suite-order sorting. Live zoom with
  <kbd>Ctrl</kbd>+<kbd>+</kbd> / <kbd>Ctrl</kbd>+<kbd>−</kbd> /
  <kbd>Ctrl</kbd>+<kbd>0</kbd>.

---

## The 12 tools

| # | Tool | Category | Needs a target folder | What it does |
|:-:|------|:--------:|:---------------------:|--------------|
| 01 | **NCSI No-Internet Fix** | NET | — | Kills the false "No Internet" globe/exclamation while the web works, and manages Windows NCSI active probing. |
| 02 | **User Folders Backup** | BACKUP | — | Backs up the six user folders to `C:\User Backup` (registry-resolved paths, OneDrive-aware, Robocopy) and merges them back without overwriting. |
| 03 | **Spotlight Icon Toggle** | DESKTOP | — | Hides the "Learn about this picture" desktop icon while keeping Spotlight wallpapers alive. |
| 04 | **OEM Driver Backup** | DRIVERS | — | Exports every third-party OEM driver with DISM/PnPUtil and reinstalls them in bulk — the lifesaver before a Windows reinstall. |
| 05 | **Display Driver Backup** | DRIVERS | — | Surgical backup/restore of GPU display-class drivers only. |
| 06 | **USB Immunizer** | USB | ✔ (USB root) | Immunizes a stick against auto-created junk folders with hidden decoy files, and brands it with a custom icon + label. |
| 07 | **DNS + Proxy Reset** | NET | — | Resets DNS to DHCP on all adapters, disables system proxies and flushes the resolver cache. First aid for "connected but nothing loads". |
| 08 | **Image + Video Renamer** | FILES | ✔ | Bulk-renames 50+ image formats to `.jpg` and 40+ video formats to `.mp4`, recursively and collision-safe. |
| 09 | **Directory Tree Exporter** | AUDIT | ✔ | Maps a folder into a UTF-8 tree report with sizes and dates. |
| 10 | **SHA-256 Tree Auditor** | AUDIT | ✔ | Everything the Tree Exporter does plus a SHA-256 checksum per file — tamper evidence for archives. |
| 11 | **Subfolder Unpacker** | FILES | ✔ | Flattens nested subfolders into the parent — perfect cleanup after multi-folder downloads. |
| 12 | **File Explorer Tweaker** | SYS | — | One click: open-on-This-PC, kill recent/cloud suggestions, enable check boxes, extensions and hidden items (8 HKCU values, then Explorer restarts itself). |

Each card opens the tool's page with its actions, safety notes and the
full in-app README. Every tool also links to its upstream repository.

---

## Getting started

1. Download `Polaris-Kit-Pro.exe` and put it anywhere (Desktop is fine).
2. Double-click. Accept the single UAC prompt — the tools need an
   elevated session, and asking once keeps the rest of the run silent.
3. Pick a tool, read its page, click the action. A live progress bar
   appears right under the button; **STOP / CANCEL** kills the run
   instantly. The LOG button (header, next to the language switcher)
   opens the log from anywhere.
4. Prefer Persian? The black language button in the header switches the
   whole app to فارسی — and back — instantly. Your choice (and the
   grid/list view) is remembered for the next launch. The app always
   starts in English unless you switch it.

Nothing is installed. The only footprint outside the app folder is a
tiny `settings.ini` (language, zoom, view mode) in your profile and a
private `%TEMP%\Polaris-Kit-Pro` folder that backs the tools' own
scratch files (no executable is ever placed there).

---

## Building from source

- **Windows, Command Prompt:** run `Build-EXE.bat`
- **Windows, PowerShell:** run `./Build-EXE.ps1`
- **Shell:** `./build.sh` for the host target, or
  `./build.sh x86_64-pc-windows-gnu` with a suitable cross toolchain
- **Development:** `cargo run` (the UI previews on any desktop; payload
  execution is Windows-only)

The Windows scripts produce `dist/Polaris-Kit-Pro.exe` **plus the
`dist/payloads/` folder** (copied automatically) — keep the two
together. Requirements: stable Rust (1.85+) and, for
the MSVC target, the Visual Studio C++ build tools.

---

## Safety model

- Payloads ship as real, readable files in `payloads/`, never
  downloaded; each is verified against its built-in SHA-256 pin at
  boot **and** before every execution. Nothing is ever written to
  `%TEMP%` for execution — no write-script-then-run dropper pattern
  exists.
- Folder-dependent tools receive the target path as an argument; the
  scripts are never patched or rewritten at runtime.
- **No self-elevating payloads.** The launcher elevates once via UAC;
  the hosted scripts merely guard and fail clean if they are not
  elevated — the `powershell Start-Process -Verb RunAs` and VBS
  `getadmin.vbs` patterns that antivirus heuristics flag are gone.
- Child processes run under a Windows Job Object; STOP terminates the
  whole tree (with a `taskkill /PID /T /F` fallback) — no orphans, no
  frozen UI.
- The target-folder picker rejects injection carriers (quotes, `&`,
  `|`, `%var%`, newlines) before any payload is patched with a path.
- Registry modules touch the documented values only — no policy keys, no
  binary blobs.

---

## FAQ

**Why does it ask for administrator?**
The utilities drive service control, DISM, Robocopy of profile folders
and machine registry keys. One elevation at launch keeps every later
action silent instead of prompting per tool.

**SmartScreen warns me on first run.**
The executable is unsigned. Choose *More info → Run anyway*, or verify
the SHA-256 published with the release first.

**Where do my backups go?**
User folders → `C:\User Backup`, OEM drivers → `C:\Backup_Driver`,
display drivers → `C:\Backup_Display_Driver`. Open the folder from
Explorer to inspect or reuse the files.

**Does it work on Windows 8 / 7?**
It targets Windows 10 and 11. Some tools may run on older builds, but
nothing there is tested or supported.

---

## Project layout

```
payloads/       the editable module scripts (batch / PowerShell / reg)
tools/
  gen_payloads.py   regenerates src/payloads.rs from payloads/ (sha256-pinned)
src/
  app.rs       state machine: routing, worker lifecycle, dialogs, boot
  views.rs     toolbar, grid/list, tool pages, slide-over log, dialogs
  widgets.rs   buttons, window controls, progress bar, painter text helpers
  bidi.rs      in-app Unicode-bidi layout engine (RTL/LTR mixed text)
  theme.rs     Nova palette, Vazirmatn font system, 1080p metrics
  data.rs      module catalog + EN/FA string tables
  readmes.rs   per-tool documentation blocks (EN/FA)
  payloads.rs  sha256 pin table for the payloads/ folder (AUTO-GENERATED)
  run.rs       process spawning, job objects, on-disk dispatch, progress, Win32 glue
  settings.rs  language + zoom + view persistence (atomic INI)
```

---

## License

Released under the **Apache License 2.0**. The embedded Vazirmatn fonts
are used under the SIL Open Font License 1.1 (`assets/fonts/OFL-Vazirmatn.txt`).
