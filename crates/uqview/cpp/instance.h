#pragma once
#include <QObject>
#include <QString>

// One uQview window per session. The first window listens on a socket in $XDG_RUNTIME_DIR; a
// later start sends it what to open and exits, unless that window is in Mark up (`busy`), which
// keeps its unsaved marks: then the later start opens its own window.
// QML: `Instance.busy` (bound to Mark up being open) and `onOpenRequested(paths, markup)`.
class Instance : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool busy READ busy WRITE setBusy NOTIFY busyChanged)

public:
    using QObject::QObject;
    static Instance *get();
    bool busy() const { return m_busy; }
    void setBusy(bool b);
    void deliver(const QString &pathsJson, bool markup) { emit openRequested(pathsJson, markup); }

signals:
    void busyChanged();
    // `paths`: a JSON array of absolute paths.
    void openRequested(const QString &paths, bool markup);

private:
    bool m_busy = false;
};
