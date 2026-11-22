import { revealSidebar } from './sidebarAssertions';
// Release-build measurement only. Run with tools/perf/app-measure.sh on the cloud box.
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { test, expect, type Page } from '@playwright/test';

test.skip(process.env.LEGATUM_PERF !== '1', 'measurement only');
test.use({ viewport: { width: 1000, height: 600 } });
test.setTimeout(30 * 60_000);

const root = resolve(process.cwd(), '..');
const base = process.env.LEGATUM_PERF_URL ?? 'http://localhost:4173';
const tag = process.env.LEGATUM_PERF_TAG ?? 'baseline';
// The late-game save's turn. A game that ends before turn 200 (the fixture can) sets this lower.
const late = Number(process.env.LEGATUM_PERF_LATE ?? 200);
type LongTask = { start: number; duration: number };
type ProbeWindow = Window & { __perfProbe: { wasm: { kind: string; ms: number }[]; tasks: LongTask[]; memory: WebAssembly.Memory | null; click: number; turnClick: number; turnPaint: number };
  legatum: { getState(): Record<string, unknown> }; legatumMap: { perf: { frames: number; drawMs: number; maxMs: number; workMs: number; buildMs: number; measureFrames: boolean; reset(): void } }; legatumPerf?: Record<string, number> };

async function probe(page: Page) {
  await page.addInitScript(() => {
    const w = window as unknown as ProbeWindow;
    const p: ProbeWindow['__perfProbe'] = w.__perfProbe = { wasm: [], tasks: [], memory: null, click: 0, turnClick: 0, turnPaint: 0 };
    new PerformanceObserver((list) => {
      for (const e of list.getEntries()) p.tasks.push({ start: e.startTime, duration: e.duration });
    }).observe({ type: 'longtask', buffered: true });
    document.addEventListener('click', (event) => {
      p.click = performance.now();
      if ((event.target as Element).closest('.seatbar')) p.turnClick = p.click;
    }, true);
    const original = WebAssembly.instantiate;
    (WebAssembly as unknown as { instantiate: (...args: unknown[]) => Promise<unknown> }).instantiate = async (...args) => {
      const t = performance.now();
      const result = await (original as (...a: unknown[]) => Promise<unknown>)(...args);
      p.wasm.push({ kind: 'instantiate', ms: performance.now() - t });
      const instance = (result as WebAssembly.WebAssemblyInstantiatedSource).instance ?? result as WebAssembly.Instance;
      p.memory = instance.exports.memory as WebAssembly.Memory ?? p.memory;
      return result;
    };
    if (WebAssembly.instantiateStreaming) {
      const streaming = WebAssembly.instantiateStreaming;
      (WebAssembly as unknown as { instantiateStreaming: (...args: unknown[]) => Promise<unknown> }).instantiateStreaming = async (...args) => {
        const t = performance.now();
        const result = await (streaming as (...a: unknown[]) => Promise<unknown>)(...args);
        p.wasm.push({ kind: 'instantiateStreaming', ms: performance.now() - t });
        p.memory = (result as WebAssembly.WebAssemblyInstantiatedSource).instance.exports.memory as WebAssembly.Memory ?? p.memory;
        return result;
      };
    }
  });
}

async function save(name: string, data: unknown) {
  await writeFile(resolve(root, 'out', `perf-app-${tag}-${name}.json`), JSON.stringify(data, null, 2));
  console.log(`APP-PERF ${name} ${name.endsWith('-partial') ? `${(data as unknown[]).length} rows` : JSON.stringify(data)}`);
}

