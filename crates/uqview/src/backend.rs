//! The QML-facing `Backend`: the list of pictures and which one is shown, the settings, the theme,
//! and the actions (trash and undo, copy, set as wallpaper, save a mark-up, show in folder).
//! Drawing and decoding stay in QML and Qt; the rules live in `uqview-core`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use serde_json::json;
use uqview_core::config::{self, Config, MarkupSave};
use uqview_core::trash::{self, TrashError, Trashed};
use uqview_core::{folder, markup, wallpaper};

use crate::app::ffi;
use crate::theme;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, theme_json)]
        #[qproperty(QString, config_json)]
        /// The pictures to step through, as a JSON array of absolute paths.
        #[qproperty(QString, files_json)]
        #[qproperty(i32, index)]
        /// Started with `--markup`: open straight in Mark up.
        #[qproperty(bool, start_markup)]
        #[qproperty(bool, can_set_wallpaper)]
        #[qproperty(bool, can_undo)]
        /// The extensions Qt can read, as a JSON array (for the Open dialog's filter).
        #[qproperty(QString, suffixes_json)]
        #[qproperty(QString, version)]
        type Backend = super::BackendRust;

        /// Open these paths (a JSON array): a picture and its folder, or a folder.
        #[qinvokable]
        fn open_json(self: Pin<&mut Backend>, paths: &QString);
        /// List the folder again, staying on the same picture (it may have changed on disk).
        #[qinvokable]
        fn reload(self: Pin<&mut Backend>);
        /// Move a picture to the trash; `notice` says how it went.
        #[qinvokable]
        fn trash(self: Pin<&mut Backend>, path: &QString);
        /// Put the last trashed picture back.
        #[qinvokable]
        fn undo(self: Pin<&mut Backend>);
        #[qinvokable]
        fn copy_image(self: Pin<&mut Backend>, path: &QString) -> bool;
        #[qinvokable]
        fn set_wallpaper(self: Pin<&mut Backend>, path: &QString);
        /// `{width, height, format, frames, animated, size, modified}` (or `{error}`) as JSON.
        #[qinvokable]
        fn info(self: &Backend, path: &QString) -> QString;
        /// Save a mark-up. `mode`: "save" (as the settings say), "copy", "overwrite" or
        /// "clipboard". Answers `{ok, path, error}` as JSON.
        #[qinvokable]
        fn save_markup(
            self: Pin<&mut Backend>,
            src: &QString,
            marks: &QString,
            mode: &QString,
        ) -> QString;
        #[qinvokable]
        fn config_set(self: Pin<&mut Backend>, key: &QString, value: &QString) -> bool;
        #[qinvokable]
        fn show_in_folder(self: &Backend, path: &QString);

        /// `kind` is "ok", "error" or "trashed" (the last offers Undo).
        #[qsignal]
        fn notice(self: Pin<&mut Backend>, kind: QString, text: QString);
    }

    impl cxx_qt::Threading for Backend {}
    impl cxx_qt::Initialize for Backend {}
}

use qobject::Backend;

struct Start {
    paths: Vec<PathBuf>,
    markup: bool,
}

static START: OnceLock<Start> = OnceLock::new();

/// What the command line asked to open; read once by the window's backend.
pub fn set_start(paths: Vec<PathBuf>, markup: bool) {
    let _ = START.set(Start { paths, markup });
}

#[derive(Default)]
pub struct BackendRust {
    theme_json: QString,
    config_json: QString,
    files_json: QString,
    index: i32,
    start_markup: bool,
    can_set_wallpaper: bool,
    can_undo: bool,
    suffixes_json: QString,
    version: QString,

    files: Vec<PathBuf>,
    exts: HashSet<String>,
    config: Config,
    last_trashed: Option<Trashed>,
}

