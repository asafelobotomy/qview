// Qview's page in uQ Settings (docs/settings-pages.md in the uQ repository). Every change goes
// through `qview config set`, which checks it; the page shows what the command answers.
import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.uq.settings
import "logic.js" as Logic

SettingsPage {
    id: page

    property var ctx
    property var theme
    property string highlight
    property var config: null
    property var defaults: undefined
    property string failure: ""
    property bool dead: false

    pageId: "qview"

    function apply(result) {
        const c = Logic.config(result);
        if (c !== null)
            page.config = c;
        page.failure = Logic.problem(result);
    }
    function set(key, value) {
        if (!page.ctx)
            return;
        page.ctx.command.run("config-set", [key, value], result => {
            if (!page.dead)
                page.apply(result);
        });
    }
    function makeDefault() {
        page.ctx.command.run("default-set", [], result => {
            if (page.dead)
                return;
            page.failure = Logic.problem(result);
            page.ctx.command.run("default", [], r => page.defaults = Logic.defaultState(r));
        });
    }

    property bool started: false
    function start() {
        if (!page.ctx || page.started)
            return;
        page.started = true;
        page.ctx.command.watch("config", result => page.apply(result));
        page.ctx.command.watch("default", result => page.defaults = Logic.defaultState(result));
    }
    onCtxChanged: start()
    Component.onCompleted: start()
    Component.onDestruction: page.dead = true

    banners: [
        ErrorCard {
            visible: page.failure !== ""
            title: "Couldn't change that"
            detail: page.failure
        }
    ]

    SettingsTab {
        title: "Qview"
        iconName: "qview"

        SettingsCard {
            title: "Pictures"
            iconName: "image-x-generic"
            wide: true
            SettingRow {
                settingId: "default"
                label: "Open pictures with Qview"
                highlight: page.highlight
                UqLabel {
                    text: page.defaults === undefined ? "Checking…" : Logic.defaultLine(page.defaults)
                    opacity: 0.75
                }
                QQC2.Button {
                    text: "Make Qview the default"
                    visible: page.defaults !== undefined && page.defaults !== null && !page.defaults.isDefault
                    onClicked: page.makeDefault()
                }
            }
        }

        SettingsCard {
            title: "Browsing"
            iconName: "view-list-icons"
            help: "Arrow keys, the mouse's side buttons or the strip step through a folder's pictures."
            SettingRow {
                settingId: "sort"
                label: "Order of pictures"
                highlight: page.highlight
                UqComboBox {
                    model: Logic.labels(Logic.SORTS)
                    currentIndex: page.config ? Logic.indexOf(Logic.SORTS, page.config.sort) : 0
                    enabled: page.config !== null
                    onActivated: index => page.set("sort", Logic.SORTS[index].value)
                }
            }
            SwitchRow {
                settingId: "sort_descending"
                label: "Newest, largest or Z first"
                highlight: page.highlight
                value: page.config ? page.config.sort_descending === true : false
                enabled: page.config !== null
                onChosen: v => page.set("sort_descending", Logic.onOff(v))
            }
            SwitchRow {
                settingId: "wrap_around"
                label: "After the last picture, start again"
                highlight: page.highlight
                value: page.config ? page.config.wrap_around === true : true
                enabled: page.config !== null
                onChosen: v => page.set("wrap_around", Logic.onOff(v))
            }
            SwitchRow {
                settingId: "show_thumbnails"
                label: "Show thumbnails"
                highlight: page.highlight
                value: page.config ? page.config.show_thumbnails === true : true
                enabled: page.config !== null
                onChosen: v => page.set("show_thumbnails", Logic.onOff(v))
            }
        }

        SettingsCard {
            title: "Look"
            iconName: "preferences-desktop-color"
            SettingRow {
                settingId: "background"
                label: "Background"
                highlight: page.highlight
                UqComboBox {
                    model: Logic.labels(Logic.BACKGROUNDS)
                    currentIndex: page.config ? Logic.indexOf(Logic.BACKGROUNDS, page.config.background) : 0
                    enabled: page.config !== null
                    onActivated: index => page.set("background", Logic.BACKGROUNDS[index].value)
                }
            }
            SwitchRow {
                settingId: "upscale_small"
                label: "Enlarge small pictures to fit"
                highlight: page.highlight
                value: page.config ? page.config.upscale_small === true : false
                enabled: page.config !== null
                onChosen: v => page.set("upscale_small", Logic.onOff(v))
            }
        }

        SettingsCard {
            title: "Mark up and trash"
            iconName: "document-edit"
            SettingRow {
                settingId: "markup_save"
                label: "Save mark-ups"
                highlight: page.highlight
                UqComboBox {
                    model: Logic.labels(Logic.MARKUP_SAVES)
                    currentIndex: page.config ? Logic.indexOf(Logic.MARKUP_SAVES, page.config.markup_save) : 0
                    enabled: page.config !== null
                    onActivated: index => page.set("markup_save", Logic.MARKUP_SAVES[index].value)
                }
            }
            SwitchRow {
                settingId: "confirm_delete"
                label: "Ask before moving to the trash"
                highlight: page.highlight
                value: page.config ? page.config.confirm_delete === true : false
                enabled: page.config !== null
                onChosen: v => page.set("confirm_delete", Logic.onOff(v))
            }
        }
    }
}
