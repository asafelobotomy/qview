import QtQuick
import org.qview.app
import "logic.js" as L

// The picture: fitted to the view until zoomed, then panned by dragging. A screen-sized decode
// shows first (fast, and the neighbours are decoded the same way ahead of time); the full-size
// decode takes over only when zoomed past it. Animated GIF/WebP/APNG play.
Item {
    id: view

    property string path: ""
    property var info: ({})
    property string nextPath: ""
    property string prevPath: ""
    property int turn: 0                 // degrees, a multiple of 90
    property bool upscale: false
    property string background: "theme" // theme, black or checker
    // 0 means "fit"; otherwise the scale (1 = 100 %).
    property real zoom: 0

    readonly property int iw: info && info.width > 0 ? info.width : shown.implicitWidth
    readonly property int ih: info && info.height > 0 ? info.height : shown.implicitHeight
    readonly property real fit: L.fitScale(width, height, iw, ih, turn, upscale)
    readonly property real scale: zoom > 0 ? zoom : fit
    readonly property bool zoomed: zoom > 0 && Math.abs(zoom - fit) > 0.0001
    readonly property bool animated: info && info.animated === true
    readonly property bool failed: path !== "" && (info && info.error !== undefined || shown.status === Image.Error)
    readonly property bool loading: path !== "" && !failed && shown.status !== Image.Ready
    readonly property real dpr: Screen.devicePixelRatio
    // The decode size for the fitted view (both sides, so every picture shares one cache key).
    readonly property size screenSize: Qt.size(Math.ceil(Math.max(width, 64) * dpr), Math.ceil(Math.max(height, 64) * dpr))

    signal stepRequested(int dir)
    signal activated()

    // Displayed size of the picture's bounding box (after the turn).
    readonly property real boxW: (turn % 180 === 0 ? iw : ih) * scale
    readonly property real boxH: (turn % 180 === 0 ? ih : iw) * scale
    property real px: 0
    property real py: 0

    function place(x, y) {
        var p = L.clampPos(x, y, boxW, boxH, width, height);
        px = p.x;
        py = p.y;
    }
    function zoomTo(s, ax, ay) {
        s = L.clamp(s, 0.02, 32);
        if (ax === undefined) {
            ax = width / 2;
            ay = height / 2;
        }
        var p = L.zoomAround(px, py, scale, s, ax, ay);
        zoom = s;
        place(p.x, p.y);
    }
    function zoomStep(dir, ax, ay) {
        zoomTo(L.stepZoom(scale, dir), ax, ay);
    }
    function fitToView() {
        zoom = 0;
        place(0, 0);
    }
    function actualSize(ax, ay) {
        zoomTo(1, ax, ay);
    }
    function rotate(deg) {
        turn = ((turn + deg) % 360 + 360) % 360;
        fitToView();
    }

    onPathChanged: {
        turn = 0;
        zoom = 0;
        place(0, 0);
    }
    onWidthChanged: place(px, py)
    onHeightChanged: place(px, py)
    onFitChanged: if (zoom === 0) place(0, 0)

    clip: true

    // ---- Background
    Rectangle {
        anchors.fill: parent
        color: view.background === "black" ? "#000000" : Theme.bg
    }
    // Under the picture only: a canvas painted once per view size, slid by the pan offset.
    Item {
        id: checker
        visible: view.background === "checker" && !view.failed && view.path !== ""
        x: Math.max(0, view.px)
        y: Math.max(0, view.py)
        width: Math.min(view.width, view.px + view.boxW) - x
        height: Math.min(view.height, view.py + view.boxH) - y
        clip: true
        Canvas {
            readonly property int cell: 12
            x: -((checker.x - view.px) % (cell * 2))
            y: -((checker.y - view.py) % (cell * 2))
            width: view.width + cell * 2
            height: view.height + cell * 2
            renderStrategy: Canvas.Cooperative
            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
            onPaint: {
                var ctx = getContext("2d");
                ctx.fillStyle = "#6e6e6e";
                ctx.fillRect(0, 0, width, height);
                ctx.fillStyle = "#9a9a9a";
                for (var r = 0; r * cell < height; r++)
                    for (var c = r % 2; c * cell < width; c += 2)
                        ctx.fillRect(c * cell, r * cell, cell, cell);
            }
        }
    }

    // ---- The picture. `box` is the turned bounding box; `pic` sits rotated inside it.
    Item {
        id: box
        x: view.px
        y: view.py
        width: view.boxW
        height: view.boxH

        Item {
            id: pic
            width: view.iw * view.scale
            height: view.ih * view.scale
            anchors.centerIn: parent
            rotation: view.turn

            Image {
                id: fast
                anchors.fill: parent
                visible: !view.animated && !(full.visible)
                source: view.animated ? "" : L.fileUrl(view.path)
                sourceSize: view.screenSize
                asynchronous: true
                autoTransform: true
                smooth: true
                mipmap: true
                fillMode: Image.Stretch
            }
            // Full size, only once the screen-sized one isn't enough.
            Image {
                id: full
                anchors.fill: parent
                readonly property bool needed: !view.animated && view.path !== ""
                    && (view.iw * view.scale > view.screenSize.width * 1.02 / view.dpr
                        || view.ih * view.scale > view.screenSize.height * 1.02 / view.dpr)
                visible: needed && status === Image.Ready
                source: needed ? L.fileUrl(view.path) : ""
                asynchronous: true
                autoTransform: true
                cache: false
                // Pixels stay sharp from 200 % up, so the picture's own pixels can be seen.
                smooth: view.scale < 2
                mipmap: view.scale < 1
            }
            AnimatedImage {
                id: anim
                anchors.fill: parent
                visible: view.animated
                source: view.animated ? L.fileUrl(view.path) : ""
                asynchronous: true
                cache: false
                playing: visible
                smooth: view.scale < 2
            }
        }
    }
    readonly property Item shown: animated ? anim : fast

    // The neighbours, decoded ahead into the pixmap cache at the same size.
    Image {
        visible: false
        source: view.nextPath !== "" && !view.loading ? L.fileUrl(view.nextPath) : ""
        sourceSize: view.screenSize
        asynchronous: true
        autoTransform: true
    }
    Image {
        visible: false
        source: view.prevPath !== "" && !view.loading ? L.fileUrl(view.prevPath) : ""
        sourceSize: view.screenSize
        asynchronous: true
        autoTransform: true
    }

    // ---- Pointer
    DragHandler {
        id: drag
        target: null
        property real sx: 0
        property real sy: 0
        cursorShape: active ? Qt.ClosedHandCursor : (view.zoomed ? Qt.OpenHandCursor : Qt.ArrowCursor)
        onActiveChanged: if (active) {
            sx = view.px;
            sy = view.py;
        }
        onTranslationChanged: if (active) view.place(sx + translation.x, sy + translation.y)
    }
    PinchHandler {
        target: null
        property real s0: 1
        onActiveChanged: if (active) s0 = view.scale
        onActiveScaleChanged: if (active) view.zoomTo(s0 * activeScale, centroid.position.x, centroid.position.y)
    }
    WheelHandler {
        // A mouse wheel zooms around the pointer; a touchpad's two-finger scroll pans (and with
        // Ctrl zooms). Pinch zooms too.
        acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
        onWheel: function (ev) {
            var touchpad = ev.pixelDelta.x !== 0 || ev.pixelDelta.y !== 0;
            if (touchpad && !(ev.modifiers & Qt.ControlModifier)) {
                if (view.zoomed)
                    view.place(view.px + ev.pixelDelta.x, view.py + ev.pixelDelta.y);
                return;
            }
            var dy = ev.angleDelta.y !== 0 ? ev.angleDelta.y : ev.pixelDelta.y;
            if (dy === 0)
                return;
            var f = Math.pow(1.0015, dy);
            view.zoomTo(view.scale * f, ev.x, ev.y);
        }
    }
    TapHandler {
        acceptedButtons: Qt.LeftButton
        onDoubleTapped: function (ev) {
            if (view.zoomed || view.scale !== 1)
                view.zoomed ? view.fitToView() : view.actualSize(ev.position.x, ev.position.y);
            else
                view.fitToView();
        }
        onTapped: view.activated()
    }
    TapHandler {
        acceptedButtons: Qt.BackButton | Qt.ForwardButton
        onTapped: function (ev, button) {
            view.stepRequested(button === Qt.BackButton ? -1 : 1);
        }
    }
}
