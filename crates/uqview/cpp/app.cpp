#include "uqview/cpp/app.h"
#include "uqview/cpp/instance.h"
#include "uqview/cpp/markupcanvas.h"
#include "uqview/cpp/paint.h"

#include <QClipboard>
#include <QDir>
#include <QLocalServer>
#include <QLocalSocket>
#include <QFileInfo>
#include <QGuiApplication>
#include <QIcon>
#include <QImageReader>
#include <QImageWriter>
#include <QJsonDocument>
#include <QJsonArray>
#include <QJsonObject>
#include <QMimeData>
#include <QQuickWindow>
#include <QSaveFile>
#include <QString>
#include <QUrl>
#include <QtQml/qqml.h>

namespace {
QString qs(rust::Str s)
{
    return QString::fromUtf8(s.data(), static_cast<qsizetype>(s.size()));
}
rust::String rs(const QString &s)
{
    const QByteArray b = s.toUtf8();
    return rust::String(b.constData(), static_cast<size_t>(b.size()));
}
} // namespace

void uqview_describe_application(rust::Str version)
{
    QGuiApplication::setApplicationName(QStringLiteral("uqview"));
    QGuiApplication::setApplicationDisplayName(QStringLiteral("uQview"));
    QGuiApplication::setApplicationVersion(qs(version));
    QGuiApplication::setDesktopFileName(QStringLiteral("io.github.asafelobotomy.uqview"));
    // Without a desktop platform theme (uQ sets one) Qt has no icon theme; use Breeze's.
    const QString theme = QIcon::themeName();
    if (theme.isEmpty() || theme == QLatin1String("hicolor"))
        QIcon::setThemeName(QStringLiteral("breeze"));
    QIcon::setFallbackThemeName(QStringLiteral("breeze"));
}

bool uqview_has_window()
{
    const auto windows = QGuiApplication::allWindows();
    for (QWindow *w : windows) {
        if (qobject_cast<QQuickWindow *>(w))
            return true;
    }
    return false;
}

rust::String uqview_suffixes()
{
    QStringList out;
    const auto formats = QImageReader::supportedImageFormats();
    for (const QByteArray &f : formats)
        out << QString::fromLatin1(f).toLower();
    // Spellings Qt's plugin names don't cover.
    if (out.contains(QStringLiteral("jpeg")))
        out << QStringLiteral("jpe") << QStringLiteral("jfif");
    if (out.contains(QStringLiteral("tiff")))
        out << QStringLiteral("tif");
    if (out.contains(QStringLiteral("heif")))
        out << QStringLiteral("heic");
    out.removeDuplicates();
    return rs(out.join(QLatin1Char(',')));
}

rust::String uqview_image_info(rust::Str path)
{
    QImageReader r(qs(path));
    r.setAutoTransform(true);
    QJsonObject o;
    QSize size = r.size();
    if (!r.canRead() && !size.isValid()) {
        o.insert(QStringLiteral("error"), r.errorString());
        return rs(QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact)));
    }
    if (r.transformation() & QImageIOHandler::TransformationRotate90)
        size.transpose();
    o.insert(QStringLiteral("width"), size.width());
    o.insert(QStringLiteral("height"), size.height());
    o.insert(QStringLiteral("format"), QString::fromLatin1(r.format()).toUpper());
    o.insert(QStringLiteral("frames"), qMax(1, r.imageCount()));
    o.insert(QStringLiteral("animated"), r.supportsAnimation() && r.imageCount() != 1);
    return rs(QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact)));
}

bool uqview_copy_image(rust::Str path)
{
    const QString p = qs(path);
    QImageReader r(p);
    r.setAutoTransform(true);
    const QImage img = r.read();
    if (img.isNull())
        return false;
    auto *mime = new QMimeData;
    mime->setImageData(img);
    // File managers paste the file itself.
    mime->setUrls({QUrl::fromLocalFile(p)});
    QGuiApplication::clipboard()->setMimeData(mime);
    return true;
}

