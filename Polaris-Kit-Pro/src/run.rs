//! Execution engine: the port of the Python worker.
//!
//! - v3.1.1 Defender-clean model: payloads are executed IN PLACE from the
//!   `payloads\` folder that ships next to the executable. Nothing is ever
//!   written to `%TEMP%`, so the write-script-then-execute dropper pattern
//!   Windows Defender heuristics flag cannot occur by construction.
//! - Before every run the on-disk payload is re-read and sha256-verified
//!   against the embedded pin table (tamper evidence without temp copies).
//! - Runs fully hidden (`CREATE_NO_WINDOW` + `SW_HIDE`), streaming decoded
//!   output lines into the in-app log. Zero popups, zero console windows.
//! - PowerShell payloads (kind "ps1") are dispatched strictly as
//!   `powershell.exe -NoProfile -ExecutionPolicy Bypass -File <payloads\...>`
//!   - never `-Command`, never `-EncodedCommand`, no base64 anywhere.
//!   Folder-dependent batch tools receive the target folder as `%1`
//!   instead of a patched script copy.
//! - `stop` kills the whole process tree (job object first, then
//!   `taskkill /PID <pid> /T /F` as the fallback), so a robocopy/dism
//!   child can never orphan or freeze the stream.
//! - Live progress: every streamed line is scanned for `NN%` markers
//!   (DISM, robocopy, PnPUtil) and reported through `RunShared::progress`
//!   for the linear progress bar under the action button.
//! - Stale per-run dirs left behind by pre-3.1.1 versions are purged at
//!   boot (24h cutoff); the folder now serves only as the child processes'
//!   private TEMP.
//!
//! On non-Windows targets the worker reports `execution needs Windows`
//! (the dev-preview behaviour of the Python app); the Windows path is the
//! deliverable.
//!
//! # Safety
//! The Windows half talks to Win32 through the `windows` crate. Every
//! `unsafe` block here is a thin, documented FFI call: the only raw
//! pointers handed over are to `Vec`/stack buffers that outlive the call,
//! and every returned handle is closed exactly once. The crate denies
//! `unsafe_code` globally; this module is the single, audited exception.
#![cfg_attr(windows, allow(unsafe_code))]
#![allow(unused_imports)]
use crate::payloads;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunEvent {
    Log(String),
    State(String),
    Done,
}

/// Shared between the UI thread and the run worker.
pub struct RunShared {
    pub stop: Arc<AtomicBool>,
    pub child_pid: Arc<Mutex<Option<u32>>>,
    pub child_exited: Arc<AtomicBool>,
    /// Job object handle (Windows) - closing it kills the whole tree.
    pub job: Arc<Mutex<Option<u64>>>,
    /// Live progress for the linear bar: 0..=1000 permille of completion.
    /// `u32::MAX` means "no determinate percentage seen yet" (indeterminate).
    pub progress: Arc<AtomicU32>,
}

/// Sentinel stored in [`RunShared::progress`] until a determinate
/// percentage is parsed from the tool output.
pub const PROGRESS_UNKNOWN: u32 = u32::MAX;

