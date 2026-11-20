#pragma once
#include <QImage>
#include <QJsonArray>
#include <QPainter>
#include <QRectF>

// The marks a person draws in Mark up, as JSON objects in picture pixels:
//   {"t":"pen"|"hl", "c":"#rrggbb", "w":4, "p":[x,y, x,y, ...]}      free hand / highlighter
//   {"t":"line"|"arrow"|"rect"|"ellipse", "c", "w", "x1","y1","x2","y2"}
//   {"t":"text", "c", "s":32, "x","y", "text":"..."}               top-left corner, size in px
//   {"t":"num", "c", "s":32, "x","y", "n":1}                        a numbered badge, centred
//   {"t":"pixelate", "x1","y1","x2","y2"}                           hides what's under it
// The same code draws the preview (MarkupCanvas, scaled) and the saved picture (scale 1), so
// what's saved is what was seen. `original` is the unmarked picture at the painter's scale;
// pixelate samples it, so marks under a pixelated area are never what's hidden.
namespace uqview {
void paintMarks(QPainter &p, const QJsonArray &marks, const QImage &original, qreal scale);
}