rust::String uqview_render_markup(rust::Str src, rust::Str marks, rust::Str dst, bool clipboard)
{
    QImageReader r(qs(src));
    r.setAutoTransform(true);
    const QImage original = r.read().convertToFormat(QImage::Format_ARGB32_Premultiplied);
    if (original.isNull())
        return rs(QStringLiteral("Couldn't read the picture: %1").arg(r.errorString()));
    const QJsonObject doc = QJsonDocument::fromJson(qs(marks).toUtf8()).object();
    QImage out = original;
    {
        QPainter p(&out);
        uqview::paintMarks(p, doc.value(QStringLiteral("marks")).toArray(), original, 1.0);
    }
    const QJsonObject crop = doc.value(QStringLiteral("crop")).toObject();
    if (!crop.isEmpty()) {
        const QRect c = QRectF(crop.value(QStringLiteral("x")).toDouble(), crop.value(QStringLiteral("y")).toDouble(),
                               crop.value(QStringLiteral("w")).toDouble(), crop.value(QStringLiteral("h")).toDouble())
                            .toAlignedRect()
                            .intersected(out.rect());
        if (c.width() >= 1 && c.height() >= 1)
            out = out.copy(c);
    }
    const QString target = qs(dst);
    if (!target.isEmpty()) {
        const QString suffix = QFileInfo(target).suffix().toLower();
        QImage toSave = out;
        // JPEG and other formats without transparency get the picture flattened on white.
        if (suffix == QLatin1String("jpg") || suffix == QLatin1String("jpeg") || suffix == QLatin1String("bmp")) {
            QImage flat(out.size(), QImage::Format_RGB32);
            flat.fill(Qt::white);
            QPainter fp(&flat);
            fp.drawImage(0, 0, out);
            fp.end();
            toSave = flat;
        }
        QSaveFile f(target);
        if (!f.open(QIODevice::WriteOnly))
            return rs(QStringLiteral("Couldn't save %1: %2").arg(target, f.errorString()));
        QImageWriter w(&f, suffix.toLatin1());
        if (suffix == QLatin1String("jpg") || suffix == QLatin1String("jpeg") || suffix == QLatin1String("webp"))
            w.setQuality(95);
        if (!w.write(toSave)) {
            f.cancelWriting();
            return rs(QStringLiteral("Couldn't save %1: %2").arg(target, w.errorString()));
        }
        if (!f.commit())
            return rs(QStringLiteral("Couldn't save %1: %2").arg(target, f.errorString()));
    }
    if (clipboard) {
        auto *mime = new QMimeData;
        mime->setImageData(out);
        if (!target.isEmpty())
            mime->setUrls({QUrl::fromLocalFile(target)});
        QGuiApplication::clipboard()->setMimeData(mime);
    }
    return rust::String();
}

void uqview_register_types()
{
    qmlRegisterType<MarkupCanvas>("org.uqview.paint", 1, 0, "MarkupCanvas");
    qmlRegisterSingletonInstance("org.uqview.instance", 1, 0, "Instance", Instance::get());
}

Instance *Instance::get()
{
    static Instance *one = new Instance(qApp);
    return one;
}

void Instance::setBusy(bool b)
{
    if (b == m_busy)
        return;
    m_busy = b;
    emit busyChanged();
}

namespace {
QString instanceSocket()
{
    // $XDG_RUNTIME_DIR: only this user can reach it. Without one every start gets a window.
    const QString dir = qEnvironmentVariable("XDG_RUNTIME_DIR");
    if (dir.isEmpty() || !QDir::isAbsolutePath(dir) || !QDir(dir).exists())
        return {};
    return dir + QStringLiteral("/uqview.socket");
}

void bringForward()
{
    for (QWindow *w : QGuiApplication::allWindows()) {
        if (!qobject_cast<QQuickWindow *>(w) || !w->isVisible())
            continue;
        if (w->windowState() & Qt::WindowMinimized)
            w->setWindowStates(w->windowStates() & ~Qt::WindowMinimized);
        w->show();
        w->raise();
        w->requestActivate();
    }
}

void answer(QLocalSocket *peer)
{
    if (!peer->canReadLine())
        return;
    const QJsonObject req = QJsonDocument::fromJson(peer->readLine()).object();
    if (Instance::get()->busy()) {
        peer->write("busy\n");
    } else {
        Instance::get()->deliver(QString::fromUtf8(QJsonDocument(req.value("paths").toArray()).toJson(QJsonDocument::Compact)),
                                 req.value("markup").toBool());
        bringForward();
        peer->write("ok\n");
    }
    peer->flush();
    peer->disconnectFromServer();
}
} // namespace

bool uqview_forward_to_running(rust::Str request)
{
    const QString path = instanceSocket();
    if (path.isEmpty())
        return false;
    {
        QLocalSocket probe;
        probe.connectToServer(path);
        if (probe.waitForConnected(1000)) {
            probe.write(QByteArray(request.data(), static_cast<qsizetype>(request.size())) + "\n");
            probe.waitForBytesWritten(1000);
            // "ok": the open window took it. "busy" (in Mark up), or no answer: open our own window,
            // and leave the socket to the window that has it.
            return probe.waitForReadyRead(3000) && probe.readLine().trimmed() == "ok";
        }
    }
    // Nobody answered: a socket left by a window that crashed is stale, so take it over.
    auto *server = new QLocalServer(qApp);
    server->setSocketOptions(QLocalServer::UserAccessOption);
    if (!server->listen(path)) {
        QLocalServer::removeServer(path);
        if (!server->listen(path))
            return false;
    }
    QObject::connect(server, &QLocalServer::newConnection, server, [server] {
        while (QLocalSocket *peer = server->nextPendingConnection()) {
            QObject::connect(peer, &QLocalSocket::disconnected, peer, &QObject::deleteLater);
            QObject::connect(peer, &QLocalSocket::readyRead, peer, [peer] { answer(peer); });
            answer(peer);
        }
    });
    QObject::connect(qApp, &QCoreApplication::aboutToQuit, server, [server] { server->close(); });
    return false;
}

void uqview_set_icon_theme(rust::Str name)
{
    const QString n = qs(name);
    if (n.isEmpty() || n == QIcon::themeName() || n.contains(QLatin1Char('/')))
        return;
    for (const QString &dir : QIcon::themeSearchPaths()) {
        if (QFileInfo::exists(dir + QLatin1Char('/') + n + QStringLiteral("/index.theme"))) {
            QIcon::setThemeName(n);
            return;
        }
    }
}
