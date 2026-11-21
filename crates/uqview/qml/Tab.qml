import QtQuick
import org.uqview.app
import "logic.js" as L

// One tab: its own Backend (folder, place in it, undo) and its own view, so a mark-up in one tab
// stays as it is while another tab is shown.
Item {
    id: tab

    // From the window's list of tabs: what to open first, and whether straight into Mark up.
    required property string paths
    required property bool markup
    property bool fullscreen: false
    readonly property alias view: view
    readonly property alias backend: backend
    readonly property string title: view.path !== "" ? L.baseName(view.path) : "uQview"

    signal openRequested()
    signal fullscreenToggled()
    signal quitRequested()

    Backend { id: backend }

    AppView {
        id: view
        anchors.fill: parent
        backend: backend
        fullscreen: tab.fullscreen
        onOpenRequested: tab.openRequested()
        onFullscreenToggled: tab.fullscreenToggled()
        onQuitRequested: tab.quitRequested()
    }

    function open(pathsJson, markup) {
        backend.openJson(pathsJson);
        if (markup)
            Qt.callLater(view.startMarkup);
    }

    Component.onCompleted: open(paths, markup)
}