impl cxx_qt::Initialize for Backend {
    fn initialize(mut self: Pin<&mut Self>) {
        let exts: HashSet<String> = ffi::uqview_suffixes()
            .split(',')
            .filter(|s| !s.is_empty() && s.len() <= 8)
            .map(str::to_string)
            .collect();
        let mut sorted: Vec<&String> = exts.iter().collect();
        sorted.sort();
        self.as_mut()
            .set_suffixes_json(QString::from(&json!(sorted).to_string()));
        self.as_mut().rust_mut().exts = exts;
        self.as_mut()
            .set_version(QString::from(env!("CARGO_PKG_VERSION")));
        self.as_mut().load_config();
        self.as_mut()
            .set_can_set_wallpaper(wallpaper::uqctl().is_some());

        let path = theme::tokens_path();
        let tokens = theme::load(path.as_ref());
        apply_icon_theme(&tokens);
        self.as_mut()
            .set_theme_json(QString::from(&tokens.to_string()));
        let qt = self.qt_thread();
        theme::watch(path, move |tokens| {
            qt.queue(move |b| {
                if let Ok(v) = serde_json::from_str(&tokens) {
                    apply_icon_theme(&v);
                }
                b.set_theme_json(QString::from(&tokens))
            })
            .is_ok()
        });

        if let Some(start) = START.get() {
            self.as_mut().set_start_markup(start.markup);
            let paths = start.paths.clone();
            self.as_mut().open_paths(&paths);
        }
    }
}

fn path_of(q: &QString) -> PathBuf {
    PathBuf::from(q.to_string())
}

impl Backend {
    fn load_config(mut self: Pin<&mut Self>) {
        let c = Config::load(config::path().as_deref());
        self.as_mut()
            .set_config_json(QString::from(&c.to_json().to_string()));
        self.as_mut().rust_mut().config = c;
    }

    fn publish(mut self: Pin<&mut Self>, files: Vec<PathBuf>, index: usize) {
        let list: Vec<String> = files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        let index = if files.is_empty() {
            -1
        } else {
            index.min(files.len() - 1) as i32
        };
        self.as_mut().rust_mut().files = files;
        // The list first, so QML sees the new index against the new list.
        self.as_mut()
            .set_files_json(QString::from(&json!(list).to_string()));
        self.as_mut().set_index(index);
    }

    fn open_paths(mut self: Pin<&mut Self>, paths: &[PathBuf]) {
        let (sort, desc) = (self.config.sort, self.config.sort_descending);
        let (files, index) = match paths {
            [] => (Vec::new(), 0),
            [one] => folder::open(one, &self.exts, sort, desc),
            // Several: just those, in the order given.
            many => (
                many.iter()
                    .map(|p| folder::absolute(p))
                    .filter(|p| p.is_file())
                    .collect(),
                0,
            ),
        };
        self.as_mut().publish(files, index);
    }

    fn current(&self) -> Option<PathBuf> {
        usize::try_from(self.index)
            .ok()
            .and_then(|i| self.files.get(i).cloned())
    }

    fn open_json(self: Pin<&mut Self>, paths: &QString) {
        let Ok(list) = serde_json::from_str::<Vec<String>>(&paths.to_string()) else {
            return;
        };
        let paths: Vec<PathBuf> = list
            .into_iter()
            .map(|s| s.strip_prefix("file://").map(decode).unwrap_or(s))
            .map(PathBuf::from)
            .collect();
        self.open_paths(&paths);
    }

    fn reload(mut self: Pin<&mut Self>) {
        let Some(cur) = self.current() else { return };
        // A picture opened on its own (with others given on the command line) keeps its list.
        if self.files.len() > 1 && self.files.iter().any(|f| f.parent() != cur.parent()) {
            return;
        }
        if !cur.exists() {
            return;
        }
        let (sort, desc) = (self.config.sort, self.config.sort_descending);
        let (files, index) = folder::around(&cur, &self.exts, sort, desc);
        if files != self.files {
            self.as_mut().publish(files, index);
        }
    }

