.pragma library

// uqview's pure rules for the window: zoom and pan maths, stepping through the folder, names and
// URLs, and the mark-up model. No Qt calls, so tests/gui/logic.test.mjs runs it under Node.

// The zoom levels + and - step through (1 is 100 %).
var STEPS = [0.05, 0.1, 0.15, 0.25, 0.33, 0.5, 0.67, 0.8, 1, 1.25, 1.5, 2, 3, 4, 6, 8, 12, 16, 24, 32];
var MIN_ZOOM = 0.02;
var MAX_ZOOM = 32;

function clamp(v, lo, hi) {
    return Math.max(lo, Math.min(hi, v));
}

// The scale that shows the whole picture in the view. `rotation` in degrees; a quarter turn
// swaps the picture's sides. Small pictures stay at 100 % unless `upscale`.
function fitScale(vw, vh, iw, ih, rotation, upscale) {
    if (!(iw > 0 && ih > 0 && vw > 0 && vh > 0))
        return 1;
    var quarter = Math.abs(rotation % 180) === 90;
    var w = quarter ? ih : iw, h = quarter ? iw : ih;
    var s = Math.min(vw / w, vh / h);
    return upscale ? s : Math.min(1, s);
}

// The next zoom level up (dir 1) or down (dir -1) from `z`.
function stepZoom(z, dir) {
    if (dir > 0) {
        for (var i = 0; i < STEPS.length; i++)
            if (STEPS[i] > z * 1.001)
                return STEPS[i];
        return MAX_ZOOM;
    }
    for (var j = STEPS.length - 1; j >= 0; j--)
        if (STEPS[j] < z / 1.001)
            return STEPS[j];
    return MIN_ZOOM;
}

// Where the picture's top-left goes so the picture point under (px, py) in the view stays under
// it when the scale changes from s0 to s1. `x`, `y` are the current top-left.
function zoomAround(x, y, s0, s1, px, py) {
    var k = s1 / s0;
    return { x: px - (px - x) * k, y: py - (py - y) * k };
}

// Keep a picture of size (w, h) placed at (x, y) in a view (vw, vh): centred along an axis it's
// smaller on, otherwise with no gap at the edges.
function clampPos(x, y, w, h, vw, vh) {
    return {
        x: w <= vw ? (vw - w) / 2 : clamp(x, vw - w, 0),
        y: h <= vh ? (vh - h) / 2 : clamp(y, vh - h, 0)
    };
}

// The index after stepping `dir` from `i` in a list of `n`; -1 when there's nowhere to go.
function step(i, n, dir, wrap) {
    if (n <= 0)
        return -1;
    var j = i + dir;
    if (j < 0)
        return wrap ? n - 1 : (i === 0 ? -1 : 0);
    if (j >= n)
        return wrap ? 0 : (i === n - 1 ? -1 : n - 1);
    return j;
}

function baseName(p) {
    var s = String(p || "");
    var i = s.lastIndexOf("/");
    return i < 0 ? s : s.slice(i + 1);
}

