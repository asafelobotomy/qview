#pragma once
#include <QImage>
#include <QJsonArray>
#include <QQuickPaintedItem>
#include <QString>

// Mark up's preview: the picture scaled to the item, with the marks drawn on it by the same code
// that draws the saved picture (paint.h). QML sizes it to the picture's shape and maps the
// pointer to picture pixels with `scale` (item width / picture width).
class MarkupCanvas : public QQuickPaintedItem
{
    Q_OBJECT
    Q_PROPERTY(QString source READ source WRITE setSource NOTIFY sourceChanged)
    Q_PROPERTY(QString marks READ marks WRITE setMarks NOTIFY marksChanged)
    Q_PROPERTY(int imageWidth READ imageWidth NOTIFY sourceChanged)
    Q_PROPERTY(int imageHeight READ imageHeight NOTIFY sourceChanged)
    Q_PROPERTY(bool ready READ ready NOTIFY sourceChanged)

public:
    explicit MarkupCanvas(QQuickItem *parent = nullptr);

    QString source() const { return m_source; }
    void setSource(const QString &s);
    QString marks() const { return m_marksText; }
    void setMarks(const QString &m);
    int imageWidth() const { return m_realSize.width(); }
    int imageHeight() const { return m_realSize.height(); }
    bool ready() const { return !m_full.isNull(); }

    void paint(QPainter *painter) override;

Q_SIGNALS:
    void sourceChanged();
    void marksChanged();

private:
    void refreshScaled();

    QString m_source;
    QString m_marksText;
    QJsonArray m_marks;
    QImage m_full;      // the picture, at most 4096 px on its long side (the preview's source)
    QImage m_shown;     // m_full at the item's size in device pixels
    QImage m_sample;    // m_full at the item's size in item pixels, for pixelate
    QSize m_shownFor;
    QSize m_realSize;   // the picture's own size, which marks are measured in
};
