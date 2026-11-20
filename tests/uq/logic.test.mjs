// Tests for the Qview page (uq/pages/qview/): logic.js and the manifest.
//   node --test 'tests/*/*.test.mjs'
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { loadLogic } from "./load-logic.mjs";
import { parseToml } from "./toml-lite.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const pageDir = join(here, "..", "..", "uq", "pages", "qview");
const L = loadLogic(join(pageDir, "logic.js"));
const manifest = parseToml(readFileSync(join(pageDir, "page.toml"), "utf8"));
const qml = readFileSync(join(pageDir, "Page.qml"), "utf8");
const configRs = readFileSync(join(here, "..", "..", "crates", "qview-core", "src", "config.rs"), "utf8");

test("the manifest is uq_api 3 and every command is used", () => {
  assert.equal(manifest.uq_api, 3);
  assert.equal(manifest.id, "qview");
  for (const c of manifest.command) {
    assert.equal(c.argv[0], "/usr/bin/qview");
    assert.ok(qml.includes(`"${c.id}"`), `command ${c.id} is never used`);
  }
});

test("config-set takes exactly the keys qview knows", () => {
  const set = manifest.command.find((c) => c.id === "config-set");
  const keys = set.args[0].pattern.replace(/^\^\(|\)\$$/g, "").split("|").sort();
  const known = [...configRs.matchAll(/^\s+\("([a-z_]+)", (?:Some|None)/gm)].map((m) => m[1]).sort();
  assert.deepEqual(keys, known);
  for (const list of [L.SORTS, L.BACKGROUNDS, L.MARKUP_SAVES])
    for (const o of list) assert.match(o.value, new RegExp(set.args[1].pattern));
});

test("answers", () => {
  assert.equal(L.config({ error: "x" }), null);
  assert.deepEqual(L.config({ code: 0, output: { sort: "date" } }), { sort: "date" });
  assert.deepEqual(L.config({ code: 0, output: { ok: true, config: { sort: "size" } } }), { sort: "size" });
  assert.equal(L.problem({ code: 2, error: "exit 2", output: { ok: false, error: "sort must be one of: name" } }), "sort must be one of: name");
  assert.equal(L.problem({ error: "Cancelled", cancelled: true }), "");
  assert.equal(L.indexOf(L.BACKGROUNDS, "checker"), 2);
  assert.equal(L.indexOf(L.BACKGROUNDS, "nope"), 0);
});

test("the default app line", () => {
  assert.equal(L.defaultLine(L.defaultState({ code: 0, output: { default: true, current: "io.github.asafelobotomy.qview.desktop" } })), "Pictures open in Qview.");
  assert.equal(L.defaultLine(L.defaultState({ code: 0, output: { default: false, current: "org.kde.gwenview.desktop" } })), "Pictures open in Gwenview.");
  assert.equal(L.defaultLine(L.defaultState({ code: 0, output: { default: false, current: "" } })), "No app is set to open pictures.");
  assert.equal(L.defaultLine(null), "Couldn't check which app opens pictures.");
});
