// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { improvementReason, upgradeGain } from './UnitImprovements';
import { UnitPanel, upgradeStepBreakdown, upgradeCheck } from './UnitPanel';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { tileIndex } from '../hex';
import type { PlayerView, UnitView } from '../types';
import { unitsNeedingOrders } from '../orders';
import { foundCitySelected, healBlocker, healForecast } from '../commands';

let container: HTMLDivElement;
let root: Root;
beforeEach(() => {
  vi.useFakeTimers();
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div'));
  root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  act(() => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
  vi.clearAllTimers();
  vi.useRealTimers();
});

const NO_IRON = 'No Iron to spare: all 6 of your Iron are in use. Bring another Iron deposit inside your borders, or build a Mine on one for +1 Iron.';

/** A Spearman whose upgrade to Phalangite (Classical, Mathematics) costs 60 Gold and uses 1 Iron, with the requirements set as asked. */
async function spearman({ tech, owned, gold, spare = 3 }: { tech: boolean; owned: boolean; gold: number; spare?: number }) {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const view: PlayerView = structuredClone(client.view(0));
  const unit: UnitView = view.units.find((u) => u.owner === view.me && u.kind === 'Spearman')!;
  unit.kind = 'Spearman';
  unit.upgrade_need = 'Iron';
  view.research.known = tech ? [...view.research.known, 'Mathematics'] : view.research.known.filter((t) => t !== 'Mathematics');
  view.tiles[tileIndex(unit.pos, view.width, view.height)].owner = owned ? view.me : null;
  view.economy.gold = gold;
  // Phalangite also uses 1 Iron while it exists (the Spearman uses none), so it needs one to spare.
  view.supply = [{ resource: 'Iron', revealed: true, revealed_by: 'IronWorking', supplied: { total: 6, parts: [['Iron near Home', 6]] }, used: { total: 6 - spare, parts: [] }, idle: [], held: [], spare }, ...(view.supply ?? []).filter((x) => x.resource !== 'Iron')];
  unit.upgrade_breakdown = { total: 60, parts: [['Base upgrade Gold', 60]] };
  // The engine reports null for every failure (tech, territory and Gold alike).
  unit.upgrade_cost = tech && owned && gold >= 60 && spare >= 1 ? 60 : null;
  // The core words the supply refusal; the panel compacts only recognized supply messages.
  unit.upgrade_supply_blocker = spare < 1 ? NO_IRON : null;
  return { view, unit, client };
}

it.each([
  [{ tech: false, owned: true, gold: 500 }, 'Needs Mathematics', ['Mathematics ✗', 'in your territory ✓', '60 Gold ✓', 'Iron supply: needs 1 spare, you have 3 ✓']],
  [{ tech: true, owned: false, gold: 500 }, 'Move into your territory', ['Mathematics ✓', 'in your territory ✗', '60 Gold ✓', 'Iron supply: needs 1 spare, you have 3 ✓']],
  // The playtest bug: owned territory, the tech, 59 Gold used to say "Only inside your territory".
  [{ tech: true, owned: true, gold: 59 }, 'Needs 60 Gold (you have 59)', ['Mathematics ✓', 'in your territory ✓', '60 Gold ✗', 'Iron supply: needs 1 spare, you have 3 ✓']],
  [{ tech: true, owned: true, gold: 60 }, null, ['Mathematics ✓', 'in your territory ✓', '60 Gold ✓', 'Iron supply: needs 1 spare, you have 3 ✓']],
  // The supply comes after the Gold, as in the engine.
  [{ tech: true, owned: true, gold: 60, spare: 0 }, NO_IRON, ['Mathematics ✓', 'in your territory ✓', '60 Gold ✓', 'Iron supply: needs 1 spare, you have none ✗']],
  // Idle users make the spare negative: never "you have -2" (8.9.16 review).
  [{ tech: true, owned: true, gold: 60, spare: -2 }, NO_IRON, ['Mathematics ✓', 'in your territory ✓', '60 Gold ✓', 'Iron supply: needs 1 spare, you have none ✗']],
])('names the exact upgrade blocker and every requirement (%o)', async (setup, blocker, needs) => {
  const { view, unit } = await spearman(setup);
  const check = upgradeCheck(unit, view, view.economy.gold)!;
  expect(check.blocker).toBe(blocker);
  expect(check.needs.map((n) => `${n.label} ${n.met ? '✓' : '✗'}`)).toEqual(needs);
});

it('offers the newest researched upgrade for the summed step price', async () => {
  const { view, unit } = await spearman({ tech: true, owned: true, gold: 200 });
  view.research.known.push('Feudalism');
  unit.upgrade_to = 'Pikeman';
  unit.upgrade_steps = ['Phalangite', 'Pikeman'];
  unit.upgrade_breakdown = { total: 144, parts: [['Base upgrade Gold', 144]] };
  unit.upgrade_cost = 144;
  const check = upgradeCheck(unit, view, 200)!;
  expect(check.to).toBe('Pikeman');
  expect(check.cost).toBe(144);
});

const upgradeReason = (container: HTMLElement) => {
  const button = [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Upgrade'))!;
  expect(button.disabled).toBe(true);
  return document.getElementById(button.getAttribute('aria-describedby')!.split(' ').find((id) => document.getElementById(id)?.classList.contains('act-reason'))!)!;
};

it.each([
  ['India', 5, 'Veteran, level 5'],
  ['Sparta', 6, 'Spartan Veteran, level 6'],
] as const)('uses one %s max-rank emblem in the unit panel', async (faction, level, label) => {
  const { view, unit, client } = await spearman({ tech: true, owned: true, gold: 100 });
  view.players.find((p) => p.id === unit.owner)!.faction = faction;
  unit.level = level;
  unit.xp = level === 6 ? 318 : 226;
  unit.xp_next = null;
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  expect(container.querySelector('.up-rank .rank-insignia')?.getAttribute('aria-label')).toBe(label);
  expect(container.querySelectorAll('.up-rank .rank-insignia g')).toHaveLength(level);
});

it('uses the same three-chevron badge below max rank', async () => {
  const { view, unit, client } = await spearman({ tech: true, owned: true, gold: 100 });
  unit.level = 3;
  unit.xp = 90;
  unit.xp_next = 150;
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  expect(container.querySelectorAll('.up-rank .rank-insignia g')).toHaveLength(3);
});

it('shows one unmet requirement as a single sentence next to the button, not repeated as a checklist', async () => {
  const { view, unit, client } = await spearman({ tech: true, owned: true, gold: 59 });
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const reason = upgradeReason(container);
  expect(reason.textContent).toContain('Needs 60 Gold (you have 59)');
  expect(reason.querySelector('.act-needs')).toBeNull();
});

it('shows marked requirements beside the row when more than one is unmet', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 500, spare: 0 });
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const reason = upgradeReason(container);
  expect(reason.textContent).toContain('Mathematics');
  expect(reason.textContent).toContain('in your territory');
  const marks = [...reason.querySelectorAll('.act-needs .ui-mark[aria-label], .act-needs .not-met-mark')].map((m) => m.getAttribute('aria-label'));
  expect(marks).toEqual(['missing', 'done', 'done', 'missing']);
  expect(reason.querySelector('.resource-reason-link')?.textContent).toBe('Iron');
  const info = container.querySelector<HTMLButtonElement>('.improve-row .tip-info')!;
  act(() => info.click());
  // The Needs line under the row is the one place that lists them: the ⓘ card has the stats and the price.
  const card = document.getElementById(info.getAttribute('aria-controls')!)!.textContent;
  expect(card).not.toContain('Iron supply: needs 1 spare');
  expect(card).toContain('Upgrade cost');
});

