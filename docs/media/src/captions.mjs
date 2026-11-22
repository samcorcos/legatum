// Renders the teaser's caption plates and end card as PNGs (Chromium, the app's bundled fonts).
//   node docs/media/src/captions.mjs <out-dir> [--square]     reads docs/media/src/teaser.json
// Plates are transparent and frame-sized (1920x1080, or 1080x1080 with --square); the type is large enough to read on a phone.
import { createRequire } from "node:module";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../../..");
const argv = process.argv.slice(2);
const square = argv.includes("--square");
const outDir = resolve(argv.find((a) => !a.startsWith("--")) ?? join(tmpdir(), "legatum-captions"));
mkdirSync(outDir, { recursive: true });
const W = square ? 1080 : 1920, H = 1080;
const appRequire = createRequire(process.env.LEGATUM_APP_PKG ?? join(root, "app/package.json"));
const { chromium } = appRequire("playwright");
const files = dirname(appRequire.resolve("@fontsource/libre-franklin/package.json")) + "/files";
const cfiles = dirname(appRequire.resolve("@fontsource/cormorant/package.json")) + "/files";
const board = JSON.parse(readFileSync(join(here, "teaser.json"), "utf8"));
const face = (family, file, weight, style = "normal") => `@font-face{font-family:'${family}';src:url('${pathToFileURL(file)}') format('woff2');font-weight:${weight};font-style:${style}}`;
const size = square ? 40 : 52;
const css = `${face("Franklin", `${files}/libre-franklin-latin-600-normal.woff2`, 600)}${face("Cormorant", `${cfiles}/cormorant-latin-600-italic.woff2`, 600, "italic")}
html,body{margin:0;background:transparent}
.plate{position:absolute;left:50%;bottom:${square ? 96 : 84}px;transform:translateX(-50%);padding:${square ? "20px 34px 21px" : "24px 60px 25px"};background:rgba(242,234,216,.96);color:#2b2419;font:600 ${size}px Franklin;letter-spacing:.16em;text-transform:uppercase;white-space:nowrap;box-shadow:0 12px 40px rgba(0,0,0,.45);border:3px solid #2b2419}
.plate::before{content:"";position:absolute;inset:6px;border:1.5px solid rgba(143,111,28,.75);pointer-events:none}
.plate.top{bottom:auto;top:${square ? 96 : 84}px}`;
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width: W, height: H }, deviceScaleFactor: 1 });
for (const [i, shot] of board.shots.entries()) {
  if (!shot.caption) continue;
  const html = `<!doctype html><meta charset=utf-8><style>${css}</style><div class="plate ${shot.captionAt ?? ""}">${shot.caption.replace(/&/g, "&amp;")}</div>`;
  const f = join(tmpdir(), `legatum-caption-${process.pid}.html`);
  writeFileSync(f, html);
  await p.goto(pathToFileURL(f).href);
  await p.evaluate(() => document.fonts.ready);
  const fits = await p.evaluate(() => { const r = document.querySelector(".plate").getBoundingClientRect(); return r.left >= 24 && r.right <= innerWidth - 24; });
  if (!fits) throw new Error(`caption ${i} ("${shot.caption}") does not fit a ${W}px frame`);
  await p.screenshot({ path: join(outDir, `cap-${String(i).padStart(2, "0")}.png`), omitBackground: true });
}
// End card: the key art under the wordmark, with the tagline ("Empires rise and fall. Some are remembered.", user 2026-10-24).
const logo = readFileSync(join(root, "app/src/assets/icons/logo-legatum.svg"), "utf8").replace("<svg ", `<svg style="width:${square ? 640 : 760}px;display:block;margin:0 auto" `);
const art = pathToFileURL(join(root, "app/src/assets/art/title-keyart-c-road-at-dawn.png")).href;
const end = board.end;
const endHtml = `<!doctype html><meta charset=utf-8><style>${css}
body{width:${W}px;height:${H}px;background:#1b1712 url('${art}') center/cover;position:relative;overflow:hidden;color:#f2ead8}
body::before{content:"";position:absolute;inset:0;background:radial-gradient(ellipse at 50% 46%,rgba(20,16,11,.86),rgba(20,16,11,.55) 60%,rgba(20,16,11,.25))}
.c{position:absolute;left:0;right:0;top:${square ? 270 : 290}px;text-align:center}
.k{font:600 ${square ? 24 : 26}px Franklin;letter-spacing:.36em;color:#d6b25c;margin:0 0 44px}
.t{font:italic 600 ${square ? 54 : 64}px/1.2 Cormorant;margin:46px 0 0}
.u{position:absolute;left:0;right:0;bottom:${square ? 130 : 116}px;text-align:center;font:600 ${square ? 28 : 32}px/1.7 Franklin;letter-spacing:.2em;color:#efe3c6}
.u i{color:#d6b25c;font-style:normal;padding:0 18px}
.frame{position:absolute;inset:34px;border:2px solid rgba(214,178,92,.55)}.frame::after{content:"";position:absolute;inset:8px;border:1px solid rgba(242,234,216,.22)}</style>
<div class=frame></div><div class=c><p class=k>${end.kicker}</p>${logo}<p class=t>${end.tagline.join("<br>")}</p></div>
<div class=u>${end.foot.map((l) => l.join("<i>·</i>")).join(square ? "<br>" : "<i>·</i>")}</div>`;
const ef = join(tmpdir(), `legatum-endcard-${process.pid}.html`);
writeFileSync(ef, endHtml);
await p.goto(pathToFileURL(ef).href);
await p.evaluate(() => document.fonts.ready);
await p.waitForTimeout(500);
await p.screenshot({ path: join(outDir, "endcard.png") });
await b.close();
console.log(`captions in ${outDir}`);
