//! qview: a fast picture viewer for uQ. Shows a picture and steps through its folder, zooms,
//! marks up (pen, highlighter, arrows, boxes, text, numbers, pixelate, crop), copies, sets the
//! wallpaper through uQ and moves pictures to the trash. On uQ it takes its colours from uQ's
//! theme tokens; elsewhere it follows the desktop palette.
//!
//! Exit codes: 0 normal, 2 bad command line or setting, 4 no window could be created.

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString, QUrl};
use qview_core::args::{self, Cmd};
use qview_core::config::{self, Config};
use qview_core::defaults;

mod app;
mod backend;
mod theme;

fn main() {
    let cmd = match args::parse(std::env::args().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("qview: {e}\n\n{}", args::USAGE);
            std::process::exit(2);
        }
    };
    match cmd {
        Cmd::Version => println!("qview {}", env!("CARGO_PKG_VERSION")),
        Cmd::Help => println!("{}", args::USAGE),
        Cmd::ConfigShow { json } => config_show(json),
        Cmd::ConfigSet { json, key, value } => config_set(json, &key, &value),
        Cmd::DefaultStatus { json } => default_status(json),
        Cmd::DefaultSet { json } => default_set(json),
        Cmd::View { paths, markup } => {
            backend::set_start(paths, markup);
            window();
        }
    }
}

fn config_show(json: bool) {
    let c = Config::load(config::path().as_deref());
    if json {
        println!("{}", c.to_json());
    } else {
        print!("{}", c.to_toml());
    }
}

fn config_set(json: bool, key: &str, value: &str) {
    let Some(path) = config::path() else {
        fail(json, "no home folder");
    };
    let mut c = Config::load(Some(&path));
    if let Err(e) = c.set(key, value).and_then(|()| c.save(&path)) {
        fail(json, &e.to_string());
    }
    if json {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "config": c.to_json() })
        );
    }
}

/// What `xdg-mime` says opens `mime`, or "" when nothing does (or xdg-mime is missing).
fn query_default(mime: &str) -> String {
    std::process::Command::new("xdg-mime")
        .args(["query", "default", mime])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn default_status(json: bool) {
    let (png, jpeg) = (query_default("image/png"), query_default("image/jpeg"));
    let is = defaults::is_default(&png, &jpeg);
    if json {
        println!("{}", serde_json::json!({ "default": is, "current": png }));
    } else if is {
        println!("Qview opens pictures.");
    } else if png.is_empty() {
        println!("No app is set to open pictures.");
    } else {
        println!("{png} opens pictures.");
    }
}

fn default_set(json: bool) {
    let argv = defaults::set_argv();
    let ok = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        fail(
            json,
            "xdg-mime couldn't set the default (is xdg-utils installed?)",
        );
    }
    if json {
        println!("{}", serde_json::json!({ "ok": true, "default": true }));
    } else {
        println!("Pictures now open in Qview.");
    }
}

fn fail(json: bool, msg: &str) -> ! {
    if json {
        println!("{}", serde_json::json!({ "ok": false, "error": msg }));
    }
    eprintln!("qview: {msg}");
    std::process::exit(2);
}

fn window() {
    // The "Basic" Qt Quick Controls style draws from the palette, which Theme sets per item.
    QQuickStyle::set_style(&QString::from("Basic"));
    let mut app = QGuiApplication::new();
    app::ffi::qview_describe_application(env!("CARGO_PKG_VERSION"));
    app::ffi::qview_register_types();
    // Qt 6.4 doesn't search qrc:/qt/qml on its own (6.5 does); the app's module lives there.
    // SAFETY: no other thread exists yet.
    unsafe { std::env::set_var("QML_IMPORT_PATH", ":/qt/qml") };
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/org/qview/app/qml/Main.qml"));
    }
    if !app::ffi::qview_has_window() {
        eprintln!("qview: the window couldn't be created (is Qt Quick installed?)");
        std::process::exit(4);
    }
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