it('puts Gold and supply costs beside the benefit, keeping the treasury forecast in details', async () => {
  const { view, unit, client } = await spearman({ tech: true, owned: true, gold: 100 });
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const row = container.querySelector('.improve-row')!;
  expect(row.querySelector('.improve-cost')!.getAttribute('aria-label')).toBe('Costs 60 Gold; uses 1 Iron');
  expect(row.querySelector('.improve-copy small')!.textContent).toMatch(/\+\d+ Strength/);
  expect(row.querySelector('.improve-supply [data-amount="Iron"]')).not.toBeNull();
  expect(row.querySelector('.improve-supply .amt-iron .amt-n')?.textContent).toBe('1');
  expect(row.querySelector('.improve-supply .amt-iron .amt-icon')).not.toBeNull();
});

it('shares one Iron reason between Upgrade and Siege and links both disabled controls to it', async () => {
  const { view, unit, client } = await spearman({ tech: true, owned: true, gold: 500, spare: 0 });
  unit.siege_offer = { price: { total: 160, parts: [] }, blocker: `Siege: ${NO_IRON[0].toLowerCase()}${NO_IRON.slice(1)}` };
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const reasons = container.querySelectorAll('.improve-reason');
  expect(reasons).toHaveLength(1);
  expect(reasons[0].textContent).toBe('Upgrade and Siege need 1 Iron. Your 6 Iron are all in use: claim another Iron deposit or build a Mine (+1).');
  expect(reasons[0].querySelector('.resource-reason-link')?.getAttribute('aria-label')).toBe('Open Iron supply');
  const buttons = [...container.querySelectorAll<HTMLButtonElement>('.improve-row button.act')];
  expect(buttons.length).toBeGreaterThan(0);
  buttons.forEach((button) => {
    expect(button.disabled).toBe(true);
    expect(button.getAttribute('aria-describedby')).toContain(reasons[0].id);
  });
});

