const log = [], images = {};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const W = window;
log.push(`dpr ${W.devicePixelRatio} ${innerWidth}x${innerHeight} ua ${navigator.userAgent}`);
for (let i = 0; i < 100 && ![...document.querySelectorAll('button')].some((b) => /^Continue/.test(b.textContent.trim())); i++) await sleep(100);
[...document.querySelectorAll('button')].find((b) => /^Continue/.test(b.textContent.trim())).click();
for (let i = 0; i < 200 && !(W.legatum && W.legatum.getState().views && W.legatum.getState().views.length); i++) await sleep(100);
W.legatum.setState({ reportOpen: false, techOpen: false }); W.legatum.getState().select(null);
const m = () => W.legatumMap;
const camTo = (pos, z) => { const s = m().toScreen(pos), cam = m().camera; cam.x += (s.x - innerWidth / 2) / cam.zoom; cam.y += (s.y - innerHeight / 2) / cam.zoom; cam.zoom = z; m().redraw(); };
const stats = () => m().terrain;
const png = () => document.querySelector('canvas[data-game-map]').toDataURL('image/png');
async function settle() { for (let i = 0; i < 80; i++) { const a = stats().renders; await sleep(1500); await frames(20); const s = stats(); if (s.missing === 0 && s.renders === a) return; } }
// An occluded WKWebView window stops animation frames for a while: wait for real frames, or a screenshot shows the last zoom.
let ticks = 0; const tick = () => { ticks++; requestAnimationFrame(tick); }; requestAnimationFrame(tick);
async function frames(n) { const t0 = ticks; for (let i = 0; i < 3000 && ticks - t0 < n; i++) await sleep(20); }
const kara = { q: 14, r: 15 };
// The app eases the camera for a moment after loading (Playwright's reduced-motion emulation is not available here): set it until it holds.
async function holdCam(z) {
  for (let i = 0, ok = 0; i < 60 && ok < 3; i++) {
    camTo(kara, z); await sleep(200);
    const p = m().toScreen(kara);
    ok = Math.abs(m().camera.zoom - z) < 1e-6 && Math.abs(p.x - innerWidth / 2) < 2 && Math.abs(p.y - innerHeight / 2) < 2 ? ok + 1 : 0;
  }
}
for (const z of [1.0, 1.4, 0.7, 1.2]) {
  await holdCam(z); await frames(20);
  images[`z${z}-early`] = png();
  await settle();
  images[`z${z}-settled`] = png();
  log.push(`z${z} ${JSON.stringify(stats())}`);
}
return { log, images };
