import { describe, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { advisor_orders } from '../wasm-pkg/legatum.js';
import { neighbors } from '../hex';
import type { Order, TechId } from '../types';
import { TECH_NAME } from '../data';
import { advisorSuggestion, advisorWhy, practiceKind, PRACTICE_TURNS, readAdvisorPreferences, useAdvisor, type AdvisorContext } from './advisorModel';

async function opening() {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('4545', 'Athens', 'India', 'Ada', 'Bo'));
  const view = client.view(0);
  const city = view.cities.find((c) => c.owner === view.me)!;
  const unit = view.units.find((u) => u.owner === view.me)!;
  const build = city.detail!.options[0].item;
  const tech = view.research.available[0].tech;
  const plan: Order[] = [
    { type: 'SetProduction', city: city.id, item: build },
    { type: 'MoveUnit', unit: unit.id, to: neighbors(unit.pos)[0] },
    { type: 'SetResearch', tech },
  ];
  const ctx = (orders: Order[] = [], selection: AdvisorContext['selection'] = null, chosen: Order[] | null = plan): AdvisorContext =>
    ({ view, orders, selection, plan: chosen, humans: 1, touch: false });
  return { view, city, unit, build, tech, plan, ctx };
}

describe('one decision at a time', () => {
  it('welcomes once, then follows turn-one city, unit, research order', async () => {
    const { city, unit, plan, ctx } = await opening();
    expect(advisorSuggestion(ctx(), [], [])?.id).toBe('welcome');
    expect(advisorSuggestion(ctx(), ['welcome'], [])).toMatchObject({ id: `city:${city.id}`, current: 0 });
    expect(advisorSuggestion(ctx([], { kind: 'city', id: city.id }), ['welcome'], [])).toMatchObject({ id: `city:${city.id}`, current: 1 });
    const cityDone = [plan[0]];
    expect(advisorSuggestion(ctx(cityDone), ['welcome'], [])?.id).toBe(`unit:${unit.id}`);
    expect(advisorSuggestion(ctx(cityDone, { kind: 'unit', id: unit.id }), ['welcome'], [])).toMatchObject({ id: `unit:${unit.id}`, current: 1 });
    const unitDone = [...cityDone, ...plan.filter((o) => !(!(o.type === 'MoveUnit') && !(o.type === 'MoveUnitPath'))).map((o) => ({ ...o, type: 'SkipUnit' as const }))];
    expect(advisorSuggestion(ctx(unitDone), ['welcome'], [])?.id).not.toBe(`unit:${unit.id}`);
    expect(advisorSuggestion(ctx(unitDone), ['welcome'], [])?.title).toBeTruthy();
  });

  it('follows the screen: the open Research panel or the selected unit comes first', async () => {
    const { city, unit, plan, ctx } = await opening();
    expect(advisorSuggestion({ ...ctx([plan[0]]), techOpen: true }, ['welcome'], [])?.id).toBe('research');
    expect(advisorSuggestion(ctx([], { kind: 'unit', id: unit.id }), ['welcome'], [])?.id).toBe(`unit:${unit.id}`);
    expect(advisorSuggestion(ctx([], { kind: 'city', id: city.id }), ['welcome'], [])?.id).toBe(`city:${city.id}`);
  });

  it('asks to close Research once research is chosen, before anything behind the panel', async () => {
    const { view, ctx } = await opening();
    const tech = view.research.available[0].tech;
    const s = advisorSuggestion({ ...ctx([{ type: 'SetResearch', tech }]), techOpen: true }, ['welcome'], []);
    expect(s).toMatchObject({ id: 'closeResearch', steps: [{ key: 'Esc', target: { kind: 'control', name: 'closeResearch' } }] });
    expect(advisorSuggestion(ctx([{ type: 'SetResearch', tech }]), ['welcome'], [])?.id).not.toBe('closeResearch');
    expect(advisorSuggestion({ ...ctx([{ type: 'SetResearch', tech }]), techOpen: true }, ['welcome'], ['closeResearch'])?.id).not.toBe('closeResearch');
    expect(advisorSuggestion({ ...ctx([]), techOpen: true }, ['welcome'], [])?.id).not.toBe('closeResearch');
  });

  it('accepts a different build and research choice as that decision made', async () => {
    const { city, view, ctx } = await opening();
    const chosen = city.detail!.options.at(-1)!.item;
    expect(advisorSuggestion(ctx([{ type: 'SetProduction', city: city.id, item: chosen }]), ['welcome'], [])?.id).not.toBe(`city:${city.id}`);
    const available = view.research.available.map((t) => t.tech);
    const alternative = available.at(-1)!;
    const orders: Order[] = [{ type: 'SetProduction', city: city.id, item: chosen }, { type: 'SetResearch', tech: alternative }];
    expect(advisorSuggestion(ctx(orders), ['welcome'], [])?.id).not.toBe('research');
  });

  it('falls back to a basic instruction if the plan is unavailable', async () => {
    const { city, ctx } = await opening();
    const s = advisorSuggestion(ctx([], null, null), ['welcome'], []);
    expect(s).toMatchObject({ id: `city:${city.id}`, title: `Choose a build in ${city.name}` });
    expect(s!.steps[1].instruction).toBe('Choose a unit or building to build.');
  });

  it.each(['SetAutoFire', 'UpgradeUnit'] as const)('keeps guiding a unit when Hard chooses %s', async (type) => {
    const { unit, plan, ctx } = await opening();
    const choice: Order = !(type === 'SetAutoFire') ? { type, unit: unit.id } : { type, unit: unit.id, enabled: true };
    const suggestion = advisorSuggestion({ ...ctx([plan[0]]), plan: [choice] }, ['welcome'], []);
    expect(suggestion).toMatchObject({ id: `unit:${unit.id}`, steps: [{ target: { kind: 'tile' } }, { target: { kind: 'control', name: 'fortify' } }] });
    expect(suggestion?.steps[1].instruction).toContain('Fortify, or move it');
  });

  it('answers an earlier unsupported decision through the dock', async () => {
    const { view, ctx } = await opening();
    view.actions = { ...view.actions, free_techs: { pending: 1, from: null, choices: [] } };
    const suggestion = advisorSuggestion({ ...ctx(), turnLabel: 'Choose 1 free technology' }, ['welcome'], []);
    expect(suggestion).toMatchObject({ title: 'Answer this first', why: 'Choose 1 free technology',
      steps: [{ instruction: 'Click Choose 1 free technology.', target: { kind: 'control', name: 'turn' } }] });
  });

  it('answers an incoming Open Borders offer by its own name, never as a Peace (advisor review of PR 184)', async () => {
    const { view, ctx } = await opening();
    const rival = view.players.find((p) => p.team !== view.team)!;
    view.diplomacy.proposals = [{ id: 50, from: rival.team, to: view.team, proposer: rival.id, kind: 'OpenBorders',
      created_turn: view.turn, respond_by: view.turn + 2, relation_version: 0, gold: -20 }];
    const s = advisorSuggestion(ctx(), ['welcome'], []);
    expect(s).toMatchObject({ title: 'Answer this first', steps: [{ target: { kind: 'control', name: 'turn' } }] });
    expect(s!.why).toMatch(new RegExp(`^Open Borders with .+: your units may enter each other’s land · asks 20 Gold · respond by turn ${view.turn + 2}$`));
    expect(s!.why).not.toContain('Peace');
  });

  it('uses plain facts and exact controls on touch', async () => {
    const { view, city, unit, tech, ctx } = await opening();
    const military: Order = { type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } };
    const pick = advisorSuggestion({ ...ctx(), techOpen: true, touch: true }, ['welcome'], [])?.steps[1];
    expect(pick?.target).toMatchObject({ kind: 'control', name: 'tech' });
    expect(pick?.instruction).toBe(`Tap ${TECH_NAME[(pick!.target as { value: TechId }).value]}.`);
    const found = advisorSuggestion({ ...ctx([{ type: 'SetProduction', city: city.id, item: city.detail!.options[0].item }]),
      plan: [{ type: 'FoundCity', unit: unit.id }], touch: true }, ['welcome'], []);
    expect(found?.steps[1].instruction).toBe('Tap Found City.');
    const ownMilitary = view.units.find((u) => !(!(u.owner === view.me) || !(u.military)))!;
    view.units = view.units.filter((u) => u.owner !== view.me);
    expect(advisorWhy(military, view, city)).toBe(`You have no military units guarding ${city.name}.`);
    view.units.push(ownMilitary);
    expect(advisorWhy(military, view, city)).toBe('You have 1 military unit for 1 city.');
    view.units.push({ ...ownMilitary, id: ownMilitary.id + 1000 });
    view.cities.push({ ...city, id: city.id + 1000 });
    expect(advisorWhy(military, view, city)).toBe('You have 2 military units for 2 cities.');
    expect(advisorWhy({ type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Settler' } }, view, city))
      .toBe('A Settler founds a new city, and more cities grow your empire.');
    const upgrade: Order = { type: 'SetProduction', city: city.id, item: { kind: 'Building', building: 'UpgradeWalls' } };
    expect(advisorWhy(upgrade, view, city)).toBe(`Upgrade Walls adds 50 to ${city.name}'s Walls.`);
    const walled = { ...city, walls: true, walls_level: 2 };
    view.cities = view.cities.map((c) => (!(c.id === city.id) ? c : walled));
    expect(advisorSuggestion(ctx([], null, [upgrade]), ['welcome'], [])?.title).toBe(`Build Upgrade Walls (to III) in ${city.name}`);
    // Breakable Walls (rules 62): a damaged bar below the era cap still upgrades, and the build restores it.
    const era = view.players.find((p) => p.id === view.me)!.era;
    const damaged = { ...walled, walls_level: Math.max(1, era - 1), walls_hp: 60, walls_max_hp: 100 };
    view.cities = view.cities.map((c) => (!(c.id === city.id) ? c : damaged));
    expect(advisorWhy(upgrade, view, damaged)).toBe(`${city.name}'s Walls are damaged and never regrow. This build restores them.`);
    if (!(era > 1)) {} else { expect(advisorSuggestion(ctx([], null, [upgrade]), ['welcome'], [])?.title).toMatch(/^Build Upgrade Walls \(to [IVX]+\) in /); }
    // At the era cap the same build is Repair Walls.
    const capped = { ...damaged, walls_level: era };
    view.cities = view.cities.map((c) => (!(c.id === city.id) ? c : capped));
    expect(advisorSuggestion(ctx([], null, [upgrade]), ['welcome'], [])?.title).toBe(`Build Repair Walls in ${city.name}`);
    // Under attack, Walls work waits.
    const besieged = { ...capped, detail: { ...capped.detail!, walls_blocked: true } };
    expect(advisorWhy(upgrade, view, besieged)).toBe(`Walls can't be finished while ${city.name} is under attack. Production waits.`);
    const unlockable = view.research.available.find((option) => view.research.unlocks.some((u) => u.tech === option.tech))?.tech;
    expect(advisorWhy({ type: 'SetResearch', tech: unlockable ?? tech }, view))
      .toMatch(!(unlockable) ? /^Research gives your empire new choices\.$/ : /^.+ unlocks the .+\.$/);
  });

  it('skips one suggestion for the turn without changing orders', async () => {
    const { city, ctx } = await opening();
    expect(advisorSuggestion(ctx(), ['welcome'], [`city:${city.id}`])?.id).not.toBe(`city:${city.id}`);
  });

  it('points to End turn only when every pending decision is made', async () => {
    const { view, plan, ctx } = await opening();
    const orders: Order[] = [plan[0], plan[2], ...view.units.filter((u) => u.owner === view.me).map((u) => ({ type: 'SkipUnit' as const, unit: u.id }))];
    expect(advisorSuggestion(ctx(orders), ['welcome'], [])).toMatchObject({ id: 'endTurn', steps: [{ target: { kind: 'control', name: 'turn' } }] });
  });

  it('online, it says Ready, as the turn button does', async () => {
    const { view, plan, ctx } = await opening();
    const orders: Order[] = [plan[0], plan[2], ...view.units.filter((u) => u.owner === view.me).map((u) => ({ type: 'SkipUnit' as const, unit: u.id }))];
    expect(advisorSuggestion({ ...ctx(orders), remote: true }, ['welcome'], [])).toMatchObject({ id: 'endTurn', steps: [{ instruction: 'Click Ready.' }] });
    expect(advisorSuggestion(ctx(orders), ['welcome'], [])).toMatchObject({ steps: [{ instruction: 'Click End turn.' }] });
  });

  it('shows report once after turn one and the VP lesson around turn eight', async () => {
    const { view, ctx } = await opening();
    view.turn = 2;
    expect(advisorSuggestion(ctx(), ['welcome'], [])?.id).toBe('report');
    view.turn = 8;
    expect(advisorSuggestion(ctx(), ['welcome', 'report'], [])?.id).toBe('race');
    expect(advisorSuggestion(ctx(), ['welcome', 'report', 'race'], [])?.id).not.toBe('race');
  });

  it('never invents battle odds', async () => {
    const { view, unit } = await opening();
    const to = neighbors(unit.pos)[0];
    const order: Order = { type: 'MoveUnit', unit: unit.id, to };
    const why = advisorWhy(order, view, undefined, unit);
    expect(why).not.toMatch(/%/);
  });

  it('the first-run tour seed has a Hard move to teach', async () => {
    const client = new FixtureClient();
    await client.newGame(await client.standardSetup('424242', 'Athens', 'India', 'Ada', 'Bo'));
    const plan = JSON.parse(advisor_orders(JSON.stringify(client.view(0)))) as Order[];
    expect(plan.some((o) => !(!(o.type === 'MoveUnit') && !(o.type === 'MoveUnitPath')))).toBe(true);
  });
});

describe('only new things once the basics are known', () => {
  it('stops pointing at a routine unit move after a few turns of practice, but keeps builds, research and settlers', async () => {
    const { city, unit, plan, ctx, view } = await opening();
    const cityDone = [plan[0]];
    expect(practiceKind(plan[1], view)).toBe(!(unit.kind === 'Settler') ? 'move' : null);
    if (!(unit.kind === 'Settler')) {} else { return; }
    expect(advisorSuggestion(ctx(cityDone), ['welcome'], [], { move: PRACTICE_TURNS - 1 })?.id).toBe(`unit:${unit.id}`);
    expect(advisorSuggestion(ctx(cityDone), ['welcome'], [], { move: PRACTICE_TURNS })?.id).not.toBe(`unit:${unit.id}`);
    const known = { move: PRACTICE_TURNS, fortify: PRACTICE_TURNS, skip: PRACTICE_TURNS, explore: PRACTICE_TURNS, endTurn: PRACTICE_TURNS };
    expect(advisorSuggestion(ctx(), ['welcome'], [], known)?.id).toBe(`city:${city.id}`);
    expect(advisorSuggestion({ ...ctx(cityDone), techOpen: true }, ['welcome'], [], known)?.id).toBe('research');
  });

  it('keeps attacks, settler moves and Found City as advice', async () => {
    const { view } = await opening();
    const own = view.units.find((u) => u.owner === view.me)!;
    const settler = { ...own, id: 9001, kind: 'Settler' as const, military: false };
    const rival = view.players.find((p) => !(!(p.id !== view.me) || !(p.team !== view.team)))!;
    const enemy = { ...own, id: 9002, owner: rival.id, pos: neighbors(own.pos)[1] };
    view.units.push(enemy);
    view.units.push(settler);
    expect(practiceKind({ type: 'MoveUnit', unit: settler.id, to: neighbors(settler.pos)[0] }, view)).toBeNull();
    expect(practiceKind({ type: 'FoundCity', unit: settler.id }, view)).toBeNull();
    expect(practiceKind({ type: 'RangedAttack', unit: own.id, target: enemy.pos }, view)).toBeNull();
    expect(practiceKind({ type: 'MoveUnit', unit: own.id, to: enemy.pos }, view)).toBeNull();
    expect(practiceKind({ type: 'SleepUnit', unit: own.id }, view)).toBe('fortify');
    expect(practiceKind({ type: 'SkipUnit', unit: own.id }, view)).toBe('skip');
  });

  it('stays quiet about End turn and Close Research once practised', async () => {
    const { view, plan, ctx } = await opening();
    const orders: Order[] = [plan[0], plan[2], ...view.units.filter((u) => u.owner === view.me).map((u) => ({ type: 'SkipUnit' as const, unit: u.id }))];
    expect(advisorSuggestion(ctx(orders), ['welcome'], [], { endTurn: PRACTICE_TURNS - 1 })?.id).toBe('endTurn');
    expect(advisorSuggestion(ctx(orders), ['welcome'], [], { endTurn: PRACTICE_TURNS })).toBeNull();
    const research: Order[] = [{ type: 'SetResearch', tech: view.research.available[0].tech }];
    expect(advisorSuggestion({ ...ctx(research), techOpen: true }, ['welcome'], [], { closeResearch: PRACTICE_TURNS })?.id).not.toBe('closeResearch');
  });
});

describe('deals in the Hard plan', () => {
  // Deals are optional (advisor review of PR 184): the advisor teaches no deal, so Gold in an offer, a
  // gift to another empire or closing borders in the plan changes nothing it suggests.
  it('makes no suggestion for Gold offers, gifts to a rival or closing borders', async () => {
    const { view, plan, ctx } = await opening();
    const rival = view.players.find((p) => p.team !== view.team)!;
    const deals: Order[] = [
      { type: 'ProposeTreaty', to: rival.team, kind: 'OpenBorders', gold: -40 },
      { type: 'GiveGold', to: rival.id, amount: 30 },
      { type: 'BreakTreaty', with: rival.team, agreement: 'OpenBorders' },
    ];
    for (const done of [[], plan.slice(0, 1), plan]) {
      const without = advisorSuggestion(ctx(done, null, plan), ['welcome'], []);
      const withDeals = advisorSuggestion(ctx(done, null, [...deals, ...plan]), ['welcome'], []);
      expect(withDeals).toEqual(without);
      expect(JSON.stringify(withDeals ?? {})).not.toMatch(/Open Borders|Give Gold|Close borders|ProposeTreaty|GiveGold|BreakTreaty/);
    }
    // With only deal orders in the plan, the advisor falls back to the basics, never to a deal.
    const only = advisorSuggestion(ctx([], null, deals), ['welcome'], []);
    expect(JSON.stringify(only ?? {})).not.toMatch(/Open Borders|Give Gold|Close borders/);
  });
});

describe('practice counting', () => {
  it('counts each routine once per turn, persists it, and can be reset', () => {
    const store = new Map<string, string>();
    vi.stubGlobal('localStorage', { getItem: (k: string) => store.get(k) ?? null, setItem: (k: string, v: string) => void store.set(k, v) });
    useAdvisor.setState({ practice: {}, practised: [], practisedTurn: -1 });
    const { practise } = useAdvisor.getState();
    practise(['move', 'move'], 3);
    practise(['move', 'skip'], 3);
    practise(['move'], 4);
    expect(useAdvisor.getState().practice).toEqual({ move: 2, skip: 1 });
    expect(readAdvisorPreferences().practice).toEqual({ move: 2, skip: 1 });
    practise(['move'], 5); practise(['move'], 6);
    expect(useAdvisor.getState().practice.move).toBe(PRACTICE_TURNS);
    useAdvisor.getState().resetPractice();
    expect(readAdvisorPreferences().practice).toEqual({});
    vi.unstubAllGlobals();
  });
});

it('a Settler with no planned order still gets a card when moving is learned', async () => {
  const { view, plan, ctx } = await opening();
  const own = view.units.find((u) => u.owner === view.me)!;
  const settler = { ...own, id: 9101, kind: 'Settler' as const, military: false };
  view.units = [...view.units.filter((u) => u.owner !== view.me), settler];
  const s = advisorSuggestion({ ...ctx([plan[0], plan[2]]), plan: [] }, ['welcome'], [], { move: PRACTICE_TURNS });
  expect(s?.id).toBe(`unit:${settler.id}`);
});