impl RunShared {
    pub fn new() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            child_pid: Arc::new(Mutex::new(None)),
            child_exited: Arc::new(AtomicBool::new(true)),
            job: Arc::new(Mutex::new(None)),
            progress: Arc::new(AtomicU32::new(PROGRESS_UNKNOWN)),
        }
    }

    /// Reset the progress tracker for a fresh run.
    pub fn reset_progress(&self) {
        self.progress.store(PROGRESS_UNKNOWN, Ordering::Relaxed);
    }

    /// Report a percentage seen in the tool output (clamped to 0..=100).
    /// Never regresses: tools occasionally restart their own progress.
    pub fn report_percent(&self, percent: f32) {
        if !percent.is_finite() {
            return;
        }
        let permille = (percent.clamp(0.0, 100.0) * 10.0).round() as u32;
        let current = self.progress.load(Ordering::Relaxed);
        if current == PROGRESS_UNKNOWN || permille > current {
            self.progress.store(permille, Ordering::Relaxed);
        }
    }

    /// The current progress fraction (0.0..=1.0), or None if unknown.
    pub fn progress_fraction(&self) -> Option<f32> {
        let raw = self.progress.load(Ordering::Relaxed);
        if raw == PROGRESS_UNKNOWN {
            None
        } else {
            Some(raw as f32 / 1000.0)
        }
    }

    /// Scan one streamed output line for a progress percentage. Covers the
    /// console progress bars of DISM (`[===  10.0%  ]`), robocopy
    /// (`95.4%`), and explicit `[PROGRESS] 42` markers the hosted scripts
    /// may emit.
    pub fn note_line(&self, line: &str) {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i].is_ascii_digit() {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                    i += 1;
                }
                // skip spaces, then expect a percent sign
                let mut j = i;
                while j < bytes.len() && bytes[j] == b' ' {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'%' {
                    if let Ok(v) = line[start..i].parse::<f32>() {
                        // sanity: percent values carry at most one decimal
                        let text = &line[start..i];
                        if text.matches('.').count() <= 1 && v <= 100.0 {
                            self.report_percent(v);
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
        const PROGRESS_TAG: &str = "[PROGRESS]";
        if let Some(pos) = line.find(PROGRESS_TAG) {
            let rest = line[pos + PROGRESS_TAG.len()..].trim();
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(v) = digits.parse::<f32>() {
                self.report_percent(v);
            }
        }
    }

    /// The UI calls this on STOP / window close: terminate the process tree.
    /// The operation uses the already-open job handle and never starts a
    /// helper process on the UI thread.
    pub fn kill(&self) {
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::{CloseHandle, HANDLE};
            use windows::Win32::System::JobObjects::TerminateJobObject;

            // Serialize termination with the worker's job-handle close to
            // avoid acting on a recycled handle value.
            let job_guard = self.job.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(raw_handle) = *job_guard {
                // SAFETY: the worker owns this live job handle; the mutex
                // prevents it from closing the handle during this call.
                let _ =
                    unsafe { TerminateJobObject(HANDLE(raw_handle as *mut core::ffi::c_void), 1) };
                return;
            }
            drop(job_guard);

            // Fallback when job-object creation failed: taskkill the whole
            // tree (`/T`) forcefully (`/F`) by PID, hidden, no console flash.
            if let Some(pid) = *self.child_pid.lock().unwrap_or_else(|e| e.into_inner()) {
                Self::taskkill_tree(pid);
            }
        }
    }

    /// `taskkill /PID <pid> /T /F` - the standard process-tree termination,
    /// used as the fallback when no job object is available. Launched
    /// hidden with the same hardened environment rules as the payloads.
    #[cfg(windows)]
    fn taskkill_tree(pid: u32) {
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            CreateProcessW, WaitForSingleObject, CREATE_NO_WINDOW,
            CREATE_UNICODE_ENVIRONMENT, PROCESS_INFORMATION, STARTF_USESHOWWINDOW,
            STARTUPINFOW,
        };
        use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

        let taskkill = match system_executable("taskkill.exe") {
            Ok(path) => path,
            Err(_) => return,
        };
        let cmdline = format!("\"{}\" /PID {} /T /F", taskkill.display(), pid);
        let mut cmd_w: Vec<u16> = cmdline.encode_utf16().chain(std::iter::once(0)).collect();
        let taskkill_w: Vec<u16> = taskkill
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let si = STARTUPINFOW {
            cb: core::mem::size_of::<STARTUPINFOW>() as u32,
            dwFlags: STARTF_USESHOWWINDOW,
            wShowWindow: SW_HIDE.0 as u16,
            ..Default::default()
        };
        let mut pi = PROCESS_INFORMATION::default();
        // SAFETY: all wide buffers are NUL-terminated and outlive the call;
        // the child handles are closed on every path below.
        let created = unsafe {
            CreateProcessW(
                PCWSTR::from_raw(taskkill_w.as_ptr()),
                Some(windows::core::PWSTR::from_raw(cmd_w.as_mut_ptr())),
                None,
                None,
                false,
                CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
                None,
                None,
                &si as *const STARTUPINFOW,
                &mut pi as *mut PROCESS_INFORMATION,
            )
        };
        if let Ok(()) = created {
            unsafe {
                let _ = WaitForSingleObject(pi.hProcess, 3000);
                let _ = CloseHandle(pi.hThread);
                let _ = CloseHandle(pi.hProcess);
            }
        }
    }
}

