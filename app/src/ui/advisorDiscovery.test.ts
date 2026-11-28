import { describe, expect, it } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { advisorSuggestion, useAdvisor, type AdvisorContext } from './advisorModel';
import { DISCOVERY_IDS, discoveriesUsed, discoverySuggestion } from './advisorDiscovery';
import type { Order } from '../types';

async function fixture() {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('4545', 'Athens', 'India', 'Ada', 'Bo'));
  const view = client.view(0);
  const city = view.cities.find((c) => c.owner === view.me)!;
  const scout = view.units.find((u) => u.owner === view.me)!;
  scout.kind = 'Scout'; scout.hp = 100; scout.goto = null; scout.standing_order = null; scout.automation = null;
  city.population = 6; city.detail!.resistance = 0; city.detail!.build_automation = null;
  view.turn = 100; view.research.era = 4;
  const ctx: AdvisorContext = { view, orders: [], selection: null, humans: 1, touch: false, plan: [] };
  return { view, city, scout, ctx };
}
const seen = ['welcome', 'report', 'howToWin'];

describe('feature discovery', () => {
  it('offers mature city automation, follows the city, and respects completion and dismissal', async () => {
    const { ctx, city } = await fixture();
    const advice = discoverySuggestion(ctx, seen, [], {});
    expect(advice).toMatchObject({ id: 'cityAutomation', current: 0, acknowledge: true, steps: [
      { target: { kind: 'tile', hex: city.pos } }, { target: { kind: 'control', name: 'automate' }, key: 'I' },
    ] });
    expect(advice!.why).toContain('tile improvements');
    expect(discoverySuggestion({ ...ctx, selection: { kind: 'city', id: city.id } }, seen, [], {})?.current).toBe(1);
    expect(discoverySuggestion(ctx, [...seen, 'cityAutomation'], [], {})?.id).not.toBe('cityAutomation');
    expect(discoverySuggestion(ctx, seen, ['cityAutomation'], {})?.id).not.toBe('cityAutomation');
    ctx.orders = [{ type: 'SetCityAutomation', city: city.id, focus: 'Economy' }];
    expect(discoverySuggestion(ctx, seen, [], {})?.id).not.toBe('cityAutomation');
  });

  it('waits until management is repetitive and leaves threats and active choices alone', async () => {
    const { ctx, view, city } = await fixture();
    view.research.era = 1; view.turn = 20;
    expect(discoverySuggestion(ctx, seen, [], {})?.id).not.toBe('cityAutomation');
    view.turn = 80;
    view.cities.push({ ...city, id: 200 }, { ...city, id: 201 });
    expect(discoverySuggestion(ctx, seen, [], {})?.id).toBe('cityAutomation');
    expect(discoverySuggestion({ ...ctx, techOpen: true }, seen, [], {})).toBeNull();
    expect(discoverySuggestion({ ...ctx, mode: 'move' }, seen, [], {})).toBeNull();
    expect(discoverySuggestion({ ...ctx, selection: { kind: 'unit', id: 999 } }, seen, [], {})).toBeNull();
    city.occupation_lock = 1;
    ctx.selection = { kind: 'city', id: city.id };
    expect(discoverySuggestion(ctx, seen, [], {})).toBeNull();
  });

  it('teaches Auto-explore without relying on a Hard AI order, but never to an injured or busy Scout', async () => {
    const { ctx, view, scout } = await fixture();
    view.turn = 5; view.research.era = 0;
    ctx.selection = { kind: 'unit', id: scout.id };
    view.tiles[0].explored = false;
    expect(discoverySuggestion(ctx, seen, [], {})).toMatchObject({ id: 'autoExplore', current: 1, steps: [
      {}, { target: { kind: 'control', name: 'explore' }, key: 'E' },
    ] });
    expect(discoverySuggestion(ctx, seen, [], { explore: 1 })?.id).not.toBe('autoExplore');
    expect(discoverySuggestion(ctx, seen, ['autoExplore'], {})?.id).not.toBe('autoExplore');
    scout.hp = 59;
    expect(discoverySuggestion(ctx, seen, [], {})?.id).not.toBe('autoExplore');
    scout.hp = 100; scout.occupation_lock = 1;
    expect(discoverySuggestion(ctx, seen, [], {})?.id).not.toBe('autoExplore');
    scout.occupation_lock = null; ctx.orders = [{ type: 'SkipUnit', unit: scout.id }];
    expect(discoverySuggestion(ctx, seen, [], {})).toBeNull();
  });

  it('recognizes using optional features and never records stopping one as learning it', () => {
    const orders: Order[] = [
      { type: 'SetCityAutomation', city: 1, focus: 'Economy' },
      { type: 'SetAutomation', unit: 2, automation: 'Explore' },
      { type: 'SleepUnit', unit: 2, until_healed: true },
      { type: 'SetAutoFire', unit: 3, enabled: true },
      { type: 'SetProductionQueue', city: 1, queue: [{ kind: 'Building', building: 'Granary' }, { kind: 'Building', building: 'Workshop' }] },
    ];
    expect(discoveriesUsed(orders)).toEqual(['cityAutomation', 'autoExplore', 'heal', 'autoFire', 'productionQueue']);
    expect(discoveriesUsed([{ type: 'SetCityAutomation', city: 1, focus: null }, { type: 'SetAutomation', unit: 2, automation: null }])).toEqual([]);
  });

  it('keeps mandatory decisions ahead of discovery', async () => {
    const { ctx, view } = await fixture();
    view.actions = { ...view.actions, free_techs: { pending: 1, from: null, choices: [] } };
    expect(advisorSuggestion(ctx, seen, [])?.id).toBe('answer:freeTechs:');
  });
  it('teaches improvement and finite build queues, but not a perpetual project queue', async () => {
    const { ctx, city, view } = await fixture();
    view.turn = 10; view.research.era = 0;
    ctx.selection = { kind: 'city', id: city.id };
    const d = city.detail!;
    const item = { kind: 'Improvement' as const, tile: city.pos, improvement: 'Farm' as const };
    d.options.push({ ...d.options[0], item, turns: 3, resource_blocker: null });
    d.producing = null;
    expect(discoverySuggestion(ctx, seen, [], {})).toMatchObject({ id: 'tileImprovements', steps: [{ target: { name: 'build' } }] });
    d.producing = { kind: 'Building', building: 'Granary' }; d.production_queue = [];
    expect(discoverySuggestion(ctx, seen, [], {})).toMatchObject({ id: 'productionQueue', steps: [{ target: { name: 'queueBuild' } }] });
    d.producing = { kind: 'Project', project: 'Gold' };
    expect(discoverySuggestion(ctx, seen, [], {})?.id).not.toBe('productionQueue');
  });

  it('teaches healing and city auto-fire only when those controls apply', async () => {
    const { ctx, city, scout } = await fixture();
    ctx.selection = { kind: 'unit', id: scout.id };
    scout.hp = 60; scout.resting_heal = { total: 10, parts: [['Rest', 10]] };
    expect(discoverySuggestion(ctx, seen, [], {})?.id).toBe('heal');
    scout.resting_heal.total = 0;
    expect(discoverySuggestion(ctx, seen, [], {})?.id).not.toBe('heal');
    scout.hp = 100; scout.kind = 'Archer'; scout.ranged = 5; scout.pos = city.pos;
    expect(discoverySuggestion(ctx, seen, [], {})?.id).toBe('autoFire');
    scout.standing_order = 'AutoFire';
    expect(discoverySuggestion(ctx, seen, [], {})?.id).not.toBe('autoFire');
  });

  it('teaches upgrades with the actual cost and honors queued spending', async () => {
    const { ctx, view, city, scout } = await fixture();
    ctx.selection = { kind: 'unit', id: scout.id };
    scout.kind = 'Spearman'; scout.pos = city.pos; scout.upgrade_to = 'Pikeman'; scout.upgrade_cost = 80; scout.upgrade_breakdown = null;
    view.research.known.push('Feudalism'); view.economy.gold = 1000;
    const tile = view.tiles.find((t) => t.owner === view.me);
    expect(tile).toBeTruthy();
    expect(discoverySuggestion(ctx, seen, [], {})).toMatchObject({ id: 'modernize', why: expect.stringContaining('80 Gold'), steps: [{ target: { name: 'upgrade' } }] });
    view.economy.gold = 0;
    expect(discoverySuggestion(ctx, seen, [], {})?.id).not.toBe('modernize');
  });

  it('teaches air controls and pings only when available', async () => {
    const { ctx, view, city } = await fixture();
    view.aircraft = [{ id: 500, owner: view.me, kind: 'Biplane', city: city.id, hp: 100 }];
    ctx.selection = { kind: 'air', id: 500 };
    expect(advisorSuggestion(ctx, seen, [])).toMatchObject({ id: 'airOperations', steps: [{ target: { name: 'airRecon' } }] });
    ctx.orders = [{ type: 'AirRecon', plane: 500, target: city.pos }];
    expect(discoverySuggestion(ctx, seen, [], {})).toBeNull();
    ctx.selection = null; ctx.orders = []; ctx.canPing = false;
    expect(discoverySuggestion(ctx, [...seen, 'cityAutomation'], [], {})?.id).not.toBe('teamPing');
    ctx.canPing = true;
    expect(discoverySuggestion(ctx, [...seen, 'cityAutomation'], [], {})?.id).toBe('teamPing');
  });

  it('discovers projects, map yields, batch cities and available siege equipment', async () => {
    const { ctx, view, city, scout } = await fixture();
    const learned = [...seen, 'cityAutomation'];
    ctx.selection = { kind: 'city', id: city.id };
    const d = city.detail!;
    d.producing = null; d.options = [{ ...d.options[0], item: { kind: 'Project', project: 'Gold' }, turns: 0 }];
    expect(discoverySuggestion(ctx, learned, [], {})?.id).toBe('projects');
    expect(discoverySuggestion(ctx, [...learned, 'projects'], [], {})?.id).toBe('mapYields');
    ctx.selection = null; view.cities.push({ ...city, id: 200 }, { ...city, id: 201 });
    expect(discoverySuggestion(ctx, learned, [], {})?.id).toBe('cities');
    ctx.selection = { kind: 'unit', id: scout.id }; scout.kind = 'Spearman'; scout.siege_offer = { price: { total: 80, parts: [['Siege', 80]] }, blocker: null };
    view.economy.gold = 1000; view.supply = [{ resource: 'Iron', spare: 2 } as NonNullable<typeof view.supply>[number]];
    expect(discoverySuggestion(ctx, [...learned, 'modernize'], [], {})?.id).toBe('siege');
    scout.siege_offer.blocker = 'Unavailable';
    expect(discoverySuggestion(ctx, [...learned, 'modernize'], [], {})?.id).not.toBe('siege');
  });

  it('does not teach manual city controls while the city is automated', async () => {
    const { ctx, city } = await fixture();
    ctx.selection = { kind: 'city', id: city.id };
    city.detail!.producing = { kind: 'Building', building: 'Granary' };
    city.detail!.production_queue = [];
    const learned = [...seen, 'cityAutomation'];
    expect(discoverySuggestion(ctx, learned, [], {})?.id).toBe('productionQueue');
    ctx.orders = [{ type: 'SetCityAutomation', city: city.id, focus: 'Economy' }];
    expect(discoverySuggestion(ctx, learned, [], {})).toBeNull();
    ctx.orders = []; city.detail!.build_automation = 'Economy';
    expect(discoverySuggestion(ctx, learned, [], {})).toBeNull();
  });

  it('preserves core unit and urgent teaching after learning Automate', async () => {
    const { ctx, view, city, scout } = await fixture();
    ctx.selection = { kind: 'city', id: city.id };
    city.detail!.producing = { kind: 'Building', building: 'Granary' };
    // Remove other routine decisions so the idle Scout is the fallback lesson.
    view.cities = [city]; view.units = [scout];
    const learned = [...seen, ...DISCOVERY_IDS];
    expect(advisorSuggestion(ctx, learned, [])?.id).toBe(`unit:${scout.id}`);
    ctx.orders = [{ type: 'SetCityAutomation', city: city.id, focus: 'Economy' }];
    // The first render precedes the effect that remembers learning the control.
    expect(advisorSuggestion(ctx, learned.filter((id) => id !== 'cityAutomation'), [])?.id).toBe(`unit:${scout.id}`);
    ctx.discoveryTurn = view.turn;
    expect(advisorSuggestion(ctx, learned, [])?.id).toBe(`unit:${scout.id}`);
    // Toggling it off still leaves the basic unit guidance available.
    ctx.orders = [{ type: 'SetCityAutomation', city: city.id, focus: null }];
    expect(advisorSuggestion(ctx, learned, [])?.id).toBe(`unit:${scout.id}`);
    const previousActions = view.actions;
    view.actions = { ...view.actions, free_techs: { pending: 1, from: null, choices: [] } };
    expect(advisorSuggestion(ctx, learned, [])?.id).toBe('answer:freeTechs:');
    view.actions = previousActions;
    view.turn += 1;
    expect(advisorSuggestion(ctx, learned, [])?.id).toBe(`unit:${scout.id}`);
  });

  it('paces optional lessons after learning one and recognizes an enabled Yields overlay', async () => {
    const { ctx, view, city } = await fixture();
    const old = useAdvisor.getState();
    try {
      useAdvisor.setState({ seen: [...seen], discoveryTurn: -1 });
      useAdvisor.getState().markSeen('cityAutomation', view.turn);
      expect(useAdvisor.getState().discoveryTurn).toBe(view.turn);
      expect(discoverySuggestion({ ...ctx, discoveryTurn: view.turn }, useAdvisor.getState().seen, [], {})).toBeNull();
      view.turn += 1;
      ctx.selection = { kind: 'city', id: city.id };
      city.detail!.producing = null; city.detail!.options = [];
      expect(discoverySuggestion({ ...ctx, discoveryTurn: view.turn - 1 }, useAdvisor.getState().seen, [], {})?.id).toBe('mapYields');
      expect(discoverySuggestion({ ...ctx, yieldsVisible: true }, useAdvisor.getState().seen, [], {})).toBeNull();
    } finally { useAdvisor.setState(old, true); }
  });

  it('teaches opening Diplomacy from the right sidebar after meeting a rival, and suppresses before contact or once used', async () => {
    const { ctx, view } = await fixture();
    view.turn = 8; view.research.era = 0;
    const rival = view.players.find((p) => p.team !== view.team)!;
    view.diplomacy.met = [view.team];
    // Before meeting a rival, no diplomacy lesson fires.
    expect(discoverySuggestion(ctx, seen, [], {})?.id).not.toBe('diplomacy');

    // Once a living rival is met with nothing selected, teach how to open Diplomacy.
    view.diplomacy.met = [view.team, rival.team];
    const advice = discoverySuggestion(ctx, seen, [], {});
    expect(advice).toMatchObject({ id: 'diplomacy', current: 0, acknowledge: true, steps: [
      { target: { kind: 'control', name: 'rivalRow', value: String(rival.team) } },
    ] });
    expect(advice!.steps[0].instruction).toContain('right sidebar');
    expect(advice!.why).toContain('offer Peace or an Alliance');
    expect(advice!.why).toContain('declare War');

    // Deals toggle controls whether Gold / Open Borders trading is promised.
    view.diplomacy.deals = true;
    expect(discoverySuggestion(ctx, seen, [], {})!.why).toContain('trade Gold or Open Borders');
    view.diplomacy.deals = false;
    expect(discoverySuggestion(ctx, seen, [], {})!.why).not.toContain('Open Borders');

    // Touch phrasing avoids "Click ... right sidebar".
    expect(discoverySuggestion({ ...ctx, touch: true }, seen, [], {})!.steps[0].instruction).toContain('Tap');

    // Suppressed when a unit/city is selected, when seen/skipped, or once a diplomatic order is queued.
    expect(discoverySuggestion({ ...ctx, selection: { kind: 'unit', id: view.units[0].id } }, seen, [], {})?.id).not.toBe('diplomacy');
    expect(discoverySuggestion(ctx, [...seen, 'diplomacy'], [], {})?.id).not.toBe('diplomacy');
    expect(discoverySuggestion(ctx, seen, ['diplomacy'], {})?.id).not.toBe('diplomacy');
    expect(discoveriesUsed([{ type: 'DeclareWar', against: rival.team }])).toContain('diplomacy');
    expect(discoverySuggestion({ ...ctx, orders: [{ type: 'DeclareWar', against: rival.team }] }, seen, [], {})?.id).not.toBe('diplomacy');
  });

});
