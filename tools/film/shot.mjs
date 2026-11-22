#!/usr/bin/env node
// Screenshot a page at one or more widths, for checking the Evolution panel and the report.
//   node tools/film/shot.mjs <url> <out-prefix> [--widths 1440,390] [--selector #evolution] [--full]
// Also prints any text elements whose boxes overlap (a cheap overlapping-text check).
import { loadPlaywright } from "./lib/pw.mjs";

const [url, out, ...rest] = process.argv.slice(2);
const opt = (k, d) => { const i = rest.indexOf(`--${k}`); return i >= 0 ? rest[i + 1] : d; };
const widths = opt("widths", "1440,390").split(",").map(Number);
const selector = opt("selector", null);
const full = rest.includes("--full");
const { chromium } = await loadPlaywright();
const browser = await chromium.launch();
for (const w of widths) {
  const page = await browser.newPage({ viewport: { width: w, height: w < 600 ? 844 : 1000 }, deviceScaleFactor: w < 600 ? 2 : 1 });
  await page.goto(url, { waitUntil: "networkidle" }).catch(() => page.goto(url));
  await page.waitForTimeout(Number(opt("wait", "1500")));
  const file = `${out}-${w}.png`;
  if (selector) await page.locator(selector).screenshot({ path: file });
  else await page.screenshot({ path: file, fullPage: full });
  // Overlap check: leaf text elements whose rectangles intersect.
  const overlaps = await page.evaluate((sel) => {
    const scope = sel ? document.querySelector(sel) : document.body;
    if (!scope) return [];
    const els = [...scope.querySelectorAll("*")].filter((e) => e.children.length === 0 && e.textContent.trim() && e.getClientRects().length && !(e.closest("details:not([open])") && !e.closest("summary")));
    // Skip text scrolled out of view inside a scroll container (it is clipped, not overlapping).
    const clippedAway = (e, r) => {
      for (let a = e.parentElement; a; a = a.parentElement) {
        const o = getComputedStyle(a);
        if (/(auto|scroll|hidden)/.test(o.overflow + o.overflowY + o.overflowX)) {
          const c = a.getBoundingClientRect();
          if (r.bottom <= c.top + 1 || r.top >= c.bottom - 1 || r.right <= c.left + 1 || r.left >= c.right - 1) return true;
        }
      }
      return false;
    };
    // Per line fragment, so a span that wraps is not mistaken for one big box.
    const rects = els.flatMap((e) => [...e.getClientRects()].map((r) => ({ e, r }))).filter(({ e, r }) => r.width > 0 && r.height > 0 && !clippedAway(e, r));
    const hits = [];
    for (let i = 0; i < rects.length; i++) for (let j = i + 1; j < rects.length; j++) {
      const a = rects[i].r, b = rects[j].r;
      const ix = Math.min(a.right, b.right) - Math.max(a.left, b.left), iy = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
      if (ix > 2 && iy > 2 && rects[i].e !== rects[j].e && !rects[i].e.contains(rects[j].e) && !rects[j].e.contains(rects[i].e)) hits.push(`${rects[i].e.textContent.trim().slice(0, 30)} <> ${rects[j].e.textContent.trim().slice(0, 30)}`);
    }
    const wide = document.documentElement.scrollWidth > innerWidth + 1 ? [`page scrolls horizontally: ${document.documentElement.scrollWidth} > ${innerWidth}`] : [];
    return [...wide, ...hits.slice(0, 20)];
  }, selector);
  console.log(`${file}: ${overlaps.length ? `${overlaps.length} issue(s)\n  ${overlaps.join("\n  ")}` : "no overlapping text"}`);
  await page.close();
}
await browser.close();
