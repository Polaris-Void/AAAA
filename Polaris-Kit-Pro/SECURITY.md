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
exits immediately. Every payload that embeds its own `admin` self-elevation
asks consent again through its own logic — the launcher never silently
suppresses a UAC prompt.

### Unpredictable, private staging
Each run stages its payload to
`%TEMP%\Polaris-Kit-Pro\run-<pid>-<random>`.
The random suffix is drawn from the OS CSPRNG, so a local attacker cannot
pre-plant a file at the path and win a race. The staging root is verified to
be a real directory (not a symlink) before use.

### Hash-verified execution
Every payload carries its sha256 digest inside the binary. The digest is
checked twice:

1. **At boot** — the full vault audit logs `[ OK ]` or `[ERR ]` per payload.
2. **At staging** — the bytes written to disk are re-read and hashed immediately
   before execution; any mismatch aborts the run before a process is spawned.

### Containment
Each tool runs inside a Windows **job object** with kill-on-close semantics.
STOP (and window close) terminate the whole process tree — long-running
`robocopy`, `dism` or `explorer` children can never orphan. Tool output is
streamed into the in-app log; nothing opens a visible console or popup.

### Hygiene
- Stale staging directories (older than 24 h) are purged at boot.
- Payload payloads are byte-exact copies of the upstream Polaris-Void scripts;
  they are never patched at runtime beyond the documented `%~dp0` marker.
- `unsafe` is denied crate-wide. The single exception (`src/run.rs`) is a thin,
  documented Win32 FFI layer: handle acquisition, job objects, pipes and DWM
  attribute calls, each with a safety comment.
- Release builds compile with LTO, stripped symbols and no debug assertions.

## Scope

The launcher only executes its own embedded, hash-pinned payloads and the
native folder picker. It does not download anything, does not open network
listeners, and does not persist credentials anywhere (the only written files
are `settings.ini`, exported logs, and the tools' own output in their vaults).
