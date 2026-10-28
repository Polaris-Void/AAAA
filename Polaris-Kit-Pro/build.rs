//! Build script: embeds the frozen lion icon + version resource into the
//! Windows executable. On every other target it is a no-op.

fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Polaris-Kit-Pro");
        res.set(
            "FileDescription",
            "Polaris-Kit-Pro - 12 Windows utilities in one launcher",
        );
        res.set("InternalName", "polaris-kit-pro");
        res.set("OriginalFilename", "Polaris-Kit-Pro.exe");
        res.set("LegalCopyright", "Polaris-Void");
        if let Err(e) = res.compile() {
            println!("cargo:warning=winresource failed: {e}");
        }
    }
}
