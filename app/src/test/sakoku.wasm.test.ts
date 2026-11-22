// Japan's Sakoku vow: the fixtureClient and the Rust core agree on the action, what a closed country blocks (war, Alliances, tech
// diffusion), the public countdown, and holding and losing the live VP.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import type { Faction, GameAction, Order, PlayerView } from '../types';
import { fromOffset, within } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture(factions: Faction[]) {
  const setup = makeStandardSetup('90212', factions[0], factions[1], 'A', 'B');
  setup.width = setup.height = 24;
  setup.players = factions.map((faction, team) => ({ ...(setup.players[team % 2]), faction, name: `P${team}`, team, is_human: true }));
  const s = createState(setup);
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  s.wonder?.fill(null); s.wondersDiscovered = [];
  for (const relation of s.diplomacy?.relations ?? []) relation.state = 'Peace';
  for (const p of s.players) p.gold = 500;
  s.players[0].goalRule = 'Sakoku';
  return s;
}

function city(s: MState, owner: number, col: number, row: number, population = 3) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: `C${s.nextCityId}`, pos: fromOffset(col, row),
    population, food: 3, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: s.cities.every((x) => x.owner !== owner), foundedTurn: 1, resistance: 0,
  };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  for (const h of within(c.pos, 6)) {
    if (c.grownTiles.length >= population - 1) break;
    const i = idxOf(s, h);
    if (i < 0 || s.owner[i] >= 0) continue;
    s.owner[i] = owner; s.tileCity[i] = c.id; c.grownTiles.push(h); c.tilesClaimed++;
  }
  return c;
}

function pair(s: MState) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  return { fixtureClient, core };
}

const action = (a: GameAction): Order => ({ type: 'Action', action: a });
const close: Order = action({ action: 'Sakoku', close: true });
const reopen: Order = action({ action: 'Sakoku', close: false });
const goalEvents = (events: { type: string }[]) => events.filter((e) => ['FactionGoalCompleted', 'FactionGoalLost', 'SakokuClosed', 'SakokuReopened', 'OrderFailed'].includes(e.type));

/** India (0) and a Mongol (1) at Peace, each with two cities. */
function world() {
  const s = fixture(['India', 'Mongols']);
  city(s, 0, 4, 4, 5); city(s, 0, 4, 12, 4);
  city(s, 1, 18, 18, 4); city(s, 1, 18, 8, 3);
  return s;
}

