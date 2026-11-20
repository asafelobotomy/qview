.pragma library

// The Qview page's pure rules: what each answer of the `qview` command means for the page.
// No Qt calls (tests/uq/logic.test.mjs runs it under Node).

var SORTS = [
    { value: "name", label: "Name" },
    { value: "date", label: "Date changed" },
    { value: "size", label: "Size" }
];
var BACKGROUNDS = [
    { value: "theme", label: "Theme colour" },
    { value: "black", label: "Black" },
    { value: "checker", label: "Checkerboard" }
];
var MARKUP_SAVES = [
    { value: "copy", label: "As a copy beside it" },
    { value: "overwrite", label: "Over the original" }
];

function labels(list) {
    return list.map(function (o) { return o.label; });
}

function indexOf(list, value) {
    for (var i = 0; i < list.length; i++)
        if (list[i].value === value)
            return i;
    return 0;
}

// The settings from `config` (or `config-set`'s `config`), or null when the answer failed.
function config(result) {
    if (!result || result.error !== undefined || result.code !== 0 || !result.output)
        return null;
    var o = result.output.config !== undefined ? result.output.config : result.output;
    return typeof o === "object" && o !== null && o.sort !== undefined ? o : null;
}

// The words `config-set` takes for a switch.
function onOff(b) {
    return b ? "on" : "off";
}

// `default`'s answer as {isDefault, current} ("" for none), or null when it failed.
function defaultState(result) {
    if (!result || result.error !== undefined || result.code !== 0 || !result.output)
        return null;
    return { isDefault: result.output["default"] === true, current: String(result.output.current || "") };
}

// "Gwenview" from "org.kde.gwenview.desktop": the line under the Default row.
function appName(desktopId) {
    var id = String(desktopId || "").replace(/\.desktop$/, "");
    if (id === "")
        return "";
    var last = id.split(".").pop();
    return last.charAt(0).toUpperCase() + last.slice(1);
}

function defaultLine(state) {
    if (state === null)
        return "Couldn't check which app opens pictures.";
    if (state.isDefault)
        return "Pictures open in Qview.";
    if (state.current === "")
        return "No app is set to open pictures.";
    return "Pictures open in " + appName(state.current) + ".";
}

// A failed command's message for the page.
function problem(result) {
    if (!result)
        return "No answer";
    if (result.cancelled)
        return "";
    if (result.output && result.output.error)
        return String(result.output.error);
    return result.error !== undefined ? String(result.error) : "";
}