async function loadSave(page: Page, name: 'start' | 'late') {
  const game = await readFile(resolve(root, 'out', `perf-save-${name === 'late' ? late : name}.json`), 'utf8');
  const turn = name === 'start' ? 1 : late + 1;
  await page.addInitScript(({ game, turn }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Me'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game, turn });
}

async function loaded(page: Page, turn: number) {
  await expect(page.getByTestId('turn-number')).toHaveText(String(turn), { timeout: 90_000 });
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  await page.evaluate(() => new Promise<void>((done) => requestAnimationFrame(() => requestAnimationFrame(() => done()))));
}

async function snapshot(page: Page) {
  return page.evaluate(() => {
    const w = window as unknown as ProbeWindow;
    const s = w.legatum.getState();
    const bytes = (v: unknown) => JSON.stringify(v)?.length ?? 0;
    const reports = s.reportHistory as unknown[];
    const views = s.views as unknown[];
    return { turn: (views?.[s.seat as number] as { turn: number })?.turn,
      heapMB: (performance as Performance & { memory: { usedJSHeapSize: number } }).memory.usedJSHeapSize / 1048576,
      wasmMB: w.__perfProbe.memory?.buffer.byteLength ? w.__perfProbe.memory.buffer.byteLength / 1048576 : null,
      viewsKB: bytes(views) / 1024, reportKB: bytes(s.report) / 1024,
      reportHistoryKB: bytes(reports) / 1024, reportCount: reports?.length ?? 0,
      eventCount: reports?.reduce<number>((n, r) => n + ((r as { events?: unknown[][] }).events?.reduce((total, seat) => total + seat.length, 0) ?? 0), 0) ?? 0,
      saveKB: (s.client as { save(): string })?.save().length / 1024 };
  });
}

async function gcSnapshot(page: Page) {
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('HeapProfiler.enable');
  await cdp.send('HeapProfiler.collectGarbage');
  await cdp.send('Performance.enable');
  const metrics = await cdp.send('Performance.getMetrics');
  const data = await snapshot(page);
  await cdp.detach();
  return { ...data, cdpHeapMB: (metrics.metrics.find((m) => m.name === 'JSHeapUsedSize')?.value ?? 0) / 1048576 };
}

async function clearTurnOverlays(page: Page) {
  // Major moments mount just after the turn number changes and can cover End Turn.
  await page.waitForTimeout(300);
  for (let i = 0; i < 12; i++) {
    if (!await page.locator('.moment-card, [data-testid="era-up-board"], [data-testid="alarm-screen"], [data-celebrate-delta], dialog[open]').count()
      && !await page.getByRole('button', { name: /^To arms/ }).count()) break;
    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);
  }
}

test('cold new game and Continue to a turn number', async ({ browser }) => {
  test.setTimeout(30 * 60_000);
  for (const mode of ['new', 'continue'] as const) {
    const context = await browser.newContext({ viewport: { width: 1000, height: 600 } });
    const page = await context.newPage();
    if (mode === 'continue') await loadSave(page, 'late');
    await probe(page);
    const cdp = await page.context().newCDPSession(page);
    await cdp.send('Network.enable');
    await cdp.send('Network.setCacheDisabled', { cacheDisabled: true });
    const started = Date.now();
    await page.goto(`${base}/?client=wasm&perf=1`, { waitUntil: 'commit' });
    await expect(page.getByRole('button', { name: mode === 'new' ? /^Single Player/ : /^Continue/ })).toBeVisible();
    const titleMs = Date.now() - started;
    if (mode === 'new') {
      await page.getByRole('button', { name: /^Single Player/ }).click();
      await page.getByRole('radiogroup', { name: 'Map size', exact: true }).getByRole('radio', { name: /^Standard/ }).click();
      await page.getByRole('button', { name: /^Start game/ }).click();
      await loaded(page, 1);
    } else {
      await page.getByRole('button', { name: /^Continue/ }).click();
      await loaded(page, late + 1);
    }
    const data = await page.evaluate(({ titleMs }) => {
      const w = window as unknown as ProbeWindow;
      const nav = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      const wasmResource = performance.getEntriesByType('resource').filter((e) => e.name.endsWith('.wasm')) as PerformanceResourceTiming[];
      return { titleMs, navToGameMs: performance.now(), clickToPaintMs: performance.now() - w.__perfProbe.click, dclMs: nav.domContentLoadedEventEnd,
        wasm: w.__perfProbe.wasm, wasmFetch: wasmResource.map((r) => ({ startMs: r.startTime, responseEndMs: r.responseEnd, transferKB: r.transferSize / 1024 })),
        longTasksOver100: w.__perfProbe.tasks.filter((t) => t.duration >= 100),
        phases: performance.getEntriesByType('measure').filter((e) => e.name.startsWith('legatum:') && e.name !== 'legatum:mapDraw').map((e) => ({ name: e.name, ms: e.duration })) };
    }, { titleMs });
    await save(`cold-${mode}`, data);
    await cdp.detach();
    await context.close();
  }
});

test('late-game pan, zoom and heavy panels', async ({ page }) => {
  test.setTimeout(30 * 60_000);
  page.setDefaultTimeout(12_000);
  await loadSave(page, 'late');
  await probe(page);
  await page.goto(`${base}/?client=wasm&perf=1`);
  await page.getByRole('button', { name: /^Continue/ }).click();
  await loaded(page, late + 1);
  await page.waitForTimeout(1200);
  const cdp = await page.context().newCDPSession(page);
  const trace: { name: string; dur?: number; ts?: number; ph?: string }[] = [];
  cdp.on('Tracing.dataCollected', ({ value }) => { trace.push(...value as unknown as typeof trace); });
  await cdp.send('Tracing.start', { categories: 'devtools.timeline,blink.user_timing', options: 'record-as-much-as-possible' });
  await page.evaluate(() => { performance.clearMeasures('legatum:mapDraw'); const perf = (window as unknown as ProbeWindow).legatumMap.perf; perf.reset(); perf.measureFrames = true; });
  const canvas = page.locator('.map-canvas').first();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('map canvas missing');
  await page.mouse.move(box.x + box.width * 0.45, box.y + box.height * 0.5);
  await page.mouse.down();
  for (let i = 0; i < 24; i++) await page.mouse.move(box.x + box.width * (0.45 + i * .01), box.y + box.height * (0.5 + i * .004));
  await page.mouse.up();
  for (let i = 0; i < 8; i++) await page.mouse.wheel(0, i % 2 ? 180 : -180);
  await page.waitForTimeout(1300);
  const frame = await page.evaluate(() => {
    const w = window as unknown as ProbeWindow;
    w.legatumMap.perf.measureFrames = false;
    const draws = performance.getEntriesByName('legatum:mapDraw', 'measure').map((e) => e.duration).sort((a, b) => a - b);
    return { overlay: w.legatumPerf, map: { ...w.legatumMap.perf }, jsFrameMeanMs: draws.reduce((a, b) => a + b, 0) / (draws.length || 1),
      jsFrameP95Ms: draws[Math.floor(draws.length * .95)] ?? 0, jsFrameCount: draws.length };
  });
  const done = new Promise<void>((resolveDone) => cdp.once('Tracing.tracingComplete', () => resolveDone()));
  await cdp.send('Tracing.end');
  await done;
  await cdp.detach();
  const js = trace.filter((e) => e.ph === 'X' && e.dur && ['FunctionCall', 'EvaluateScript', 'EventDispatch', 'FireAnimationFrame'].includes(e.name));
  const jsMs = js.reduce((a, e) => a + e.dur! / 1000, 0);
  await save('map', { frame, traceJsMs: jsMs, traceEvents: js.length, traceJsP95Ms: js.map((e) => e.dur! / 1000).sort((a, b) => a - b)[Math.floor(js.length * .95)] ?? 0 });

  const panels = [
    { name: 'cities', click: () => page.getByTestId('cities-button').click(), visible: '[data-testid="cities-overview"][open]', close: () => page.getByRole('button', { name: 'Close cities' }).click() },
    { name: 'research', click: () => page.locator('.research-chip').click(), visible: '.tech-modal[open]', close: () => page.getByRole('button', { name: 'Close research' }).click() },
    { name: 'diplomacy', click: async () => { await revealSidebar(page, 'standings'); await page.getByRole('button', { name: /^Diplomacy( \(\d+\))?$/ }).click(); }, visible: '.diplomacy-panel', close: () => page.getByRole('button', { name: 'Close diplomacy' }).click() },
  ];
  const panelRows = [];
  for (const panel of panels) {
    console.log(`APP-PERF opening ${panel.name}`);
    await panel.click();
    console.log(`APP-PERF clicked ${panel.name}`);
    await expect(page.locator(panel.visible)).toBeVisible();
    const row = await page.evaluate(() => {
      const w = window as unknown as ProbeWindow;
      return new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(() => {
        const ms = performance.now() - w.__perfProbe.click;
        done({ clickToPaintMs: ms, tasksOver100: w.__perfProbe.tasks.filter((t) => t.start >= w.__perfProbe.click && t.duration >= 100) });
      })));
    });
    panelRows.push({ panel: panel.name, ...row as object });
    console.log(`APP-PERF painted ${panel.name}`);
    await panel.close();
  }
  const cities = await page.evaluate(() => {
    const s = (window as unknown as ProbeWindow).legatum.getState();
    const v = (s.views as { cities: { id: number; owner: number; detail?: unknown }[]; me: number }[])[s.seat as number];
    const city = v.cities.find((c) => c.owner === v.me && c.detail);
    if (city) (s.select as (selection: unknown) => void)({ kind: 'city', id: city.id });
    return city?.id ?? null;
  });
  if (cities !== null) {
    const first = page.locator('[data-testid="where-to-build"] summary').first();
    if (await first.count()) {
      await first.click();
      await expect(page.locator('[data-testid="where-to-build"][open]').first()).toBeVisible();
      panelRows.push({ panel: 'where-to-build', ...await page.evaluate(() => {
        const w = window as unknown as ProbeWindow;
        return new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(() => done({ clickToPaintMs: performance.now() - w.__perfProbe.click,
          tasksOver100: w.__perfProbe.tasks.filter((t) => t.start >= w.__perfProbe.click && t.duration >= 100) }))));
      }) as object });
    }
  }
  await save('panels', panelRows);
});

