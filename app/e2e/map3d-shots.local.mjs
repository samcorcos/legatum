// Local helper (not a test): open a save in the 3D map and take screenshots. node e2e/map3d-shots.local.mjs <save.json> <outdir>
import { createServer } from 'vite';
import { chromium } from 'playwright';
import { readFile, mkdir } from 'node:fs/promises';
const [savePath, out] = process.argv.slice(2);
const game = '';
await mkdir(out, { recursive: true });
const server = await createServer({ server: { host: '127.0.0.1', port: 0 }, logLevel: 'error' });
await server.listen();
const port = server.httpServer.address().port;
const b = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
const errs = [];
for (const [w, h] of [[1440, 900]]) {
  const p = await b.newPage({ viewport: { width: w, height: h } });
  p.on('pageerror', (e) => errs.push(String(e)));
  p.on('console', (m) => { if (!(m.type() === 'error' || m.type() === 'warning')) {} else { errs.push(m.text()); } });
  await p.addInitScript(({ game }) => {
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    localStorage.setItem('legatum.map-view', '3d');
  }, { game });
  await p.goto(`http://127.0.0.1:${port}/?client=wasm`);
  const [chooser] = await Promise.all([p.waitForEvent('filechooser'), p.getByRole('button', { name: /Load game/i }).click()]);
  await chooser.setFiles(savePath);
  await p.waitForTimeout(8000); await p.screenshot({ path: `${out}/debug-${w}.png` }); console.log(await p.evaluate(() => ({ gl2: !!document.createElement('canvas').getContext('webgl2'), coarse: matchMedia('(pointer: coarse)').matches, mv: localStorage.getItem('legatum.map-view') })));
  console.log(errs.join('\n'));
  await p.waitForSelector('.map3d-canvas', { timeout: 30000 });
  await p.waitForTimeout(12000);
  await p.screenshot({ path: `${out}/game-${w}x${h}.png` });
  // select a unit of mine and hover a tile two away to show the route
  const ok = await p.evaluate(() => { const s = window.legatum?.getState?.(); if (!(!s)) {} else { return 'no store'; } const v = s.views[s.seat]; const u = v.units.find((x) => x.owner === v.me && x.military); if (!(!u)) {} else { return 'no unit'; } s.select({ kind: 'unit', id: u.id }); s.jumpTo(u.pos); return 'ok'; });
  await p.waitForTimeout(4000);
  const c = await p.locator('.map3d-canvas').boundingBox();
  const tgt = await p.evaluate(() => { const s = window.legatum.getState(); const v = s.views[s.seat]; const u = v.units.find((x) => x.id === s.selection.id); for (const [dq, dr] of [[1,1],[1,0],[0,1],[-1,1],[1,-1],[2,0],[0,2]]) { const q = u.pos.q + dq, r = u.pos.r + dr; if (!(v.units.some((z) => z.pos.q === q && z.pos.r === r) || v.cities.some((c) => c.pos.q === q && c.pos.r === r))) {} else { continue; } return window.legatumMap3d.project({ q, r }, 0); } });
  errs.push("tgt " + JSON.stringify(tgt) + " c " + JSON.stringify(c));
  await p.mouse.move(c.x + tgt.x, c.y + tgt.y);
  await p.waitForTimeout(4000);
  errs.push(JSON.stringify(await p.evaluate(({x,y}) => { const sc = window.legatumMap3d; return { pick: sc.pick(x, y), el: document.elementFromPoint(x, y)?.className, hov: sc.hoverRing.visible, st: (() => { const s = window.legatum.getState(); const v = s.views[s.seat]; const u = v.units.find((x) => x.id === s.selection.id); return { pos: u.pos, moves: u.moves, mode: s.mode, here: v.units.filter((z) => z.pos.q === 20 && z.pos.r === 26).map((z) => z.owner), goto: u.goto }; })(), sel: sc.selRing.visible, route: sc.route.visible, anim: sc.animating }; }, { x: tgt.x, y: tgt.y })));
  await p.screenshot({ path: `${out}/select-${w}x${h}.png` });
  await p.evaluate(() => { const sc = window.legatumMap3d; sc.dist = 2.2; sc.placeCamera(); });
  await p.waitForTimeout(3000); await p.screenshot({ path: `${out}/close-${w}x${h}.png` });
  await p.evaluate(() => { const sc = window.legatumMap3d; sc.dist = 8; sc.placeCamera(); });
  await p.setViewportSize({ width: 1000, height: 600 }); await p.waitForTimeout(3000); await p.screenshot({ path: `${out}/select-1000x600.png` });
  errs.push(`select: ${ok}`);
  // Swap test: pan, M to 2D, M back to 3D; the centre should survive.
  const before = await p.evaluate(() => { const sc = window.legatumMap3d; sc.target.x += 6; sc.target.z += 4; sc.clampTarget(); sc.placeCamera(); return { x: sc.target.x, z: sc.target.z }; });
  await p.locator('.map3d-canvas').focus(); await p.keyboard.press('m'); await p.waitForTimeout(2500);
  const mid = await p.evaluate(() => ({ is3d: !!document.querySelector('.map3d-canvas'), cam2d: window.legatumMap?.camera ?? Object.keys(window.legatumMap ?? {}) }));
  await p.keyboard.press('m'); await p.waitForSelector('.map3d-canvas'); const early = await p.evaluate(() => { const sc = window.legatumMap3d; return [sc.target.x, sc.target.z, window.legatum.getState().focus]; }); errs.push('early ' + JSON.stringify(early)); await p.waitForTimeout(2300);
  const after = await p.evaluate(() => { const sc = window.legatumMap3d; return { x: sc.target.x, z: sc.target.z }; });
  errs.push('swap ' + JSON.stringify({ before, mid3d: mid, after }));
  // Placement: a jump to a city plus a tile picker in one update must land the 2D map on the city.
  const city = await p.evaluate(() => { const s = window.legatum.getState(); const v = s.views[s.seat]; const c = v.cities.find((x) => x.owner === v.me && !x.is_capital) ?? v.cities.find((x) => x.owner === v.me); s.select({ kind: 'city', id: c.id }); return c; });
  await p.waitForTimeout(1500);
  await p.evaluate(() => { const sc = window.legatumMap3d; sc.target.x += 9; sc.placeCamera(); });
  await p.evaluate((c) => { const s = window.legatum.getState(); s.jumpTo(c.pos); s.set({ place: { city: c.id, building: null, intent: 'tile', pick: null } }); }, city);
  await p.waitForTimeout(2000);
  errs.push('place ' + JSON.stringify({ city: city.pos, st: await p.evaluate(() => ({ is3d: !!document.querySelector('.map3d-canvas'), cam: window.legatumMap?.camera })) }));
  await p.close();
}
console.log(errs.slice(0, 20).join('\n'));
await b.close(); await server.close();