it('does not compact unrelated engine refusals or call reserved/idle supply in use', async () => {
  const { view } = await spearman({ tech: true, owned: true, gold: 500, spare: 0 });
  expect(improvementReason('Move into your territory', 'Iron', view)).toBe('Move into your territory');
  view.supply![0].held = ['Forge in Kyoto'];
  expect(improvementReason(NO_IRON, 'Iron', view)).toContain('reserved for construction');
  view.supply![0].idle = ['Pikeman'];
  expect(improvementReason(NO_IRON, 'Iron', view)).toContain('You are already 1 Iron short: claim another Iron deposit or build a Mine (+1).');
  expect(improvementReason(`Siege: ${NO_IRON[0].toLowerCase()}${NO_IRON.slice(1)}`, 'Iron', view)).toMatch(/^Siege needs 1 Iron\. You are already 1 Iron short: /);
});

it('computes promotion-aware upgrade gains and preserves a lost bow in the visible effect', async () => {
  const { unit } = await spearman({ tech: true, owned: true, gold: 500 });
  unit.strength = 20; unit.level = 2; unit.ranged = 0;
  expect(upgradeGain(unit, 'Sparta', 'Phalangite')).toBe('+6 Strength');
  unit.kind = 'Cataphract'; unit.ranged = 20;
  expect(upgradeGain(unit, 'Mongols', 'Knight')).toContain('Loses its bow');
});

it('says a unit short of its supply is idle, fights weaker, and shows the strength it fights with', async () => {
  const { view, unit, client } = await spearman({ tech: true, owned: true, gold: 100 });
  unit.unsupplied = 'Iron';
  unit.strength = 36;
  unit.level = 0;
  unit.unsupplied_loss = 9;
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  expect(container.querySelector('[data-testid="unit-unsupplied"]')!.textContent!.trim()).toBe('Idle: you are short of Iron. It fights 25% weaker until supply returns.');
  expect(container.querySelector('[data-testid="unit-strength-idle"]')!.textContent).toBe('27');
  expect(container.querySelector('[data-testid="unit-strength-was"]')!.textContent).toBe('was 36: no Iron');
});

it('an unsupplied unit with no loss from the core shows its strength unchanged', async () => {
  const { view, unit, client } = await spearman({ tech: true, owned: true, gold: 100 });
  unit.strength = 36; unit.level = 0;
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  expect(container.querySelector('[data-testid="unit-strength-idle"]')).toBeNull();
});

it('prices an upgrade one line per step, each with its own faction part', () => {
  const b = upgradeStepBreakdown({ total: 164, parts: [['Base upgrade Gold', 60], ['Faction', 20], ['Base upgrade Gold', 84]] }, 'Spearman', ['Phalangite', 'Pikeman'], 'Byzantium');
  expect(b.parts.map(([, v]) => v)).toEqual([80, 84]);
  expect(b.parts[0][0]).toMatch(/^Spearman → /);
  expect(b.total).toBe(164);
});