    fn trash(mut self: Pin<&mut Self>, path: &QString) {
        let p = path_of(path);
        let name = display_name(&p);
        match trash::trash(&p) {
            Ok(t) => {
                self.as_mut().rust_mut().last_trashed = Some(t);
                self.as_mut().set_can_undo(true);
                self.as_mut().drop_file(&p);
                self.as_mut().notice(
                    QString::from("trashed"),
                    QString::from(&format!("Moved {name} to the trash")),
                );
            }
            Err(TrashError::OtherDrive) => {
                // The drive's own trash: `gio trash` knows where it is. No undo from here.
                let ok = Command::new("gio")
                    .args(["trash", "--"])
                    .arg(&p)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .is_ok_and(|s| s.success());
                if ok {
                    self.as_mut().rust_mut().last_trashed = None;
                    self.as_mut().set_can_undo(false);
                    self.as_mut().drop_file(&p);
                    self.as_mut().notice(
                        QString::from("ok"),
                        QString::from(&format!("Moved {name} to the trash")),
                    );
                } else {
                    self.as_mut().notice(
                        QString::from("error"),
                        QString::from(&format!("Couldn't move {name} to the trash")),
                    );
                }
            }
            Err(e) => self.as_mut().notice(
                QString::from("error"),
                QString::from(&format!("Couldn't move {name} to the trash: {e}")),
            ),
        }
    }

    fn drop_file(mut self: Pin<&mut Self>, p: &Path) {
        let mut files = self.files.clone();
        let Some(i) = files.iter().position(|f| f == p) else {
            return;
        };
        files.remove(i);
        let index = i.min(files.len().saturating_sub(1));
        self.as_mut().publish(files, index);
    }

    fn undo(mut self: Pin<&mut Self>) {
        let Some(t) = self.as_mut().rust_mut().last_trashed.take() else {
            return;
        };
        self.as_mut().set_can_undo(false);
        match trash::restore(&t) {
            Ok(()) => {
                let (sort, desc) = (self.config.sort, self.config.sort_descending);
                let (files, index) = folder::around(&t.original, &self.exts, sort, desc);
                self.as_mut().publish(files, index);
                self.as_mut().notice(
                    QString::from("ok"),
                    QString::from(&format!("Put {} back", display_name(&t.original))),
                );
            }
            Err(e) => self.as_mut().notice(
                QString::from("error"),
                QString::from(&format!("Couldn't put it back: {e}")),
            ),
        }
    }

    fn copy_image(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let ok = ffi::uqview_copy_image(&path.to_string());
        let (kind, text) = if ok {
            ("ok", "Copied to the clipboard")
        } else {
            ("error", "Couldn't copy the picture")
        };
        self.as_mut()
            .notice(QString::from(kind), QString::from(text));
        ok
    }

