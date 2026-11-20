import QtQuick
import org.qview.app
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.qview.paint
import "logic.js" as L

// Mark up: draw on the picture, then save a copy (or over it, or to the clipboard). The marks
// are kept in picture pixels; the canvas draws them with the same code that saves them.
Rectangle {
    id: mk

    required property var backend
    property string path: ""
    property string tool: "arrow"
    property int level: 1
    property string colour: "#e5484d"
    property var marks: []
    property var crop: null
    property var draft: null
    property var undoStack: []
    property var redoStack: []
    property bool busy: false
    readonly property bool dirty: marks.length > 0 || crop !== null
    readonly property var swatches: [String(Theme.accent), "#e5484d", "#f5a524", "#46a758", "#3e8ef7", "#111111", "#ffffff"]

    signal closed()
    signal saved(string path)
    signal message(string kind, string text)

    color: Theme.bg

    function reset() {
        marks = [];
        crop = null;
        draft = null;
        undoStack = [];
        redoStack = [];
        textEdit.visible = false;
    }
    function snapshot() {
        return { marks: marks.slice(), crop: crop };
    }
    function commit(newMarks, newCrop) {
        undoStack = undoStack.concat([snapshot()]);
        redoStack = [];
        marks = newMarks;
        crop = newCrop;
    }
    function undo() {
        if (undoStack.length === 0)
            return;
        redoStack = redoStack.concat([snapshot()]);
        var s = undoStack[undoStack.length - 1];
        undoStack = undoStack.slice(0, -1);
        marks = s.marks;
        crop = s.crop;
    }
    function redo() {
        if (redoStack.length === 0)
            return;
        undoStack = undoStack.concat([snapshot()]);
        var s = redoStack[redoStack.length - 1];
        redoStack = redoStack.slice(0, -1);
        marks = s.marks;
        crop = s.crop;
    }
    function save(mode) {
        if (busy)
            return;
        commitText();
        busy = true;
        var r = JSON.parse(backend.saveMarkup(path, L.document(marks, crop), mode));
        busy = false;
        if (!r.ok) {
            message("error", r.error);
            return;
        }
        if (mode === "clipboard") {
            message("ok", "Copied the marked-up picture");
            return;
        }
        message("ok", "Saved " + L.baseName(r.path));
        reset();
        saved(r.path);
    }
    function cancel() {
        if (dirty && !discard.visible) {
            discard.open();
            return;
        }
        reset();
        closed();
    }
    function commitText() {
        if (!textEdit.visible)
            return;
        var m = textEdit.mark;
        m.text = textEdit.text;
        textEdit.visible = false;
        if (L.worthKeeping(m, 0))
            commit(marks.concat([m]), crop);
    }

    onPathChanged: reset()
    focus: visible
    Keys.onPressed: function (ev) {
        if (textEdit.visible)
            return;
        if (ev.key === Qt.Key_Escape) {
            cancel();
        } else if (ev.matches(StandardKey.Undo)) {
            undo();
        } else if (ev.matches(StandardKey.Redo) || (ev.key === Qt.Key_Z && ev.modifiers === (Qt.ControlModifier | Qt.ShiftModifier))) {
            redo();
        } else if (ev.matches(StandardKey.Save)) {
            save("save");
        } else if (ev.matches(StandardKey.Copy)) {
            save("clipboard");
        } else if (ev.modifiers === Qt.NoModifier && ev.text >= "1" && ev.text <= "3") {
            level = Number(ev.text) - 1;
        } else if (ev.modifiers === Qt.NoModifier || ev.modifiers === Qt.ShiftModifier) {
            var t = L.toolByKey(ev.text);
            if (t === "")
                return;
            tool = t;
        } else {
            return;
        }
        ev.accepted = true;
    }

    // ---- Toolbar
    Rectangle {
        id: bar
        anchors.left: parent.left
        anchors.right: parent.right
        height: Theme.barHeight
        color: Theme.surface
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Theme.border }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.gap
            anchors.rightMargin: Theme.gap
            spacing: 2

            Repeater {
                model: L.TOOLS
                IconButton {
                    required property var modelData
                    iconName: modelData.icon
                    text: modelData.label
                    tip: modelData.label
                    shortcut: modelData.key
                    checkable: true
                    checked: mk.tool === modelData.id
                    onClicked: {
                        mk.commitText();
                        mk.tool = modelData.id;
                    }
                }
            }
            Rectangle { width: 1; height: 22; color: Theme.border; Layout.leftMargin: 6; Layout.rightMargin: 6 }
            Repeater {
                model: mk.swatches
                Rectangle {
                    required property string modelData
                    width: 22
                    height: 22
                    radius: 11
                    color: modelData
                    border.width: mk.colour === modelData ? 3 : 1
                    border.color: mk.colour === modelData ? Theme.text : Theme.border
                    Accessible.role: Accessible.RadioButton
                    Accessible.name: "Colour " + modelData
                    MouseArea {
                        anchors.fill: parent
                        anchors.margins: -3
                        cursorShape: Qt.PointingHandCursor
                        onClicked: mk.colour = parent.modelData
                    }
                }
            }
            Rectangle { width: 1; height: 22; color: Theme.border; Layout.leftMargin: 6; Layout.rightMargin: 6 }
            Repeater {
                model: [{ l: "Thin", d: 4 }, { l: "Medium", d: 8 }, { l: "Thick", d: 14 }]
                IconButton {
                    required property var modelData
                    required property int index
                    tip: modelData.l
                    shortcut: String(index + 1)
                    checkable: true
                    checked: mk.level === index
                    onClicked: mk.level = index
                    contentItem: Item {
                        Rectangle {
                            anchors.centerIn: parent
                            width: modelData.d
                            height: modelData.d
                            radius: width / 2
                            color: parent.parent.fg
                        }
                    }
                }
            }
            Rectangle { width: 1; height: 22; color: Theme.border; Layout.leftMargin: 6; Layout.rightMargin: 6 }
            IconButton { iconName: "edit-undo"; tip: "Undo"; shortcut: "Ctrl+Z"; enabled: mk.undoStack.length > 0; onClicked: mk.undo() }
            IconButton { iconName: "edit-redo"; tip: "Redo"; shortcut: "Ctrl+Shift+Z"; enabled: mk.redoStack.length > 0; onClicked: mk.redo() }

            Item { Layout.fillWidth: true }

            IconButton { text: "Cancel"; showText: true; tip: "Cancel"; shortcut: "Esc"; onClicked: mk.cancel() }
            IconButton { iconName: "edit-copy"; text: "Copy"; showText: true; tip: "Copy"; shortcut: "Ctrl+C"; enabled: mk.dirty; onClicked: mk.save("clipboard") }
            IconButton {
                id: saveButton
                primary: true
                text: "Save"
                tip: "Save"
                enabled: mk.dirty && !mk.busy
                onClicked: mk.save("save")
            }
            IconButton {
                iconName: "arrow-down"
                text: "▾"
                tip: "More ways to save"
                enabled: mk.dirty && !mk.busy
                onClicked: saveMenu.open()
                QQC2.Menu {
                    id: saveMenu
                    y: parent.height
                    QQC2.MenuItem { text: "Save as a copy"; onTriggered: mk.save("copy") }
                    QQC2.MenuItem { text: "Save over the original"; onTriggered: mk.save("overwrite") }
                }
            }
        }
    }

    // ---- The canvas, fitted below the toolbar
    Item {
        id: area
        anchors.top: bar.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 24
        clip: false

        readonly property real s: canvas.imageWidth > 0
            ? Math.min(width / canvas.imageWidth, height / canvas.imageHeight, 4) : 1

        MarkupCanvas {
            id: canvas
            source: mk.path
            marks: JSON.stringify(mk.draft ? mk.marks.concat([mk.draft]) : mk.marks)
            width: imageWidth * area.s
            height: imageHeight * area.s
            anchors.centerIn: parent

            // Outside the crop is dimmed.
            Item {
                anchors.fill: parent
                readonly property var c: mk.draft && mk.draft.t === "crop" ? L.cropFrom(mk.draft, canvas.imageWidth, canvas.imageHeight) : mk.crop
                visible: c !== null
                Rectangle { color: "#99000000"; x: 0; y: 0; width: parent.width; height: parent.c ? parent.c.y * area.s : 0 }
                Rectangle { color: "#99000000"; x: 0; y: parent.c ? (parent.c.y + parent.c.h) * area.s : 0; width: parent.width; height: parent.height - y }
                Rectangle { color: "#99000000"; x: 0; y: parent.c ? parent.c.y * area.s : 0; width: parent.c ? parent.c.x * area.s : 0; height: parent.c ? parent.c.h * area.s : 0 }
                Rectangle { color: "#99000000"; x: parent.c ? (parent.c.x + parent.c.w) * area.s : 0; y: parent.c ? parent.c.y * area.s : 0; width: parent.width - x; height: parent.c ? parent.c.h * area.s : 0 }
                Rectangle {
                    color: "transparent"
                    border.color: "white"
                    border.width: 1
                    x: parent.c ? parent.c.x * area.s : 0
                    y: parent.c ? parent.c.y * area.s : 0
                    width: parent.c ? parent.c.w * area.s : 0
                    height: parent.c ? parent.c.h * area.s : 0
                }
            }

            MouseArea {
                anchors.fill: parent
                preventStealing: true
                cursorShape: mk.tool === "text" ? Qt.IBeamCursor : Qt.CrossCursor
                function pt(m) {
                    return { x: m.x / area.s, y: m.y / area.s };
                }
                onPressed: function (m) {
                    mk.forceActiveFocus();
                    var p = pt(m);
                    var iw = canvas.imageWidth, ih = canvas.imageHeight;
                    if (mk.tool === "text") {
                        mk.commitText();
                        textEdit.mark = { t: "text", c: mk.colour, s: L.textSize(mk.level, iw, ih), x: Math.round(p.x), y: Math.round(p.y), text: "" };
                        textEdit.text = "";
                        textEdit.visible = true;
                        textEdit.forceActiveFocus();
                        return;
                    }
                    mk.commitText();
                    if (mk.tool === "num") {
                        mk.commit(mk.marks.concat([{ t: "num", c: mk.colour, s: L.badgeSize(mk.level, iw, ih), x: Math.round(p.x), y: Math.round(p.y), n: L.nextNumber(mk.marks) }]), mk.crop);
                        return;
                    }
                    mk.draft = L.begin(mk.tool, p.x, p.y, mk.colour, mk.level, iw, ih);
                }
                onPositionChanged: function (m) {
                    if (mk.draft === null)
                        return;
                    var p = pt(m);
                    mk.draft = L.extend(mk.draft, p.x, p.y, (m.modifiers & Qt.ShiftModifier) !== 0);
                }
                onReleased: {
                    var d = mk.draft;
                    mk.draft = null;
                    if (d === null)
                        return;
                    if (d.t === "crop") {
                        var c = L.cropFrom(d, canvas.imageWidth, canvas.imageHeight);
                        if (c !== null)
                            mk.commit(mk.marks, c);
                        return;
                    }
                    if (L.worthKeeping(d, 3 / area.s))
                        mk.commit(mk.marks.concat([d]), mk.crop);
                }
            }

            // Typing a text mark: shown where it will be, in its colour and size.
            QQC2.TextField {
                id: textEdit
                property var mark: ({})
                visible: false
                x: (mark.x || 0) * area.s - leftPadding
                y: (mark.y || 0) * area.s - topPadding
                width: Math.max(160, contentWidth + 40)
                font.pixelSize: Math.max(10, (mark.s || 20) * area.s)
                font.bold: true
                color: mark.c || "red"
                padding: 2
                placeholderText: "Type, then Enter"
                background: Rectangle { color: "#40000000"; border.color: Theme.accent; radius: 3 }
                Keys.onReturnPressed: mk.commitText()
                Keys.onEnterPressed: mk.commitText()
                Keys.onEscapePressed: {
                    visible = false;
                    mk.forceActiveFocus();
                }
            }
        }
    }

    QQC2.Dialog {
        id: discard
        anchors.centerIn: parent
        modal: true
        title: "Discard your marks?"
        // A set width: the Basic style's own sizing loops between the title and the buttons.
        implicitWidth: Math.min(parent.width - 32, 360)
        standardButtons: QQC2.Dialog.Discard | QQC2.Dialog.Cancel
        onDiscarded: {
            close();
            mk.reset();
            mk.closed();
        }
        QQC2.Label {
            text: "The picture stays as it was."
            color: Theme.text
        }
    }
}
