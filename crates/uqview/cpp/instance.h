#pragma once
#include <QObject>
#include <QString>

// One uQview window per session. The first window listens on a socket in $XDG_RUNTIME_DIR; a
// later start sends it what to open and exits, and the window opens it in a new tab (other
// tabs, and any mark-up in them, stay as they are).
// QML: `onOpenRequested(paths, markup)`.
class Instance : public QObject
{
    Q_OBJECT

public:
    using QObject::QObject;
    static Instance *get();
    void deliver(const QString &pathsJson, bool markup) { emit openRequested(pathsJson, markup); }

signals:
    // `paths`: a JSON array of absolute paths.
    void openRequested(const QString &paths, bool markup);
};