    fn set_wallpaper(self: Pin<&mut Self>, path: &QString) {
        let p = path_of(path);
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let (kind, text) = match wallpaper::uqctl() {
                None => (
                    "error".to_string(),
                    "Setting the wallpaper needs uQ".to_string(),
                ),
                Some(uqctl) => {
                    let argv = wallpaper::command(&uqctl, &p);
                    match Command::new(&argv[0])
                        .args(&argv[1..])
                        .stdin(Stdio::null())
                        .output()
                    {
                        Ok(o) if o.status.success() => ("ok".into(), "Set as the wallpaper".into()),
                        Ok(o) => {
                            let err = String::from_utf8_lossy(&o.stderr);
                            let err = err.lines().last().unwrap_or("").trim();
                            let why = if err.contains("unrecognized subcommand") {
                                "this needs a newer uQ".to_string()
                            } else if err.is_empty() {
                                "uqctl failed".to_string()
                            } else {
                                err.trim_start_matches("error: ").to_string()
                            };
                            ("error".into(), format!("Couldn't set the wallpaper: {why}"))
                        }
                        Err(e) => ("error".into(), format!("Couldn't set the wallpaper: {e}")),
                    }
                }
            };
            let _ = qt.queue(move |mut b| {
                b.as_mut()
                    .notice(QString::from(&kind), QString::from(&text))
            });
        });
    }

    fn info(&self, path: &QString) -> QString {
        let p = path_of(path);
        let mut v: serde_json::Value =
            serde_json::from_str(&ffi::uqview_image_info(&p.to_string_lossy()))
                .unwrap_or_else(|_| json!({}));
        if let Ok(meta) = std::fs::metadata(&p) {
            v["size"] = json!(folder::human_size(meta.len()));
            if let Ok(m) = meta.modified()
                && let Ok(d) = m.duration_since(std::time::UNIX_EPOCH)
            {
                v["modified"] = json!(d.as_millis() as u64);
            }
        }
        v["name"] = json!(display_name(&p));
        v["folder"] = json!(p.parent().map(|d| d.to_string_lossy().into_owned()));
        QString::from(&v.to_string())
    }

    fn save_markup(
        mut self: Pin<&mut Self>,
        src: &QString,
        marks: &QString,
        mode: &QString,
    ) -> QString {
        let src_path = path_of(src);
        let mode = mode.to_string();
        let mode = match mode.as_str() {
            "save" => match self.config.markup_save {
                MarkupSave::Copy => "copy",
                MarkupSave::Overwrite => "overwrite",
            },
            m => m,
        };
        let dst = match mode {
            "copy" => Some(markup::copy_path(&src_path)),
            "overwrite" => Some(src_path.clone()),
            "clipboard" => None,
            _ => return QString::from(&json!({"ok": false, "error": "unknown mode"}).to_string()),
        };
        let dst_s = dst
            .as_ref()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        let err = ffi::uqview_render_markup(
            &src_path.to_string_lossy(),
            &marks.to_string(),
            &dst_s,
            dst.is_none(),
        );
        if !err.is_empty() {
            return QString::from(&json!({"ok": false, "error": err}).to_string());
        }
        if let Some(d) = &dst {
            // Show the saved picture next.
            let (sort, desc) = (self.config.sort, self.config.sort_descending);
            let (files, index) = folder::around(d, &self.exts, sort, desc);
            self.as_mut().publish(files, index);
        }
        QString::from(&json!({"ok": true, "path": dst_s}).to_string())
    }

    fn config_set(mut self: Pin<&mut Self>, key: &QString, value: &QString) -> bool {
        let Some(path) = config::path() else {
            return false;
        };
        let mut c = self.config.clone();
        if c.set(&key.to_string(), &value.to_string()).is_err() || c.save(&path).is_err() {
            return false;
        }
        let resort = c.sort != self.config.sort || c.sort_descending != self.config.sort_descending;
        self.as_mut()
            .set_config_json(QString::from(&c.to_json().to_string()));
        self.as_mut().rust_mut().config = c;
        if resort && let Some(cur) = self.current() {
            let (sort, desc) = (self.config.sort, self.config.sort_descending);
            let (files, index) = folder::around(&cur, &self.exts, sort, desc);
            self.as_mut().publish(files, index);
        }
        true
    }

    fn show_in_folder(&self, path: &QString) {
        let p = path_of(path);
        let uri = format!("file://{}", encode(&p.to_string_lossy()));
        // The file manager's ShowItems selects the file; without one, open the folder.
        let shown = Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                "--dest=org.freedesktop.FileManager1",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
            ])
            .arg(format!("array:string:{uri}"))
            .arg("string:")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if !shown
            && let Some(dir) = p.parent()
            && let Ok(mut child) = Command::new("xdg-open")
                .arg(dir)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
        {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
    }
}

fn apply_icon_theme(tokens: &serde_json::Value) {
    if let Some(name) = tokens.get("iconTheme").and_then(|v| v.as_str()) {
        ffi::uqview_set_icon_theme(name);
    }
}

fn display_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.to_string_lossy().into_owned())
}

/// Percent-encode a path for a file:// URI (unreserved characters and `/` stay).
fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Undo `encode` (and any other percent-encoding) for paths dropped as file:// URLs.
fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Some(v) = std::str::from_utf8(&b[i + 1..i + 3])
                .ok()
                .and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};

    #[test]
    fn uris_round_trip() {
        let p = "/home/u/My Pics/50% off #1.png";
        assert_eq!(encode(p), "/home/u/My%20Pics/50%25%20off%20%231.png");
        assert_eq!(decode(&encode(p)), p);
        assert_eq!(decode("/a%zz"), "/a%zz");
    }
}
