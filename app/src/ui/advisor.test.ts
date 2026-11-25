import { describe, expect, it } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { advisor_orders } from '../wasm-pkg/legatum.js';
import { neighbors } from '../hex';
import type { Order } from '../types';
import { advisorSuggestion, advisorWhy, type AdvisorContext } from './advisorModel';

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
    const unitDone = [...cityDone, ...plan.filter((o) => o.type === 'MoveUnit' || o.type === 'MoveUnitPath').map((o) => ({ ...o, type: 'SkipUnit' as const }))];
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
    const choice: Order = type === 'SetAutoFire' ? { type, unit: unit.id, enabled: true } : { type, unit: unit.id };
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

  it('uses plain facts and exact controls on touch', async () => {
    const { view, city, unit, tech, ctx } = await opening();
    const military: Order = { type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } };
    expect(advisorSuggestion({ ...ctx(), techOpen: true, touch: true }, ['welcome'], [])?.steps[1].instruction).toMatch(/^Tap Research now on /);
    const found = advisorSuggestion({ ...ctx([{ type: 'SetProduction', city: city.id, item: city.detail!.options[0].item }]),
      plan: [{ type: 'FoundCity', unit: unit.id }], touch: true }, ['welcome'], []);
    expect(found?.steps[1].instruction).toBe('Tap Found City.');
    const ownMilitary = view.units.find((u) => u.owner === view.me && u.military)!;
    view.units = view.units.filter((u) => u.owner !== view.me);
    expect(advisorWhy(military, view, city)).toBe(`You have no military units guarding ${city.name}.`);
    view.units.push(ownMilitary);
    expect(advisorWhy(military, view, city)).toBe('You have 1 military unit for 1 city.');
    view.units.push({ ...ownMilitary, id: ownMilitary.id + 1000 });
    view.cities.push({ ...city, id: city.id + 1000 });
    expect(advisorWhy(military, view, city)).toBe('You have 2 military units for 2 cities.');
    expect(advisorWhy({ type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Settler' } }, view, city))
      .toBe('A Settler founds a new city, and more cities grow your empire.');
    const unlockable = view.research.available.find((option) => view.research.unlocks.some((u) => u.tech === option.tech))?.tech;
    expect(advisorWhy({ type: 'SetResearch', tech: unlockable ?? tech }, view))
      .toMatch(unlockable ? /^.+ unlocks the .+\.$/ : /^Research gives your empire new choices\.$/);
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
    expect(plan.some((o) => o.type === 'MoveUnit' || o.type === 'MoveUnitPath')).toBe(true);
  });
});