impl Default for RunShared {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------- elevation
/// Are we running elevated? (Python: `IsUserAnAdmin`.)
pub fn is_admin() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::UI::Shell::IsUserAnAdmin;
        // SAFETY: stateless query, no arguments.
        unsafe { IsUserAnAdmin().as_bool() }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Relaunch elevated via UAC and exit the current process. Called once at
/// launch, before any window exists (same as the Python app).
#[cfg(windows)]
pub fn elevate_self() {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = std::env::current_exe().unwrap_or_default();
    let exe_w: Vec<u16> = exe
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let joined = args
        .iter()
        .map(|argument| quote_windows_arg(argument))
        .collect::<Vec<_>>()
        .join(" ");
    let args_w: Vec<u16> = joined.encode_utf16().chain(std::iter::once(0)).collect();
    let op_w: Vec<u16> = w("runas");
    // SAFETY: all wide buffers are NUL-terminated and live to the end of
    // the function; the pointers are only read by the call.
    let _ = unsafe {
        ShellExecuteW(
            None,
            PCWSTR::from_raw(op_w.as_ptr()),
            PCWSTR::from_raw(exe_w.as_ptr()),
            PCWSTR::from_raw(args_w.as_ptr()),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    std::process::exit(0);
}

#[cfg(windows)]
fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn quote_windows_arg(argument: &str) -> String {
    if !argument.is_empty()
        && !argument
            .chars()
            .any(|character| character.is_whitespace() || character == '"')
    {
        return argument.to_owned();
    }

    let mut quoted = String::with_capacity(argument.len() + 2);
    quoted.push('"');
    let mut backslashes = 0usize;
    for character in argument.chars() {
        if character == '\\' {
            backslashes += 1;
            continue;
        }
        if character == '"' {
            quoted.extend(std::iter::repeat('\\').take(backslashes * 2 + 1));
            quoted.push('"');
        } else {
            quoted.extend(std::iter::repeat('\\').take(backslashes));
            quoted.push(character);
        }
        backslashes = 0;
    }
    quoted.extend(std::iter::repeat('\\').take(backslashes * 2));
    quoted.push('"');
    quoted
}

#[cfg(windows)]
fn windows_root() -> Result<PathBuf, String> {
    let root = std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("WINDIR"))
        .ok_or_else(|| "Windows system directory is unavailable".to_string())?;
    let root = PathBuf::from(root);
    if !root.is_absolute() {
        return Err("Windows system directory is not absolute".into());
    }
    let root = std::fs::canonicalize(root).map_err(|e| format!("system directory: {e}"))?;
    if !root.join("System32").is_dir() {
        return Err("Windows System32 directory is unavailable".into());
    }
    Ok(root)
}

/// Resolve trusted built-in executables explicitly; do not trust COMSPEC or
/// PATH when this process is elevated.
#[cfg(windows)]
fn system_executable(name: &str) -> Result<PathBuf, String> {
    if !safe_stage_filename(name) {
        return Err("invalid system executable name".into());
    }
    let path = windows_root()?.join("System32").join(name);
    let metadata = std::fs::metadata(&path).map_err(|e| format!("system tool missing: {e}"))?;
    if !metadata.is_file() {
        return Err("system tool is not a file".into());
    }
    std::fs::canonicalize(path).map_err(|e| format!("system tool path: {e}"))
}


/// Like [`system_executable`] but for tools that live in a subdirectory of
/// System32 (e.g. `WindowsPowerShell\v1.0\powershell.exe`). Every component
/// is validated against the safe-name whitelist before joining.
#[cfg(windows)]
fn system_executable_rel(components: &[&str]) -> Result<PathBuf, String> {
    if components.is_empty() {
        return Err("empty system tool path".into());
    }
    for component in components {
        if !safe_stage_filename(component) {
            return Err("invalid system tool path component".into());
        }
    }
    let mut path = windows_root()?;
    for component in components {
        path = path.join(component);
    }
    let metadata = std::fs::metadata(&path).map_err(|e| format!("system tool missing: {e}"))?;
    if !metadata.is_file() {
        return Err("system tool is not a file".into());
    }
    std::fs::canonicalize(path).map_err(|e| format!("system tool path: {e}"))
}

/// Inherit ordinary user settings but replace the execution-sensitive
/// variables so elevated payloads cannot resolve a user-controlled executable
/// through PATH or COMSPEC.
#[cfg(windows)]
fn safe_environment_block(private_temp: &Path) -> Result<Vec<u16>, String> {
    use std::os::windows::ffi::OsStrExt;

    let root = windows_root()?;
    if !private_temp.is_absolute() {
        return Err("private temp directory is not absolute".into());
    }
    let system_dir = root.join("System32");
    let path_parts = [
        system_dir.clone(),
        system_dir.join("Wbem"),
        system_dir.join("WindowsPowerShell").join("v1.0"),
    ];
    let path_value = path_parts
        .iter()
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join(";");
    let comspec = system_dir.join("cmd.exe");

    let mut variables: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os()
        .filter(|(key, _)| {
            ![
                "PATH",
                "COMSPEC",
                "SYSTEMROOT",
                "WINDIR",
                "TEMP",
                "TMP",
                "PATHEXT",
                "PSMODULEPATH",
                "NODEFAULTCURRENTDIRECTORYINEXEPATH",
            ]
            .iter()
            .any(|reserved| key.to_string_lossy().eq_ignore_ascii_case(reserved))
        })
        .collect();
    variables.push(("SystemRoot".into(), root.as_os_str().to_owned()));
    variables.push(("WINDIR".into(), root.as_os_str().to_owned()));
    variables.push(("ComSpec".into(), comspec.into_os_string()));
    variables.push(("PATH".into(), path_value.into()));
    variables.push(("PATHEXT".into(), ".COM;.EXE;.BAT;.CMD".into()));
    variables.push((
        "PSModulePath".into(),
        system_dir
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("Modules")
            .into_os_string(),
    ));
    variables.push(("TEMP".into(), private_temp.as_os_str().to_owned()));
    variables.push(("TMP".into(), private_temp.as_os_str().to_owned()));
    variables.push(("NoDefaultCurrentDirectoryInExePath".into(), "1".into()));
    variables.sort_by(|a, b| {
        a.0.to_string_lossy()
            .to_ascii_uppercase()
            .cmp(&b.0.to_string_lossy().to_ascii_uppercase())
    });

    let mut block = Vec::new();
    for (key, value) in variables {
        block.extend(key.encode_wide());
        block.push(b'=' as u16);
        block.extend(value.encode_wide());
        block.push(0);
    }
    block.push(0);
    Ok(block)
}

/// Same taskbar identity as the frozen exe (grouping + icon).
#[cfg(windows)]
pub fn set_app_id() {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
    // SAFETY: static string, outlives the call.
    let _ = unsafe {
        SetCurrentProcessExplicitAppUserModelID(&HSTRING::from("PolarisVoid.PolarisKitPro"))
    };
}

#[cfg(not(windows))]
pub fn set_app_id() {}

/// Ask Windows DWM to apply its rounded-window clipping on supported systems
/// and drop the light 1px system border (the white corner fringe).
#[cfg(windows)]
#[allow(deprecated)] // the rwh_05 compat trait is the shortest path to the HWND
pub fn apply_window_rounding(frame: &eframe::Frame) -> bool {
    use winit::platform::windows::{CornerPreference, WindowExtWindows};
    let Some(window) = frame.winit_window() else {
        return false;
    };
    window.set_corner_preference(CornerPreference::Round);
    // Kill the 1px system border: on Windows 11 it paints a light rim that
    // is most visible hugging the rounded corners (white arcs on the dark
    // theme). DWMWA_BORDER_COLOR = DWMWA_COLOR_NONE removes it, leaving
    // clean rounded corners with no light fringe.
    use winit::raw_window_handle::HasRawWindowHandle as _;
    let hwnd_ok = window.raw_window_handle().ok().and_then(|raw| match raw {
        winit::raw_window_handle::RawWindowHandle::Win32(handle) => Some(handle),
        _ => None,
    });
    if let Some(handle) = hwnd_ok {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR};
        const DWMWA_COLOR_NONE: u32 = 0xFFFF_FFFE;
        let hwnd = HWND(handle.hwnd.get() as *mut _);
        let color = DWMWA_COLOR_NONE;
        // SAFETY: hwnd is the live window handle borrowed from winit; the
        // call only changes a DWM visual attribute and returns immediately.
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_BORDER_COLOR,
                &color as *const u32 as *const std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
    true
}

#[cfg(not(windows))]
pub fn apply_window_rounding(_frame: &eframe::Frame) -> bool {
    true
}

#[cfg(windows)]
fn close_shared_job(shared: &RunShared, job: windows::Win32::Foundation::HANDLE) {
    use windows::Win32::Foundation::CloseHandle;
    if job.0.is_null() {
        return;
    }
    let mut shared_job = shared.job.lock().unwrap_or_else(|e| e.into_inner());
    if *shared_job == Some(job.0 as u64) {
        *shared_job = None;
    }
    // SAFETY: this worker owns the job handle, and the mutex serializes it
    // with RunShared::kill so it cannot be closed during termination.
    let _ = unsafe { CloseHandle(job) };
}

// ----------------------------------------------------------------- payload location
/// The `payloads\` folder that ships next to the executable. Since v3.1.1
/// this is the ONLY execution source: the real script files are run where
/// they are (auditable, user-readable, no temp copies). The folder must be
/// a directory; symlinks are rejected.
pub fn payload_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("executable path: {e}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "executable has no parent directory".to_string())?
        .join("payloads");
    let metadata = std::fs::symlink_metadata(&dir)
        .map_err(|_| "payloads folder not found next to the executable".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("payloads path is not a plain folder".into());
    }
    std::fs::canonicalize(dir).map_err(|e| format!("payloads path: {e}"))
}

// ----------------------------------------------------------------- private temp
/// `%TEMP%\Polaris-Kit-Pro` (or the OS temp directory on POSIX).
/// Since v3.1.1 this folder is NOT an execution source: it only backs the
/// child processes' TEMP/TMP environment (tools like DISM keep scratch
/// files there) and collects stale per-run dirs from older releases, which
/// [`clean_old_stages`] purges. The base is rejected if it is a symlink or
/// non-directory.
pub fn stage_base() -> std::io::Result<PathBuf> {
    let base = std::env::temp_dir().join("Polaris-Kit-Pro");
    match std::fs::create_dir(&base) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }

    let metadata = std::fs::symlink_metadata(&base)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "private temp root is not a directory",
        ));
    }

    // On POSIX, tighten an existing directory as well as a newly created one.
    // Windows inherits the per-user ACL of the user's temp directory.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(base)
}

