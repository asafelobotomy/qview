#!/bin/sh
# Installs everything qview ships into DESTDIR (default /): the program, the desktop entry and
# icon, and the uQ Settings page. Used by the PKGBUILD; also handy for a manual install:
#   cargo build --release --workspace && sudo scripts/install-files.sh
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
d=${DESTDIR:-}
install -Dm755 "$here/target/release/qview" "$d/usr/bin/qview"
install -Dm644 "$here/packaging/desktop/io.github.asafelobotomy.qview.desktop" \
    "$d/usr/share/applications/io.github.asafelobotomy.qview.desktop"
install -Dm644 "$here/packaging/desktop/qview.svg" "$d/usr/share/icons/hicolor/scalable/apps/qview.svg"
install -Dm644 -t "$d/usr/share/uq/settings/pages/qview/" "$here"/uq/pages/qview/*.qml \
    "$here/uq/pages/qview/logic.js" "$here/uq/pages/qview/page.toml"
install -Dm644 "$here/LICENSE" "$d/usr/share/licenses/qview/LICENSE"
