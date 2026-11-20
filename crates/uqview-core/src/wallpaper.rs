//! "Set as wallpaper": on uQ, through `uqctl wallpaper set`, which stores it in uQ's settings
//! (so Settings › Wallpaper shows it and it survives a restart).

use std::path::{Path, PathBuf};

/// `uqctl` on PATH, if uQ is installed.
pub fn uqctl() -> Option<PathBuf> {
    which("uqctl")
}

/// The command that makes `image` the wallpaper on every display.
pub fn command(uqctl: &Path, image: &Path) -> Vec<String> {
    vec![
        uqctl.to_string_lossy().into_owned(),
        "wallpaper".into(),
        "set".into(),
        "--".into(),
        image.to_string_lossy().into_owned(),
    ]
}

/// A program on PATH.
pub fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(name))
        .find(|p| is_executable(p))
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_the_command() {
        let c = command(
            Path::new("/usr/bin/uqctl"),
            Path::new("/home/u/-odd name.png"),
        );
        assert_eq!(
            c,
            [
                "/usr/bin/uqctl",
                "wallpaper",
                "set",
                "--",
                "/home/u/-odd name.png"
            ]
        );
    }
}
