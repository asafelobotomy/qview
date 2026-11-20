//! uQ's theme tokens (`~/.local/state/uq/theme/tokens.json`, written by uqd) as the QML `Theme`
//! singleton reads them. Without uQ, `present` is false and the window follows the desktop palette.

use std::path::PathBuf;
use std::time::SystemTime;

use serde_json::{Map, Value, json};

const COLORS: [&str; 12] = [
    "bg",
    "surface",
    "surface_alt",
    "overlay",
    "text",
    "text_muted",
    "accent",
    "accent_text",
    "border",
    "success",
    "warning",
    "error",
];

/// Check the tokens file once a second and hand new tokens (JSON) to `send` until it returns false.
pub fn watch(path: Option<PathBuf>, send: impl Fn(String) -> bool + Send + 'static) {
    std::thread::Builder::new()
        .name("uq-theme".into())
        .spawn(move || {
            let mut seen = path.as_ref().and_then(modified);
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let now = path.as_ref().and_then(modified);
                if now != seen {
                    seen = now;
                    if !send(load(path.as_ref()).to_string()) {
                        return;
                    }
                }
            }
        })
        .expect("spawning the theme watcher");
}

pub fn tokens_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("QVIEW_UQ_TOKENS") {
        return Some(PathBuf::from(p));
    }
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))?;
    Some(state.join("uq/theme/tokens.json"))
}

fn modified(path: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// The tokens as `{present, colors, radius, fontUi, fontMono, fontSize}`; `{present: false}` when
/// uQ isn't there or the file doesn't parse.
pub fn load(path: Option<&PathBuf>) -> Value {
    let absent = json!({ "present": false });
    let Some(text) = path.and_then(|p| std::fs::read_to_string(p).ok()) else {
        return absent;
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return absent;
    };
    let colors_in = v.get("colors").and_then(Value::as_object);
    let mut colors = Map::new();
    for k in COLORS {
        if let Some(c) = colors_in.and_then(|m| m.get(k)).and_then(Value::as_str) {
            colors.insert(k.into(), Value::String(c.into()));
        }
    }
    if colors.is_empty() {
        return absent;
    }
    let font = |k: &str| {
        v.pointer(&format!("/fonts/{k}"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    json!({
        "present": true,
        "colors": colors,
        "radius": v.pointer("/ui/radius").and_then(Value::as_u64).unwrap_or(8),
        "fontUi": font("ui"),
        "fontMono": font("mono"),
        "fontSize": v.pointer("/fonts/size").and_then(Value::as_u64).unwrap_or(0),
        "iconTheme": v.pointer("/icons/theme").and_then(Value::as_str).unwrap_or(""),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_or_bad_tokens_mean_no_uq() {
        assert_eq!(load(None), json!({"present": false}));
        assert_eq!(
            load(Some(&PathBuf::from("/nonexistent/tokens.json"))),
            json!({"present": false})
        );
    }
}
