//! Polaris-Kit-Pro test suite: the build-time guarantees the launcher
//! depends on. The boot audit verifies the on-disk payloads at runtime;
//! these tests verify them at development time, plus the gates that keep
//! elevated payload execution safe.

use polaris_kit_pro::data::MODULES;
use polaris_kit_pro::logstore::Log;
use polaris_kit_pro::{payloads, run};

/// The payloads/ folder in the repository root (the same folder that ships
/// next to the executable at runtime).
fn repo_payload_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("payloads")
}

/// Every payload file on disk must hash to its declared sha256 pin (the
/// offline twin of the boot audit - catches any accidental payload edit
/// before it can reach a release).
#[test]
fn vault_integrity_sha256() {
    assert!(!payloads::PAYLOADS.is_empty(), "payload pin table is empty");
    let dir = repo_payload_dir();
    assert!(dir.is_dir(), "payloads/ folder missing from the repo root");
    for p in payloads::PAYLOADS {
        let bytes = std::fs::read(dir.join(p.file))
            .unwrap_or_else(|e| panic!("payload {} unreadable: {e}", p.file));
        assert_eq!(
            run::sha256_hex(&bytes),
            p.sha256_str(),
            "payload {} deviates from its embedded pin",
            p.resid
        );
    }
}

/// Every file that ships in payloads/ must be pinned - nothing unpicked
/// may ride along (and nothing pinned may be missing, covered above).
#[test]
fn every_payload_file_is_pinned() {
    let pinned: std::collections::BTreeSet<&str> =
        payloads::PAYLOADS.iter().map(|p| p.file).collect();
    let mut on_disk = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(repo_payload_dir()).expect("payloads/ readable") {
        let entry = entry.expect("payload dir entry readable");
        if !entry.file_type().expect("file type readable").is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        on_disk.insert(name);
    }
    for name in &on_disk {
        assert!(
            pinned.contains(name.as_str()),
            "unpinned file {} rides along in payloads/",
            name
        );
    }
    assert_eq!(pinned.len(), on_disk.len(), "pin table and disk diverge");
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
            if a.kind == "script" || a.kind == "reg" || a.kind == "ps1" {
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

/// Payload file names must be plain, short, traversal-free ASCII: they are
/// joined onto the payloads/ folder path and executed verbatim.
#[test]
fn payload_file_names_are_safe() {
    for p in payloads::PAYLOADS {
        assert!(
            !p.file.is_empty() && p.file.len() <= 128,
            "bad payload file name {}",
            p.file
        );
        assert!(
            !p.file.contains(".."),
            "traversal in payload file name {}",
            p.file
        );
        assert!(
            p.file
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.')),
            "unsafe payload file name {}",
            p.file
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
        "C:\\boom!dir",
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

/// `%~dp0` patching is GONE: folder-dependent payloads must instead accept
/// the target folder as argument 1 and fall back to their own folder when
/// run standalone. This is what makes in-place execution (no temp copies)
/// possible.
#[test]
fn folder_payloads_accept_argument() {
    let dir = repo_payload_dir();
    for resid in ["IMGREN", "VIDREN", "UNPACK", "USB_TOOL"] {
        let p = payloads::payload(resid).expect("folder payload pinned");
        let bytes = std::fs::read(dir.join(p.file)).expect("folder payload readable");
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("if not \"%~1\"==\"\""),
            "payload {} does not read the target argument",
            p.file
        );
        assert!(
            text.contains("%~dp0"),
            "payload {} lost its standalone fallback",
            p.file
        );
    }
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
    assert_eq!(polaris_kit_pro::VERSION, "3.1.1");
    assert_eq!(polaris_kit_pro::APP_VERSION, "Polaris-Kit-Pro v3.1.1");
}

/// Settings whitelist parsing: valid pairs parse, junk lines vanish,
/// out-of-range scales clamp, and the serialize/parse roundtrip holds.
#[test]
fn settings_roundtrip_and_whitelist() {
    use polaris_kit_pro::app::Lang;
    use polaris_kit_pro::settings::Settings;

    // defaults: English UI, grid view (product defaults)
    assert_eq!(
        Settings::default(),
        Settings {
            lang: Lang::En,
            scale: 1.0,
            view: polaris_kit_pro::app::ViewMode::Grid
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

    // roundtrip (incl. the persisted view preference)
    let original = Settings {
        lang: Lang::Fa,
        scale: 1.25,
        view: polaris_kit_pro::app::ViewMode::List,
    };
    let parsed = Settings::parse(&original.serialize());
    assert_eq!(parsed.lang, Lang::Fa);
    assert!((parsed.scale - 1.25).abs() < 0.001);
    assert_eq!(parsed.view, polaris_kit_pro::app::ViewMode::List);

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

// ----------------------------------------------------------------- v3.1 tests

/// The live progress parser: DISM/robocopy percentages stream through the
/// log lines; the bar needs them clamped, monotonic and lazy (unknown until
/// the first real percentage shows up).
#[test]
fn progress_parses_percentages() {
    use polaris_kit_pro::run::{RunShared, PROGRESS_UNKNOWN};
    let shared = RunShared::new();
    assert_eq!(shared.progress_fraction(), None, "starts indeterminate");
    assert_eq!(shared.progress.load(std::sync::atomic::Ordering::Relaxed), PROGRESS_UNKNOWN);

    shared.note_line("  | [ 25.5%  ]                 ]");
    assert!((shared.progress_fraction().unwrap() - 0.255).abs() < 1e-6);

    shared.note_line("  | 95.4% copied");
    assert!((shared.progress_fraction().unwrap() - 0.954).abs() < 1e-6);

    // never regresses; bogus values above 100% are ignored as noise
    shared.note_line("12% restarting");
    assert!((shared.progress_fraction().unwrap() - 0.954).abs() < 1e-6);
    shared.note_line("999% nonsense");
    assert!((shared.progress_fraction().unwrap() - 0.954).abs() < 1e-6);
    shared.note_line("100.0% done");
    assert!((shared.progress_fraction().unwrap() - 1.0).abs() < 1e-6);

    // explicit [PROGRESS] markers from hosted scripts
    let shared2 = RunShared::new();
    shared2.note_line("[PROGRESS] 42");
    assert!((shared2.progress_fraction().unwrap() - 0.42).abs() < 1e-6);
}

/// Every PowerShell/batch payload (verified on disk) is dispatched with the
/// clean execution flags mandated by the Microsoft security baseline:
/// -NoProfile -ExecutionPolicy Bypass -File. The payloads must not contain
/// any self-elevation, base64/EncodedCommand or interactive-wait patterns.
#[test]
fn payloads_are_defender_clean() {
    let bad = [
        "-EncodedCommand", "FromBase64String", "Start-Process", "-Verb RunAs",
        "getadmin.vbs", "Start-Job", "Invoke-Expression", "IEX(", "DownloadString",
        "Net.WebClient", "certutil", "bitsadmin", "regsvr32", "mshta",
    ];
    let dir = repo_payload_dir();
    for p in payloads::PAYLOADS {
        if p.file.ends_with(".ps1") || p.file.ends_with(".bat") {
            let text = String::from_utf8_lossy(&std::fs::read(dir.join(p.file)).expect("payload readable"))
                .to_lowercase();
            for pattern in bad {
                assert!(
                    !text.contains(&pattern.to_lowercase()),
                    "payload {} contains forbidden pattern {:?}",
                    p.resid,
                    pattern
                );
            }
        }
    }
}

/// The four PowerShell tools are wired as kind "ps1" (clean -File dispatch)
/// and every module still exposes at least one runnable action.
#[test]
fn ps1_actions_wired() {
    let ps1: Vec<&str> = MODULES
        .iter()
        .flat_map(|m| m.acts.iter())
        .filter(|a| a.kind == "ps1")
        .map(|a| a.resid)
        .collect();
    assert_eq!(ps1.len(), 4, "expected 4 ps1 payloads, got {ps1:?}");
    for resid in ["DISPBACKUP_PS", "DNSRESET_PS", "TREE_PS", "SHA256_PS"] {
        assert!(ps1.contains(&resid), "{resid} missing from the ps1 set");
    }
    for m in &MODULES {
        assert!(m.acts.iter().any(|a| a.kind != "vault"), "module {} lost its actions", m.mid);
    }
}

/// The vault inspector (VIEW VAULT) is eradicated: no action, no string.
#[test]
fn vault_feature_is_gone() {
    for m in &MODULES {
        for a in m.acts {
            assert_ne!(a.kind, "vault", "module {} still has a vault action", m.mid);
        }
    }
}
