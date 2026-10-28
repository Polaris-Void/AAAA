# Contributing to Polaris-Kit-Pro

Thanks for considering a contribution! The project is a single Rust crate
with a deliberate, small design system — please keep changes tight and
focused.

## Getting started

1. Install [Rust](https://rustup.rs) (stable, 1.85+).
2. `git clone` the repository, then `cargo test` — the full suite must pass
   before and after your change.
3. Use `Build-EXE.bat` (Windows) or `./build.sh` (sh) to produce `dist/Polaris-Kit-Pro.exe`.

## Ground rules

- **`unsafe` stays denied.** Win32 FFI belongs in `src/run.rs`, inside the
  existing audited section, with a `// Safety:` comment on every block.
- **No hard-coded colors.** The UI reads everything from `theme::Tokens` /
  `theme::*` constants. If you need a new token, add it there first.
- **Bilingual by default.** Every user-visible string needs an `en` and a `fa`
  entry in `data::STRINGS` (or the module tables). The product name
  `Polaris-Kit-Pro` is the only string that is never translated.
- **Payload vault is frozen.** Payloads are byte-exact copies of the upstream
  Polaris-Void tools. Do not edit `src/payloads.rs` by hand — a vault change
  must come from the generator and must update the embedded sha256 digests
  (the test suite enforces this).
- **Keep the tests green.** `cargo test` covers vault integrity, staging-name
  safety, folder validation, `%~dp0` patching, log bounds and the 12-module
  catalog. Add a test for any new safety gate.
- **Format before you push:** `cargo fmt` and `cargo clippy` (warnings are
  treated as errors in review).

## Commit / PR style

- Small, single-purpose commits (`fix: ...`, `feat: ...`, `docs: ...`).
- PRs must describe *what* changed and *why*; screenshots are welcome for UI
  changes.
- New modules: one `Module` entry in `src/data.rs` + payload(s) in the vault +
  README blocks for both languages + a suite test bump if the module count
  changes.