test('End Turn and retained memory through turn 300', async ({ page }) => {
  test.setTimeout(30 * 60_000);
  page.setDefaultTimeout(12_000);
  const orders = (await readFile(resolve(root, 'out', 'perf-orders.jsonl'), 'utf8')).trim().split('\n').map((line) => JSON.parse(line) as Record<string, unknown[]>);
  await loadSave(page, 'start');
  await probe(page);
  await page.goto(`${base}/?client=wasm&perf=1`);
  await page.getByRole('button', { name: /^Continue/ }).click();
  await loaded(page, 1);
  await page.evaluate(() => {
    const w = window as unknown as ProbeWindow;
    const number = document.querySelector('[data-testid="turn-number"]')!;
    new MutationObserver(() => {
      const expected = number.textContent;
      requestAnimationFrame(() => requestAnimationFrame(() => {
        if (number.textContent === expected) w.__perfProbe.turnPaint = performance.now();
      }));
    }).observe(number, { childList: true, characterData: true, subtree: true });
  });
  const turns = [];
  const memory = [await gcSnapshot(page)];
  for (let turn = 1; turn <= Number(process.env.LEGATUM_PERF_TURNS ?? 300); turn++) {
    await clearTurnOverlays(page);
    await page.evaluate((mine) => {
      const w = window as unknown as ProbeWindow;
      w.__perfProbe.turnPaint = 0;
      const s = w.legatum.getState();
      (s.set as (patch: unknown) => void)({ orders: [mine] });
    }, orders[turn - 1]['0'] ?? []);
    await page.locator('.seatbar').getByRole('button', { name: /^(Ready anyway|End turn|Ready\b)/ }).click();
    const confirm = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
    if (await confirm.isVisible()) await confirm.getByRole('button', { name: 'End turn anyway', exact: true }).click();
    await loaded(page, turn + 1);
    const row = await page.evaluate(() => {
      const w = window as unknown as ProbeWindow;
      const t = w.__perfProbe.turnClick;
      const painted = w.__perfProbe.turnPaint || performance.now();
      const tasks = w.__perfProbe.tasks.filter((e) => e.start >= t && e.start <= painted);
      return { turn: (w.legatum.getState().views as { turn: number }[])[0].turn - 1,
        clickToPaintMs: painted - t, blockedMs: tasks.reduce((a, x) => a + x.duration, 0), maxTaskMs: Math.max(0, ...tasks.map((x) => x.duration)),
        phases: performance.getEntriesByType('measure').filter((e) => e.name.startsWith('legatum:') && e.name !== 'legatum:mapDraw' && e.startTime >= t).map((e) => ({ name: e.name, ms: e.duration })),
        mapBuildMs: w.legatumMap.perf.buildMs };
    });
    turns.push(row);
    if (turn % 25 === 0) {
      memory.push(await gcSnapshot(page));
      await save('turns-partial', turns);
      await save('memory-partial', memory);
    }
  }
  await save('turns', turns);
  await save('memory', memory);
});

