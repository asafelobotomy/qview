#include "uqview/cpp/markupcanvas.h"
#include "uqview/cpp/paint.h"

#include <QImageReader>
#include <QJsonDocument>
#include <QPainter>
#include <QQuickWindow>

MarkupCanvas::MarkupCanvas(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{
    setAntialiasing(true);
    setOpaquePainting(false);
    connect(this, &QQuickItem::widthChanged, this, [this] { update(); });
    connect(this, &QQuickItem::heightChanged, this, [this] { update(); });
}

void MarkupCanvas::setSource(const QString &s)
{
    if (s == m_source)
        return;
    m_source = s;
    if (s.isEmpty()) {
        // Mark up closed: drop the picture (and don't ask Qt to open no file).
        m_full = QImage();
        m_shown = m_sample = QImage();
        m_shownFor = m_realSize = QSize();
        Q_EMIT sourceChanged();
        update();
        return;
    }
    QImageReader r(s);
    r.setAutoTransform(true);
    const QSize size = r.size();
    // A preview needs no more than a 4K screen's worth of pixels; saving reads the full file.
    if (size.isValid() && qMax(size.width(), size.height()) > 4096)
        r.setScaledSize(size.scaled(4096, 4096, Qt::KeepAspectRatio));
    m_full = r.read().convertToFormat(QImage::Format_ARGB32_Premultiplied);
    // Keep the picture's own size so marks map to its real pixels.
    // (QImageReader::size() is before the EXIF turn; a quarter turn swaps the sides.)
    if (m_full.isNull())
        m_realSize = QSize();
    else if (size.isValid())
        m_realSize = (r.transformation() & QImageIOHandler::TransformationRotate90) ? size.transposed() : size;
    else
        m_realSize = m_full.size();
    m_shown = QImage();
    m_sample = QImage();
    Q_EMIT sourceChanged();
    update();
}

void MarkupCanvas::setMarks(const QString &m)
{
    if (m == m_marksText)
        return;
    m_marksText = m;
    m_marks = QJsonDocument::fromJson(m.toUtf8()).array();
    Q_EMIT marksChanged();
    update();
}

void MarkupCanvas::refreshScaled()
{
    const qreal dpr = window() ? window()->effectiveDevicePixelRatio() : 1.0;
    const QSize logical(qMax(1, qRound(width())), qMax(1, qRound(height())));
    const QSize device(qMax(1, qRound(width() * dpr)), qMax(1, qRound(height() * dpr)));
    if (m_shownFor == device && !m_shown.isNull())
        return;
    m_shownFor = device;
    m_shown = m_full.scaled(device, Qt::IgnoreAspectRatio, Qt::SmoothTransformation);
    m_shown.setDevicePixelRatio(dpr);
    m_sample = m_full.scaled(logical, Qt::IgnoreAspectRatio, Qt::SmoothTransformation);
}

void MarkupCanvas::paint(QPainter *p)
{
    if (m_full.isNull() || width() < 1 || height() < 1)
        return;
    refreshScaled();
    p->drawImage(QRectF(0, 0, width(), height()), m_shown);
    uqview::paintMarks(*p, m_marks, m_sample, width() / imageWidth());
}
