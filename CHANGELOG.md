# Changelog

## 0.1.2

One window per session: a picture opened while uQview is open shows in that window (with its
folder) and brings it forward, instead of a second uQview. While that window is in Mark up, a new
picture gets its own window, so unsaved marks are never replaced. `uqview --markup FILE` (uQ's
screenshot preview) opens Mark up in the open window the same way. `uqview --new-window` opens
another window regardless.

## 0.1.1

Renamed from Qview to uQview: the command, the package, the desktop entry
(`io.github.asafelobotomy.uqview`), the settings file (`~/.config/uqview/config.toml`) and the
uQ Settings page are now `uqview`, because the AUR's `qview` is another app (qView).

## 0.1.0

First release: view, zoom and step through a folder; Mark up (pen, highlighter, arrow, line, box,
ellipse, text, numbers, pixelate, crop) saved as a copy, over the original or to the clipboard;
Set as wallpaper through uQ; copy; trash with undo; details; uQ theme; uQ Settings page.