it('prices the whole path as one line when a free step leaves no part to split on', () => {
  const b = upgradeStepBreakdown({ total: 84, parts: [['Base upgrade Gold', 84]] }, 'Spearman', ['Phalangite', 'Pikeman'], 'India');
  expect(b.parts).toHaveLength(1);
  expect(b.parts[0][1]).toBe(84);
});

it.each(['Spearman', 'Pikeman', 'Musketman', 'Archer'] as const)(
  'describes Sparta %s health from the core wound-immunity data', async (kind) => {
    const { view, unit, client } = await spearman({ tech: true, owned: true, gold: 100 });
    view.players.find((p) => p.id === unit.owner)!.faction = 'Sparta';
    unit.kind = kind;
    unit.hp = 50;
    useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
    act(() => root.render(<UnitPanel unit={unit} view={view} />));
    const health = container.querySelector<HTMLElement>('.up-stat')!;
    act(() => health.click());
    const tip = document.getElementById(health.getAttribute('aria-describedby')!)!;
    expect(tip.textContent).toContain(kind === 'Archer'
      ? 'Strength drops by 1 per 10 HP lost'
      : 'Sparta infantry ignore wound strength penalties');
  },
);

it('Fortify persists visibly with its keyboard hint, and Skip is the only other idle order', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 100 });
  unit.hp = 70; unit.fortify = 6; unit.standing_order = 'Sleep';
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, selection: { kind: 'unit', id: unit.id } });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const button = (label: string) => [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith(label));
  expect(button('Fortify')!.getAttribute('aria-pressed')).toBe('true');
  expect(button('Fortify')!.textContent).toContain('F');
  expect(button('Fortify')!.closest('.act-wrap')?.classList.contains('primary')).toBe(false);
  expect(button('Skip this turn')!.closest('.act-wrap')?.classList.contains('primary')).toBe(false);
  expect(container.textContent).toContain('Resting defence +6');
  expect(container.textContent).toContain('Fortified until woken');
  expect(button('Wake')).toBeUndefined();
  act(() => button('Skip this turn')!.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SkipUnit', unit: unit.id }]);
});

it('a unit locked in a captured city says it is the garrison once, and offers no Move, Disband, Fortify or Skip', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 100 });
  unit.occupation_lock = (view.me + 1) as typeof unit.occupation_lock;
  unit.standing_order = 'Sleep';
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, selection: { kind: 'unit', id: unit.id } });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const labels = [...container.querySelectorAll<HTMLButtonElement>('button.act')].map((b) => b.textContent ?? '');
  expect(container.querySelectorAll('[data-testid="occupation-lock"]')).toHaveLength(1);
  expect(container.querySelector('[data-testid="occupation-lock"]')!.textContent).toMatch(/^\s*Garrisoning .+: holding the city down until Peace with /);
  expect(container.querySelector('[data-testid="occupation-lock-note"]')!.textContent).toMatch(/people will not accept your rule while .+ still fights\. This unit cannot leave or disband\.$/);
  expect(container.textContent).not.toContain('Needs orders');
  for (const label of ['Move', 'Disband', 'Fortify', 'Skip', 'Auto-explore']) expect(labels.some((l) => l.startsWith(label)), label).toBe(false);
  expect(container.textContent!.match(/holding/g)).toHaveLength(1);
});

