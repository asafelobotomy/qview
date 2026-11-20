#!/bin/sh
# Saves a screenshot of the Qview page: tests/uq/shots.sh <outdir>
set -e
out=${1:?usage: shots.sh <outdir>}
mkdir -p "$out"
here=$(cd "$(dirname "$0")" && pwd)
tmp="$here/tst_shots_tmp.qml"
trap 'rm -f "$tmp"' EXIT
sed "s|property string shotsDir: \"\"|property string shotsDir: \"$out\"|" "$here/tst_page.qml" > "$tmp"
QT_QPA_PLATFORM=offscreen /usr/lib/qt6/bin/qmltestrunner -import "$here/stub" -input "$tmp"
