<div align="center">

# Polaris-Kit-Pro

**12 native Windows utilities — one launcher, one file, zero installs.**

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

The launcher carries every payload inside the executable — scripts,
fonts, icons — and stages each one to a fresh, hash-verified temporary
directory per run. Nothing is installed, nothing is left behind.

---

## Highlights

- **Single-file portable EXE** — no installer, no runtime dependencies,
  no .NET, no Python. Copy it anywhere and run it.
- **One elevation, zero popups** — UAC is requested once at launch; every
  tool then runs hidden (`CREATE_NO_WINDOW`) with its output streamed
  line-by-line into the in-app log.
- **Byte-exact payload vault** — all 20 script/registry payloads are
  embedded and SHA-256-verified at boot and again before every run.
- **English + فارسی in one click** — the whole interface (except the
  product name) switches language instantly, powered by an in-app
  Unicode-bidi engine so mixed Persian/Latin lines always render in the
  correct order. Everything is set in **Vazirmatn** — one type family
  for both languages, body and log text alike.
- **Live log console** — a proper left-to-right terminal view with
  timestamps, color-coded severities, autoscroll, one-click export and
  a process-tree STOP that never freezes the UI.
- **In-app README renderer** — each tool's full documentation (headings,
  tables, code blocks, clickable links) in both languages, rendered
  inside the app.
- **Vault inspector** — browse backup folders (user folders, OEM
  drivers, display drivers) inside the app with sizes and totals.
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
3. Pick a tool, read its page, click the action. Watch the live LOG
   (top-right button) for output; use **STOP** if you change your mind.
4. Prefer Persian? The language button on the far left of the toolbar
  switches the whole app to فارسی — and back — instantly. Your choice
  is remembered for the next launch.

Nothing is installed. The only footprint outside the EXE is a tiny
`settings.ini` (language + zoom) in your profile and per-run staging
folders under `%TEMP%` that are cleaned up automatically.

---

## Building from source

- **Windows, Command Prompt:** run `Build-EXE.bat`
- **Windows, PowerShell:** run `./Build-EXE.ps1`
- **Shell:** `./build.sh` for the host target, or
  `./build.sh x86_64-pc-windows-gnu` with a suitable cross toolchain
- **Development:** `cargo run` (the UI previews on any desktop; payload
  execution is Windows-only)

The Windows scripts produce `dist/Polaris-Kit-Pro.exe` with the icon and
version resources baked in. Requirements: stable Rust (1.85+) and, for
the MSVC target, the Visual Studio C++ build tools.

---

## Safety model

- Payloads are embedded, never downloaded; each is verified against its
  built-in SHA-256 at boot **and** before every execution.
- Staging happens in a per-run, OS-randomized folder under `%TEMP%`,
  isolated per launch and removed afterwards.
- Child processes run under a Windows Job Object; STOP terminates the
  whole tree — no orphans, no frozen UI.
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
display drivers → `C:\Backup_Display_Driver`. The vault inspector shows
each one inside the app.

**Does it work on Windows 8 / 7?**
It targets Windows 10 and 11. Some tools may run on older builds, but
nothing there is tested or supported.

---

## Project layout

```
src/
  app.rs       state machine: routing, worker lifecycle, dialogs, boot
  views.rs     toolbar, grid, tool pages, log console, dialogs
  widgets.rs   buttons, window controls, painter text helpers
  bidi.rs      in-app Unicode-bidi layout engine (RTL/LTR mixed text)
  theme.rs     Nova palette, Vazirmatn font system, 1080p metrics
  data.rs      module catalog + EN/FA string tables
  readmes.rs   per-tool documentation blocks (EN/FA)
  payloads.rs  embedded, SHA-256-pinned payload vault
  run.rs       process spawning, job objects, staging, Win32 glue
  settings.rs  language + zoom persistence (atomic INI)
```

---

## License

Released under the **Apache License 2.0**. The embedded Vazirmatn fonts
are used under the SIL Open Font License 1.1 (`assets/fonts/OFL-Vazirmatn.txt`).