it('Heal rests a damaged unit like Fortify, shows its rate and turns to full, and returns it to the queue by itself', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 100 });
  unit.hp = 55;
  // The core validates the order against its own copy of the unit, so wound that one too.
  client.debugState().units.find((u) => u.id === unit.id)!.hp = 55;
  unit.resting_heal = { total: 20, parts: [['Home territory', 20]] };
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, selection: { kind: 'unit', id: unit.id } });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const button = (label: string) => [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith(label))!;
  const heal = button('Heal');
  expect(heal.disabled).toBe(false);
  expect(heal.textContent).toContain('H');
  expect(heal.getAttribute('aria-pressed')).toBe('false');
  expect(heal.querySelector('.action-icon')).not.toBeNull();
  // The tooltip gives the rate where it stands and the turns to full (45 HP at 20 a turn: 3).
  act(() => heal.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SleepUnit', unit: unit.id, until_healed: true }]);
  expect(button('Heal').getAttribute('aria-pressed')).toBe('true');
  expect(button('Heal').classList.contains('active')).toBe(true);
  expect(button('Heal').querySelector('.action-check')).not.toBeNull();
  expect(button('Fortify').getAttribute('aria-pressed')).toBe('false');
  expect(button('Fortify').querySelector('.action-check')).toBeNull();
  expect(container.querySelector('.up-status')?.textContent).toContain('Healing until full');
  expect(unitsNeedingOrders(view, useGame.getState().orders[0]).some((u) => u.id === unit.id)).toBe(false);
  // The order is on the unit once resolved: still pressed, still out of the queue.
  unit.standing_order = 'Heal';
  useGame.setState({ orders: [[]] });
  act(() => root.render(<UnitPanel unit={{ ...unit }} view={view} />));
  expect(button('Heal').getAttribute('aria-pressed')).toBe('true');
  expect(container.querySelector('.up-status')?.textContent).toContain('Healing');
  expect(unitsNeedingOrders(view, []).some((u) => u.id === unit.id)).toBe(false);
  // Fortify replaces it, and Skip does too.
  act(() => button('Fortify').click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SleepUnit', unit: unit.id }]);
});

it('the Heal tooltip names the rate and the turns to full', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 100 });
  unit.hp = 55;
  unit.resting_heal = { total: 20, parts: [['Home territory', 20]] };
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, selection: { kind: 'unit', id: unit.id } });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const heal = [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Heal'))!;
  expect(healForecast(unit)).toBe('Heals 20 HP a turn in your territory: full in 3 turns.');
  act(() => heal.focus());
  const tip = document.getElementById(heal.getAttribute('aria-describedby')!.split(' ')[0])!;
  expect(tip.textContent).toContain('Heals 20 HP a turn in your territory: full in 3 turns.');
  unit.resting_heal = { total: 0, parts: [['Closed foreign territory', 0]] };
  expect(healForecast(unit)).toBeNull();
  expect(healBlocker(unit)).toContain('nothing heals here');
  unit.hp = 100;
  expect(healBlocker(unit)).toContain('full health');
});

it('Heal is offered only to a damaged unit, and where nothing heals it is off with the reason under the row', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 100 });
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, selection: { kind: 'unit', id: unit.id } });
  const heal = () => [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Heal'));
  unit.hp = 100;
  act(() => root.render(<UnitPanel unit={{ ...unit }} view={view} />));
  expect(heal()).toBeUndefined();
  // Closed foreign land heals nothing: the button is off and the reason sits right under the row.
  unit.hp = 60;
  unit.resting_heal = { total: 0, parts: [['Closed foreign territory', 0]] };
  act(() => root.render(<UnitPanel unit={{ ...unit }} view={view} />));
  expect(heal()!.disabled).toBe(true);
  const reason = container.querySelector('[data-testid="heal-blocked"]')!;
  expect(reason.textContent).toContain('nothing heals here');
  expect(heal()!.getAttribute('aria-describedby')).toBe(reason.id);
  act(() => heal()!.click());
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('offers the auto-fire toggle to a ranged city garrison and queues its setting', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 100 });
  unit.kind = 'Archer'; unit.ranged = 12; unit.range = 2;
  view.cities.push({ id: 99, owner: view.me, pos: unit.pos, name: 'Home', detail: null } as PlayerView['cities'][number]);
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, selection: { kind: 'unit', id: unit.id } });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const button = [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Auto-fire from city'))!;
  expect(button).toBeDefined();
  act(() => button.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SetAutoFire', unit: unit.id, enabled: true }]);
});

it('civilians say Wait while a Scout leads with Auto-explore and a Settler with Found City', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 100 });
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0 });
  for (const kind of ['Scout', 'Settler'] as const) {
    const u = { ...unit, kind, military: kind === 'Scout' };
    act(() => root.render(<UnitPanel unit={u} view={view} />));
    const primaries = [...container.querySelectorAll('.actions .act-wrap.primary')].map((el) => el.textContent);
    expect(primaries).toHaveLength(1);
    expect(primaries[0]).toMatch(kind === 'Scout' ? /^Auto-explore/ : /^Found City/);
    expect([...container.querySelectorAll<HTMLButtonElement>('button.act')].some((b) => b.textContent?.startsWith('Wait'))).toBe(true);
  }
});

