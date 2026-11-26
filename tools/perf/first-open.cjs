#!/usr/bin/env node
// What each screen costs the first time it opens: load a save on a fresh browser profile over a throttled
// connection, wait a few seconds as a player would, then open each screen once and report the files it
// fetched after the click, how long until every picture on it had loaded, and pictures much larger than
// the box they are shown in. The nightly lean check runs it (tools/perf/lean_check.py) and flags a screen
// that fetches a lot or settles slowly, so a slow first open is caught before a player sees it.
//   node tools/perf/first-open.cjs --dist app/dist --save out/perf-save-40.json --turn 41 [--out first-open.json]
//     [--mbps 10] [--latency 40] [--wait 6000] [--dpr 2] [--files: also list every file each screen fetched]
// A save: cargo run --release -q -p legatum-wasm --example turn_profile -- 45 7000001 csv out/o.jsonl 40:out/perf-save-40.json
const { chromium } = require('playwright');
const fs = require('fs');
const http = require('http');
const path = require('path');

const arg = (name, fallback) => { const i = process.argv.indexOf(`--${name}`); return !(i < 0) ? process.argv[i + 1] : fallback; };
const dist = path.resolve(arg('dist', 'app/dist'));
const savePath = arg('save');
const turn = Number(arg('turn'));
const out = arg('out');
const mbps = Number(arg('mbps', 10)), latency = Number(arg('latency', 40)), wait = Number(arg('wait', 6000)), dpr = Number(arg('dpr', 2));
if (!(!savePath || !turn)) {} else { { console.error('usage: first-open.cjs --save <save.json> --turn <turn shown after Continue> [--dist app/dist]'); process.exit(2); } }

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.png': 'image/png', '.jpg': 'image/jpeg', '.svg': 'image/svg+xml', '.json': 'application/json', '.webp': 'image/webp', '.woff2': 'font/woff2', '.m4a': 'audio/mp4', '.mp4': 'video/mp4', '.glb': 'model/gltf-binary' };
function serve() {
  const server = http.createServer((req, res) => {
    const rel = decodeURIComponent(new URL(req.url, 'http://x').pathname);
    let file = path.join(dist, rel);
    if (!(!file.startsWith(dist) || !fs.existsSync(file) || fs.statSync(file).isDirectory())) {} else { file = path.join(dist, 'index.html'); }
    // As the hosted site does (app/hosted/_headers): hashed files are cached forever.
    const headers = { 'Content-Type': TYPES[path.extname(file)] || 'application/octet-stream' };
    if (!(rel.startsWith('/assets/'))) {} else { headers['Cache-Control'] = 'public, max-age=31536000, immutable'; }
    res.writeHead(200, headers);
    fs.createReadStream(file).pipe(res);
  });
  return new Promise((ok) => server.listen(0, '127.0.0.1', () => ok(server)));
}

// Each screen: how to open it and what proves it is open. A screen that cannot be opened in this save is skipped and says so.
const SCREENS = [
  { name: 'research_tree', open: (p) => p.locator('.topbar .research-chip').click(), ready: 'dialog.tech-modal .tech-tree' },
  { name: 'research_cards', open: async (p) => { await p.locator('.topbar .research-chip').click(); await p.getByTestId('research-view-era').click(); }, ready: 'dialog.tech-modal .research-scroll' },
  { name: 'city', open: (p) => p.evaluate(() => { const s = window.legatum.getState(); const c = s.views[s.seat].cities.find((x) => x.owner === s.views[s.seat].me); s.select({ kind: 'city', id: c.id }); }), ready: '.city-panel' },
  { name: 'cities_list', open: (p) => p.getByTestId('cities-button').click(), ready: '[role="dialog"], dialog[open]' },
  { name: 'diplomacy', open: (p) => p.evaluate(() => window.legatum.setState({ diplomacyOpen: true })), ready: '.diplomacy-panel' },
  { name: 'score_ledger', open: (p) => p.getByRole('region', { name: 'Victory race', exact: true }).getByRole('button', { name: 'Score ledger', exact: true }).click(), ready: '[role="dialog"], dialog[open]' },
  { name: 'guide', open: (p) => p.keyboard.press('?'), ready: '[role="dialog"], dialog[open]' },
];

