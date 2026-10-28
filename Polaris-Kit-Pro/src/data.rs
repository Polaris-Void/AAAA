//! Static, allocation-free app catalog generated from the bundled Polaris-Kit-Pro catalog.

#[derive(Clone, Copy, Debug)]
pub struct Run(pub &'static str, pub Style, pub &'static str);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Style {
    Norm,
    Bold,
    Code,
    Link,
}

pub enum Block {
    H1(&'static [Run]),
    H2(&'static [Run]),
    H3(&'static [Run]),
    P(&'static [Run]),
    Quote(&'static [Run]),
    Ul(&'static [&'static [Run]]),
    Ol(&'static [&'static [Run]]),
    Code(&'static str),
    Hr,
    Table(&'static [&'static [&'static [Run]]]),
}

#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub label: &'static str,
    pub label_fa: &'static str,
    pub kind: &'static str,
    pub resid: &'static str,
    pub argv: &'static str,
    pub args: &'static str,
    pub path: &'static str,
    pub need_folder: bool,
    pub confirm: Option<&'static str>,
    pub copy_ico: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Module {
    pub mid: &'static str,
    pub title: &'static str,
    pub title_fa: &'static str,
    pub cat: &'static str,
    pub tech: &'static str,
    pub tech_fa: &'static str,
    pub blurb_en: &'static str,
    pub blurb_fa: &'static str,
    pub desc_official: &'static str,
    pub accent: [u8; 3],
    pub url: &'static str,
    pub acts: &'static [Action],
}

#[derive(Clone, Copy, Debug)]
pub struct Str {
    pub key: &'static str,
    pub en: &'static str,
    pub fa: &'static str,
}

pub const GITHUB: &str = "https://github.com/Polaris-Void";

pub const TABS: [(&str, &str, &str); 9] = [
    ("ALL", "ALL", "همه"),
    ("NET", "NET", "شبکه"),
    ("BACKUP", "BACKUP", "پشتیبان"),
    ("DESKTOP", "DESKTOP", "دسکتاپ"),
    ("DRIVERS", "DRIVERS", "درایور"),
    ("USB", "USB", "فلش"),
    ("FILES", "FILES", "فایل"),
    ("AUDIT", "AUDIT", "بازرسی"),
    ("SYS", "SYS", "سیستم"),
];

pub static MODULES: [Module; 12] = [
    Module {
        mid: "NCSI-FIX",
        title: "NCSI No-Internet Fix",
        title_fa: "رفع خطای کاذب اینترنت",
        cat: "NET",
        tech: "registry tweak + NlaSvc restart",
        tech_fa: "تغییر رجیستری + ری‌استارت NlaSvc",
        blurb_en: "Fix false \"No Internet\" alerts (globe icon / yellow bang while the web works), kill surprise captive-portal popups, and control Microsoft active probing telemetry.",
        blurb_fa: "رفع خطای کاذب No Internet (آیکون کره یا علامت زرد با وجود اینترنت سالم)، توقف پاپ‌آپ‌های captive-portal و کنترل درخواست‌های دوره‌ای مایکروسافت، بدون نیاز به ریبوت.",
        desc_official: "Fix false \"No Internet Access\" alerts and manage Windows NCSI internet probing with a one-click service restarter.",
        accent: [ 0x4d, 0x8d, 0xff ],
        url: "https://github.com/Polaris-Void/Windows-No-Internet-Fix",
        acts: &[
            Action { label: "DISABLE PROBING", label_fa: "غیرفعال‌سازی پروب", kind: "reg", resid: "NCSI_DISABLE", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
            Action { label: "ENABLE PROBING", label_fa: "فعال‌سازی پروب", kind: "reg", resid: "NCSI_ENABLE", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
            Action { label: "RESTART NLA SVC", label_fa: "ری‌استارت سرویس NLA", kind: "script", resid: "NCSI_RESTART", argv: "admin", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "USER-BACKUP",
        title: "User Folders Backup",
        title_fa: "بکاپ پوشه‌های کاربر",
        cat: "BACKUP",
        tech: "robocopy + registry path resolution",
        tech_fa: "Robocopy + خواندن مسیر از رجیستری",
        blurb_en: "Backup Desktop, Downloads, Documents, Pictures, Music, Videos to C:\\User Backup. Resolves real paths from the registry (OneDrive aware). Restore merges back without overwriting anything.",
        blurb_fa: "بکاپ خودکار پوشه‌های کاربر در C:\\User Backup با Robocopy؛ مسیرها از رجیستری خوانده می‌شود (سازگار با OneDrive) و بازیابی هیچ فایلی را بازنویسی نمی‌کند.",
        desc_official: "A lightweight, robust Windows Batch script for automated user shell folder backup using dynamic registry path resolution and optimized Robocopy.",
        accent: [ 0xff, 0x5c, 0x47 ],
        url: "https://github.com/Polaris-Void/Win-User-Backup",
        acts: &[
            Action { label: "BACKUP NOW", label_fa: "شروع بکاپ", kind: "script", resid: "USERBACKUP", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
            Action { label: "RESTORE (MERGE)", label_fa: "بازیابی (ادغام)", kind: "script", resid: "USERRESTORE", argv: "", args: "", path: "", need_folder: false, confirm: Some("restore"), copy_ico: false },
            Action { label: "VIEW VAULT", label_fa: "مشاهده صندوق", kind: "vault", resid: "", argv: "", args: "", path: "C:\\User Backup", need_folder: false, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "SPOTLIGHT",
        title: "Spotlight Icon Toggle",
        title_fa: "مدیریت آیکون Spotlight",
        cat: "DESKTOP",
        tech: "HKCU registry tweak",
        tech_fa: "تغییر رجیستری HKCU",
        blurb_en: "Remove the annoying \"Learn about this picture\" desktop icon while keeping Spotlight wallpapers alive. One click to hide, one click to bring it back.",
        blurb_fa: "حذف آیکون مزاحم «Learn about this picture» از دسکتاپ بدون غیرفعال‌کردن والپیپر Spotlight؛ مخفی یا نمایش با یک کلیک.",
        desc_official: "Clean up your Windows desktop by removing the \"Learn about this picture\" icon without disabling Spotlight wallpapers.",
        accent: [ 0x9b, 0x6b, 0xff ],
        url: "https://github.com/Polaris-Void/Win-Spotlight-Icon-Toggle",
        acts: &[
            Action { label: "HIDE ICON", label_fa: "مخفی‌کردن آیکون", kind: "reg", resid: "SPOT_HIDE", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
            Action { label: "SHOW ICON", label_fa: "نمایش آیکون", kind: "reg", resid: "SPOT_SHOW", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
            Action { label: "RESTART EXPLORER", label_fa: "ری‌استارت اکسپلورر", kind: "cmd", resid: "", argv: "", args: "taskkill /F /IM explorer.exe & timeout /t 1 >nul & start explorer.exe", path: "", need_folder: false, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "DRV-BACKUP",
        title: "OEM Driver Backup",
        title_fa: "بکاپ درایورهای OEM",
        cat: "DRIVERS",
        tech: "DISM + PnPUtil, zero dependencies",
        tech_fa: "DISM و PnPUtil، بدون وابستگی",
        blurb_en: "Export every third-party OEM driver to a vault, and reinstall them in bulk later. The lifesaver you run before reinstalling Windows.",
        blurb_fa: "بکاپ کامل درایورهای OEM با DISM و نصب گروهی آن‌ها هنگام بازیابی؛ نجات‌دهنده قبل از تعویض ویندوز، بدون هیچ وابستگی.",
        desc_official: "Lightweight, zero-dependency Windows Batch utility to back up and restore OEM drivers natively using DISM and PnPUtil.",
        accent: [ 0xff, 0xc1, 0x4d ],
        url: "https://github.com/Polaris-Void/Win-Drv-Backup",
        acts: &[
            Action { label: "BACKUP ALL OEM", label_fa: "بکاپ همه OEM", kind: "script", resid: "DRVBACKUP", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
            Action { label: "RESTORE DRIVERS", label_fa: "بازیابی درایورها", kind: "script", resid: "DRVRESTORE", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
            Action { label: "VIEW VAULT", label_fa: "مشاهده صندوق", kind: "vault", resid: "", argv: "", args: "", path: "%SystemDrive%\\Backup_Driver", need_folder: false, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "DISP-BACKUP",
        title: "Display Driver Backup",
        title_fa: "بکاپ درایور گرافیک",
        cat: "DRIVERS",
        tech: "Get-WindowsDriver + PnPUtil export",
        tech_fa: "Get-WindowsDriver و خروجی PnPUtil",
        blurb_en: "Surgical backup/restore of GPU display-class drivers only. Filters the Display class GUID and exports matching oem*.inf packages.",
        blurb_fa: "بکاپ و بازیابی تخصصی درایور گرافیک (فقط کلاس Display) با فیلتر GUID و خروجی PnPUtil؛ سریع و تمیز.",
        desc_official: "One-click Windows batch scripts to back up and restore third-party display (GPU) drivers using PnPUtil. Auto-elevates, no dependencies.",
        accent: [ 0x2d, 0xd4, 0xbf ],
        url: "https://github.com/Polaris-Void/Win-Display-Drv-Backup",
        acts: &[
            Action { label: "BACKUP DISPLAY", label_fa: "بکاپ گرافیک", kind: "script", resid: "DISPBACKUP", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
            Action { label: "RESTORE DISPLAY", label_fa: "بازیابی گرافیک", kind: "script", resid: "DISPRESTORE", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
            Action { label: "VIEW VAULT", label_fa: "مشاهده صندوق", kind: "vault", resid: "", argv: "", args: "", path: "%SystemDrive%\\Backup_Display_Driver", need_folder: false, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "USB-SHIELD",
        title: "USB Immunizer",
        title_fa: "ایمن‌سازی فلش USB",
        cat: "USB",
        tech: "decoy files + hidden/system attributes",
        tech_fa: "فایل طعمه + اتریبیوت مخفی و سیستمی",
        blurb_en: "Immunize a USB stick against auto-created junk (Android, System Volume Information) using hidden decoy files, and brand it with a custom icon + label. Pick the drive root first.",
        blurb_fa: "ایمن‌سازی فلش USB در برابر پوشه‌های زائد خودکار با فایل‌طعمه Hidden/System و برندسازی آیکون و لیبل درایو؛ ابتدا ریشه درایو را انتخاب کن.",
        desc_official: "A Windows batch utility to immunize USB drives against auto-created junk folders and customize drive branding with custom icons.",
        accent: [ 0xff, 0x5c, 0xa8 ],
        url: "https://github.com/Polaris-Void/USB-Immunizer-Customizer",
        acts: &[
            Action { label: "IMMUNIZE TARGET", label_fa: "ایمن‌سازی هدف", kind: "script", resid: "USB_TOOL", argv: "", args: "", path: "", need_folder: true, confirm: None, copy_ico: true },
        ],
    },
    Module {
        mid: "DNS-RESET",
        title: "DNS + Proxy Reset",
        title_fa: "ریست DNS و پروکسی",
        cat: "NET",
        tech: "NetAdapter reset + proxy off + flushdns",
        tech_fa: "ریست آداپتور + قطع پروکسی + flushdns",
        blurb_en: "Reset DNS to automatic (DHCP) on all adapters, disable system proxy (HKCU+HKLM), and flush the resolver cache. First aid for \"connected but nothing loads\".",
        blurb_fa: "ریست DNS به حالت خودکار (DHCP)، غیرفعال‌سازی پروکسی سیستم و Flush کش DNS؛ کمک‌های اولیه برای «وصلم ولی هیچی باز نمی‌شود».",
        desc_official: "A lightweight Windows Batch script to reset DNS settings to automatic (DHCP), disable system proxies, and flush DNS cache to resolve connectivity issues.",
        accent: [ 0x51, 0xd8, 0x8a ],
        url: "https://github.com/Polaris-Void/Reset-DNS-Proxy",
        acts: &[
            Action { label: "RESET DNS+PROXY", label_fa: "ریست DNS و پروکسی", kind: "script", resid: "DNSRESET", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "MEDIA-RENAME",
        title: "Image + Video Renamer",
        title_fa: "تغییرنام عکس و ویدیو",
        cat: "FILES",
        tech: "recursive batch rename, collision-safe",
        tech_fa: "تغییرنام بازگشتی، ضد تداخل",
        blurb_en: "Bulk-rename 50+ image formats to .jpg and 40+ video formats to .mp4, recursively. Name clashes get auto (1), (2) suffixes - nothing is ever overwritten or deleted.",
        blurb_fa: "تغییرنام گروهی بازگشتی: ده‌ها فرمت عکس به jpg. و ویدیو به mp4.؛ تداخل نام با پسوند خودکار حل می‌شود و هیچ فایلی بازنویسی یا حذف نمی‌شود.",
        desc_official: "Bulk rename 50+ image formats to .jpg and 40+ video formats to .mp4 recursively with collision protection.",
        accent: [ 0x34, 0xd3, 0x99 ],
        url: "https://github.com/Polaris-Void/Image-Video-Renamer",
        acts: &[
            Action { label: "IMAGES -> .JPG", label_fa: "عکس‌ها به JPG", kind: "script", resid: "IMGREN", argv: "", args: "", path: "", need_folder: true, confirm: None, copy_ico: false },
            Action { label: "VIDEOS -> .MP4", label_fa: "ویدیوها به MP4", kind: "script", resid: "VIDREN", argv: "", args: "", path: "", need_folder: true, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "TREE-EXPORT",
        title: "Directory Tree Exporter",
        title_fa: "خروجی درخت پوشه‌ها",
        cat: "AUDIT",
        tech: "batch + PowerShell, UTF-8 report",
        tech_fa: "بچ + PowerShell، گزارش UTF-8",
        blurb_en: "Map a folder into a visual tree report with sizes and creation/modification dates. Wide UTF-8 table, self-excluding log, stats summary included.",
        blurb_fa: "تولید گزارش درختی دایرکتوری با حجم و تاریخ ساخت/تغییر در فایل UTF-8؛ مناسب مستندسازی پروژه و آرشیو.",
        desc_official: "A lightweight Windows Batch and PowerShell script to generate and export recursive visual directory tree reports in UTF-8 encoding.",
        accent: [ 0x63, 0x66, 0xf1 ],
        url: "https://github.com/Polaris-Void/Directory-Tree-Exporter",
        acts: &[
            Action { label: "EXPORT TREE", label_fa: "خروجی درخت", kind: "script", resid: "TREE", argv: "", args: "", path: "", need_folder: true, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "SHA256-AUDIT",
        title: "SHA-256 Tree Auditor",
        title_fa: "ممیزی درخت SHA-256",
        cat: "AUDIT",
        tech: "Get-FileHash per file, tamper evidence",
        tech_fa: "هش SHA-256 تک‌تک فایل‌ها",
        blurb_en: "Everything Tree Exporter does, plus a SHA-256 checksum for every file. Prove integrity, detect corruption or tampering. Locked files are tagged, never fatal.",
        blurb_fa: "ممیزی دقیق: درخت دایرکتوری + متادیتا + هش SHA-256 تک‌تک فایل‌ها برای اثبات یکپارچگی و تشخیص دستکاری یا خرابی.",
        desc_official: "High-precision Windows utility to map directory trees, extract metadata, and compute SHA-256 hashes.",
        accent: [ 0x22, 0xd3, 0xee ],
        url: "https://github.com/Polaris-Void/Directory-SHA256-Tree-Exporter",
        acts: &[
            Action { label: "RUN SHA-256 AUDIT", label_fa: "اجرای ممیزی SHA-256", kind: "script", resid: "SHA256", argv: "", args: "", path: "", need_folder: true, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "UNPACK",
        title: "Subfolder Unpacker",
        title_fa: "تجمیع ساب‌فولدرها",
        cat: "FILES",
        tech: "deep recursive move-to-parent",
        tech_fa: "انتقال بازگشتی به پوشه والد",
        blurb_en: "Flatten chaos: move every file from all nested subfolders straight into the parent folder. Perfect cleanup after extracting archives or multi-folder downloads.",
        blurb_fa: "صاف‌کردن پوشه‌ها: انتقال همه فایل‌ها از ساب‌فولدرهای تودرتو به پوشه اصلی؛ عالی برای تمیزکاری بعد از اکسترکت آرشیوها.",
        desc_official: "Move all files from nested subdirectories to the parent folder in Windows.",
        accent: [ 0xff, 0x8c, 0x42 ],
        url: "https://github.com/Polaris-Void/Unpack-Subfolders",
        acts: &[
            Action { label: "UNPACK TO PARENT", label_fa: "انتقال به پوشه اصلی", kind: "script", resid: "UNPACK", argv: "", args: "", path: "", need_folder: true, confirm: None, copy_ico: false },
        ],
    },
    Module {
        mid: "EXPLORER-TWEAK",
        title: "File Explorer Tweaker",
        title_fa: "تنظیم فایل اکسپلورر",
        cat: "SYS",
        tech: "8 HKCU values + explorer self-restart",
        tech_fa: "۸ مقدار رجیستری HKCU + ری‌استارت اکسپلورر",
        blurb_en: "One click makes File Explorer open on This PC, kills recent/frequent/cloud suggestions, and enables item check boxes, file name extensions and hidden items. Per-user (HKCU) only, navigation pane untouched, Explorer restarts itself.",
        blurb_fa: "پیکربندی یک‌کلیکی فایل اکسپلورر: باز شدن روی This PC، قطع پیشنهادهای فایل‌های اخیر/مکرر/ابر و فعال‌سازی جعبه‌های تیک، پسوند فایل‌ها و موارد پنهان. فقط سطح کاربر (HKCU)، نوار ناوبری دست‌نخورده و اکسپلورر خودش ری‌استارت می‌شود.",
        desc_official: "A lightweight, self-elevating Windows Batch script to optimize File Explorer settings for privacy and power users via native registry tweaks.",
        accent: [ 0x3b, 0x82, 0xf6 ],
        url: "https://github.com/Polaris-Void/Explorer-Settings-Tweaker",
        acts: &[
            Action { label: "APPLY SETTINGS", label_fa: "اعمال تنظیمات اکسپلورر", kind: "script", resid: "EXPLORER_SETTINGS", argv: "", args: "", path: "", need_folder: false, confirm: None, copy_ico: false },
        ],
    },
];

pub static STRINGS: &[Str] = &[
    Str {
        key: "subtitle",
        en: "Built for Windows 10 & 11 — best experienced on Windows 10 and 11",
        fa: "برای ویندوز ۱۰ و ۱۱ ساخته شده — بهترین تجربه در ویندوز ۱۰ و ۱۱",
    },
    Str {
        key: "tools",
        en: "TOOLS",
        fa: "ابزار",
    },
    Str {
        key: "modules",
        en: "MODULES",
        fa: "ماژول‌ها",
    },
    Str {
        key: "all",
        en: "ALL",
        fa: "همه",
    },
    Str {
        key: "open",
        en: "OPEN",
        fa: "باز کردن",
    },
    Str {
        key: "back_all",
        en: "‹ ALL MODULES",
        fa: "› همه ابزارها",
    },
    Str {
        key: "log",
        en: "LOG",
        fa: "لاگ",
    },
    Str {
        key: "log_title",
        en: "LIVE LOG",
        fa: "لاگ زنده",
    },
    Str {
        key: "search_ph",
        en: "Search tools...",
        fa: "جست‌وجوی ابزار…",
    },
    Str {
        key: "sort_az",
        en: "A – Z",
        fa: "الفبا",
    },
    Str {
        key: "sort_order",
        en: "01 – 12",
        fa: "ترتیب",
    },
    Str {
        key: "footer_os",
        en: "WINDOWS 10 / 11 · X64",
        fa: "ویندوز ۱۰ / ۱۱ · X64",
    },
    Str {
        key: "open_project",
        en: "OPEN PROJECT",
        fa: "باز کردن پروژه",
    },
    Str {
        key: "actions",
        en: "ACTIONS",
        fa: "عملیات",
    },
    Str {
        key: "target",
        en: "TARGET FOLDER:",
        fa: "پوشه هدف:",
    },
    Str {
        key: "browse",
        en: "BROWSE...",
        fa: "انتخاب...",
    },
    Str {
        key: "notarget",
        en: "(no target selected)",
        fa: "(هدفی انتخاب نشده)",
    },
    Str {
        key: "export",
        en: "EXPORT",
        fa: "ذخیره گزارش",
    },
    Str {
        key: "clear",
        en: "CLEAR",
        fa: "پاک‌سازی",
    },
    Str {
        key: "stop",
        en: "STOP",
        fa: "توقف",
    },
    Str {
        key: "github",
        en: "GitHub",
        fa: "گیت‌هاب",
    },
    Str {
        key: "lang",
        en: "فارسی",
        fa: "EN",
    },
    Str {
        key: "panel_title",
        en: "TOOL DETAILS",
        fa: "جزئیات ابزار",
    },
    Str {
        key: "readme",
        en: "README",
        fa: "راهنما",
    },
    Str {
        key: "st_module",
        en: "module",
        fa: "ماژول",
    },
    Str {
        key: "st_of",
        en: "of",
        fa: "از",
    },
    Str {
        key: "st_target",
        en: "target",
        fa: "هدف",
    },
    Str {
        key: "vault_title",
        en: "VAULT INSPECTOR",
        fa: "محتوای صندوق",
    },
    Str {
        key: "vault_copy",
        en: "COPY PATH",
        fa: "کپی مسیر",
    },
    Str {
        key: "vault_refresh",
        en: "REFRESH",
        fa: "به‌روزرسانی",
    },
    Str {
        key: "vault_close",
        en: "CLOSE",
        fa: "بستن",
    },
    Str {
        key: "confirm_title",
        en: "CONFIRM ACTION",
        fa: "تأیید عملیات",
    },
    Str {
        key: "yes",
        en: "YES, RUN IT",
        fa: "بله، اجرا کن",
    },
    Str {
        key: "no",
        en: "CANCEL",
        fa: "انصراف",
    },
    Str {
        key: "category",
        en: "CATEGORY",
        fa: "دسته‌بندی",
    },
    Str {
        key: "action",
        en: "ACTION",
        fa: "عملیات",
    },
    Str {
        key: "no_match",
        en: "no tools match",
        fa: "ابزاری یافت نشد",
    },
    Str {
        key: "empty",
        en: "(empty)",
        fa: "(خالی)",
    },
    Str {
        key: "scanning",
        en: "(scanning vault...)",
        fa: "(در حال اسکن صندوق…)",
    },
    Str {
        key: "running_hint",
        en: "RUNNING — watch the LOG (top bar) · STOP halts it.",
        fa: "در حال اجرا — لاگ (نوار بالا) را دنبال کن؛ دکمهٔ «توقف» متوقفش می‌کند.",
    },
];

static EMPTY_STR: Str = Str {
    key: "",
    en: "",
    fa: "",
};
pub fn t(key: &str) -> &'static Str {
    STRINGS
        .iter()
        .find(|entry| entry.key == key)
        .unwrap_or(&EMPTY_STR)
}

pub const CONFIRM: &[(&str, &str, &str)] = &[
    ("restore", "Merge missing files from C:\\User Backup back into your profile?\nExisting files are NEVER overwritten or deleted.", "فایل‌های گمشده از C:\\User Backup به پروفایل برگردانده شود؟\nفایل‌های موجود هرگز بازنویسی یا حذف نمی‌شوند."),
];
