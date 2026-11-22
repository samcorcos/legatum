import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import { test } from "node:test";
import { editorial, editorialWordmark, lookTokens } from "./brand.mjs";
import { buildReport } from "../film/report.mjs";

test("the exported report embeds editorial fonts and the canonical wordmark offline", () => {
  const html = buildReport({
    dir: "/tmp/brand-fixture",
    run: { run_id: "brand-fixture", config: {} },
    generations: [{ gen: 0, best: { fitness: 1, proxy_win_rate: 0.5 }, archetype_share: { Science: 1 } }],
    events: [], checkpoints: [], coach: [],
  });
  assert.match(html, /data-look="court" data-variant="editorial"/);
  assert.match(html, /data:font\/woff2;base64,/);
  assert.match(html, /<svg role="img" aria-label="Legatum"/);
  assert.doesNotMatch(html, /fonts\.googleapis|Cinzel|Inter[,;]|JetBrains/);
  assert.equal(editorial.bg, lookTokens("court").bg);
  assert.equal(editorial.ink, lookTokens("court").ink);
  assert.equal(editorial.display, lookTokens("atlas").display);
  assert.equal(editorial.sans, lookTokens("atlas").sans);
});

test("film title and end cards load the same wordmark and use the editorial type payload", async () => {
  const draws = [], labels = [];
  // Exercise the renderer's API without launching a browser or claiming pixel verification.
  const ctx = new Proxy({
    globalAlpha: 1,
    drawImage: (...args) => draws.push(args),
    fillText: (text) => labels.push({ text, font: ctx.font }),
    measureText: (text) => ({ width: text.length * 20 }),
    createLinearGradient: () => ({ addColorStop() {} }),
    createRadialGradient: () => ({ addColorStop() {} }),
  }, { get: (target, key) => key in target ? target[key] : () => {} });
  const document = { getElementById: () => ({ getContext: () => ctx }), fonts: [], documentElement: { style: {} } };
  const window = {};
  const Image = class { async decode() { this.decoded = true; } };
  vm.runInNewContext(fs.readFileSync(new URL("../film/scene/film.js", import.meta.url), "utf8"), { document, window, Image });
  await window.FILM.init({
    brand: editorial, wordmark: editorialWordmark,
    run: { archetypes: [{ name: "Science", color: "#8fcf72" }], gens: [] },
    scenes: [{ type: "title", dur: 6, params: { heading: "The title" } },
      { type: "end", dur: 6, params: { heading: "The end", lines: ["Source: run log"] } }],
  });
  window.FILM.draw(0, 3);
  window.FILM.draw(1, 3);
  assert.equal(draws.length, 2);
  assert.ok(draws.every(([image]) => image.src === editorialWordmark && image.decoded));
  assert.ok(labels.filter((l) => ["The title", "The end"].includes(l.text)).every((l) => l.font.includes("Cormorant")));
  assert.ok(labels.find((l) => l.text === "Source: run log").font.includes("Libre Franklin"));
  assert.ok(!labels.some((l) => l.text === "LEGATUM"));
  assert.equal(document.documentElement.style.background, editorial.bg);
});
