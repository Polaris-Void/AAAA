//! Polaris-Kit-Pro test suite: the build-time guarantees the launcher
//! depends on. The boot audit verifies the vault at runtime; these tests
//! verify it at development time, plus the gates that keep elevated
//! payload execution safe.

use polaris_kit_pro::data::MODULES;
use polaris_kit_pro::logstore::Log;
use polaris_kit_pro::{payloads, run};

/// Every embedded payload must hash to its declared sha256 (the offline
/// twin of the boot audit - catches any accidental vault edit immediately).
#[test]
fn vault_integrity_sha256() {
    assert!(!payloads::PAYLOADS.is_empty(), "payload vault is empty");
    for p in payloads::PAYLOADS {
        let got = run::sha256_hex(p.data);
        let want: String = p.sha256.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            got, want,
            "payload {} deviates from its embedded hash",
            p.resid
        );
    }
}

/// Every module action must resolve to a vault payload, and every vault
/// payload must be reachable from the catalog (no dead weight, no dangling
/// reference) - the two directions of the vault/catalog contract.
#[test]
fn vault_catalog_is_consistent() {
    let mut used = std::collections::BTreeSet::new();
    for m in &MODULES {
        assert!(!m.acts.is_empty(), "module {} has no actions", m.mid);
        for a in m.acts {
            if a.kind == "script" || a.kind == "reg" {
                assert!(
                    payloads::payload(a.resid).is_some(),
                    "module {} action {} references missing payload {}",
                    m.mid,
                    a.label,
                    a.resid
                );
                assert!(used.insert(a.resid), "payload {} is wired twice", a.resid);
            }
        }
    }
    // the immunizer icon is copied by `copy_ico`, not dispatched as a payload
    assert!(payloads::payload("USB_ICO").is_some(), "USB_ICO missing");
    for p in payloads::PAYLOADS {
        if p.resid == "USB_ICO" {
            continue;
        }
        assert!(
            used.contains(p.resid),
            "vault payload {} is unreachable from the catalog",
            p.resid
        );
    }
}

/// Staged filenames must be plain, short, traversal-free ASCII: they are
/// joined onto the per-run staging directory and executed verbatim.
#[test]
fn vault_stage_names_are_safe() {
    for p in payloads::PAYLOADS {
        assert!(
            !p.stage.is_empty() && p.stage.len() <= 128,
            "bad stage name {}",
            p.stage
        );
        assert!(
            !p.stage.contains(".."),
            "traversal in stage name {}",
            p.stage
        );
        assert!(
            p.stage
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.')),
            "unsafe stage name {}",
            p.stage
        );
    }
}

/// The folder gate rejects command-injection carriers before any payload
/// is patched with the target path.
#[test]
fn folder_validation() {
    assert!(run::check_folder("").is_err(), "empty path accepted");
    for bad in [
        "C:\\bad\"quote",
        "C:\\cmd&whoami",
        "C:\\a|b",
        "C:\\a>b",
        "C:\\var%path%",
    ] {
        assert!(
            run::check_folder(bad).is_err(),
            "unsafe path accepted: {bad}"
        );
    }
    assert!(
        run::check_folder("line\nbreak").is_err(),
        "control characters accepted"
    );
    assert!(run::check_folder("/").is_ok(), "a real folder was rejected");
    assert!(
        run::check_folder("/definitely/not/a/real/dir-pkp").is_err(),
        "missing folder accepted"
    );
}

/// `%~dp0` patching: every marker becomes the target plus a trailing
/// backslash, and the bytes around the markers are untouched.
#[test]
fn patch_dp0_replaces_marker() {
    let data = b"@echo off\r\ncd /d \"%~dp0\"\r\ncopy \"%~dp0tool.ico\" \"%~dp0out\"\r\n".to_vec();
    let out = run::patch_dp0(&data, std::path::Path::new("C:\\Target"));
    let text = String::from_utf8(out).unwrap();
    assert!(!text.contains("%~dp0"), "marker survived patching");
    assert_eq!(
        text.matches("C:\\Target\\").count(),
        3,
        "marker replacement count"
    );
    assert!(
        text.starts_with("@echo off\r\n"),
        "bytes outside markers changed"
    );
}

