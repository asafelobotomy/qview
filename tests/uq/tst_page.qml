// Loads the Qview page with a fake ctx (as uQ Settings would):
//   QT_QPA_PLATFORM=offscreen /usr/lib/qt6/bin/qmltestrunner -import tests/uq/stub -input tests/uq/tst_page.qml
// tests/uq/shots.sh runs it with shotsDir set, to save a screenshot.
import QtQuick
import QtTest

Item {
    id: root
    width: 1000
    height: 760

    property string shotsDir: ""
    property var cfg: ({ sort: "name", sort_descending: false, background: "theme", show_thumbnails: true, wrap_around: true, confirm_delete: false, upscale_small: false, markup_save: "copy" })

    FakeContext {
        id: fake
        commands: ({
                config: () => ({ code: 0, output: root.cfg }),
                "config-set": args => {
                    const c = Object.assign({}, root.cfg);
                    c[args[0]] = args[1] === "on" ? true : (args[1] === "off" ? false : args[1]);
                    root.cfg = c;
                    return { code: 0, output: { ok: true, config: c } };
                },
                default: { code: 0, output: { default: false, current: "org.kde.gwenview.desktop" } },
                "default-set": { code: 0, output: { ok: true, default: true } }
            })
    }

    Loader {
        id: loader
        anchors.fill: parent
    }

    TestCase {
        name: "QviewPage"
        when: windowShown

        function find(item, pred) {
            if (pred(item))
                return item;
            for (let i = 0; i < item.children.length; i++) {
                const r = find(item.children[i], pred);
                if (r)
                    return r;
            }
            return null;
        }

        function test_loads_and_sets() {
            fake.calls = [];
            loader.setSource("../../uq/pages/qview/Page.qml", { ctx: fake.ctx });
            tryVerify(() => loader.status === Loader.Ready);
            // The page reads both commands through watch.
            tryVerify(() => fake.calls.some(c => c[0] === "watch" && c[1] === "config"));
            tryVerify(() => loader.item.config !== null);
            tryVerify(() => loader.item.defaults && loader.item.defaults.current === "org.kde.gwenview.desktop");
            loader.item.set("sort", "date");
            tryVerify(() => loader.item.config.sort === "date");
            compare(fake.calls.filter(c => c[1] === "config-set")[0][2], ["sort", "date"]);
            if (root.shotsDir !== "") {
                wait(300);
                const img = grabImage(root);
                img.save(root.shotsDir + "/qview-page.png");
            }
            loader.item.makeDefault();
            tryVerify(() => fake.calls.some(c => c[1] === "default-set"));
        }
    }
}