it('a military unit has Auto-explore (E) as a secondary action, a Settler has none', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 100 });
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, selection: { kind: 'unit', id: unit.id } });
  const explore = () => [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Auto-explore'));
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  expect(unit.military).toBe(true);
  expect(explore()).toBeDefined();
  expect(explore()!.closest('.act-wrap')?.classList.contains('primary')).toBe(false);
  expect(container.querySelector('.actions .act-wrap.primary')?.textContent).not.toMatch(/^Auto-explore/);
  expect([...container.querySelectorAll<HTMLButtonElement>('button.act')].some((b) => b.textContent?.startsWith('Fortify'))).toBe(true);
  act(() => explore()!.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SetAutomation', unit: unit.id, automation: 'Explore' }]);
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  expect([...container.querySelectorAll<HTMLButtonElement>('button.act')].some((b) => b.textContent?.startsWith('Stop auto-explore'))).toBe(true);
  const settler = { ...unit, kind: 'Settler' as const, military: false };
  act(() => root.render(<UnitPanel unit={settler} view={view} />));
  expect(explore()).toBeUndefined();
});

it('shows Sparta Fortify enabled at the ordinary rest-heal rate', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 60 });
  view.players.find((p) => p.id === unit.owner)!.faction = 'Sparta';
  unit.hp = 40;
  unit.resting_heal = { total: 20, parts: [['Home territory', 20]] };
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0, selection: { kind: 'unit', id: unit.id } });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const fortify = [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Fortify'))!;
  expect(fortify.disabled).toBe(false);
  expect(fortify.closest('.act-wrap')?.classList.contains('blocked') ?? false).toBe(false);
  expect(container.querySelector('.up-hints')?.textContent ?? '').toContain('heal up to 20 HP');
});

it('shows resting defence only with a current bonus, inside the panel instead of floating notes', async () => {
  const { view, unit } = await spearman({ tech: false, owned: true, gold: 60 });
  useGame.setState({ orders: [[]], errors: [[]], seat: 0 });
  unit.fortify = 0;
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  expect(container.querySelector('.up-resting')).toBeNull();
  unit.fortify = 3;
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  expect(container.querySelector('.up-head .up-resting')?.textContent).toBe('Resting defence +3');
  expect(container.querySelector('.up-hints')?.textContent).not.toContain('Resting defence');
});

it('Fortify takes a unit out of the order cycle and Skip, an order of its own, wakes it before resolving', async () => {
  const { view, unit, client } = await spearman({ tech: false, owned: true, gold: 60 });
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0, selection: { kind: 'unit', id: unit.id } });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const button = (label: string) => [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith(label))!;
  act(() => button('Fortify').click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SleepUnit', unit: unit.id }]);
  expect(button('Fortify').getAttribute('aria-pressed')).toBe('true');
  expect(unitsNeedingOrders(view, useGame.getState().orders[0]).some((u) => u.id === unit.id)).toBe(false);
  act(() => button('Skip this turn').click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SkipUnit', unit: unit.id }]);
  expect(button('Fortify').getAttribute('aria-pressed')).toBe('false');
});

/** After founding the capital, stand a Settler (the Spearman, relabelled in the view) at `offset` from it. */
async function settlerNearCapital(offset: { q: number; r: number }) {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  await client.endTurn({});
  const view: PlayerView = structuredClone(client.view(0));
  const capital = view.cities.find((c) => c.owner === view.me)!;
  const unit: UnitView = { ...view.units.find((u) => u.owner === view.me && u.kind === 'Spearman')!, kind: 'Settler', military: false,
    pos: { q: capital.pos.q + offset.q, r: capital.pos.r + offset.r } };
  view.units = view.units.map((u) => (u.id === unit.id ? unit : u));
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0, selection: { kind: 'unit', id: unit.id } });
  return { view, unit, capital };
}

