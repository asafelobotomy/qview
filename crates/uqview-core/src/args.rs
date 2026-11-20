//! The command line.

use std::path::PathBuf;

pub const USAGE: &str = "\
Usage: uqview [FILE|FOLDER]...        view pictures (a folder opens at its first picture)
       uqview --markup FILE           open FILE straight in Mark up
       uqview config show [--json]    print the settings
       uqview config set [--json] KEY VALUE
       uqview default status|set [--json]   is uQview the app pictures open in / make it so
       uqview --version | --help

Keys: sort (name|date|size), sort_descending, background (theme|black|checker),
      show_thumbnails, wrap_around, confirm_delete, upscale_small, markup_save (copy|overwrite)
Switches take on or off.";

#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    View {
        paths: Vec<PathBuf>,
        markup: bool,
    },
    ConfigShow {
        json: bool,
    },
    ConfigSet {
        json: bool,
        key: String,
        value: String,
    },
    DefaultStatus {
        json: bool,
    },
    DefaultSet {
        json: bool,
    },
    Version,
    Help,
}

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Cmd, String> {
    let args: Vec<String> = args.into_iter().collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => return Ok(Cmd::Version),
        Some("--help" | "-h") => return Ok(Cmd::Help),
        Some("config") => return parse_config(&args[1..]),
        Some("default") => return parse_default(&args[1..]),
        _ => {}
    }
    let mut paths = Vec::new();
    let mut markup = false;
    let mut rest_are_paths = false;
    for a in args {
        if rest_are_paths {
            paths.push(PathBuf::from(a));
            continue;
        }
        match a.as_str() {
            "--" => rest_are_paths = true,
            "--markup" | "-m" => markup = true,
            // Desktop files may pass URLs (%U); only local ones are opened.
            u if u.starts_with("file://") => paths.push(PathBuf::from(decode_file_url(u)?)),
            o if o.starts_with('-') && o.len() > 1 => return Err(format!("unknown option {o}")),
            p => paths.push(PathBuf::from(p)),
        }
    }
    if markup && paths.len() != 1 {
        return Err("--markup takes one file".into());
    }
    Ok(Cmd::View { paths, markup })
}

fn parse_config(rest: &[String]) -> Result<Cmd, String> {
    let json = rest.iter().any(|a| a == "--json");
    let words: Vec<&str> = rest
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--json")
        .collect();
    match words.as_slice() {
        ["show"] => Ok(Cmd::ConfigShow { json }),
        ["set", key, value] => Ok(Cmd::ConfigSet {
            json,
            key: key.to_string(),
            value: value.to_string(),
        }),
        _ => {
            Err("usage: uqview config show [--json] | uqview config set [--json] KEY VALUE".into())
        }
    }
}

fn parse_default(rest: &[String]) -> Result<Cmd, String> {
    let json = rest.iter().any(|a| a == "--json");
    let words: Vec<&str> = rest
        .iter()
        .map(String::as_str)
        .filter(|a| *a != "--json")
        .collect();
    match words.as_slice() {
        ["status"] => Ok(Cmd::DefaultStatus { json }),
        ["set"] => Ok(Cmd::DefaultSet { json }),
        _ => Err("usage: uqview default status|set [--json]".into()),
    }
}

/// `file:///a%20b.png` -> `/a b.png`.
fn decode_file_url(u: &str) -> Result<String, String> {
    let rest = &u["file://".len()..];
    let path = rest.strip_prefix("localhost").unwrap_or(rest);
    if !path.starts_with('/') {
        return Err(format!("not a local file: {u}"));
    }
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex =
                std::str::from_utf8(&bytes[i + 1..i + 3]).map_err(|_| format!("bad URL {u}"))?;
            out.push(u8::from_str_radix(hex, 16).map_err(|_| format!("bad URL {u}"))?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| format!("bad URL {u}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(v: &[&str]) -> Result<Cmd, String> {
        parse(v.iter().map(|s| s.to_string()))
    }

    #[test]
    fn views() {
        assert_eq!(
            p(&[]),
            Ok(Cmd::View {
                paths: vec![],
                markup: false
            })
        );
        assert_eq!(
            p(&["a.png", "--", "-b.png"]),
            Ok(Cmd::View {
                paths: vec!["a.png".into(), "-b.png".into()],
                markup: false
            })
        );
        assert_eq!(
            p(&["--markup", "/s.png"]),
            Ok(Cmd::View {
                paths: vec!["/s.png".into()],
                markup: true
            })
        );
        assert!(p(&["--markup"]).is_err());
        assert!(p(&["--frob"]).is_err());
        assert_eq!(
            p(&["file:///home/u/a%20b.png"]),
            Ok(Cmd::View {
                paths: vec!["/home/u/a b.png".into()],
                markup: false
            })
        );
        assert!(p(&["file://host/x.png"]).is_err());
    }

    #[test]
    fn config() {
        assert_eq!(
            p(&["config", "show", "--json"]),
            Ok(Cmd::ConfigShow { json: true })
        );
        assert_eq!(
            p(&["config", "set", "--json", "sort", "date"]),
            Ok(Cmd::ConfigSet {
                json: true,
                key: "sort".into(),
                value: "date".into()
            })
        );
        assert!(p(&["config", "set", "sort"]).is_err());
        assert_eq!(p(&["--version"]), Ok(Cmd::Version));
        assert_eq!(
            p(&["default", "set", "--json"]),
            Ok(Cmd::DefaultSet { json: true })
        );
        assert_eq!(
            p(&["default", "status"]),
            Ok(Cmd::DefaultStatus { json: false })
        );
        assert!(p(&["default"]).is_err());
    }
}
