// Renders docs/media/banner.jpg: the key art ("the road at dawn") under the Legatum wordmark.
//   node docs/media/src/banner.mjs      (needs Playwright's Chromium and the app's bundled @fontsource fonts)
//   node docs/media/src/banner.mjs og   the same design at 1200x630 as app/public/og.jpg, the image a link to
//                                       the web build unfurls with (Open Graph and Twitter card, app/index.html)
import { createRequire } from "node:module";
import { readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../../..");
const appRequire = createRequire(process.env.LEGATUM_APP_PKG ?? join(root, "app/package.json"));
const { chromium } = appRequire("playwright");
const fonts = dirname(appRequire.resolve("@fontsource/cormorant/package.json")) + "/files";
const sans = dirname(appRequire.resolve("@fontsource/libre-franklin/package.json")) + "/files";
const logo = readFileSync(join(root, "app/src/assets/icons/logo-legatum.svg"), "utf8").replace("<svg ", '<svg style="width:620px;display:block" ');
const art = pathToFileURL(join(root, "app/src/assets/art/title-keyart-c-road-at-dawn-1600.jpg")).href;
const face = (family, file, weight, style = "normal") => `@font-face{font-family:'${family}';src:url('${pathToFileURL(file)}') format('woff2');font-weight:${weight};font-style:${style}}`;
const og = process.argv[2] === "og";
const [width, height] = og ? [1200, 630] : [1600, 560];
const out = og ? "app/public/og.jpg" : "docs/media/banner.jpg";
// The card is narrower and taller than the banner: a smaller wordmark, and the art kept clear on the right.
const fit = og ? ".in{left:72px;top:118px}.in svg{width:520px!important}.t{font-size:38px;max-width:600px}.f{left:72px;bottom:52px}.b::before{background:linear-gradient(90deg,rgba(24,19,14,.94) 0%,rgba(24,19,14,.84) 40%,rgba(24,19,14,.30) 70%,rgba(24,19,14,0) 90%),linear-gradient(0deg,rgba(24,19,14,.55),rgba(24,19,14,0) 38%)}" : "";
const html = `<!doctype html><meta charset=utf-8><style>
${face("Cormorant", `${fonts}/cormorant-latin-600-italic.woff2`, 600, "italic")}
${face("Cormorant", `${fonts}/cormorant-latin-500-normal.woff2`, 500)}
${face("Franklin", `${sans}/libre-franklin-latin-600-normal.woff2`, 600)}
html,body{margin:0}
.b{position:relative;width:${width}px;height:${height}px;overflow:hidden;background:#1b1712 url('${art}') center 46%/cover}
.b::before{content:"";position:absolute;inset:0;background:linear-gradient(90deg,rgba(24,19,14,.94) 0%,rgba(24,19,14,.82) 34%,rgba(24,19,14,.30) 62%,rgba(24,19,14,0) 82%),linear-gradient(0deg,rgba(24,19,14,.55),rgba(24,19,14,0) 38%)}
.in{position:absolute;left:96px;top:96px;color:#f2ead8}
.k{font:600 15px Franklin;letter-spacing:.34em;color:#d6b25c;margin:0 0 30px}
.t{font:italic 600 42px/1.18 Cormorant;margin:30px 0 0;max-width:760px;color:#f2ead8}
.f{position:absolute;left:96px;bottom:44px;font:600 14px Franklin;letter-spacing:.26em;color:#e8dcc0;opacity:.88}
.f i{color:#d6b25c;font-style:normal;padding:0 12px}
.frame{position:absolute;inset:18px;border:1.5px solid rgba(214,178,92,.55);pointer-events:none}
.frame::after{content:"";position:absolute;inset:6px;border:1px solid rgba(242,234,216,.22)}
${fit}</style><div class=b><div class=frame></div><div class=in><p class=k>A HISTORICAL 4X ON A HEX MAP</p>${logo}<p class=t>Empires rise and fall.<br>Some are remembered.</p></div>
<div class=f>OPEN SOURCE<i>·</i>RUST ENGINE<i>·</i>PLAY IN YOUR BROWSER</div></div>`;
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
const page = join(tmpdir(), `legatum-${og ? "og" : "banner"}.html`);
writeFileSync(page, html);
await p.goto(pathToFileURL(page).href);
await p.evaluate(() => document.fonts.ready);
await p.waitForTimeout(400);
await p.screenshot({ path: join(root, out), type: "jpeg", quality: 88 });
await b.close();
console.log(`wrote ${out}`);