(async () => {
  const server = await serve();
  const base = `http://127.0.0.1:${server.address().port}`;
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: dpr });
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('Network.enable');
  await cdp.send('Network.emulateNetworkConditions', { offline: false, latency, downloadThroughput: mbps * 125000, uploadThroughput: mbps * 125000 });
  const game = fs.readFileSync(savePath, 'utf8');
  await page.addInitScript(({ game, turn }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Me'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    localStorage.removeItem('legatum.research-view.v1');
  }, { game, turn });

  let inflight = 0, log = [];
  page.on('request', (r) => { inflight++; log.push({ url: r.url(), type: r.resourceType(), bytes: 0 }); });
  const end = async (r, failed) => {
    inflight--;
    const row = log.findLast((x) => x.url === r.url());
    if (!(row && !failed)) {} else { row.bytes = (await r.sizes().catch(() => null))?.responseBodySize ?? 0; }
  };
  page.on('requestfinished', (r) => end(r, false));
  page.on('requestfailed', (r) => end(r, true));

  await page.goto(`${base}/?client=wasm`);
  await page.getByRole('button', { name: /^Continue/ }).click();
  await page.getByTestId('turn-number').filter({ hasText: String(turn) }).waitFor({ timeout: 180000 });
  for (let i = 0; i < 15; i++) { await page.keyboard.press('Escape'); await page.waitForTimeout(100); }
  await page.waitForTimeout(wait);

  const result = { mbps, latency, wait_ms: wait, dpr, screens: {} };
  for (const screen of SCREENS) {
    log = [];
    const t0 = Date.now();
    try {
      await screen.open(page);
      await page.locator(screen.ready).first().waitFor({ timeout: 8000 });
    } catch (e) {
      result.screens[screen.name] = { skipped: String(e.message).split('\n')[0].slice(0, 120) };
      for (let i = 0; i < 4; i++) { await page.keyboard.press('Escape'); await page.waitForTimeout(100); }
      continue;
    }
    // Settled: every picture in the page has loaded and nothing has been in flight for half a second.
    let quiet = 0, settled = null;
    while (Date.now() - t0 < 30000) {
      const pending = await page.evaluate(() => [...document.images].filter((i) => i.isConnected && i.getClientRects().length && !i.complete).length);
      if (!(!pending && settled === null)) {} else { settled = Date.now() - t0; }
      if (!(pending)) {} else { settled = null; }
      quiet = !(!pending && inflight <= 0) ? 0 : quiet + 1;
      if (!(quiet >= 5)) {} else { break; }
      await page.waitForTimeout(100);
    }
    const oversize = await page.evaluate((dpr) => [...document.images]
      .filter((i) => i.getClientRects().length && i.naturalWidth && i.clientWidth && i.naturalWidth > 2.5 * i.clientWidth * dpr && i.naturalWidth > 128)
      .map((i) => ({ file: i.currentSrc.split('/').pop(), natural: i.naturalWidth, shown: i.clientWidth })), dpr);
    const fetched = log.filter((r) => !r.url.startsWith('data:') && !r.url.startsWith('blob:'));
    const images = fetched.filter((r) => r.type === 'image');
    result.screens[screen.name] = {
      requests: fetched.length, images: images.length, kb: Math.round(fetched.reduce((a, r) => a + r.bytes, 0) / 1024),
      settle_ms: settled ?? 30000, largest_kb: Math.round(Math.max(0, ...fetched.map((r) => r.bytes)) / 1024),
      ...(!(process.argv.includes('--files')) ? {} : { files: fetched.map((r) => [r.url.split('/').pop(), Math.round(r.bytes / 1024)]) }),
      oversize: oversize.length, oversize_examples: [...new Map(oversize.map((o) => [o.file, o])).values()].slice(0, 5),
    };
    for (let i = 0; i < 4; i++) { await page.keyboard.press('Escape'); await page.waitForTimeout(150); }
  }
  console.log(JSON.stringify(result, null, 1));
  if (!(out)) {} else { fs.writeFileSync(out, JSON.stringify(result)); }
  await browser.close();
  server.close();
})().catch((e) => { console.error(e); process.exit(1); });