#[cfg(windows)]
fn safe_stage_filename(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name != "."
        && name != ".."
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

/// Write an asset without following or overwriting an existing path.
#[cfg(windows)]
fn write_new_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;

    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)
}

/// Purge leftover run dirs older than 24h (crashed/killed runs).
pub fn clean_old_stages() {
    let Ok(base) = stage_base() else { return };
    let Ok(entries) = std::fs::read_dir(&base) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.file_name().to_string_lossy().starts_with("run-") {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if !kind.is_dir() || kind.is_symlink() {
            continue;
        }
        let Ok(metadata) = std::fs::symlink_metadata(entry.path()) else {
            continue;
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if modified
            .elapsed()
            .map(|age| age.as_secs() > 86_400)
            .unwrap_or(false)
        {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

// ----------------------------------------------------------------- folder validation
/// Characters that change the meaning of a batch line / cmd command line:
/// a target folder containing any of them would break (or worse, inject
/// commands into) the generated .bat -> reject with a clear message.
/// `!` corrupts delayed-expansion values inside the folder-dependent tools.
const UNSAFE_PATH_CHARS: &[char] = &['"', '&', '|', '<', '>', '^', '%', '(', ')', '!'];

/// Returns the usable folder or a human-readable reason (the pass-7 logic).
pub fn check_folder(d: &str) -> Result<PathBuf, String> {
    if d.is_empty() {
        return Err("empty path".into());
    }
    if d.contains('\0') || d.contains('\r') || d.contains('\n') {
        return Err("path contains control characters".into());
    }
    if d.contains('`') {
        return Err("path contains a backtick".into());
    }
    let bad: std::collections::BTreeSet<char> = d
        .chars()
        .filter(|c| UNSAFE_PATH_CHARS.contains(c))
        .collect();
    if !bad.is_empty() {
        let list: String = bad.iter().collect();
        return Err(format!("path contains unsafe characters: {list}"));
    }
    let p = PathBuf::from(d);
    if !p.is_dir() {
        return Err("path does not exist or is not a folder".into());
    }
    Ok(p)
}

/// Decodes one stream line: UTF-8 first, then the OEM codepage (Python:
/// "mbcs"), then lossy.
pub fn decode_line(b: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(b) {
        return s.to_string();
    }
    #[cfg(windows)]
    if let Some(s) = decode_mbcs(b) {
        return s;
    }
    String::from_utf8_lossy(b).into_owned()
}

#[cfg(windows)]
fn decode_mbcs(b: &[u8]) -> Option<String> {
    use windows::Win32::Globalization::{
        MultiByteToWideChar, CP_OEMCP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS,
    };
    let flags = MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0);
    // SAFETY: both calls pass owned buffers that live across the call.
    let n = unsafe { MultiByteToWideChar(CP_OEMCP, flags, b, None) };
    if n <= 0 {
        return None;
    }
    let mut w = vec![0u16; n as usize];
    let got = unsafe { MultiByteToWideChar(CP_OEMCP, flags, b, Some(&mut w)) };
    if got <= 0 {
        return None;
    }
    w.truncate(got as usize);
    String::from_utf16(&w).ok()
}

// lines that are artefacts of headless runs, not real problems
#[cfg(windows)]
const STREAM_SKIP: &[&str] = &[
    "Input redirection is not supported",
    "Press any key to continue",
];

/// The standalone scripts print an auto-close countdown at their end;
/// inside the app that line is noise, so keep it out of the log.
#[cfg(windows)]
fn is_skip_line(line: &str) -> bool {
    if STREAM_SKIP.iter().any(|s| line.contains(s)) {
        return true;
    }
    let lower = line.to_ascii_lowercase();
    if let Some(pos) = lower.rfind(" in ") {
        let pre = &lower[..pos];
        if pre.ends_with("closing automatically") || pre.ends_with("closing") {
            let rest = lower[pos + 4..].trim_start();
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if !digits.is_empty() {
                return true;
            }
        }
    }
    false
}

// ----------------------------------------------------------------- worker
/// The run worker (background thread). Mirrors the Python `worker` 1:1:
/// think phase -> stage -> verify -> dispatch -> stream -> cleanup.
pub fn spawn_worker(
    module: &'static crate::data::Module,
    action: &'static crate::data::Action,
    folder: String,
    answer: Option<String>,
    tx: SyncSender<RunEvent>,
    shared: Arc<RunShared>,
) -> std::io::Result<()> {
    std::thread::Builder::new()
        .name(format!("pkp-run-{}", module.mid))
        .spawn(move || worker_main(module, action, folder, answer, tx, shared))
        .map(|_| ())
}

fn worker_main(
    module: &'static crate::data::Module,
    action: &'static crate::data::Action,
    folder: String,
    answer: Option<String>,
    tx: SyncSender<RunEvent>,
    shared: Arc<RunShared>,
) {
    let log = |tx: &SyncSender<RunEvent>, s: &str| {
        let _ = tx.send(RunEvent::Log(s.to_string()));
    };
    let state = |tx: &SyncSender<RunEvent>, s: &str| {
        let _ = tx.send(RunEvent::State(s.to_string()));
    };
    log(
        &tx,
        &format!("[THINK] neural link :: module {} authenticated", module.mid),
    );
    std::thread::sleep(std::time::Duration::from_millis(150));
    if action.need_folder {
        log(&tx, &format!("[THINK] target resolved :: {folder}"));
    } else {
        log(&tx, "[THINK] scope :: LOCAL SYSTEM (no folder needed)");
    }
    std::thread::sleep(std::time::Duration::from_millis(150));
    if is_admin() {
        log(
            &tx,
            "[THINK] privilege scan :: ELEVATED session, no more prompts",
        );
    } else {
        log(
            &tx,
            "[THINK] privilege scan :: STANDARD session (dev preview)",
        );
    }
    std::thread::sleep(std::time::Duration::from_millis(150));

    #[cfg(not(windows))]
    {
        let _ = (module, action, answer, shared);
        log(
            &tx,
            &format!(
                "[ERR ] execution needs Windows (dev preview on {}).",
                std::env::consts::OS
            ),
        );
        state(&tx, "aborted :: needs windows");
        let _ = tx.send(RunEvent::Done);
    }

    #[cfg(windows)]
    {
        use std::sync::Arc;
        let result: Result<i32, String> = (|| {
            // defense in depth: re-validate at dispatch time (the folder
            // could have been deleted/renamed since it was picked)
            if action.need_folder {
                if let Err(why) = check_folder(&folder) {
                    log(&tx, &format!("[ABORT] target folder invalid :: {why}"));
                    state(&tx, "aborted :: bad target");
                    return Err("bad-target".into());
                }
            }

            let comspec = system_executable("cmd.exe")?;
            let reg_exe = system_executable("reg.exe")?;
            let comspec_text = comspec.to_string_lossy();
            let reg_text = reg_exe.to_string_lossy();
            let (cmdline, cwd): (String, PathBuf);

            if matches!(action.kind, "script" | "reg" | "ps1") {
                // v3.1.1 Defender-clean dispatch: resolve the payload INSIDE
                // the payloads\ folder next to the executable, verify its
                // bytes against the pin table, and run it WHERE IT IS.
                let pin = payloads::payload(action.resid)
                    .ok_or_else(|| "payload missing from the pin table".to_string())?;
                if !safe_stage_filename(pin.file) {
                    return Err("unsafe payload file name".into());
                }
                let pdir = payload_dir().map_err(|e| {
                    log(&tx, "[ERR ] payload folder missing - keep payloads\\ next to the exe.");
                    e
                })?;
                let ppath = pdir.join(pin.file);
                let on_disk = std::fs::read(&ppath)
                    .map_err(|e| format!("payload read ({}): {e}", pin.file))?;
                if sha256_hex(&on_disk) != pin.sha256_str() {
                    log(&tx, "[ERR ] payload integrity check failed - aborting.");
                    log(&tx, "[ERR ] the file in payloads\\ does not match its sha256 pin.");
                    state(&tx, "aborted :: integrity");
                    return Err("integrity".into());
                }
                log(
                    &tx,
                    &format!(
                        "[THINK] payload :: {} ({} bytes, sha256-verified on disk)",
                        pin.file,
                        on_disk.len()
                    ),
                );
                std::thread::sleep(std::time::Duration::from_millis(100));

                if action.copy_ico {
                    if let Some(ico) = payloads::payload("USB_ICO") {
                        let ipath = pdir.join(ico.file);
                        match std::fs::read(&ipath).map_err(|e| format!("icon read: {e}")) {
                            Ok(bytes) => {
                                let dest = Path::new(&folder).join(".autorun.ico");
                                match write_new_file(&dest, &bytes) {
                                    Ok(()) => {
                                        log(&tx, "[ OK ] brand icon deployed :: .autorun.ico")
                                    }
                                    Err(error)
                                        if error.kind() == std::io::ErrorKind::AlreadyExists =>
                                    {
                                        log(
                                            &tx,
                                            "[SKIP] .autorun.ico already present - kept as-is.",
                                        );
                                    }
                                    Err(error) => log(
                                        &tx,
                                        &format!("[WARN] icon deploy failed ({error}) - continuing."),
                                    ),
                                }
                            }
                            Err(error) => {
                                log(&tx, &format!("[WARN] icon source failed ({error}) - continuing."))
                            }
                        }
                    }
                }
                if answer.is_some() {
                    log(
                        &tx,
                        "[THINK] consent :: operator confirmed, answering prompt",
                    );
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                log(
                    &tx,
                    &format!("[EXEC] dispatching :: {} / {}", module.mid, action.label),
                );
                state(&tx, &format!("running :: {}", action.label));
                if ppath.to_string_lossy().contains('"')
                    || comspec_text.contains('"')
                    || reg_text.contains('"')
                {
                    log(&tx, "[ABORT] command path contains a quote - refusing.");
                    state(&tx, "aborted :: bad path");
                    return Err("bad-path".into());
                }
                if action.kind == "script" {
                    let mut tail = if action.argv.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", action.argv)
                    };
                    // folder-dependent tools receive the target as %1 - the
                    // scripts read it and fall back to their own folder when
                    // run standalone. No patched copies are ever written.
                    if action.need_folder {
                        tail.push_str(&format!(" \"{}\"", folder));
                    }
                    // NOTE: single string, NOT a list. Python's list2cmdline
                    // would escape the inner quotes and cmd.exe would fail.
                    // /d = skip AutoRun, /s = deterministic quote stripping.
                    cmdline = format!(
                        "\"{}\" /d /s /c \"\"{}\"{}\"",
                        comspec.display(),
                        ppath.display(),
                        tail
                    );
                    cwd = if action.need_folder {
                        Path::new(&folder).to_path_buf()
                    } else {
                        pdir
                    };
                } else if action.kind == "ps1" {
                    // Microsoft best-practice flags, verbatim: no profile,
                    // bypass for the shipped script, -File (never -Command,
                    // never -EncodedCommand, no base64 payloads anywhere).
                    let ps = system_executable_rel(&[
                        "WindowsPowerShell",
                        "v1.0",
                        "powershell.exe",
                    ])?;
                    let ps_text = ps.to_string_lossy();
                    if ps_text.contains('"') {
                        log(&tx, "[ABORT] powershell path contains a quote - refusing.");
                        state(&tx, "aborted :: bad path");
                        return Err("bad-path".into());
                    }
                    cmdline = format!(
                        "\"{}\" -NoProfile -ExecutionPolicy Bypass -File \"{}\"",
                        ps.display(),
                        ppath.display()
                    );
                    cwd = if action.need_folder {
                        Path::new(&folder).to_path_buf()
                    } else {
                        pdir
                    };
                } else {
                    cmdline = format!("\"{}\" import \"{}\"", reg_exe.display(), ppath.display());
                    cwd = pdir;
                }
            } else {
                // cmd (restart explorer)
                log(&tx, "[THINK] payload :: direct shell command (hidden)");
                std::thread::sleep(std::time::Duration::from_millis(100));
                log(
                    &tx,
                    &format!("[EXEC] dispatching :: {} / {}", module.mid, action.label),
                );
                state(&tx, &format!("running :: {}", action.label));
                cmdline = format!("\"{}\" /d /s /c \"{}\"", comspec.display(), action.args);
                cwd = stage_base().map_err(|error| format!("private temp: {error}"))?;
            }

            // the child's TEMP/TMP points at the private folder; nothing is
            // executed from there (on-disk payload dispatch only)
            let private_temp = stage_base().map_err(|error| format!("private temp: {error}"))?;
            Ok(run_hidden_windows(
                &cmdline,
                &cwd,
                &private_temp,
                answer.as_deref(),
                &tx,
                &shared,
            ))
        })();

        match result {
            Err(e) => {
                log(&tx, &format!("[ERR ] worker fault :: {e}"));
                state(&tx, "worker fault");
            }
            Ok(code) => {
                if code == -2 {
                    log(&tx, "[WARN] terminated by operator.");
                    state(&tx, "stopped by operator");
                } else if code == -1 {
                    state(&tx, "spawn failed");
                } else if code == 0 {
                    log(&tx, "[ OK ] process exited clean :: code 0");
                    state(&tx, "done :: exit 0");
                } else {
                    log(&tx, &format!("[WARN] process ended :: exit code {code}"));
                    state(&tx, &format!("done :: exit {code}"));
                }
                if module.mid == "SPOTLIGHT" && action.kind == "reg" && code == 0 {
                    log(&tx, "[TIP ] if the icon lingers: press RESTART EXPLORER.");
                }
            }
        }
        // nothing was written to %TEMP% for this run (on-disk dispatch);
        // stale dirs from pre-3.1.1 versions are purged at boot instead
        let _ = tx.send(RunEvent::Done);
    }
}

/// The Windows hidden run: job object + CREATE_NO_WINDOW + SW_HIDE, stdout
/// pipe streamed into the log. Returns exit code (-1 spawn, -2 stopped).
#[cfg(windows)]
fn run_hidden_windows(
    cmdline: &str,
    cwd: &Path,
    private_temp: &Path,
    input: Option<&str>,
    tx: &SyncSender<RunEvent>,
    shared: &Arc<RunShared>,
) -> i32 {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows::Win32::System::Pipes::CreatePipe;
    use windows::Win32::System::Threading::{
        CreateProcessW, GetExitCodeProcess, TerminateProcess, WaitForSingleObject,
        CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, INFINITE, PROCESS_INFORMATION,
        STARTF_USESHOWWINDOW, STARTUPINFOW,
    };
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

    let null = HANDLE::default();
    let mut environment = match safe_environment_block(private_temp) {
        Ok(environment) => environment,
        Err(error) => {
            let _ = tx.send(RunEvent::Log(format!(
                "[ERR ] secure environment setup failed :: {error}"
            )));
            return -1;
        }
    };
    // SAFETY: all FFI below passes owned, NUL-terminated buffers that outlive
    // the call; every handle is closed exactly once on every exit path.
    unsafe {
        // job object: closing it kills the whole tree (defense in depth for
        // the UI STOP, which also fires taskkill /F /T)
        let mut job = match CreateJobObjectW(None, PCWSTR::null()) {
            Ok(j) => j,
            Err(_) => {
                let _ = tx.send(RunEvent::Log(
                    "[WARN] job create failed - continuing without tree-kill job".into(),
                ));
                null
            }
        };
        if job.0 != null.0 {
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let _ = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const core::ffi::c_void,
                core::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            *shared.job.lock().unwrap_or_else(|e| e.into_inner()) = Some(job.0 as u64);
        }

        // pipes: stdout (+stderr merged) and optional stdin
        let mut stdout_r = null;
        let mut stdout_w = null;
        if CreatePipe(&mut stdout_r, &mut stdout_w, None, 0).is_err() {
            close_shared_job(shared, job);
            let _ = tx.send(RunEvent::Log("[ERR ] spawn failed :: stdout pipe".into()));
            return -1;
        }
        let have_stdin = input.is_some();
        let mut stdin_r = null;
        let mut stdin_w = null;
        if have_stdin && CreatePipe(&mut stdin_r, &mut stdin_w, None, 0).is_err() {
            let _ = CloseHandle(stdout_r);
            let _ = CloseHandle(stdout_w);
            close_shared_job(shared, job);
            let _ = tx.send(RunEvent::Log("[ERR ] spawn failed :: stdin pipe".into()));
            return -1;
        }

        let mut cmd_w: Vec<u16> = cmdline.encode_utf16().chain(std::iter::once(0)).collect();
        let cwd_w: Option<Vec<u16>> = cwd
            .to_str()
            .map(|s| {
                s.encode_utf16()
                    .chain(std::iter::once(0))
                    .collect::<Vec<u16>>()
            })
            .filter(|v| !v.is_empty());

        let si = STARTUPINFOW {
            cb: core::mem::size_of::<STARTUPINFOW>() as u32,
            lpReserved: windows::core::PWSTR::null(),
            lpDesktop: windows::core::PWSTR::null(),
            lpTitle: windows::core::PWSTR::null(),
            dwX: 0,
            dwY: 0,
            dwXSize: 0,
            dwYSize: 0,
            dwXCountChars: 0,
            dwYCountChars: 0,
            dwFillAttribute: 0,
            dwFlags: STARTF_USESHOWWINDOW,
            wShowWindow: SW_HIDE.0 as u16,
            cbReserved2: 0,
            lpReserved2: core::ptr::null_mut(),
            hStdInput: if have_stdin { stdin_r } else { null },
            hStdOutput: stdout_w,
            hStdError: stdout_w, // merged, like stderr=STDOUT
        };
        let mut pi = PROCESS_INFORMATION::default();

        let created = CreateProcessW(
            PCWSTR::null(),
            Some(windows::core::PWSTR::from_raw(cmd_w.as_mut_ptr())),
            None,
            None,
            true,
            CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
            Some(environment.as_mut_ptr() as *const core::ffi::c_void),
            match &cwd_w {
                Some(c) => PCWSTR::from_raw(c.as_ptr()),
                None => PCWSTR::null(),
            },
            &si as *const STARTUPINFOW,
            &mut pi as *mut PROCESS_INFORMATION,
        );

        // this process keeps only the read ends
        let _ = CloseHandle(stdout_w);
        if have_stdin {
            // write the answer, then close our end so the child sees EOF
            if let Some(answer) = input {
                let bytes = answer.as_bytes();
                let mut written = 0usize;
                while written < bytes.len() {
                    let mut n = 0u32;
                    if WriteFile(stdin_w, Some(&bytes[written..]), Some(&mut n), None).is_err() {
                        break;
                    }
                    if n == 0 {
                        break;
                    }
                    written += n as usize;
                }
            }
            let _ = CloseHandle(stdin_w);
        }
        let _ = CloseHandle(stdin_r);

        if created.is_err() {
            let _ = CloseHandle(stdout_r);
            close_shared_job(shared, job);
            let _ = tx.send(RunEvent::Log("[ERR ] spawn failed".into()));
            return -1;
        }

        *shared.child_pid.lock().unwrap_or_else(|e| e.into_inner()) = Some(pi.dwProcessId);
        let pid = pi.dwProcessId;
        if job.0 != null.0 && AssignProcessToJobObject(job, pi.hProcess).is_err() {
            let _ = tx.send(RunEvent::Log(
                "[WARN] job assignment failed - using direct process termination".into(),
            ));
            close_shared_job(shared, job);
            job = null;
        }
        if shared.stop.load(Ordering::Relaxed) {
            if job.0 != null.0 {
                let _ = windows::Win32::System::JobObjects::TerminateJobObject(job, 1);
            } else {
                let _ = TerminateProcess(pi.hProcess, 1);
            }
        }
        let _ = CloseHandle(pi.hThread);
        let _ = tx.send(RunEvent::Log(format!(
            "[EXEC] PID {pid} :: streaming output (hidden, in-app)..."
        )));

        // beat: every 20s while the child is alive, so a silent tool still
        // reports life (the Python _beat thread, 1:1)
        let start = std::time::Instant::now();
        let beat_exit = shared.child_exited.clone();
        let beat_tx = tx.clone();
        let beat = std::thread::spawn(move || {
            let mut last_beat = 0u64;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if beat_exit.load(Ordering::Relaxed) {
                    break;
                }
                let elapsed = start.elapsed().as_secs();
                if elapsed >= last_beat + 20 {
                    last_beat = elapsed;
                    let _ = beat_tx.send(RunEvent::Log(format!(
                        "[....] still running ({elapsed}s elapsed) - output streams live..."
                    )));
                }
            }
        });

        // stream the output line by line
        let mut out: Vec<u8> = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            if shared.child_exited.load(Ordering::Relaxed) {
                break;
            }
            let mut n = 0u32;
            match ReadFile(stdout_r, Some(&mut buf), Some(&mut n), None) {
                Ok(()) if n == 0 => break,
                Ok(()) => {
                    out.extend_from_slice(&buf[..n as usize]);
                    while let Some(pos) = out.iter().position(|&b| b == b'\n') {
                        let line_b: Vec<u8> = out.drain(..=pos).collect();
                        let line = decode_line(&line_b);
                        let line = line.trim_end_matches(['\r', '\n']);
                        // live progress for the linear bar (DISM/robocopy %)
                        shared.note_line(line);
                        if !line.trim().is_empty() && !is_skip_line(line) {
                            let _ = tx.send(RunEvent::Log(format!("  | {line}")));
                        }
                    }
                }
                Err(_) => {
                    let _ = tx.send(RunEvent::Log("[WARN] stream fault :: read error".into()));
                    break;
                }
            }
        }

        // wait for exit (the child may live briefly past the last line)
        let _ = WaitForSingleObject(pi.hProcess, INFINITE);
        let mut code: u32 = 1;
        let _ = GetExitCodeProcess(pi.hProcess, &mut code);
        shared.child_exited.store(true, Ordering::Relaxed);
        let _ = beat.join();

        let _ = CloseHandle(stdout_r);
        close_shared_job(shared, job);
        *shared.child_pid.lock().unwrap_or_else(|e| e.into_inner()) = None;
        let _ = CloseHandle(pi.hProcess);

        if shared.stop.load(Ordering::Relaxed) {
            -2
        } else {
            code as i32
        }
    }
}

// ----------------------------------------------------------------- stop
/// The UI calls this on STOP: kill the tree of the running tool.
pub fn stop_tree(shared: &RunShared) {
    shared.kill();
}

/// sha256 hex of a byte slice (staging integrity check + boot audit).
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
