import QtQuick
import org.qview.app
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import "logic.js" as L

// The picture's details: name, size in pixels, file size, type, date and folder.
Rectangle {
    id: panel

    property var info: ({})
    property real zoom: 1
    signal showInFolder()

    color: Theme.surface
    implicitWidth: 260
    Rectangle { anchors.left: parent.left; width: 1; height: parent.height; color: Theme.border }

    component Field: ColumnLayout {
        property string label: ""
        property string value: ""
        visible: value !== ""
        spacing: 2
        Layout.fillWidth: true
        QQC2.Label {
            text: parent.label
            color: Theme.textMuted
            font.family: Theme.fontUi
            font.pointSize: Theme.smallSize
            textFormat: Text.PlainText
        }
        QQC2.Label {
            Layout.fillWidth: true
            text: parent.value
            color: Theme.text
            font.family: Theme.fontUi
            font.pointSize: Theme.bodySize
            wrapMode: Text.WrapAnywhere
            textFormat: Text.PlainText
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        anchors.leftMargin: 17
        spacing: 14
        QQC2.Label {
            text: "Details"
            color: Theme.text
            font.family: Theme.fontUi
            font.pointSize: Theme.bodySize + 2
            font.bold: true
        }
        Field { label: "Name"; value: panel.info.name || "" }
        Field { label: "Dimensions"; value: L.dims(panel.info) }
        Field { label: "File size"; value: panel.info.size || "" }
        Field {
            label: "Type"
            value: (panel.info.format || "") + (panel.info.frames > 1 ? " · " + panel.info.frames + " frames" : "")
        }
        Field { label: "Modified"; value: L.formatDate(panel.info.modified) }
        Field { label: "Zoom"; value: L.percent(panel.zoom) }
        Field { label: "Folder"; value: panel.info.folder || "" }
        IconButton {
            iconName: "folder-open"
            text: "Show in folder"
            showText: true
            tip: "Show in folder"
            onClicked: panel.showInFolder()
        }
        Item { Layout.fillHeight: true }
    }
}
