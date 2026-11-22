// Store integration: solo and hot-seat games through the real Rust core.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { GameClient } from './client';
import { coopVsAiSetup, teamTarget } from './gameSetup';
import { applyPreset, buildSetup, newLobby, removeSlot, setDifficulty, setSlot } from './lobby';
import { FixtureClient } from './test/fixtureEngine';
import { saveFile, stepReport, toggleLastTurnReport, useGame, viewedReport } from './store';
import { contextFromViews, filterLines } from './format';
import { neighbors } from './hex';

let makeClient: () => Promise<GameClient>;
// Archive transport/storage is exercised separately against IndexedDB and HTTP. This store
// integration runs in Node, where a relative /api URL is not a browser network destination.
vi.mock('./platform/archive', () => ({
  recordGameHistory: vi.fn().mockResolvedValue(undefined),
  abandonGameHistory: vi.fn().mockResolvedValue(undefined),
}));
vi.mock('./client', async (orig) => ({
  ...(await orig<typeof import('./client')>()),
  createClient: () => makeClient(),
}));

async function until(pred: () => boolean, ms = 20000) {
  const end = Date.now() + ms;
  while (!pred()) {
    if (Date.now() > end) throw new Error('timed out');
    await new Promise((r) => setTimeout(r, 5));
  }
}

/** The capital is already founded; press Ready (Enter) every turn. */
async function playSoloTurns(turns: number) {
  for (let t = 0; t < turns; t++) {
    const before = useGame.getState().views![0].turn;
    useGame.getState().toggleReady();
    await until(() => !useGame.getState().busy);
    expect(useGame.getState().error).toBeNull();
    expect(useGame.getState().views![0].turn).toBe(before + 1);
    expect(useGame.getState().ready).toEqual([false]);
  }
}

const wasmPkg = fileURLToPath(new URL('./wasm-pkg/legatum.js', import.meta.url));
const hasWasm = existsSync(wasmPkg);

const clients: [string, boolean, () => Promise<() => Promise<GameClient>>][] = [
  [
    'wasm',
    hasWasm,
    async () => {
      const pkg = './wasm-pkg/legatum.js';
      const mod = await import(/* @vite-ignore */ pkg);
      await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('./wasm-pkg/legatum_bg.wasm', import.meta.url))) });
      const { WasmClient } = await import('./wasmClient');
      return async () => new WasmClient(mod.Game);
    },
  ],
];

