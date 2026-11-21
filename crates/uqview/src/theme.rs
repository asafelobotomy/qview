//! uQ's theme tokens for the QML `Theme` singleton, from the vendored uq-theme module
//! (`uq_theme.rs`, copied unchanged from uQ's `native/uq-theme`; CI checks it with
//! `scripts/check-uq-theme.sh`). Without uQ, `present` is false and the window follows the desktop.
//! The file is followed with inotify, not polled.

use std::path::PathBuf;

pub use crate::uq_theme::{load, watch};

/// `$UQVIEW_UQ_TOKENS` (tests and screenshots), else uQ's state directory.
pub fn tokens_path() -> Option<PathBuf> {
    crate::uq_theme::tokens_path("UQVIEW_UQ_TOKENS")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_file_means_no_uq() {
        assert_eq!(load(None)["present"], false);
    }
}
