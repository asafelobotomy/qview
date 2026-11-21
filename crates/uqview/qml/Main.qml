// uqview's window: a tab per picture (or set of pictures) opened, each a Tab with its own view
// (AppView). This wires the tabs to the theme, the Open dialog, drops, full screen and pictures
// opened while the window is open, and asks before closing a tab or the window over unsaved marks.
import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Dialogs
import QtQuick.Layouts
import org.uqview.app
import org.uqview.instance
import "logic.js" as L

Window {
    id: win

    width: 1200
    height: 800
    minimumWidth: 560
    minimumHeight: 380
    visible: true
    title: tab ? (tab.view.path !== "" ? tab.title + " — uQview" : "uQview") : "uQview"
    color: Theme.bg
    // The Basic style's dialogs, menus and tooltips draw from the palette: uQ's colours here.
    palette.window: Theme.surface
    palette.windowText: Theme.text
    palette.base: Theme.bg
    palette.text: Theme.text
    palette.button: Theme.surfaceAlt
    palette.buttonText: Theme.text
    palette.light: Theme.overlay
    palette.midlight: Theme.surfaceAlt
    palette.mid: Theme.overlay
    palette.dark: Theme.border
    palette.shadow: "#000000"
    palette.highlight: Theme.accent
    palette.highlightedText: Theme.accentText
    palette.toolTipBase: Theme.overlay
    palette.toolTipText: Theme.text
    palette.placeholderText: Theme.textMuted

    readonly property bool fullscreen: visibility === Window.FullScreen
    readonly property Item tab: tabs.count > 0 && stack.currentIndex >= 0 ? tabs.itemAt(stack.currentIndex) : null
    // What the tab strip shows; re-read when a tab's title or marks change.
    readonly property var strip: {
        var a = [];
        for (var i = 0; i < tabs.count; i++) {
            var t = tabs.itemAt(i);
            a.push(t ? { title: t.title, unsaved: t.view.unsaved } : { title: "", unsaved: false });
        }
        return a;
    }
    // Set while closing after every tab's marks were dealt with.
    property bool leaving: false
    // What waits on the Save your marks? answer: {kind: "tab", index} or {kind: "window"}.
    property var pending: null

    // The theme (one watcher for all tabs) and what the command line asked to open.
    Backend { id: shell }
    Binding {
        target: Theme
        property: "tokens"
        value: shell.themeJson !== "" ? JSON.parse(shell.themeJson) : ({ present: false })
    }

    ListModel { id: model }

    function plain(p) {
        p = String(p);
        return p.indexOf("file://") === 0 ? decodeURIComponent(p.slice(7)) : p;
    }
    // Open pictures: in the tab already showing that picture, in this tab when it shows nothing,
    // else in a new tab. Tabs in Mark up are never replaced.
    function openPictures(list, markup) {
        list = list.map(plain);
        var shown = [];
        for (var i = 0; i < tabs.count; i++)
            shown.push(tabs.itemAt(i) ? tabs.itemAt(i).view.path : "");
        var at = L.findShown(shown, list);
        if (at >= 0) {
            show(at);
            if (markup)
                tabs.itemAt(at).view.startMarkup();
            return;
        }
        if (tab && tab.view.path === "" && !tab.view.marking) {
            tab.open(JSON.stringify(list), markup);
            return;
        }
        model.append({ paths: JSON.stringify(list), markup: markup });
        show(model.count - 1);
    }
    function show(i) {
        if (i < 0 || i >= tabs.count)
            return;
        stack.currentIndex = i;
        var t = tabs.itemAt(i);
        if (t)
            t.view.refocus();
    }
    function closeTab(i) {
        var t = tabs.itemAt(i);
        if (!t)
            return;
        if (t.view.unsaved) {
            show(i);
            ask({ kind: "tab", index: i });
            return;
        }
        if (model.count === 1) {
            win.close();
            return;
        }
        var next = L.afterClose(i, stack.currentIndex, model.count);
        model.remove(i);
        show(next);
    }
    function ask(what) {
        pending = what;
        saveMarks.open();
    }
    // After Save or Discard: carry on with what was asked.
    function resume() {
        var p = pending;
        pending = null;
        if (!p)
            return;
        if (p.kind === "tab")
            closeTab(p.index);
        else
            win.close();
    }

    onClosing: function (close) {
        if (leaving)
            return;
        for (var i = 0; i < tabs.count; i++) {
            if (tabs.itemAt(i) && tabs.itemAt(i).view.unsaved) {
                close.accepted = false;
                show(i);
                ask({ kind: "window" });
                return;
            }
        }
        leaving = true;
    }

    Component.onCompleted: {
        var start = shell.takeStart();
        var s = start !== "" ? JSON.parse(start) : { paths: [], markup: false };
        model.append({ paths: JSON.stringify(s.paths), markup: s.markup });
        show(0);
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        TabStrip {
            Layout.fillWidth: true
            visible: model.count > 1 && !win.fullscreen
            tabs: win.strip
            current: stack.currentIndex
            onPicked: function (i) { win.show(i); }
            onCloseRequested: function (i) { win.closeTab(i); }
            onNewRequested: dialog.open()
        }

        StackLayout {
            id: stack
            Layout.fillWidth: true
            Layout.fillHeight: true

            Repeater {
                id: tabs
                model: model
                Tab {
                    fullscreen: win.fullscreen
                    onOpenRequested: dialog.open()
                    onFullscreenToggled: win.visibility = win.fullscreen ? Window.Windowed : Window.FullScreen
                    onQuitRequested: win.close()
                }
            }
        }

        // Closing a tab, or the window, over marks not yet saved.
        QQC2.Dialog {
            id: saveMarks
            anchors.centerIn: parent
            modal: true
            focus: true
            title: "Save your marks?"
            // A set width: the Basic style's own sizing loops between the title and the buttons.
            implicitWidth: Math.min(win.width - 32, 380)
            standardButtons: QQC2.Dialog.Save | QQC2.Dialog.Discard | QQC2.Dialog.Cancel
            onAccepted: {
                if (win.tab && win.tab.view.saveMarks())
                    win.resume();
                else
                    win.pending = null;
            }
            onDiscarded: {
                close();
                if (win.tab)
                    win.tab.view.discardMarks();
                win.resume();
            }
            onRejected: win.pending = null
            onOpened: forceActiveFocus()
            onClosed: if (win.tab) win.tab.view.refocus()
            palette: win.palette
            // Enter saves (Esc cancels). Inside the dialog: a modal dialog blocks shortcuts outside it.
            Shortcut { sequences: ["Return", "Enter"]; enabled: saveMarks.visible; onActivated: saveMarks.accept() }
            QQC2.Label {
                width: saveMarks.availableWidth
                wrapMode: Text.Wrap
                text: (win.tab ? win.tab.title : "This picture") + " has marks you haven't saved. If you discard them, the picture stays as it was."
                color: Theme.text
                textFormat: Text.PlainText
            }
        }
    }

    // Pictures opened while uQview is open come here (one window per session), each in a tab.
    Connections {
        target: Instance
        function onOpenRequested(paths, markup) {
            win.openPictures(JSON.parse(paths), markup);
        }
    }

    Shortcut { sequences: [StandardKey.Close]; onActivated: win.closeTab(stack.currentIndex) }
    Shortcut { sequences: ["Ctrl+Tab", "Ctrl+PgDown"]; enabled: model.count > 1; onActivated: win.show(L.cycle(stack.currentIndex, model.count, 1)) }
    Shortcut { sequences: ["Ctrl+Shift+Tab", "Ctrl+Backtab", "Ctrl+PgUp"]; enabled: model.count > 1; onActivated: win.show(L.cycle(stack.currentIndex, model.count, -1)) }

    // Back in the window after a while away: the folder may have changed.
    onActiveChanged: if (active && tab) tab.backend.reload()

    // A setting changed in one tab (sort order, thumbnails) applies to every tab.
    Connections {
        target: win.tab ? win.tab.backend : null
        function onConfigJsonChanged() {
            for (var i = 0; i < tabs.count; i++) {
                var t = tabs.itemAt(i);
                if (t && t !== win.tab)
                    t.backend.refreshConfig();
            }
        }
    }

    DropArea {
        anchors.fill: parent
        keys: ["text/uri-list"]
        onDropped: function (drop) {
            if (!drop.hasUrls)
                return;
            var list = [];
            for (var i = 0; i < drop.urls.length; i++)
                list.push(String(drop.urls[i]));
            win.openPictures(list, false);
            drop.accept();
        }
    }

    FileDialog {
        id: dialog
        title: "Open pictures"
        fileMode: FileDialog.OpenFiles
        nameFilters: {
            var s = JSON.parse(shell.suffixesJson || "[]");
            return ["Pictures (" + s.map(function (e) { return "*." + e; }).join(" ") + ")", "All files (*)"];
        }
        onAccepted: {
            var list = [];
            for (var i = 0; i < selectedFiles.length; i++)
                list.push(String(selectedFiles[i]));
            win.openPictures(list, false);
        }
    }

}