describe.each(clients)('store on the %s client', (_name, available, load) => {
  beforeEach(async () => {
    if (available) makeClient = await load();
  });

  it('explains why a save from the retired TypeScript engine cannot load', async () => {
    await useGame.getState().startGame(coopVsAiSetup('776', [{ name: 'Ann', faction: 'India' }]));
    const current = useGame.getState();
    const save = saveFile(current.client!, current.views!);
    await current.loadSave({ ...save, client: 'mock' as typeof save.client });
    expect(useGame.getState().error).toContain('retired TypeScript engine');
  });

  it('reads each human view once when starting and loading', async () => {
    const client = new FixtureClient();
    const view = vi.spyOn(client, 'view');
    makeClient = async () => client;
    const setup = coopVsAiSetup('776', [
      { name: 'Ann', faction: 'India' },
      { name: 'Bo', faction: 'Byzantium' },
    ]);
    await useGame.getState().startGame(setup);
    expect(useGame.getState().humans).toEqual([0, 1]);
    expect(view.mock.calls).toEqual([[0], [1]]);

    const save = saveFile(client, useGame.getState().views!);
    view.mockClear();
    await useGame.getState().loadSave(save);
    expect(view.mock.calls).toEqual([[0], [1]]);
  });

  it.skipIf(!available)('only auto-opens the report for diplomacy involving the viewer team', async () => {
    await useGame.getState().startGame(coopVsAiSetup('140', [{ name: 'Ann', faction: 'India' }]));
    const client = useGame.getState().client!;
    const original = client.endTurn.bind(client);
    const spy = vi.spyOn(client, 'endTurn');
    try {
      for (const subject of [2, 0]) {
        spy.mockImplementation((orders) => {
          const result = original(orders);
          const event = { type: 'AllianceBetrayed' as const, betrayal: { by: 1, against: subject, turn: result.turn, until: result.turn + 20 } };
          return { ...result, events: [event], events_by_player: { 0: [event] } };
        });
        useGame.getState().toggleReady();
        await until(() => !useGame.getState().busy);
        expect(useGame.getState().reportOpen).toBe(subject === 0);
      }
    } finally { spy.mockRestore(); }
  });

  it.skipIf(!available)('keeps recent turn reports to step through; L reopens the latest; a new game starts empty', async () => {
    await useGame.getState().startGame(coopVsAiSetup('142', [{ name: 'Ann', faction: 'India' }]));
    expect(useGame.getState().reportHistory).toEqual([]);
    await playSoloTurns(3);
    const s = useGame.getState();
    const turns = s.reportHistory.map((r) => r.turn);
    expect(turns).toHaveLength(3);
    expect(turns).toEqual([turns[0], turns[0] + 1, turns[0] + 2]);
    expect(s.reportHistory.at(-1)).toBe(s.report);
    expect(viewedReport(s)).toBe(s.report);
    // The capital's first growth (10 Food) can open the latest report by itself; start closed.
    if (useGame.getState().reportOpen) toggleLastTurnReport();
    toggleLastTurnReport();
    expect(useGame.getState().reportOpen).toBe(true);
    stepReport(-1);
    expect(viewedReport(useGame.getState())!.turn).toBe(turns[1]);
    stepReport(-1);
    stepReport(-1); // already the oldest: stays
    expect(viewedReport(useGame.getState())!.turn).toBe(turns[0]);
    stepReport(1);
    stepReport(1);
    expect(useGame.getState().reportTurn).toBeNull();
    expect(viewedReport(useGame.getState())).toBe(useGame.getState().report);
    stepReport(-1);
    toggleLastTurnReport(); // closes
    expect(useGame.getState().reportOpen).toBe(false);
    toggleLastTurnReport(); // reopens on the latest turn
    expect(useGame.getState()).toMatchObject({ reportOpen: true, reportTurn: null });
    // The next turn is appended and shown as the latest.
    await playSoloTurns(1);
    expect(useGame.getState().reportHistory.map((r) => r.turn)).toEqual([...turns, turns[2] + 1]);
    expect(useGame.getState().reportTurn).toBeNull();
    await useGame.getState().startGame(coopVsAiSetup('143', [{ name: 'Ann', faction: 'India' }]));
    expect(useGame.getState()).toMatchObject({ report: null, reportHistory: [], reportTurn: null, reportArrow: null });
  });

  it.skipIf(!available)('an invalid-order toast does not outlive the turn it was about', async () => {
    await useGame.getState().startGame(coopVsAiSetup('141', [{ name: 'Ann', faction: 'India' }]));
    const v = useGame.getState().views![0];
    const scout = v.units.find((u) => u.owner === v.me && u.kind === 'Scout')!;
    // An order for a unit that does not exist is invalid and raises visible feedback.
    useGame.getState().editOrders((o) => [...o, { type: 'MoveUnit', unit: 999999, to: scout.pos }], 'commit');
    expect(useGame.getState().error).not.toBeNull();
    useGame.getState().editOrders((o) => o.filter((x) => !(x.type === 'MoveUnit' && x.unit === 999999)));
    useGame.getState().toggleReady();
    await until(() => !useGame.getState().busy);
    expect(useGame.getState().views![0].turn).toBe(v.turn + 1);
    expect(useGame.getState().error).toBeNull();
  });

  it.skipIf(!available)('resuming the same recording keeps it live; replacing it archives its last turns', async () => {
    const { abandonGameHistory } = await import('./platform/archive');
    await useGame.getState().startGame(coopVsAiSetup('901', [{ name: 'Ann', faction: 'India' }]));
    await playSoloTurns(2);
    const current = useGame.getState();
    const save = saveFile(current.client!, current.views!);
    const history = current.client!.archive!();
    vi.mocked(abandonGameHistory).mockClear();
    await current.loadSave(save);
    expect(abandonGameHistory).not.toHaveBeenCalled();
    expect(useGame.getState().client!.archive!()).toBe(history);
    await useGame.getState().startGame(coopVsAiSetup('902', [{ name: 'Ann', faction: 'India' }]));
    expect(abandonGameHistory).toHaveBeenCalledExactlyOnceWith(history);
  });

  it.skipIf(!available)('plays a solo game: one human seat, Ready resolves the turn at once', async () => {
    await useGame.getState().startGame(coopVsAiSetup('12345', [{ name: 'Ann', faction: 'India' }]));
    const s = useGame.getState();
    expect(s.error).toBeNull();
    expect(s.humans).toEqual([0]);
    expect(s.views).toHaveLength(1);
    const v = s.views![0];
    expect(v.players).toHaveLength(8);
    expect(v.players.filter((p) => p.is_human).map((p) => p.id)).toEqual([0]);
    expect(new Set(v.players.map((p) => p.team)).size).toBe(8);
    const mine = v.standings.find((t) => t.team === v.team);
    if (mine) expect(mine.target).toBe(teamTarget(1));
    await playSoloTurns(12);
    const after = useGame.getState().views![0];
    expect(after.cities.some((c) => c.owner === after.me)).toBe(true);
  });

  it.skipIf(!available)('plays a lobby game: small map, teams with an AI ally, easy and hard AIs', async () => {
    let l = applyPreset(newLobby('solo', '31337'), 'teams');
    for (const i of [7, 6, 5]) l = removeSlot(l, i);
    l = setSlot(setDifficulty({ ...l, mapSize: 'Small' }, 'Hard'), 2, { difficulty: 'Easy' });
    await useGame.getState().startGame(buildSetup(l));
    const v = useGame.getState().views![0];
    expect([v.width, v.height, v.turn_limit, v.players.length]).toEqual([40, 26, null, 5]);
    const mine = v.standings.find((t) => t.team === v.team)!;
    expect(mine.members).toHaveLength(2);
    expect(mine.target).toBe(teamTarget(2));
    await playSoloTurns(10);
  });

  it.skipIf(!available)('hot seat: the turn waits for every human seat', async () => {
    await useGame.getState().startGame(
      coopVsAiSetup('777', [
        { name: 'Ann', faction: 'India' },
        { name: 'Bo', faction: 'Byzantium' },
      ]),
    );
    const s = useGame.getState();
    expect(s.humans).toEqual([0, 1]);
    expect(s.views!.map((v) => v.me)).toEqual([0, 1]);
    const turn = s.views![0].turn;
    s.toggleReady(0);
    await new Promise((r) => setTimeout(r, 60));
    expect(useGame.getState().views![0].turn).toBe(turn);
    expect(useGame.getState().ready).toEqual([true, false]);
    useGame.getState().setSeat(1);
    useGame.getState().toggleReady();
    await until(() => !useGame.getState().busy);
    expect(useGame.getState().views![0].turn).toBe(turn + 1);
    expect(useGame.getState().ready).toEqual([false, false]);
  });

  it.skipIf(!available)('hot seat: passing the device covers the next player until they take it', async () => {
    const { readyAndPass, confirmEndTurn } = await import('./ui/SeatBar');
    await useGame.getState().startGame(coopVsAiSetup('778', [{ name: 'Ann', faction: 'India' }, { name: 'Bo', faction: 'Byzantium' }]));
    expect(useGame.getState().handoff).toBeNull();
    const endPlanning = () => { readyAndPass(); if (useGame.getState().endTurnConfirmation) confirmEndTurn(); };
    endPlanning();
    expect(useGame.getState()).toMatchObject({ seat: 1, handoff: 1 });
    useGame.getState().set({ handoff: null }); // Bo takes the device
    endPlanning();
    await until(() => !useGame.getState().busy);
    // The turn resolves with Bo still at the device: no cover until Bo passes it on.
    expect(useGame.getState()).toMatchObject({ seat: 1, handoff: null });
    endPlanning();
    expect(useGame.getState()).toMatchObject({ seat: 0, handoff: 0 });
    // Solo play never shows it.
    await useGame.getState().startGame(coopVsAiSetup('779', [{ name: 'Ann', faction: 'India' }]));
    endPlanning();
    await until(() => !useGame.getState().busy);
    expect(useGame.getState().handoff).toBeNull();
  }, 15_000);

  it.skipIf(!available)('an eliminated solo human can resolve the remaining AI game to its end', async () => {
    const fixture = new FixtureClient();
    makeClient = async () => fixture;
    const setup = coopVsAiSetup('42', [{ name: 'Ann', faction: 'India' }]);
    setup.turn_limit = 5;
    await useGame.getState().startGame(setup);
    const view = useGame.getState().views![0];
    const state = fixture.debugState();
    const capital = state.cities.find((city) => city.owner === view.me)!;
    state.cities = state.cities.filter((city) => city.id !== capital.id);
    state.units = state.units.filter((unit) => unit.owner !== view.me);
    state.tileCity.forEach((id, i) => { if (id === capital.id) { state.tileCity[i] = -1; state.owner[i] = -1; } });
    useGame.setState({ views: [fixture.view(view.me)] });
    useGame.getState().toggleReady();
    await until(() => !useGame.getState().busy);
    expect(useGame.getState().views![0].players.find((p) => p.id === view.me)?.alive).toBe(false);
    expect(useGame.getState().eliminationTurns[view.me]).toBe(1);
    expect(useGame.getState().views![0].finished).toBe(false);
    useGame.getState().resolveToEnd();
    await until(() => !useGame.getState().resolvingToEnd);
    expect(useGame.getState().error).toBeNull();
    expect(useGame.getState().views![0].finished).toBe(true);
    expect(useGame.getState().orders).toEqual([[]]);
  });

  it.skipIf(!available).each([1, 2])('starts without a capital founding report and filters animation paths for each of %i local seats', async (count) => {
    const setup = coopVsAiSetup('12345', [
      { name: 'Ann', faction: 'India' }, { name: 'Bo', faction: 'Byzantium' },
    ].slice(0, count) as Parameters<typeof coopVsAiSetup>[1]);
    setup.players.forEach((p, i) => { p.team = i; });
    await useGame.getState().startGame(setup);
    const client = useGame.getState().client!;
    const resolve = vi.spyOn(client, 'endTurn');
    const before = useGame.getState().views!;
    for (let seat = 0; seat < count; seat++) {
      useGame.getState().setSeat(seat);
      const v = before[seat];
      const scout = v.units.find((u) => u.owner === v.me && u.kind === 'Scout')!;
      const to = neighbors(scout.pos).find((h) => !client.validate(v.me, [{ type: 'MoveUnit', unit: scout.id, to: h }]).length)!;
      useGame.getState().editOrders(() => [{ type: 'MoveUnit', unit: scout.id, to }]);
      useGame.getState().toggleReady();
    }
    await until(() => !useGame.getState().busy);
    expect(useGame.getState().error).toBeNull();
    const raw = resolve.mock.results[0].value.events as import('./types').Event[];
    expect(raw.some((event) => event.type === 'CityFounded')).toBe(false);
    const state = useGame.getState();
    for (let seat = 0; seat < count; seat++) {
      const v = state.views![seat];
      expect(state.report!.events[seat].some((event) => event.type === 'CityFounded')).toBe(false);
      expect(filterLines(state.report!.lines[seat], 'all', contextFromViews(before[seat], v)).some((line) =>
        line.category === 'cities' && line.text.includes('founded'))).toBe(false);
    }
    const active = state.views![state.seat];
    expect(state.anim?.paths.size).toBeGreaterThan(0);
    for (const id of state.anim!.paths.keys()) {
      expect(active.units.find((u) => u.id === id)?.owner).toBe(active.me);
    }
    if (count > 1) {
      useGame.getState().setSeat(0);
      expect(useGame.getState().anim).toBeNull(); // Do not carry the other seat's animation across.
      expect(useGame.getState().report!.lines[0]).toEqual(state.report!.lines[0]);
    }
    resolve.mockRestore();
  });
});
