//! "Open pictures with Qview": the picture types qview's desktop entry lists, and the
//! `xdg-mime` commands that check and set it as their default app.

pub const DESKTOP_ID: &str = "io.github.asafelobotomy.qview.desktop";

/// Every type in the desktop entry's MimeType (packaging/desktop), in that order. The first
/// two decide whether qview counts as the default.
pub const MIMES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/bmp",
    "image/tiff",
    "image/svg+xml",
    "image/avif",
    "image/heif",
    "image/heic",
    "image/jxl",
    "image/apng",
    "image/x-icon",
    "image/vnd.microsoft.icon",
    "image/x-tga",
    "image/x-portable-anymap",
    "image/x-portable-bitmap",
    "image/x-portable-graymap",
    "image/x-portable-pixmap",
    "image/x-xbitmap",
    "image/x-xpixmap",
    "image/x-exr",
    "image/x-dds",
    "image/qoi",
    "image/jp2",
    "image/vnd.adobe.photoshop",
    "image/x-xcf",
];

/// `xdg-mime default <qview> <every type>`.
pub fn set_argv() -> Vec<String> {
    let mut v = vec!["xdg-mime".to_string(), "default".into(), DESKTOP_ID.into()];
    v.extend(MIMES.iter().map(|m| m.to_string()));
    v
}

/// Whether the answers of `xdg-mime query default` for PNG and JPEG name qview.
pub fn is_default(png: &str, jpeg: &str) -> bool {
    png.trim() == DESKTOP_ID && jpeg.trim() == DESKTOP_ID
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_entry_lists_every_type() {
        let entry =
            include_str!("../../../packaging/desktop/io.github.asafelobotomy.qview.desktop");
        let line = entry.lines().find(|l| l.starts_with("MimeType=")).unwrap();
        let listed: Vec<&str> = line["MimeType=".len()..]
            .split(';')
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(listed, MIMES);
        assert_eq!(&set_argv()[..3], ["xdg-mime", "default", DESKTOP_ID]);
        assert!(is_default(&format!("{DESKTOP_ID}\n"), DESKTOP_ID));
        assert!(!is_default("org.kde.gwenview.desktop", DESKTOP_ID));
    }
}