test('late End Turn CPU profile', async ({ page }) => {
  test.skip(process.env.LEGATUM_PERF_PROFILE !== '1', 'profile on demand');
  test.setTimeout(5 * 60_000);
  await loadSave(page, 'late');
  await probe(page);
  await page.goto(`${base}/?client=wasm&perf=1`);
  await page.getByRole('button', { name: /^Continue/ }).click();
  await loaded(page, late + 1);
  await clearTurnOverlays(page);
  const orders = (await readFile(resolve(root, 'out', 'perf-orders.jsonl'), 'utf8')).trim().split('\n');
  await page.evaluate((mine) => {
    const s = (window as unknown as ProbeWindow).legatum.getState();
    (s.set as (patch: unknown) => void)({ orders: [mine] });
  }, (JSON.parse(orders[late]) as Record<string, unknown[]>)['0'] ?? []);
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('Profiler.enable');
  await cdp.send('Profiler.start');
  await page.locator('.seatbar').getByRole('button', { name: /^(Ready anyway|End turn|Ready\b)/ }).click();
  const confirm = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
  if (await confirm.isVisible()) await confirm.getByRole('button', { name: 'End turn anyway', exact: true }).click();
  await loaded(page, late + 2);
  const { profile } = await cdp.send('Profiler.stop');
  await cdp.detach();
  await writeFile(resolve(root, 'out', `perf-app-${tag}-cpu-profile.json`), JSON.stringify(profile));
  const totals = new Map<number, number>();
  for (let i = 0; i < (profile.samples?.length ?? 0); i++) {
    const id = profile.samples![i];
    totals.set(id, (totals.get(id) ?? 0) + (profile.timeDeltas?.[i] ?? 0) / 1000);
  }
  const top = profile.nodes.map((node) => ({ ms: Math.round(totals.get(node.id) ?? 0), name: node.callFrame.functionName,
    url: node.callFrame.url, line: node.callFrame.lineNumber + 1, column: node.callFrame.columnNumber + 1 }))
    .sort((a, b) => b.ms - a.ms).slice(0, 30);
  await save('cpu-top', top);
});

