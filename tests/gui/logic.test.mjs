// Tests for the window's pure rules (crates/uqview/qml/logic.js).
//   node --test 'tests/*/*.test.mjs'
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { readFileSync, writeFileSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
// logic.js starts `.pragma library`; without that line it's plain CommonJS.
const src = readFileSync(join(here, "..", "..", "crates", "uqview", "qml", "logic.js"), "utf8").replace(/^\.pragma library$/m, "");
const dir = mkdtempSync(join(tmpdir(), "uqview-logic-"));
writeFileSync(join(dir, "logic.cjs"), src);
const L = createRequire(import.meta.url)(join(dir, "logic.cjs"));

test("fit", () => {
  assert.equal(L.fitScale(1000, 500, 2000, 1000, 0, false), 0.5);
  assert.equal(L.fitScale(1000, 500, 100, 100, 0, false), 1);
  assert.equal(L.fitScale(1000, 500, 100, 100, 0, true), 5);
  // A quarter turn swaps the sides.
  assert.equal(L.fitScale(1000, 500, 2000, 1000, 90, false), 0.25);
  assert.equal(L.fitScale(0, 0, 0, 0, 0, false), 1);
});

test("zoom steps and anchoring", () => {
  assert.equal(L.stepZoom(1, 1), 1.25);
  assert.equal(L.stepZoom(1, -1), 0.8);
  assert.equal(L.stepZoom(0.9, 1), 1);
  assert.equal(L.stepZoom(32, 1), 32);
  assert.equal(L.stepZoom(0.05, -1), 0.02);
  // The point under the pointer stays put.
  const p = L.zoomAround(0, 0, 1, 2, 100, 50);
  assert.deepEqual(p, { x: -100, y: -50 });
  assert.deepEqual(L.clampPos(500, 500, 100, 100, 400, 300), { x: 150, y: 100 });
  assert.deepEqual(L.clampPos(50, -900, 800, 600, 400, 300), { x: 0, y: -300 });
});

test("stepping through a folder", () => {
  assert.equal(L.step(0, 3, 1, false), 1);
  assert.equal(L.step(2, 3, 1, true), 0);
  assert.equal(L.step(2, 3, 1, false), -1);
  assert.equal(L.step(0, 3, -1, true), 2);
  assert.equal(L.step(0, 0, 1, true), -1);
});

test("names and URLs", () => {
  assert.equal(L.baseName("/a/b/c.png"), "c.png");
  assert.equal(L.fileUrl("/a b/#1?.png"), "file:///a%20b/%231%3F.png");
  assert.equal(L.counter(2, 27), "3 of 27");
  assert.equal(L.counter(0, 1), "");
  assert.equal(L.percent(0.756), "76 %");
  assert.equal(L.percent(0.05), "5 %");
  assert.equal(L.dims({ width: 1920, height: 1080 }), "1920 × 1080");
});

test("marks", () => {
  const a = L.begin("arrow", 10, 10, "#ff0000", 1, 1920, 1080);
  assert.equal(a.t, "arrow");
  assert.equal(a.w, L.strokeWidth(1, 1920, 1080));
  const b = L.extend(a, 110, 14, true);
  assert.ok(Math.abs(b.x2 - 110) < 0.2 && b.y2 === 10, "snapped to horizontal");
  assert.ok(L.worthKeeping(b, 3));
  assert.ok(!L.worthKeeping(a, 3));
  const sq = L.extend(L.begin("rect", 0, 0, "#000", 0, 100, 100), 30, -10, true);
  assert.deepEqual([sq.x2, sq.y2], [30, -30]);
  let pen = L.begin("pen", 0, 0, "#000", 1, 1000, 1000);
  pen = L.extend(pen, 0.1, 0, false);
  assert.equal(pen.p.length, 2, "points closer than half a stroke are skipped");
  pen = L.extend(pen, 50, 0, false);
  assert.equal(pen.p.length, 4);
  assert.equal(L.nextNumber([{ t: "num", n: 2 }, { t: "pen" }]), 3);
  assert.equal(L.toolByKey("x"), "pixelate");
  assert.equal(L.toolByKey("?"), "");
  assert.deepEqual(L.cropFrom({ x1: 90, y1: 80, x2: -5, y2: 5 }, 100, 100), { x: 0, y: 5, w: 90, h: 75 });
  assert.equal(L.cropFrom({ x1: 1, y1: 1, x2: 2, y2: 2 }, 100, 100), null);
  assert.deepEqual(JSON.parse(L.document([], null)), { marks: [], crop: null });
});
