//! uqview's settings: `~/.config/uqview/config.toml`. Every value is checked before it's written
//! (`uqview config set` is what uQ Settings' page calls), and a file with a bad or unknown value
//! loads with that value at its default.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    #[default]
    Name,
    Date,
    Size,
}

/// What's drawn behind the picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Background {
    /// The theme's background colour.
    #[default]
    Theme,
    Black,
    /// A grey checkerboard, to see transparency.
    Checker,
}

/// Where a marked-up picture goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkupSave {
    /// A new file beside the original (`name-marked.png`); the original stays as it was.
    #[default]
    Copy,
    /// Over the original.
    Overwrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub sort: Sort,
    pub sort_descending: bool,
    pub background: Background,
    /// The strip of thumbnails along the bottom.
    pub show_thumbnails: bool,
    /// Going past the last picture comes back to the first.
    pub wrap_around: bool,
    /// Ask before moving a picture to the trash.
    pub confirm_delete: bool,
    /// Pictures smaller than the window are shown at their own size, not blown up.
    pub upscale_small: bool,
    pub markup_save: MarkupSave,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sort: Sort::Name,
            sort_descending: false,
            background: Background::Theme,
            show_thumbnails: true,
            wrap_around: true,
            confirm_delete: false,
            upscale_small: false,
            markup_save: MarkupSave::Copy,
        }
    }
}

/// Every key `set` takes, with the words it accepts (`None`: on/off).
pub const KEYS: [(&str, Option<&[&str]>); 8] = [
    ("sort", Some(&["name", "date", "size"])),
    ("sort_descending", None),
    ("background", Some(&["theme", "black", "checker"])),
    ("show_thumbnails", None),
    ("wrap_around", None),
    ("confirm_delete", None),
    ("upscale_small", None),
    ("markup_save", Some(&["copy", "overwrite"])),
];

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    UnknownKey(String),
    BadValue { key: String, allowed: String },
    Io(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownKey(k) => write!(f, "unknown setting {k:?}"),
            Self::BadValue { key, allowed } => write!(f, "{key} must be one of: {allowed}"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("UQVIEW_CONFIG") {
        return Some(PathBuf::from(p));
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("uqview/config.toml"))
}

impl Config {
    /// The file's settings; defaults for anything missing or wrong, and for a missing file.
    pub fn load(path: Option<&Path>) -> Self {
        let Some(text) = path.and_then(|p| std::fs::read_to_string(p).ok()) else {
            return Self::default();
        };
        Self::parse(&text)
    }

    /// Lenient: each key that's wrong falls back to its default on its own.
    pub fn parse(text: &str) -> Self {
        let mut c = Self::default();
        let Ok(table) = text.parse::<toml::Table>() else {
            return c;
        };
        for (k, v) in table {
            let word = match &v {
                toml::Value::Boolean(b) => if *b { "on" } else { "off" }.to_string(),
                toml::Value::String(s) => s.clone(),
                _ => continue,
            };
            let _ = c.set(&k, &word);
        }
        c
    }

    /// Change one setting from its word (`on`/`off`/`true`/`false` for switches).
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), ConfigError> {
        let (_, words) = KEYS
            .iter()
            .find(|(k, _)| *k == key)
            .ok_or_else(|| ConfigError::UnknownKey(key.into()))?;
        let bad = || ConfigError::BadValue {
            key: key.into(),
            allowed: words.map_or("on, off".into(), |w| w.join(", ")),
        };
        if words.is_none() {
            let b = match value {
                "on" | "true" | "yes" | "1" => true,
                "off" | "false" | "no" | "0" => false,
                _ => return Err(bad()),
            };
            match key {
                "sort_descending" => self.sort_descending = b,
                "show_thumbnails" => self.show_thumbnails = b,
                "wrap_around" => self.wrap_around = b,
                "confirm_delete" => self.confirm_delete = b,
                "upscale_small" => self.upscale_small = b,
                _ => unreachable!("every switch in KEYS is handled"),
            }
            return Ok(());
        }
        match (key, value) {
            ("sort", "name") => self.sort = Sort::Name,
            ("sort", "date") => self.sort = Sort::Date,
            ("sort", "size") => self.sort = Sort::Size,
            ("background", "theme") => self.background = Background::Theme,
            ("background", "black") => self.background = Background::Black,
            ("background", "checker") => self.background = Background::Checker,
            ("markup_save", "copy") => self.markup_save = MarkupSave::Copy,
            ("markup_save", "overwrite") => self.markup_save = MarkupSave::Overwrite,
            _ => return Err(bad()),
        }
        Ok(())
    }

    pub fn to_toml(&self) -> String {
        toml::to_string(self).unwrap_or_default()
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| json!({}))
    }

    /// Write the file (and its folder), atomically.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let io = |e: std::io::Error| ConfigError::Io(format!("{}: {e}", path.display()));
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, self.to_toml()).map_err(io)?;
        std::fs::rename(&tmp, path).map_err(io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_round_trip() {
        let mut c = Config::default();
        c.set("sort", "date").unwrap();
        c.set("show_thumbnails", "off").unwrap();
        c.set("markup_save", "overwrite").unwrap();
        let back = Config::parse(&c.to_toml());
        assert_eq!(back, c);
        assert_eq!(c.to_json()["sort"], "date");
        assert_eq!(c.to_json()["show_thumbnails"], false);
    }

    #[test]
    fn refuses_unknown_and_bad_values() {
        let mut c = Config::default();
        assert_eq!(
            c.set("nope", "on"),
            Err(ConfigError::UnknownKey("nope".into()))
        );
        assert!(matches!(
            c.set("sort", "random"),
            Err(ConfigError::BadValue { .. })
        ));
        assert!(matches!(
            c.set("wrap_around", "maybe"),
            Err(ConfigError::BadValue { .. })
        ));
        assert_eq!(c, Config::default());
    }

    #[test]
    fn lenient_load() {
        let c =
            Config::parse("sort = \"size\"\nbackground = \"pink\"\nwrap_around = false\nx = 1\n");
        assert_eq!(c.sort, Sort::Size);
        assert_eq!(c.background, Background::Theme);
        assert!(!c.wrap_around);
        assert_eq!(Config::parse("not toml ["), Config::default());
    }

    #[test]
    fn saves_atomically() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("a/config.toml");
        let mut c = Config::default();
        c.set("background", "checker").unwrap();
        c.save(&p).unwrap();
        assert_eq!(Config::load(Some(&p)), c);
    }
}
