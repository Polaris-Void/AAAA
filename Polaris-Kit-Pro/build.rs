//! Build script: embeds the frozen lion icon + version resource into the
//! Windows executable. On every other target it is a no-op.

fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        // Application manifest: declares Windows 10/11 compatibility,
        // per-monitor DPI awareness and the (asInvoker) execution level -
        // the launcher elevates itself via UAC at startup. A complete,
        // honest manifest keeps SmartScreen / Defender heuristics calm.
        res.set_manifest_file("assets/app.manifest");
        res.set("ProductName", "Polaris-Kit-Pro");
        res.set(
            "FileDescription",
            "Polaris-Kit-Pro - 12 Windows utilities in one launcher",
        );
        res.set("InternalName", "polaris-kit-pro");
        res.set("OriginalFilename", "Polaris-Kit-Pro.exe");
        res.set("CompanyName", "Polaris-Void");
        res.set("LegalCopyright", "Polaris-Void :: MIT License");
        // A complete, honest version block keeps file-reputation heuristics
        // (SmartScreen / Defender) from scoring the binary as generated junk.
        res.set("FileVersion", "3.1.1.0");
        res.set("ProductVersion", "3.1.1");
        if let Err(e) = res.compile() {
            println!("cargo:warning=winresource failed: {e}");
        }
    }
}
