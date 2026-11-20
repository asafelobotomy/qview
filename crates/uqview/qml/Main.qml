// uqview's window. The layout is AppView; this wires it to the backend, the theme, the Open
// dialog, drops and full screen.
import QtQuick
import QtQuick.Dialogs
import org.uqview.app

Window {
    id: win

    width: 1200
    height: 800
    minimumWidth: 560
    minimumHeight: 380
    visible: true
    title: view.path !== "" ? view.path.slice(view.path.lastIndexOf("/") + 1) + " — uQview" : "uQview"
    color: Theme.bg

    Backend { id: backend }
    Binding {
        target: Theme
        property: "tokens"
        value: backend.themeJson !== "" ? JSON.parse(backend.themeJson) : ({ present: false })
    }

    AppView {
        id: view
        anchors.fill: parent
        backend: backend
        fullscreen: win.visibility === Window.FullScreen
        onOpenRequested: dialog.open()
        onFullscreenToggled: win.visibility = win.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen
        onQuitRequested: win.close()
    }

    // Back in the window after a while away: the folder may have changed.
    onActiveChanged: if (active) backend.reload()

    DropArea {
        anchors.fill: parent
        keys: ["text/uri-list"]
        onDropped: function (drop) {
            if (!drop.hasUrls)
                return;
            var list = [];
            for (var i = 0; i < drop.urls.length; i++)
                list.push(String(drop.urls[i]));
            backend.openJson(JSON.stringify(list));
            drop.accept();
        }
    }

    FileDialog {
        id: dialog
        title: "Open pictures"
        fileMode: FileDialog.OpenFiles
        nameFilters: {
            var s = JSON.parse(backend.suffixesJson || "[]");
            return ["Pictures (" + s.map(function (e) { return "*." + e; }).join(" ") + ")", "All files (*)"];
        }
        onAccepted: {
            var list = [];
            for (var i = 0; i < selectedFiles.length; i++)
                list.push(String(selectedFiles[i]));
            backend.openJson(JSON.stringify(list));
        }
    }
}
