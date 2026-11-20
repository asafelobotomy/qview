#include "uqview/cpp/paint.h"

#include <QColor>
#include <QFont>
#include <QFontMetricsF>
#include <QJsonObject>
#include <QLineF>
#include <QPainterPath>
#include <QPolygonF>
#include <QtMath>

namespace {

QColor colorOf(const QJsonObject &o)
{
    QColor c(o.value(QStringLiteral("c")).toString());
    return c.isValid() ? c : QColor(Qt::red);
}

QRectF rectOf(const QJsonObject &o, qreal s)
{
    const QPointF a(o.value(QStringLiteral("x1")).toDouble() * s, o.value(QStringLiteral("y1")).toDouble() * s);
    const QPointF b(o.value(QStringLiteral("x2")).toDouble() * s, o.value(QStringLiteral("y2")).toDouble() * s);
    return QRectF(a, b).normalized();
}

QPen penOf(const QJsonObject &o, qreal s, qreal alpha = 1.0)
{
    QColor c = colorOf(o);
    c.setAlphaF(alpha);
    const qreal w = qBound(1.0, o.value(QStringLiteral("w")).toDouble(4), 200.0) * s;
    return QPen(c, w, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin);
}

void arrow(QPainter &p, const QJsonObject &o, qreal s)
{
    const QPen pen = penOf(o, s);
    const QLineF line(o.value(QStringLiteral("x1")).toDouble() * s, o.value(QStringLiteral("y1")).toDouble() * s,
                      o.value(QStringLiteral("x2")).toDouble() * s, o.value(QStringLiteral("y2")).toDouble() * s);
    if (line.length() < 0.5)
        return;
    const qreal head = qMax(10.0 * s, pen.widthF() * 4.0);
    const qreal angle = qAtan2(line.dy(), line.dx());
    const QPointF tip = line.p2();
    const QPointF left = tip - QPointF(qCos(angle - M_PI / 7) * head, qSin(angle - M_PI / 7) * head);
    const QPointF right = tip - QPointF(qCos(angle + M_PI / 7) * head, qSin(angle + M_PI / 7) * head);
    // The shaft stops inside the head so its round cap doesn't poke through the point.
    const QPointF shaftEnd = tip - QPointF(qCos(angle) * head * 0.6, qSin(angle) * head * 0.6);
    p.setPen(pen);
    p.drawLine(line.p1(), line.length() > head ? shaftEnd : line.p1());
    p.setPen(Qt::NoPen);
    p.setBrush(pen.color());
    p.drawPolygon(QPolygonF({tip, left, right}));
    p.setBrush(Qt::NoBrush);
}

void pixelate(QPainter &p, const QJsonObject &o, const QImage &original, qreal s)
{
    const QRect r = rectOf(o, s).toAlignedRect().intersected(original.rect());
    if (r.width() < 2 || r.height() < 2)
        return;
    // Blocks of about 1/90 of the picture's long side (at least 8 picture pixels).
    const qreal longSide = qMax(original.width(), original.height()) / s;
    const qreal block = qMax(8.0, longSide / 90.0) * s;
    const int w = qMax(1, qRound(r.width() / block));
    const int h = qMax(1, qRound(r.height() / block));
    const QImage small = original.copy(r).scaled(w, h, Qt::IgnoreAspectRatio, Qt::SmoothTransformation);
    p.save();
    p.setRenderHint(QPainter::SmoothPixmapTransform, false);
    p.drawImage(QRectF(r), small);
    p.restore();
}

void text(QPainter &p, const QJsonObject &o, qreal s)
{
    const QString t = o.value(QStringLiteral("text")).toString().left(2000);
    if (t.isEmpty())
        return;
    QFont f = p.font();
    f.setPixelSize(qMax(1, qRound(qBound(6.0, o.value(QStringLiteral("s")).toDouble(32), 800.0) * s)));
    f.setBold(true);
    p.setFont(f);
    const QPointF at(o.value(QStringLiteral("x")).toDouble() * s, o.value(QStringLiteral("y")).toDouble() * s);
    const QFontMetricsF fm(f);
    // A thin dark or light outline keeps the text readable on any picture.
    const QColor c = colorOf(o);
    const QColor halo = c.lightnessF() > 0.5 ? QColor(0, 0, 0, 160) : QColor(255, 255, 255, 190);
    QPainterPath path;
    const QStringList lines = t.split(QLatin1Char('\n'));
    for (int i = 0; i < lines.size(); ++i)
        path.addText(at + QPointF(0, fm.ascent() + i * fm.lineSpacing()), f, lines.at(i));
    p.setPen(QPen(halo, qMax(1.0, f.pixelSize() / 12.0), Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
    p.setBrush(Qt::NoBrush);
    p.drawPath(path);
    p.setPen(Qt::NoPen);
    p.setBrush(c);
    p.drawPath(path);
    p.setBrush(Qt::NoBrush);
}

void badge(QPainter &p, const QJsonObject &o, qreal s)
{
    const qreal d = qBound(12.0, o.value(QStringLiteral("s")).toDouble(32), 400.0) * s;
    const QPointF at(o.value(QStringLiteral("x")).toDouble() * s, o.value(QStringLiteral("y")).toDouble() * s);
    const QColor c = colorOf(o);
    p.setPen(QPen(QColor(255, 255, 255, 230), qMax(1.0, d / 14.0)));
    p.setBrush(c);
    p.drawEllipse(at, d / 2, d / 2);
    QFont f = p.font();
    f.setPixelSize(qMax(1, qRound(d * 0.55)));
    f.setBold(true);
    p.setFont(f);
    p.setPen(c.lightnessF() > 0.6 ? Qt::black : Qt::white);
    p.drawText(QRectF(at.x() - d / 2, at.y() - d / 2, d, d), Qt::AlignCenter,
               QString::number(o.value(QStringLiteral("n")).toInt(1)));
    p.setBrush(Qt::NoBrush);
}

} // namespace

namespace uqview {

void paintMarks(QPainter &p, const QJsonArray &marks, const QImage &original, qreal s)
{
    p.setRenderHint(QPainter::Antialiasing, true);
    p.setRenderHint(QPainter::TextAntialiasing, true);
    for (const QJsonValue &v : marks) {
        const QJsonObject o = v.toObject();
        const QString t = o.value(QStringLiteral("t")).toString();
        if (t == QLatin1String("pen") || t == QLatin1String("hl")) {
            const QJsonArray pts = o.value(QStringLiteral("p")).toArray();
            if (pts.size() < 2)
                continue;
            QPainterPath path(QPointF(pts.at(0).toDouble() * s, pts.at(1).toDouble() * s));
            for (int i = 2; i + 1 < pts.size(); i += 2)
                path.lineTo(pts.at(i).toDouble() * s, pts.at(i + 1).toDouble() * s);
            if (pts.size() == 2)
                path.lineTo(path.currentPosition() + QPointF(0.01, 0));
            const bool hl = t == QLatin1String("hl");
            QPen pen = penOf(o, s, hl ? 0.4 : 1.0);
            if (hl) {
                pen.setCapStyle(Qt::SquareCap);
                pen.setJoinStyle(Qt::BevelJoin);
            }
            p.setPen(pen);
            p.setBrush(Qt::NoBrush);
            // The highlighter multiplies, so the text under it stays dark and readable.
            if (hl)
                p.setCompositionMode(QPainter::CompositionMode_Multiply);
            p.drawPath(path);
            p.setCompositionMode(QPainter::CompositionMode_SourceOver);
        } else if (t == QLatin1String("line")) {
            p.setPen(penOf(o, s));
            const QJsonObject &l = o;
            p.drawLine(QPointF(l.value(QStringLiteral("x1")).toDouble() * s, l.value(QStringLiteral("y1")).toDouble() * s),
                       QPointF(l.value(QStringLiteral("x2")).toDouble() * s, l.value(QStringLiteral("y2")).toDouble() * s));
        } else if (t == QLatin1String("arrow")) {
            arrow(p, o, s);
        } else if (t == QLatin1String("rect")) {
            QPen pen = penOf(o, s);
            pen.setJoinStyle(Qt::MiterJoin);
            p.setPen(pen);
            p.setBrush(Qt::NoBrush);
            p.drawRect(rectOf(o, s));
        } else if (t == QLatin1String("ellipse")) {
            p.setPen(penOf(o, s));
            p.setBrush(Qt::NoBrush);
            p.drawEllipse(rectOf(o, s));
        } else if (t == QLatin1String("text")) {
            text(p, o, s);
        } else if (t == QLatin1String("num")) {
            badge(p, o, s);
        } else if (t == QLatin1String("pixelate")) {
            pixelate(p, o, original, s);
        }
    }
}

} // namespace uqview