describe.skipIf(!available)('Sakoku core behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('shows a legacy scenario the vow and nobody else, the same way', () => {
    const { fixtureClient, core } = pair(world());
    const view = core.view(0).actions!.sakoku!;
    expect(view).toMatchObject({ closed: false, turns: 0, target: 30, holding: false, blocker: null });
    (expect(fixtureClient.view(0).actions).toMatchSnapshot(), void (core.view(0).actions));
    expect(core.view(1).actions!.sakoku ?? null).toBeNull();
    (expect(fixtureClient.view(1).actions).toMatchSnapshot(), void (core.view(1).actions));
    (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
    expect(core.view(0).faction_goal).toMatchObject({ value: 0, target: 30, live: true, done: false });
    // Public: the country is open.
    expect(core.view(1).players[0].sakoku_turns ?? null).toBeNull();
    (expect(fixtureClient.view(1).players.map((p) => p.sakoku_turns ?? null)).toMatchSnapshot(), void (core.view(1).players.map((p) => p.sakoku_turns ?? null)));
  });

  it('validates the vow the same way: Alliance, riders, other factions', () => {
    const s = world();
    const { fixtureClient, core } = pair(s);
    (expect(fixtureClient.validate(0, [close])).toMatchSnapshot(), void (core.validate(0, [close])));
    expect(core.validate(0, [close])).toEqual([]);
    expect(core.validate(1, [close])[0].message).toContain('Only an empire with Sakoku');
    (expect(fixtureClient.validate(1, [close])).toMatchSnapshot(), void (core.validate(1, [close])));
    (expect(fixtureClient.validate(0, [reopen])).toMatchSnapshot(), void (core.validate(0, [reopen])));
    expect(core.validate(0, [reopen])[0].message).toContain('already open');
    const war: Order = { type: 'DeclareWar', against: 1, breach: false };
    (expect(fixtureClient.validate(0, [war, close])).toMatchSnapshot(), void (core.validate(0, [war, close])));
    expect(core.validate(0, [war, close]).some((e) => e.message.includes("can't declare war"))).toBe(true);
    const ally: Order = { type: 'ProposeTreaty', to: 1, kind: 'Alliance' };
    (expect(fixtureClient.validate(0, [ally, close])).toMatchSnapshot(), void (core.validate(0, [ally, close])));
    (expect(fixtureClient.validate(0, [close, close])).toMatchSnapshot(), void (core.validate(0, [close, close])));
    // In an Alliance the vow is refused until it is left.
    const allied = world();
    for (const r of allied.diplomacy?.relations ?? []) r.state = 'Alliance';
    const a = pair(allied);
    expect(a.core.validate(0, [close])[0]?.message ?? '').toContain('Leave your Alliance');
    (expect(a.fixtureClient.validate(0, [close])).toMatchSnapshot(), void (a.core.validate(0, [close])));
  });

  it('closes, blocks war and Alliances, counts publicly and holds the VP after 30 turn ends, as core does', () => {
    const { fixtureClient, core } = pair(world());
    const a = fixtureClient.endTurn({ 0: [close] }), b = core.endTurn({ 0: [close] });
    (expect(goalEvents(a.events)).toMatchSnapshot(), void (goalEvents(b.events)));
    expect(b.events.find((e) => e.type === 'SakokuClosed')).toMatchObject({ player: 0 });
    for (const me of [0, 1]) {
      (expect(fixtureClient.view(me).players.map((p) => p.sakoku_turns ?? null)).toMatchSnapshot(), void (core.view(me).players.map((p) => p.sakoku_turns ?? null)));
    }
    expect(core.view(1).players[0].sakoku_turns).toBe(1);
    // A closed country can't declare war or ally, and nobody can ally with it; the reasons agree.
    const war: Order = { type: 'DeclareWar', against: 1, breach: false };
    const ally: Order = { type: 'ProposeTreaty', to: 1, kind: 'Alliance' };
    const asIndia = [war, ally];
    (expect(fixtureClient.validate(0, asIndia)).toMatchSnapshot(), void (core.validate(0, asIndia)));
    expect(core.validate(0, [war])[0].message).toContain('closed the country');
    expect(core.validate(0, [ally])[0].message).toContain('closed the country');
    const toIndia: Order = { type: 'ProposeTreaty', to: 0, kind: 'Alliance' };
    (expect(fixtureClient.validate(1, [toIndia])).toMatchSnapshot(), void (core.validate(1, [toIndia])));
    expect(core.validate(1, [toIndia])[0].message).toContain('closed the country');
    (expect(fixtureClient.view(0).diplomacy.actions.map((x) => x.blocked_reason ?? null)).toMatchSnapshot(), void (core.view(0).diplomacy.actions.map((x) => x.blocked_reason ?? null)));
    // The countdown: 29 more turn ends reach 30, and the VP holds then, not before.
    for (let n = 2; n <= 30; n++) {
      const x = fixtureClient.endTurn({}), y = core.endTurn({});
      expect(goalEvents(x.events)).toEqual(goalEvents(y.events));
      expect(core.view(0).players[0].faction_goal_done).toBe(n >= 30);
      expect(fixtureClient.view(0).players[0].faction_goal_done).toBe(n >= 30);
    }
    expect(core.view(0).players[0].vp_sources).toContainEqual([{ kind: 'FactionGoal', of: 0 }, 2]);
    (expect(fixtureClient.view(0).players[0].vp_sources).toMatchSnapshot(), void (core.view(0).players[0].vp_sources));
    (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
    expect(core.view(0).actions!.sakoku).toMatchObject({ closed: true, turns: 30, holding: true });
    (expect(fixtureClient.view(0).actions).toMatchSnapshot(), void (core.view(0).actions));
  });

  it('keeps the VP while an enemy unit stands in the land', () => {
    const s = world();
    s.players[0].sakokuTurns = 30; s.players[0].factionGoalDone = true; s.players[0].factionGoalReachedBefore = true;
    // The rival is at war with India and has a Soldier on India's land.
    for (const r of s.diplomacy?.relations ?? []) r.state = 'War';
    // A tile beside India's first city, in its sight and its land.
    const inside = within(s.cities[0].pos, 1).find((h) => h.q !== s.cities[0].pos.q || h.r !== s.cities[0].pos.r)!;
    spawnUnit(s, 1, 'Soldier', inside, false);
    const { fixtureClient, core } = pair(s);
    const x = fixtureClient.endTurn({}), y = core.endTurn({});
    expect(goalEvents(x.events)).toEqual(goalEvents(y.events));
    expect(goalEvents(y.events).some((e) => e.type === 'FactionGoalLost')).toBe(false);
    expect(core.view(0).players[0].faction_goal_done).toBe(true);
    (expect(fixtureClient.view(0).faction_goal).toMatchSnapshot(), void (core.view(0).faction_goal));
    expect(core.view(0).actions!.sakoku!.holding).toBe(true);
    (expect(fixtureClient.view(0).actions).toMatchSnapshot(), void (core.view(0).actions));
  });

  it('reopening loses the VP and restarts the count, identically', () => {
    const s = world();
    s.players[0].sakokuTurns = 33; s.players[0].factionGoalDone = true; s.players[0].factionGoalReachedBefore = true;
    const { fixtureClient, core } = pair(s);
    const x = fixtureClient.endTurn({ 0: [reopen] }), y = core.endTurn({ 0: [reopen] });
    expect(goalEvents(x.events)).toEqual(goalEvents(y.events));
    expect(y.events.find((e) => e.type === 'SakokuReopened')).toMatchObject({ player: 0, turns: 33 });
    expect(core.view(1).players[0].sakoku_turns ?? null).toBeNull();
    expect(core.view(0).players[0].faction_goal_done).toBe(false);
    (expect(fixtureClient.view(0).players[0].vp_sources).toMatchSnapshot(), void (core.view(0).players[0].vp_sources));
    const again = fixtureClient.endTurn({ 0: [close] }), core2 = core.endTurn({ 0: [close] });
    expect(goalEvents(again.events)).toEqual(goalEvents(core2.events));
    expect(core.view(1).players[0].sakoku_turns).toBe(1);
  });

  it('keeps foreign units out of a closed country at Peace, and out of research discounts', () => {
    const s = world();
    // A Mongol Scout is already barred at Peace; a closed country bars foreign units too.
    spawnUnit(s, 1, 'Scout', fromOffset(10, 4), false);
    // The rival knows a technology India could research: with the country open the price drops, closed it does not.
    const tech = pair(world()).core.view(0).research.available[0].tech;
    s.players[1].techs = [tech];
    const open = pair(s);
    const priceOf = (v: PlayerView) => v.research.available.find((t) => t.tech === tech)?.cost.total;
    const openCore = priceOf(open.core.view(0)), openMock = priceOf(open.fixtureClient.view(0));
    s.players[0].sakokuTurns = 4;
    const closed = pair(s);
    const closedPrice = priceOf(closed.core.view(0));
    // Closed, both engines charge the full price; open, each charges less (it knows the rival has it).
    expect(priceOf(closed.fixtureClient.view(0))).toBe(closedPrice);
    expect(closedPrice).toBeGreaterThan(openCore!);
    expect(closedPrice).toBeGreaterThan(openMock!);
    const scout = closed.core.view(1).units.find((u) => u.kind === 'Scout' && u.owner === 1)!;
    const into: Order = { type: 'MoveUnit', unit: scout.id, to: s.cities[0].grownTiles[0] ?? fromOffset(5, 4) };
    (expect(closed.fixtureClient.validate(1, [into])).toMatchSnapshot(), void (closed.core.validate(1, [into])));
    for (const c of [closed.core, closed.fixtureClient]) {
      for (let i = 0; i < 4; i++) c.endTurn({ 1: [into] });
    }
    (expect(closed.fixtureClient.view(1).units.map((u) => [u.id, u.pos])).toMatchSnapshot(), void (closed.core.view(1).units.map((u) => [u.id, u.pos])));
  });
});