// A path as an Image `source` (every character a URL treats specially is escaped).
function fileUrl(p) {
    if (!p)
        return "";
    return "file://" + encodeURI(p).replace(/#/g, "%23").replace(/\?/g, "%3F");
}

// "3 of 27", or "" for a single picture.
function counter(i, n) {
    return n > 1 && i >= 0 ? (i + 1) + " of " + n : "";
}

function percent(s) {
    var p = s * 100;
    return (p < 10 ? p.toFixed(1).replace(/\.0$/, "") : Math.round(p)) + " %";
}

// "1920 × 1080"
function dims(info) {
    return info && info.width > 0 ? info.width + " × " + info.height : "";
}

function formatDate(ms) {
    if (!ms)
        return "";
    var d = new Date(ms);
    function two(n) { return n < 10 ? "0" + n : "" + n; }
    return d.getFullYear() + "-" + two(d.getMonth() + 1) + "-" + two(d.getDate()) + " " + two(d.getHours()) + ":" + two(d.getMinutes());
}

// ---- Mark up

// Stroke widths, text and badge sizes follow the picture's size, so marks look the same on a
// phone photo and on a 4K screenshot. level: 0 small, 1 medium, 2 large.
function unit(iw, ih) {
    return Math.max(1.5, Math.max(iw, ih) / 480);
}
function strokeWidth(level, iw, ih) {
    return Math.round(unit(iw, ih) * [1.5, 3, 6][level] * 10) / 10;
}
function textSize(level, iw, ih) {
    return Math.round(unit(iw, ih) * [9, 14, 22][level]);
}
function badgeSize(level, iw, ih) {
    return Math.round(unit(iw, ih) * [12, 18, 28][level]);
}

// The tools: id, label, the theme icon, the key that picks it, and whether it's dragged.
var TOOLS = [
    { id: "pen", label: "Pen", icon: "draw-freehand", key: "P" },
    { id: "hl", label: "Highlighter", icon: "draw-highlight", key: "H" },
    { id: "arrow", label: "Arrow", icon: "draw-arrow", key: "A" },
    { id: "line", label: "Line", icon: "draw-line", key: "L" },
    { id: "rect", label: "Box", icon: "draw-rectangle", key: "B" },
    { id: "ellipse", label: "Ellipse", icon: "draw-ellipse", key: "E" },
    { id: "text", label: "Text", icon: "draw-text", key: "T" },
    { id: "num", label: "Number", icon: "draw-number", key: "N" },
    { id: "pixelate", label: "Pixelate", icon: "view-private", key: "X" },
    { id: "crop", label: "Crop", icon: "transform-crop", key: "C" }
];

function toolByKey(k) {
    for (var i = 0; i < TOOLS.length; i++)
        if (TOOLS[i].key === String(k).toUpperCase())
            return TOOLS[i].id;
    return "";
}

// The next number for the Number tool.
function nextNumber(marks) {
    var n = 0;
    for (var i = 0; i < marks.length; i++)
        if (marks[i].t === "num")
            n = Math.max(n, marks[i].n);
    return n + 1;
}

// A mark started at picture point (x, y) with the given tool, colour and size level.
function begin(tool, x, y, color, level, iw, ih) {
    var w = strokeWidth(level, iw, ih);
    if (tool === "pen")
        return { t: "pen", c: color, w: w, p: [r1(x), r1(y)] };
    if (tool === "hl")
        return { t: "hl", c: color, w: w * 4, p: [r1(x), r1(y)] };
    if (tool === "pixelate")
        return { t: "pixelate", x1: r1(x), y1: r1(y), x2: r1(x), y2: r1(y) };
    return { t: tool, c: color, w: w, x1: r1(x), y1: r1(y), x2: r1(x), y2: r1(y) };
}

// The same mark with the pointer now at (x, y). `straight` (Shift) keeps lines at 45° steps
// and boxes square.
function extend(m, x, y, straight) {
    var o = JSON.parse(JSON.stringify(m));
    if (o.t === "pen" || o.t === "hl") {
        var n = o.p.length;
        var dx = x - o.p[n - 2], dy = y - o.p[n - 1];
        // Skip points closer than half a stroke: smaller saves, same look.
        if (dx * dx + dy * dy >= Math.pow(o.w / 2, 2))
            o.p.push(r1(x), r1(y));
        return o;
    }
    if (straight) {
        var ddx = x - o.x1, ddy = y - o.y1;
        if (o.t === "line" || o.t === "arrow") {
            var a = Math.round(Math.atan2(ddy, ddx) / (Math.PI / 4)) * (Math.PI / 4);
            var len = Math.sqrt(ddx * ddx + ddy * ddy);
            x = o.x1 + Math.cos(a) * len;
            y = o.y1 + Math.sin(a) * len;
        } else {
            var side = Math.max(Math.abs(ddx), Math.abs(ddy));
            x = o.x1 + (ddx < 0 ? -side : side);
            y = o.y1 + (ddy < 0 ? -side : side);
        }
    }
    o.x2 = r1(x);
    o.y2 = r1(y);
    return o;
}

// Whether a finished mark is worth keeping (a click with the box tool draws nothing).
function worthKeeping(m, minSize) {
    if (m.t === "pen" || m.t === "hl")
        return m.p.length >= 2;
    if (m.t === "text")
        return String(m.text || "").trim() !== "";
    if (m.t === "num")
        return true;
    var dx = Math.abs(m.x2 - m.x1), dy = Math.abs(m.y2 - m.y1);
    if (m.t === "line" || m.t === "arrow")
        return Math.sqrt(dx * dx + dy * dy) >= minSize;
    return dx >= minSize && dy >= minSize;
}

// A crop from a dragged mark, inside the picture: {x, y, w, h}, or null when too small.
function cropFrom(m, iw, ih) {
    var x1 = clamp(Math.min(m.x1, m.x2), 0, iw), x2 = clamp(Math.max(m.x1, m.x2), 0, iw);
    var y1 = clamp(Math.min(m.y1, m.y2), 0, ih), y2 = clamp(Math.max(m.y1, m.y2), 0, ih);
    if (x2 - x1 < 4 || y2 - y1 < 4)
        return null;
    return { x: Math.round(x1), y: Math.round(y1), w: Math.round(x2 - x1), h: Math.round(y2 - y1) };
}

// What's sent to the renderer.
function document(marks, crop) {
    return JSON.stringify({ marks: marks, crop: crop || null });
}

function r1(v) {
    return Math.round(v * 10) / 10;
}

// For Node tests.
if (typeof module !== "undefined")
    module.exports = { STEPS: STEPS, fitScale: fitScale, stepZoom: stepZoom, zoomAround: zoomAround, clampPos: clampPos, step: step, baseName: baseName, fileUrl: fileUrl, counter: counter, percent: percent, dims: dims, formatDate: formatDate, unit: unit, strokeWidth: strokeWidth, textSize: textSize, badgeSize: badgeSize, TOOLS: TOOLS, toolByKey: toolByKey, nextNumber: nextNumber, begin: begin, extend: extend, worthKeeping: worthKeeping, cropFrom: cropFrom, document: document };
