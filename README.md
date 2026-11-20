# Qview

A fast picture viewer for [uQ](https://github.com/asafelobotomy/uq), in the family of uQ's native
apps. It replaces Gwenview as uQ's image viewer and brings back marking up screenshots.

- **View**: opens a picture and steps through its folder (arrow keys, Space, the mouse's side
  buttons, the thumbnail strip, or clicking the window's edges). Pictures are decoded at screen
  size first and the next and previous ones are decoded ahead, so stepping is instant; the
  full-size decode only happens when you zoom past it. Animated GIF, WebP and APNG play.
- **Zoom and pan**: mouse wheel or pinch zooms around the pointer, `+`/`-`, `1` for 100 %,
  `F` or `0` to fit, double-click toggles, drag to pan. `R`/`Shift+R` turn the view.
- **Mark up** (`M`): pen, highlighter, arrow, line, box, ellipse, text, numbered badges,
  pixelate (to hide passwords and tokens) and crop, in seven colours and three sizes, with undo and
  redo. Save as a copy beside the original (`name-marked.png`), over it, or straight to the
  clipboard. What's saved is drawn by the same code as the preview, at the picture's full size.
- **Set as wallpaper** straight into uQ (`uqctl wallpaper set`), so Settings › Wallpaper shows it.
- **Copy** (`Ctrl+C`, the picture and its file), **move to the trash** (`Del`) with Undo,
  **details** (`I`), **show in folder**, full screen (`F11`), drag and drop.
- **uQ theme**: colours, corner radius, fonts and icon theme follow uQ's theme live; elsewhere it
  follows the desktop palette.
- **uQ Settings page** (Apps › Qview): make Qview the default picture app, the order of pictures,
  wrap-around, thumbnails, background (theme, black, checkerboard), enlarging small pictures,
  how mark-ups are saved, asking before the trash.

Formats are whatever Qt's image plugins read: PNG, JPEG, GIF, WebP, BMP, TIFF, SVG, ICO, plus
AVIF, HEIF, JPEG XL, EXR, PSD, XCF and more through `qt6-imageformats` and `kimageformats`.

## Command line

```
qview [FILE|FOLDER]...        view pictures (a folder opens at its first picture)
qview --markup FILE           open FILE straight in Mark up (uQ's screenshot preview uses it)
qview config show|set [--json] KEY VALUE
qview default status|set [--json]
```

Settings live in `~/.config/qview/config.toml`.

## Building

Needs Rust and Qt 6 (base, declarative, svg). On Arch:

```
cargo build --release --workspace
sudo scripts/install-files.sh
```

Or install `qview` from the AUR.

## Testing

`cargo test` (the rules in `qview-core`), `node --test tests/*/*.test.mjs` (the window's and
the Settings page's pure logic), and `qmltestrunner -import tests/uq/stub -input tests/uq/tst_page.qml`
(the Settings page with a fake uQ). CI also starts the window offscreen and fails on any QML error.

## Licence

GPL-3.0-or-later.
