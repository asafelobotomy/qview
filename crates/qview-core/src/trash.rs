//! Moving a picture to the trash (the freedesktop.org trash in your home, the one file managers
//! show), and putting it back. Only same-filesystem moves are done here; for a picture on
//! another drive [`trash`] says so and the caller falls back to `gio trash`.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A picture in the trash, enough to put it back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trashed {
    pub original: PathBuf,
    pub file: PathBuf,
    pub info: PathBuf,
}

#[derive(Debug)]
pub enum TrashError {
    /// The picture is on another filesystem than the home trash.
    OtherDrive,
    Io(io::Error),
}

impl std::fmt::Display for TrashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OtherDrive => write!(f, "the picture is on another drive"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for TrashError {}

/// `$XDG_DATA_HOME/Trash`, else `~/.local/share/Trash`.
pub fn home_trash() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(data.join("Trash"))
}

/// Move `file` into `trash` (a trash folder with `files/` and `info/`).
pub fn trash_into(file: &Path, trash: &Path) -> Result<Trashed, TrashError> {
    let files = trash.join("files");
    let info_dir = trash.join("info");
    std::fs::create_dir_all(&files).map_err(TrashError::Io)?;
    std::fs::create_dir_all(&info_dir).map_err(TrashError::Io)?;
    let name = file
        .file_name()
        .ok_or_else(|| TrashError::Io(io::Error::other("not a file")))?
        .to_string_lossy()
        .into_owned();
    let (stem, ext) = match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i..]),
        _ => (name.as_str(), ""),
    };
    // The info file is created first and exclusively, so two trashings never pick one name.
    for n in 1..10_000 {
        let candidate = if n == 1 {
            name.clone()
        } else {
            format!("{stem}.{n}{ext}")
        };
        let info = info_dir.join(format!("{candidate}.trashinfo"));
        let target = files.join(&candidate);
        if target.exists() {
            continue;
        }
        let mut f = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&info)
        {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(TrashError::Io(e)),
        };
        use std::io::Write;
        let body = format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            encode_path(file),
            deletion_date(SystemTime::now())
        );
        if let Err(e) = f.write_all(body.as_bytes()) {
            let _ = std::fs::remove_file(&info);
            return Err(TrashError::Io(e));
        }
        return match std::fs::rename(file, &target) {
            Ok(()) => Ok(Trashed {
                original: file.to_path_buf(),
                file: target,
                info,
            }),
            Err(e) => {
                let _ = std::fs::remove_file(&info);
                if e.raw_os_error() == Some(18) {
                    Err(TrashError::OtherDrive)
                } else {
                    Err(TrashError::Io(e))
                }
            }
        };
    }
    Err(TrashError::Io(io::Error::other(
        "the trash has too many files with this name",
    )))
}

/// Move `file` to the home trash.
pub fn trash(file: &Path) -> Result<Trashed, TrashError> {
    let trash = home_trash().ok_or_else(|| TrashError::Io(io::Error::other("no home folder")))?;
    trash_into(file, &trash)
}

/// Put a trashed picture back where it was. Refuses if something else is there now.
pub fn restore(t: &Trashed) -> io::Result<()> {
    if t.original.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "a file with that name is there now",
        ));
    }
    std::fs::rename(&t.file, &t.original)?;
    let _ = std::fs::remove_file(&t.info);
    Ok(())
}

/// The spec's `Path=`: an absolute path, percent-encoded except for unreserved characters and `/`.
fn encode_path(p: &Path) -> String {
    use std::os::unix::ffi::OsStrExt;
    let mut out = String::new();
    for &b in p.as_os_str().as_bytes() {
        if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `YYYY-MM-DDThh:mm:ss` in UTC (the spec asks for local time; file managers accept either).
fn deletion_date(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn trash_and_restore() {
        let t = tempfile::tempdir().unwrap();
        let pic = t.path().join("my pic.png");
        std::fs::write(&pic, "x").unwrap();
        let trash = t.path().join("Trash");
        let a = trash_into(&pic, &trash).unwrap();
        assert!(!pic.exists() && a.file.exists());
        let info = std::fs::read_to_string(&a.info).unwrap();
        assert!(info.contains("my%20pic.png"), "{info}");
        // A second picture of the same name gets its own slot.
        std::fs::write(&pic, "y").unwrap();
        let b = trash_into(&pic, &trash).unwrap();
        assert!(b.file.ends_with("my pic.2.png"), "{:?}", b.file);
        restore(&b).unwrap();
        assert_eq!(std::fs::read_to_string(&pic).unwrap(), "y");
        assert!(!b.info.exists());
        // The first can't come back over it.
        assert!(restore(&a).is_err());
    }

    #[test]
    fn dates() {
        assert_eq!(deletion_date(UNIX_EPOCH), "1970-01-01T00:00:00");
        let t = UNIX_EPOCH + Duration::from_secs(1_795_185_144);
        assert_eq!(deletion_date(t), "2026-11-20T14:32:24");
    }
}
