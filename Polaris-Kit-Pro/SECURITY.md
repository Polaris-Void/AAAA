# Security Policy

Polaris-Kit-Pro executes system-level maintenance tools with administrator
rights, so its security posture matters. This document explains how the
launcher protects your system and how to report a vulnerability.

## Reporting a vulnerability

Please **do not open a public issue** for security reports. Use GitHub's
private vulnerability reporting (the *Security* tab → *Report a
vulnerability*) or contact the maintainer directly. You will get an initial
response within 7 days. Fixes ship in the next patch release and are credited
in the changelog (unless you prefer to stay anonymous).

## The security model

### Elevation gate
The launcher refuses to run tools without administrator rights. On start it
relaunches itself elevated through UAC (`runas`); a standard-token process
exits immediately. The payloads contain no self-elevation of their own —
the launcher is the single UAC decision point and never silently
suppresses a prompt.

### On-disk payload execution (nothing is staged)
Since v3.1.1 the launcher executes payloads **in place** from the
`payloads\` folder that ships next to the executable. It never writes a
script to `%TEMP%` and never runs one from there — the
write-then-execute dropper pattern that antivirus heuristics flag cannot
occur by construction. The only `%TEMP%` footprint is a private
`%TEMP%\Polaris-Kit-Pro` folder that backs the child processes' own
TEMP/TMP environment (verified to be a real directory, not a symlink);
no executable is ever placed there.

Folder-dependent tools receive the target path as argument 1; the script
files themselves are never patched or rewritten at runtime.

### Hash-verified execution
Every payload carries its sha256 pin inside the binary. The pin is
checked twice:

1. **At boot** — the full payloads audit logs `[ OK ]` or `[ERR ]` per file.
2. **Before every dispatch** — the on-disk file is re-read and hashed
   immediately before execution; any mismatch (or a missing
   `payloads\` folder) aborts the run before a process is spawned.

### Containment
Each tool runs inside a Windows **job object** with kill-on-close semantics.
STOP (and window close) terminate the whole process tree — long-running
`robocopy`, `dism` or `explorer` children can never orphan. Tool output is
streamed into the in-app log; nothing opens a visible console or popup.

### Hygiene
- Stale per-run directories left behind by pre-3.1.1 releases (older
  than 24 h) are purged at boot.
- Payloads are byte-exact copies of the upstream Polaris-Void scripts;
  they are never patched at runtime (folder targets ride in as `%1`).
- `unsafe` is denied crate-wide. The single exception (`src/run.rs`) is a thin,
  documented Win32 FFI layer: handle acquisition, job objects, pipes and DWM
  attribute calls, each with a safety comment.
- Release builds compile with LTO, stripped symbols and no debug assertions.

## Scope

The launcher only executes its own hash-pinned payloads from the
`payloads\` folder next to it, plus the native folder picker. It does not download anything, does not open network
listeners, and does not persist credentials anywhere (the only written files
are `settings.ini`, exported logs, and the tools' own output in their vaults).
