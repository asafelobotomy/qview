//! Where a marked-up picture is saved.

use std::path::{Path, PathBuf};

/// A new file beside `src` for its marked-up copy: `name-marked.png`, then `name-marked-2.png`
/// and so on, never an existing file. Always PNG, so the marks stay sharp.
pub fn copy_path(src: &Path) -> PathBuf {
    let dir = src.parent().unwrap_or(Path::new("."));
    let stem = src
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "picture".into());
    // Marking up a marked copy again doesn't stack "-marked-marked".
    let base = strip_marked(&stem);
    (1..)
        .map(|n| {
            if n == 1 {
                dir.join(format!("{base}-marked.png"))
            } else {
                dir.join(format!("{base}-marked-{n}.png"))
            }
        })
        .find(|p| !p.exists())
        .expect("an unused name exists")
}

fn strip_marked(stem: &str) -> &str {
    if let Some(i) = stem.rfind("-marked") {
        let rest = &stem[i + "-marked".len()..];
        if rest.is_empty()
            || (rest.starts_with('-')
                && rest[1..].bytes().all(|b| b.is_ascii_digit())
                && rest.len() > 1)
        {
            return &stem[..i];
        }
    }
    stem
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_names() {
        let t = tempfile::tempdir().unwrap();
        let src = t.path().join("shot.jpg");
        assert_eq!(copy_path(&src), t.path().join("shot-marked.png"));
        std::fs::write(t.path().join("shot-marked.png"), "").unwrap();
        assert_eq!(copy_path(&src), t.path().join("shot-marked-2.png"));
        assert_eq!(
            copy_path(&t.path().join("shot-marked.png")),
            t.path().join("shot-marked-2.png")
        );
        assert_eq!(
            copy_path(&t.path().join("a-marked-x.png")),
            t.path().join("a-marked-x-marked.png")
        );
    }
}
