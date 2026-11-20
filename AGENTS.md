# Agent notes for Qview

| Path | What |
|---|---|
| `crates/qview-core` | pure: `folder` (listing, natural sort), `config`, `trash` (freedesktop home trash + undo), `markup` (copy names), `defaults` (MIME types, xdg-mime), `wallpaper` (uqctl command), `args` |
| `crates/qview` | the `qview` binary: cxx-qt `Backend` (`src/backend.rs`), C++ picture work (`cpp/`: clipboard, image info, mark-up renderer `paint.cpp` shared by the preview `MarkupCanvas` and the saved file), QML in `qml/` (`logic.js` holds the pure rules) |
| `uq/pages/qview` | the uQ Settings page (`uq_api` 3), everything through `qview config` / `qview default` |

Rules: keep `qview-core` pure (no Qt, no processes); the window starts fast, so no Kirigami;
marks are stored in picture pixels and drawn by `cpp/paint.cpp` only; keep files under ~400
lines (Markup.qml and AppView.qml are the exceptions until split).
Verify: `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test`,
`node --test tests/*/*.test.mjs`, and on Arch `cargo clippy -p qview -- -D warnings`.

Releasing: bump `version` in Cargo.toml (and CHANGELOG) in a PR; merging it makes
`tag-release.yml` tag it and start `release.yml`, whose `aur` job uploads the AUR files.