it('disables Found City where a known city is too close, says why beside the button, and issues no order', async () => {
  const { view, unit, capital } = await settlerNearCapital({ q: 2, r: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const found = [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Found City'))!;
  expect(found.disabled).toBe(true);
  const reason = document.getElementById(found.getAttribute('aria-describedby')!)!;
  expect(reason.textContent).toBe(`Too close to ${capital.name}: cities must be at least 3 tiles apart.`);
  // The B key goes through the same command: still no order.
  foundCitySelected();
  expect(useGame.getState().orders[0]).toEqual([]);
  // The phone's pinned action row carries the same reason.
  act(() => root.render(<UnitPanel unit={unit} view={view} peek />));
  const phone = [...container.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.textContent?.startsWith('Found City'))!;
  expect(phone.disabled).toBe(true);
  expect(document.getElementById(phone.getAttribute('aria-describedby')!)!.textContent).toBe(`Too close to ${capital.name}`);
});

it('disables Found City at an explored site blocked only by an unseen city, without naming it', async () => {
  const { view, unit } = await settlerNearCapital({ q: 4, r: 0 });
  const tile = view.tiles[tileIndex(unit.pos, view.width, view.height)];
  Object.assign(tile, { explored: true, terrain: 'Plains', owner: null });
  view.city_gap_blocked = [unit.pos];
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const found = [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Found City'))!;
  expect(found.disabled).toBe(true);
  expect(document.getElementById(found.getAttribute('aria-describedby')!)?.textContent)
    .toBe('Too close to another city to found one here');
  foundCitySelected();
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('keeps Found City available where the site looks legal from what the player knows', async () => {
  const { view, unit } = await settlerNearCapital({ q: 4, r: 0 });
  const i = tileIndex(unit.pos, view.width, view.height);
  view.tiles[i] = { ...view.tiles[i], owner: null, terrain: 'Plains', explored: true };
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const found = [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Found City'))!;
  expect(found.disabled).toBe(false);
  expect(container.querySelector('.act-reason')).toBeNull();
  act(() => found.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'FoundCity', unit: unit.id }]);
});

it('lets a full-health Roman veteran found with no Gold and explains the unit cost', async () => {
  const { view, unit } = await settlerNearCapital({ q: 4, r: 0 });
  view.players.find((p) => p.id === view.me)!.faction = 'Rome';
  view.economy.gold = 0;
  unit.kind = 'Spearman'; unit.military = true; unit.level = 1; unit.hp = 100;
  const i = tileIndex(unit.pos, view.width, view.height);
  view.tiles[i] = { ...view.tiles[i], owner: null, terrain: 'Plains', explored: true };
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const found = [...container.querySelectorAll<HTMLButtonElement>('button.act')].find((b) => b.textContent?.startsWith('Found City'))!;
  expect(found.disabled).toBe(false);
  expect(found.getAttribute('title') ?? '').not.toMatch(/Gold|price/);
  act(() => found.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'FoundCity', unit: unit.id }]);
});

it('after the game ends the panel shows the unit but awaits no orders and offers no actions (8.9.4 review)', async () => {
  const { view, unit, client } = await spearman({ tech: true, owned: true, gold: 500 });
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  expect(container.textContent).toContain('Needs orders');
  expect(container.querySelector('.actions')).not.toBeNull();
  const done: PlayerView = { ...view, finished: true, winner: view.team };
  act(() => root.render(<UnitPanel unit={unit} view={done} />));
  expect(container.textContent).not.toContain('Needs orders');
  expect(container.querySelector('.actions')).toBeNull();
  expect(container.textContent).toContain('Strength');
});


it.each(['upgrade', 'siege', 'move'] as const)('emphasizes exactly one useful action: %s', async (choice) => {
  const { view, unit, client } = await spearman({ tech: choice === 'upgrade', owned: true, gold: 500 });
  if (choice === 'siege') unit.siege_offer = { price: { total: 160, parts: [] }, blocker: null };
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0, selection: { kind: 'unit', id: unit.id } });
  act(() => root.render(<UnitPanel unit={unit} view={view} />));
  const primary = container.querySelectorAll<HTMLButtonElement>('button.primary');
  expect(primary).toHaveLength(1);
  expect(primary[0].textContent).toContain(choice === 'upgrade' ? 'Upgrade to' : choice === 'siege' ? 'Siege' : 'Move or attack');
  act(() => primary[0].click());
  if (choice === 'move') expect(useGame.getState().mode).toBe('move');
  else expect(useGame.getState().orders[0]).toContainEqual({ type: choice === 'upgrade' ? 'UpgradeUnit' : 'UpgradeSiege', unit: unit.id });
});
