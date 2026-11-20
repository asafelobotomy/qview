#pragma once
#include "rust/cxx.h"

// Application setup and the picture work that needs Qt's image code (see src/app.rs).
void uqview_describe_application(rust::Str version);
bool uqview_has_window();
// The extensions Qt's image plugins read, comma-separated and lower-case.
rust::String uqview_suffixes();
// {width, height, format, frames} as JSON, or {error} when the file can't be read as a picture.
rust::String uqview_image_info(rust::Str path);
// Put the picture (and its file) on the clipboard. False if it couldn't be read.
bool uqview_copy_image(rust::Str path);
// Draw the marks (JSON, see paint.h) on the full-size picture, crop it, then save it to dst
// (when dst isn't empty) and/or put it on the clipboard. "" when done, else why not.
rust::String uqview_render_markup(rust::Str src, rust::Str marks, rust::Str dst, bool clipboard);
// Make MarkupCanvas available to QML as org.uqview.paint 1.0.
void uqview_register_types();
// Use uQ's icon theme (from its tokens) when it names one that's installed.
void uqview_set_icon_theme(rust::Str name);
