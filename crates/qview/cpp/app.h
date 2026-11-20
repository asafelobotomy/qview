#pragma once
#include "rust/cxx.h"

// Application setup and the picture work that needs Qt's image code (see src/app.rs).
void qview_describe_application(rust::Str version);
bool qview_has_window();
// The extensions Qt's image plugins read, comma-separated and lower-case.
rust::String qview_suffixes();
// {width, height, format, frames} as JSON, or {error} when the file can't be read as a picture.
rust::String qview_image_info(rust::Str path);
// Put the picture (and its file) on the clipboard. False if it couldn't be read.
bool qview_copy_image(rust::Str path);
// Draw the marks (JSON, see paint.h) on the full-size picture, crop it, then save it to dst
// (when dst isn't empty) and/or put it on the clipboard. "" when done, else why not.
rust::String qview_render_markup(rust::Str src, rust::Str marks, rust::Str dst, bool clipboard);
// Make MarkupCanvas available to QML as org.qview.paint 1.0.
void qview_register_types();
// Use uQ's icon theme (from its tokens) when it names one that's installed.
void qview_set_icon_theme(rust::Str name);