/// The live log stays bounded, always renders bracketed timestamps, and
/// exports under the product name.
#[test]
fn log_is_bounded_and_timestamped() {
    let mut log = Log::new();
    for i in 0..20_000 {
        log.push(&format!("line {i} :: {}", "x".repeat(20)));
    }
    assert!(
        log.chars <= Log::MAX_CHARS,
        "log exceeded its bound: {} chars (max {})",
        log.chars,
        Log::MAX_CHARS
    );
    assert!(!log.lines.is_empty());
    for line in &log.lines {
        assert!(
            line.ts.starts_with('[') && line.ts.ends_with(']'),
            "bad timestamp {:?}",
            line.ts
        );
    }
    let export = log.export_text();
    assert!(export.starts_with("Polaris-Kit-Pro [PRO] :: log export"));
    log.clear();
    assert_eq!(log.lines.len(), 1, "clear must leave the purge notice");
}

/// Decoding child-process output never panics: UTF-8 first, then lossy
/// fallback for the OEM codepage bytes cmd.exe emits.
#[test]
fn decode_line_never_panics() {
    assert_eq!(run::decode_line(b"plain ascii"), "plain ascii");
    assert_eq!(run::decode_line("ch\u{2013}ar".as_bytes()), "ch\u{2013}ar");
    let decoded = run::decode_line(&[0xff, 0xfe, b'a']);
    assert!(
        decoded.contains('a'),
        "payload byte lost in decode: {decoded:?}"
    );
}

/// The full module count the UI, README and build metadata promise.
#[test]
fn suite_has_twelve_modules() {
    assert_eq!(MODULES.len(), 12, "the PRO edition ships 12 modules");
    let mids: std::collections::BTreeSet<&str> = MODULES.iter().map(|m| m.mid).collect();
    assert_eq!(mids.len(), 12, "module ids must be unique");
    assert!(mids.contains("EXPLORER-TWEAK"), "module 12 missing");
}

// ----------------------------------------------------------------- v3 tests

/// Product identity: the full name and version string the UI, log banner,
/// export header and settings file all rely on.
#[test]
fn identity_is_exact() {
    assert_eq!(polaris_kit_pro::APP_NAME, "Polaris-Kit-Pro");
    assert_eq!(polaris_kit_pro::VERSION, "3.0.0");
    assert_eq!(polaris_kit_pro::APP_VERSION, "Polaris-Kit-Pro v3.0.0");
}

/// Settings whitelist parsing: valid pairs parse, junk lines vanish,
/// out-of-range scales clamp, and the serialize/parse roundtrip holds.
#[test]
fn settings_roundtrip_and_whitelist() {
    use polaris_kit_pro::app::Lang;
    use polaris_kit_pro::settings::Settings;

    // defaults
    assert_eq!(
        Settings::default(),
        Settings {
            lang: Lang::En,
            scale: 1.0
        }
    );

    // junk tolerance
    let s = Settings::parse("garbage\nlang=fa\nbroken====\nscale=9999\n; c\n# c\n");
    assert_eq!(s.lang, Lang::Fa);
    assert_eq!(s.scale, Settings::SCALE_MAX);

    // non-finite scales fall back to 1.0
    for bad in ["nan", "inf", "-inf"] {
        let s = Settings::parse(&format!("scale={bad}"));
        assert_eq!(s.scale, 1.0);
    }

    // roundtrip
    let original = Settings {
        lang: Lang::Fa,
        scale: 1.25,
    };
    let parsed = Settings::parse(&original.serialize());
    assert_eq!(parsed.lang, Lang::Fa);
    assert!((parsed.scale - 1.25).abs() < 0.001);

    // scale cycle wraps in order and after the top step
    assert_eq!(Settings::next_scale(0.90), 1.00);
    assert_eq!(Settings::next_scale(1.00), 1.10);
    assert_eq!(Settings::next_scale(1.10), 1.25);
    assert_eq!(Settings::next_scale(1.25), 0.90);
}

/// The v3 NATIVE-1080 metrics are design-token guarantees, so they are
/// checked at COMPILE time via const assertions (and re-verified here).
#[test]
fn native_1080_metrics() {
    // Compile-time: the layout contract cannot regress silently.
    const _: () = assert!(
        polaris_kit_pro::theme::CONTROL_H >= 36.0,
        "controls too small for 1080p"
    );
    const _: () = assert!(
        polaris_kit_pro::theme::FOOTER_H <= 32.0,
        "footer must stay a thin strip"
    );
    const _: () = assert!(
        polaris_kit_pro::theme::CARD_MIN_W >= 288.0,
        "cards too narrow"
    );
    // Runtime: derive a non-constant value so this check stays meaningful.
    let sum = polaris_kit_pro::theme::CONTROL_H
        + polaris_kit_pro::theme::FOOTER_H
        + polaris_kit_pro::theme::CARD_MIN_W;
    assert!(sum.is_finite() && sum > 300.0, "metrics degenerated: {sum}");
}
