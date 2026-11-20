//! The images next to the one that was opened, in the order the viewer steps through them.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::config::Sort;

/// Compare names the way people read them: case-insensitively, with runs of digits compared as
/// numbers, so `img2.png` comes before `img10.png`.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let na = take_digits(&mut a);
                let nb = take_digits(&mut b);
                // Compare by value: strip leading zeros, then length, then digits.
                let (ta, tb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
                let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
                if ord != Ordering::Equal {
                    return ord;
                }
                let ord = na.len().cmp(&nb.len());
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(x), Some(y)) => {
                let ord = x
                    .to_lowercase()
                    .cmp(y.to_lowercase())
                    .then_with(|| x.cmp(&y));
                if ord != Ordering::Equal {
                    return ord;
                }
                a.next();
                b.next();
            }
        }
    }
}

fn take_digits(it: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut s = String::new();
    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
        s.push(c);
        it.next();
    }
    s
}

/// The lower-case extension of `p`, if it has one.
pub fn extension(p: &Path) -> Option<String> {
    p.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
}

/// Whether `p` looks like an image this viewer can show (by extension).
pub fn is_image(p: &Path, exts: &HashSet<String>) -> bool {
    extension(p).is_some_and(|e| exts.contains(&e))
}

struct Entry {
    path: PathBuf,
    name: String,
    modified: SystemTime,
    size: u64,
}

/// Every image in `dir` (not hidden ones, not in subfolders), sorted.
pub fn list(
    dir: &Path,
    exts: &HashSet<String>,
    sort: Sort,
    descending: bool,
) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for e in std::fs::read_dir(dir)? {
        let Ok(e) = e else { continue };
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || !is_image(&path, exts) {
            continue;
        }
        // Follows symlinks, so a link to an image counts and a link to a folder doesn't.
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        entries.push(Entry {
            path,
            name,
            modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            size: meta.len(),
        });
    }
    entries.sort_by(|a, b| {
        let ord = match sort {
            Sort::Name => natural_cmp(&a.name, &b.name),
            Sort::Date => a.modified.cmp(&b.modified),
            Sort::Size => a.size.cmp(&b.size),
        };
        let ord = ord.then_with(|| natural_cmp(&a.name, &b.name));
        if descending { ord.reverse() } else { ord }
    });
    Ok(entries.into_iter().map(|e| e.path).collect())
}

/// The images to step through when `file` is opened, and where `file` is among them. A file
/// whose extension isn't a known image one is still shown (Qt may read it by its contents).
pub fn around(
    file: &Path,
    exts: &HashSet<String>,
    sort: Sort,
    descending: bool,
) -> (Vec<PathBuf>, usize) {
    let dir = file
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut files = list(dir, exts, sort, descending).unwrap_or_default();
    match files.iter().position(|p| p == file) {
        Some(i) => (files, i),
        None => {
            files.insert(0, file.to_path_buf());
            (files, 0)
        }
    }
}

/// What a command-line argument opens: an image and its folder, or a folder from its first image.
pub fn open(
    arg: &Path,
    exts: &HashSet<String>,
    sort: Sort,
    descending: bool,
) -> (Vec<PathBuf>, usize) {
    let abs = absolute(arg);
    if abs.is_dir() {
        (list(&abs, exts, sort, descending).unwrap_or_default(), 0)
    } else {
        around(&abs, exts, sort, descending)
    }
}

/// `p` made absolute against the current folder, with `.` parts removed.
pub fn absolute(p: &Path) -> PathBuf {
    let joined = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    };
    joined
        .components()
        .filter(|c| !matches!(c, std::path::Component::CurDir))
        .collect()
}

/// A file size the way people say it: "812 B", "4.2 KB", "13.0 MB".
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1000 {
        return format!("{bytes} B");
    }
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1000.0 && u < UNITS.len() - 1 {
        v /= 1000.0;
        u += 1;
    }
    format!("{v:.1} {}", UNITS[u])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exts() -> HashSet<String> {
        ["png", "jpg", "jpeg", "webp"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn natural_order() {
        let mut v = vec![
            "img10.png",
            "img2.png",
            "Img1.png",
            "img02.png",
            "b.png",
            "A.png",
        ];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(
            v,
            [
                "A.png",
                "b.png",
                "Img1.png",
                "img2.png",
                "img02.png",
                "img10.png"
            ]
        );
        assert_eq!(natural_cmp("a", "a"), Ordering::Equal);
        assert_eq!(natural_cmp("a1b", "a1"), Ordering::Greater);
        assert_eq!(
            natural_cmp("x99999999999999999999999", "x100000000000000000000000"),
            Ordering::Less
        );
    }

    #[test]
    fn lists_images_only_sorted() {
        let t = tempfile::tempdir().unwrap();
        for n in [
            "b10.png",
            "b2.JPG",
            "a.webp",
            ".hidden.png",
            "notes.txt",
            "c.png",
        ] {
            std::fs::write(t.path().join(n), n).unwrap();
        }
        std::fs::create_dir(t.path().join("dir.png")).unwrap();
        let names = |v: Vec<PathBuf>| -> Vec<String> {
            v.iter()
                .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
                .collect()
        };
        let l = list(t.path(), &exts(), Sort::Name, false).unwrap();
        assert_eq!(names(l), ["a.webp", "b2.JPG", "b10.png", "c.png"]);
        let l = list(t.path(), &exts(), Sort::Name, true).unwrap();
        assert_eq!(names(l), ["c.png", "b10.png", "b2.JPG", "a.webp"]);
        let l = list(t.path(), &exts(), Sort::Size, false).unwrap();
        assert_eq!(names(l)[0], "c.png");
    }

    #[test]
    fn around_finds_the_file_or_keeps_it_first() {
        let t = tempfile::tempdir().unwrap();
        for n in ["1.png", "2.png", "3.png", "odd.xyz"] {
            std::fs::write(t.path().join(n), n).unwrap();
        }
        let (files, i) = around(&t.path().join("2.png"), &exts(), Sort::Name, false);
        assert_eq!((files.len(), i), (3, 1));
        let (files, i) = around(&t.path().join("odd.xyz"), &exts(), Sort::Name, false);
        assert_eq!((files.len(), i), (4, 0));
        assert!(files[0].ends_with("odd.xyz"));
        let (files, i) = open(t.path(), &exts(), Sort::Name, false);
        assert_eq!((files.len(), i), (3, 0));
    }

    #[test]
    fn sizes() {
        assert_eq!(human_size(812), "812 B");
        assert_eq!(human_size(4200), "4.2 KB");
        assert_eq!(human_size(13_000_000), "13.0 MB");
    }
}