test('selected city: idle frames and End Turn', async ({ page }) => {
  // A selection pulses the map; a player nearly always has one. Counts the full-map draws and the main-thread
  // time a 3 s idle stretch costs with and without a selection, then End Turn with the selection held.
  test.setTimeout(10 * 60_000);
  page.setDefaultTimeout(12_000);
  await loadSave(page, 'late');
  await probe(page);
  await page.goto(`${base}/?client=wasm&perf=1`);
  await page.getByRole('button', { name: /^Continue/ }).click();
  await loaded(page, late + 1);
  await clearTurnOverlays(page);
  await page.waitForTimeout(1500);
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('Performance.enable');
  const idle = async (selected: boolean) => {
    await page.evaluate((on) => {
      const s = (window as unknown as ProbeWindow).legatum.getState();
      const v = (s.views as { cities: { id: number; owner: number }[]; me: number }[])[s.seat as number];
      const city = v.cities.find((c) => c.owner === v.me);
      (s.select as (selection: unknown) => void)(on && city ? { kind: 'city', id: city.id } : null);
    }, selected);
    await page.waitForTimeout(800);
    const metric = async () => Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map((m) => [m.name, m.value]));
    await page.evaluate(() => { performance.clearMeasures('legatum:mapDraw'); (window as unknown as ProbeWindow).legatumMap.perf.measureFrames = true; });
    const before = await metric();
    await page.waitForTimeout(3000);
    const after = await metric();
    const map = await page.evaluate(() => { const d = performance.getEntriesByName('legatum:mapDraw', 'measure'); return { frames: d.length, drawMs: d.reduce((a, e) => a + e.duration, 0) }; });
    return { selected, fullDraws: map.frames, drawMs: map.drawMs, taskMs: (after.TaskDuration - before.TaskDuration) * 1000,
      scriptMs: (after.ScriptDuration - before.ScriptDuration) * 1000, layoutMs: (after.LayoutDuration - before.LayoutDuration) * 1000 };
  };
  const idleRows = [await idle(false), await idle(true)];
  const orders = (await readFile(resolve(root, 'out', 'perf-orders.jsonl'), 'utf8')).trim().split('\n').map((line) => JSON.parse(line) as Record<string, unknown[]>);
  await page.evaluate(() => {
    const w = window as unknown as ProbeWindow;
    const number = document.querySelector('[data-testid="turn-number"]')!;
    new MutationObserver(() => {
      const expected = number.textContent;
      requestAnimationFrame(() => requestAnimationFrame(() => { if (number.textContent === expected) w.__perfProbe.turnPaint = performance.now(); }));
    }).observe(number, { childList: true, characterData: true, subtree: true });
  });
  const turns: Record<string, number>[] = [];
  for (let turn = late + 1; turn <= late + 15; turn++) {
    await clearTurnOverlays(page);
    await page.evaluate(({ mine }) => {
      const w = window as unknown as ProbeWindow;
      w.__perfProbe.turnPaint = 0;
      const s = w.legatum.getState();
      const v = (s.views as { cities: { id: number; owner: number }[]; me: number }[])[s.seat as number];
      const city = v.cities.find((c) => c.owner === v.me);
      (s.set as (patch: unknown) => void)({ orders: [mine] });
      (s.select as (selection: unknown) => void)(city ? { kind: 'city', id: city.id } : null);
    }, { mine: orders[turn - 1]['0'] ?? [] });
    await page.waitForTimeout(300);
    await page.evaluate(() => { performance.clearMeasures('legatum:mapDraw'); (window as unknown as ProbeWindow).legatumMap.perf.measureFrames = true; });
    await page.locator('.seatbar').getByRole('button', { name: /^(Ready anyway|End turn|Ready\b)/ }).click();
    const confirm = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
    if (await confirm.isVisible()) await confirm.getByRole('button', { name: 'End turn anyway', exact: true }).click();
    await loaded(page, turn + 1);
    turns.push(await page.evaluate(() => {
      const w = window as unknown as ProbeWindow;
      const t = w.__perfProbe.turnClick;
      const painted = w.__perfProbe.turnPaint || performance.now();
      const tasks = w.__perfProbe.tasks.filter((e) => e.start >= t && e.start <= painted);
      return { clickToPaintMs: painted - t, blockedMs: tasks.reduce((a, x) => a + x.duration, 0), mapFrames: performance.getEntriesByName('legatum:mapDraw', 'measure').length, mapDrawMs: performance.getEntriesByName('legatum:mapDraw', 'measure').reduce((a, e) => a + e.duration, 0) };
    }));
  }
  await cdp.detach();
  const mean = (key: string) => turns.reduce((a, r) => a + r[key], 0) / turns.length;
  await save('selected', { idle: idleRows, endTurn: { clickToPaintMs: mean('clickToPaintMs'), blockedMs: mean('blockedMs'), mapFrames: mean('mapFrames'), mapDrawMs: mean('mapDrawMs') }, turns });
});
