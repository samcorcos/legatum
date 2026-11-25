import { beforeEach, describe, expect, it } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import type { CityView, Order, PlayerView } from '../types';
import { cityOccupiedLine, occupationLead, occupationLockWarning, occupationNote, occupationRefusal, occupationTileLine, assaultSummary, civilianCaptureRace, reinforcementNote, attackerRisk, bankruptcyWarning, cityFireOutcome, cityShotTarget, cityStrikeWarning, cityProductionWarning, disbandUnitLabel, forecastWinLabel, garrisonHeadline, idleCitiesNeedingProduction, idleProductionWarning, productionStallText, wallsLine, woundedNearEnemies } from './forecastWarnings';
import type { BattleForecast, UnitView } from '../types';
import { neighbors } from '../hex';
import { ownerName } from '../barbarians';
import { damage, exactOdds, siegeCover, wallDamagePercent } from '../combat';
import { BUILT_IN_SIEGE_NOTE, builtInSiege, canHaveSiege, itemName } from '../data';
import { forecastModel } from './ForecastCard';
import { Game } from '../wasm-pkg/legatum.js';
import { loadWasmModule } from '../wasmClient';

let view: PlayerView;
let city: CityView;
it('warns when a visible barbarian can reach a civilian before a distant capture order', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'Switzerland', 'India', 'Ada', 'Bo'));
  const v = client.view(0);
  const mover = v.units.find((u) => u.owner === v.me && u.military)!;
  const target = { q: 5, r: 5 };
  const enemy = v.players.find((p) => p.team !== v.team)!;
  v.diplomacy.relations.find((r) => [r.a, r.b].includes(enemy.team) && [r.a, r.b].includes(v.team))!.state = 'War';
  v.units.push({ ...mover, id: 900, owner: enemy.id, kind: 'Settler', military: false, pos: target });
  v.units.push({ ...mover, id: 901, owner: 255, kind: 'Spearman', military: true, pos: { q: 4, r: 5 } });
  expect(civilianCaptureRace(v, mover, target, 2)).toBe('Reaches the Settler in 2 steps: barbarian Spearman is already next to it and may capture it first.');
  expect(civilianCaptureRace(v, mover, target, 1)).toBeNull();
  v.units[v.units.length - 1].owner = enemy.id;
  expect(civilianCaptureRace(v, mover, target, 2)).toBeNull();
});
beforeEach(async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'Switzerland', 'India', 'Ada', 'Bo'));
  client.endTurn({});
  view = client.view(0);
  city = view.cities.find((c) => c.owner === view.me)!;
  city.detail!.idle_production_cap = 60;
  city.detail!.production_stored = 58;
  city.detail!.idle_production_waste = 3;
});

describe('bankruptcy warnings', () => {
  it('names the exact unit and the negative treasury, not the per-turn income', () => {
    view.economy.gold_per_turn.total = -7;
    view.economy.bankruptcy = { gold: { total: -3, parts: [['Treasury', 4], ['Unit upkeep', -7]] },
      disband: { unit: 12, kind: 'Pikeman', pos: { q: 1, r: 2 }, city: null } };
    expect(bankruptcyWarning(view)).toBe('Next turn −3 Gold: Reisläufer #12 will be disbanded');
    view.players.find((p) => p.id === view.me)!.faction = 'Byzantium';
    expect(disbandUnitLabel(view)).toBe('Tagmatic Soldier #12');
  });

  it('identifies a completing unit by its city without inventing a unit id', () => {
    view.economy.bankruptcy = { gold: { total: -2, parts: [] },
      disband: { unit: null, kind: 'Archer', pos: city.pos, city: city.id } };
    expect(bankruptcyWarning(view)).toBe(`Next turn −2 Gold: Archer (completing in ${city.name}) will be disbanded`);
  });

  it('does not warn for an affordable deficit or an exact-zero forecast', () => {
    view.economy.gold_per_turn.total = -5;
    view.economy.bankruptcy = null;
    expect(bankruptcyWarning(view)).toBeNull();
    view.economy.bankruptcy = { gold: { total: 0, parts: [] }, disband: null };
    expect(bankruptcyWarning(view)).toBeNull();
  });

  it('handles bankruptcy without any military unit to disband', () => {
    view.economy.bankruptcy = { gold: { total: -1, parts: [] }, disband: null };
    expect(bankruptcyWarning(view)).toBe('Next turn −1 Gold: treasury will reset to 0 (no military unit to disband)');
  });
});

describe('city production warnings', () => {
  it('previews the exact no-choice warning for an empty or finishing delegated city', async () => {
    await loadWasmModule();
    city.detail!.build_automation = 'Science';
    city.detail!.production_queue = [];
    city.detail!.options = city.detail!.options.filter((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman');
    expect(city.detail!.options).toHaveLength(1);
    expect(cityProductionWarning(city, [], undefined, { ...view })).toBe("Nothing here fits its automation focus. Choose this turn's build.");
    expect(idleCitiesNeedingProduction({ ...view }, [])).toContain(city);
    city.detail!.producing = { kind: 'Unit', unit: 'Scout' };
    city.detail!.turns_to_complete = 1;
    expect(cityProductionWarning(city, [], undefined, { ...view })).toContain('Nothing here fits its automation focus');
    expect(idleCitiesNeedingProduction({ ...view }, [])).toContain(city);
    city.detail!.turns_to_complete = 2;
    expect(cityProductionWarning(city, [], undefined, { ...view })).toBeNull();
    expect(idleCitiesNeedingProduction({ ...view }, [])).not.toContain(city);
  });

  it('never asks for production in a city ordered razed (8.9.29 turn-102 loop)', async () => {
    await loadWasmModule();
    city.detail!.build_automation = null;
    city.detail!.producing = null;
    city.detail!.production_queue = [];
    expect(idleCitiesNeedingProduction({ ...view }, [])).toContain(city);
    expect(idleCitiesNeedingProduction({ ...view }, [{ type: 'Raze', city: city.id }])).not.toContain(city);
  });

  it('shows the exact next-turn waste and storage limit', () => {
    expect(idleProductionWarning(city.detail)).toBe('3 Production will be wasted next turn (58/60 stored).');
    city.detail!.production_stored = 80;
    city.detail!.idle_production_waste = 5;
    expect(cityProductionWarning(city)).toBe('5 Production will be wasted next turn (80/60 stored).');
  });

  it('does not call stored or suspended production waste', () => {
    city.detail!.idle_production_waste = 0;
    expect(cityProductionWarning(city)).toBeNull();
    city.detail!.idle_production_waste = 5;
    city.detail!.resistance = 1;
    expect(cityProductionWarning(city)).toBeNull();
    city.detail!.resistance = 0;
    city.detail!.producing = { kind: 'Unit', unit: 'Spearman' };
    expect(idleProductionWarning(city.detail)).toBeNull();
    expect(idleProductionWarning(null)).toBeNull();
  });

  it('explains population and blocked spawn stalls', () => {
    expect(productionStallText({ kind: 'Population', required: 2, available: 1 })).toBe(
      'Build stalled: Settler needs population 2 (1 after growth next turn). Its progress stays banked; grow the city or choose another build.');
    expect(productionStallText({ kind: 'SpawnBlocked' })).toBe(
      'Build stalled: no free spawn tile in or beside the city. Move a blocking unit to make room.');
    expect(productionStallText(null)).toBeNull();
  });

  it('clears an idle warning when production is queued and restores it on cancellation', () => {
    const orders = [{ type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } } as const];
    expect(cityProductionWarning(city, orders)).toBeNull();
    expect(idleCitiesNeedingProduction(view, orders)).not.toContain(city);
    expect(cityProductionWarning(city, [])).toContain('3 Production');
  });

  it('does not apply the current build stall to a queued replacement', () => {
    city.detail!.producing = { kind: 'Unit', unit: 'Settler' };
    city.detail!.production_stall = { kind: 'Population', required: 2, available: 1 };
    expect(cityProductionWarning(city)).toContain('Settler needs population 2');
    expect(cityProductionWarning(city, [{ type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }])).toBeNull();
    expect(cityProductionWarning({ ...city, detail: null })).toBeNull();
  });

  it('treats an explicitly cleared queue as idle and drops the old stall', () => {
    city.detail!.producing = { kind: 'Unit', unit: 'Settler' };
    city.detail!.production_stall = { kind: 'Population', required: 2, available: 1 };
    const orders: Order[] = [{ type: 'SetProductionQueue', city: city.id, queue: [] }];
    expect(cityProductionWarning(city, orders)).toBeNull();
    expect(idleCitiesNeedingProduction(view, orders)).toContain(city);
  });

  it('prioritizes the most wasteful owned idle city without mutating the view', () => {
    const second = { ...city, id: 20, detail: { ...city.detail!, idle_production_waste: 8 } };
    const ally = { ...second, id: 21, owner: 1 };
    view.cities = [city, second, ally];
    expect(idleCitiesNeedingProduction(view, []).map((c) => c.id)).toEqual([20, city.id]);
    expect(view.cities.map((c) => c.id)).toEqual([city.id, 20, 21]);
  });
});

it('explains city entry for ranged units even when using a move attack', async () => {
  const { cityEntryWarning } = await import('./forecastWarnings');
  expect(cityEntryWarning({ kind: 'Archer' })).toBe("Ranged units can't capture cities; move a melee unit in to take it.");
  expect(cityEntryWarning({ kind: 'Spearman' })).toContain('most health');
  // A Keshig charges and captures; only its shot cannot.
  expect(cityEntryWarning({ kind: 'Cataphract' })).toContain('most health');
  expect(cityEntryWarning({ kind: 'Cataphract' }, true)).toBe("A shot can't take a city. Charge it to capture it.");
});

describe('G18 combat warnings', () => {
  const forecast = (over: Partial<BattleForecast>): BattleForecast => ({
    attacker_strength: { total: 20, parts: [] }, defender_strength: { total: 20, parts: [] }, win_chance: 0, loss_chance: 0,
    expected_damage_to_attacker: 30, expected_damage_to_defender: 30, attacker_hp: 100, defender_hp: 100, ranged: false, ...over,
  });

  it('warns on the forecast card when the attacker could be destroyed, including chances that round to 0%', () => {
    expect(attackerRisk(forecast({}), 'Spearman')).toBeNull();
    expect(attackerRisk(forecast({ loss_chance: 35, attacker_hp: 36 }), 'Spearman')).toBe('Your Spearman could be destroyed (35% chance).');
    // Worst roll (120%) of 20 vs 20 deals 36: a 36 HP attacker can die even when the chance rounds down to 0%.
    expect(attackerRisk(forecast({ attacker_hp: 36 }), 'Spearman')).toBe('Your Spearman could be destroyed (under 1% chance).');
    expect(attackerRisk(forecast({ ranged: true, loss_chance: 0, attacker_hp: 1 }), 'Archer')).toBeNull();
  });

  it('flags own units under half HP standing next to enemy military, weakest first', () => {
    const own = view.units.find((u) => u.owner === view.me && u.kind === 'Spearman')!;
    const unit = (id: number, owner: number, pos: UnitView['pos'], hp: number): UnitView => ({ ...own, id, owner, pos, hp });
    const near = neighbors(own.pos)[0];
    view.units = [unit(1, view.me, own.pos, 42), unit(2, view.me, neighbors(near)[0], 30), unit(3, 255, near, 100)];
    expect(woundedNearEnemies(view).map((w) => w.text)).toEqual(['Spearman is wounded next to enemies (30 HP)', 'Spearman is wounded next to enemies (42 HP)']);
    view.units = [unit(1, view.me, own.pos, 50), unit(3, 255, near, 100)];
    expect(woundedNearEnemies(view)).toEqual([]);
    view.units = [unit(1, view.me, own.pos, 20)];
    expect(woundedNearEnemies(view)).toEqual([]);
  });
});

describe('city strike forecast', () => {
  const f = { ranged: false, attacker_hp: 60, city_strike: { city: 1, damage: 21,
    city_strength: { total: 16, parts: [['City', 10], ['Population', 6]] },
    attacker_defense: { total: 16, parts: [['Spearman', 20], ['Wounded', -4]] } } } as unknown as BattleForecast;
  it('says the city fires back, how hard, and at whom, with both breakdowns', () => {
    const w = cityStrikeWarning(f, 'Redoubt', 'Spearman')!;
    expect(w.text).toBe('Redoubt will fire back: about −21 health to your Spearman. It fires at the weakest unit in range.');
    expect(w.details).toContain('City 16 (City 10, Population +6) vs your defence 16 (Spearman 20, Wounded −4)');
    expect(w.deadly).toBe(false);
  });
  it('warns when the shot could destroy the attacker, and is silent without a shot', () => {
    expect(cityStrikeWarning({ ...f, attacker_hp: 20 }, 'Redoubt', 'Spearman')).toMatchObject({ deadly: true, text: expect.stringContaining('about −20 health to your Spearman, and it could destroy it') });
    expect(cityStrikeWarning({ ...f, city_strike: undefined }, 'Redoubt', 'Spearman')).toBeNull();
  });
});

describe('walled city fire folded into the forecast (playtest-6 H1)', () => {
  // The walls fixture's numbers: a Spearman (20) attacks a population-3 walled city (26, 200 HP).
  const strike = { city: 1, damage: damage(26, 20, 100), city_strength: { total: 26, parts: [] }, attacker_defense: { total: 20, parts: [] } };
  const odds = exactOdds(20, 26, 100, 200, false);
  const f = {
    ranged: false, attacker_strength: { total: 20, parts: [['Spearman', 20]] }, defender_strength: { total: 26, parts: [] },
    win_chance: odds.win, loss_chance: odds.loss, expected_damage_to_attacker: odds.expToAtk, expected_damage_to_defender: odds.expToDef,
    attacker_hp: 100, defender_hp: 200, city_strike: strike,
  } as unknown as BattleForecast;

  it('takes the shot first, then fights at the wounded strength, over every roll', () => {
    const out = cityFireOutcome(f, 'India', 'Spearman')!;
    expect(out).toEqual(JSON.parse(Game.combat_projection(JSON.stringify({ op: 'cityFireOutcome', forecast: f, faction: 'India', kind: 'Spearman' }))));
    // Mean shot over the city's 41 rolls.
    let shots = 0;
    for (let r = 80; r <= 120; r++) shots += damage(26, 20, r);
    expect(out.fire).toBe(Math.trunc(shots / 41));
    // Wounded by the shot, the Spearman attacks weaker: it takes more and deals less than without it.
    expect(out.attack).toBeGreaterThan(f.expected_damage_to_attacker);
    expect(out.dealt).toBeLessThan(f.expected_damage_to_defender);
    expect(out.hpAfter).toBe(100 - out.fire - out.attack);
    expect(out.hpAfter).toBeLessThan(100 - f.expected_damage_to_attacker - 30);
    expect(out).toMatchObject({ win: 0, loss: 0, canDie: false });
  });

  it('counts a shot that destroys the attacker before it attacks', () => {
    const out = cityFireOutcome({ ...f, attacker_hp: 30 }, 'India', 'Spearman')!;
    expect(out.canDie).toBe(true);
    expect(out.loss).toBeGreaterThan(0);
    expect(out.hpAfter).toBeGreaterThanOrEqual(0);
  });

  it('leaves a ranged strike’s damage alone: it fires from the same snapshot as the city', () => {
    const ranged = { ...f, ranged: true, expected_damage_to_attacker: 0, expected_damage_to_defender: exactOdds(20, 26, 100, 200, true).expToDef };
    const out = cityFireOutcome(ranged, 'India', 'Archer')!;
    expect(out.dealt).toBe(ranged.expected_damage_to_defender);
    expect(out.attack).toBe(0);
    expect(out.hpAfter).toBe(100 - out.fire);
  });

  it('shows Wall-reduced fire damage in the card and derives the Siege hint from it', () => {
    const walled = { ...f, ranged: true, walls: { tier: 1, covered: 0, percent: 50, built_in: false } } as BattleForecast;
    const out = cityFireOutcome(walled, 'India', 'Archer')!;
    expect(out.dealt).toBe(exactOdds(20, 26, 100, 200, false, 50).expToDef);
    expect(wallsLine(walled, 'Athens', 'Archer', 1, out.dealt)?.text)
      .toBe(`Walls I halve this attack on Athens: ${out.dealt} health, or ${out.dealt * 2} with Siege.`);
    const attacker = view.units.find((u) => u.owner === view.me && u.military)!;
    const target = neighbors(attacker.pos)[0];
    view.units = [attacker];
    view.cities.push({ ...city, id: city.id + 100, owner: 1, name: 'Athens', pos: target, walls: true });
    expect(forecastModel(walled, true, view, attacker, target, []).dealt).toBe(out.dealt);
  });

  it('ignores wounds for Sparta line infantry, and is null without a shot', () => {
    const wounded = cityFireOutcome(f, 'India', 'Spearman')!, oath = cityFireOutcome(f, 'Sparta', 'Spearman')!;
    expect(oath.dealt).toBeGreaterThan(wounded.dealt);
    expect(cityFireOutcome({ ...f, city_strike: undefined }, 'India', 'Spearman')).toBeNull();
  });

  it('names the city’s target: the weakest of your units in range, by HP then id', () => {
    const rival = { ...city, id: 77, owner: view.players.find((p) => p.team !== view.team)!.id };
    view.diplomacy = { ...view.diplomacy, legacy: true } as PlayerView['diplomacy'];
    const near = neighbors(city.pos);
    const u = (id: number, hp: number, pos = near[0]) => ({ id, owner: view.me, kind: 'Spearman', pos, hp, military: true }) as unknown as UnitView;
    view.units = [u(5, 80), u(3, 80, near[1]), u(9, 100, near[2])];
    expect(cityShotTarget(view, rival)?.id).toBe(3);
    view.units = [u(5, 40), u(3, 80, near[1])];
    expect(cityShotTarget(view, rival)?.id).toBe(5);
    view.units = [u(5, 40, city.pos)];
    expect(cityShotTarget(view, rival)).toBeNull();
  });
});

describe('garrisoned city forecasts (core combat::city_tile_defender)', () => {
  const base: BattleForecast = { attacker_strength: { total: 20, parts: [] }, defender_strength: { total: 16, parts: [] },
    win_chance: 0, loss_chance: 0, expected_damage_to_attacker: 25, expected_damage_to_defender: 34,
    attacker_hp: 100, defender_hp: 200, ranged: false };

  it('says city damage can overflow into the garrison', () => {
    const f = { ...base, garrison: { unit: 9, owner: 1, kind: 'Archer' as const, hp: 100, shielded: true } };
    expect(garrisonHeadline(f, view, 'Epoch', false)).toBe(
      'Epoch absorbs damage first. The hit that empties its health carries on to the Archer garrison with the part of its damage the city did not absorb; later hits strike the garrison in full. A melee attacker captures if the garrison falls.');
    expect(garrisonHeadline({ ...f, ranged: true }, view, 'Epoch', true)).toBe('Epoch absorbs damage first. The hit that empties its health carries on to the Archer garrison with the part of its damage the city did not absorb; later hits strike the garrison in full. Ranged fire cannot capture.');
    expect(forecastWinLabel(f, true, false, false)).toBe('drop it to 0 health');
  });

  it('names the exposed garrison of a fallen city and what destroying it does', () => {
    const f = { ...base, defender_hp: 60, garrison: { unit: 9, owner: 1, kind: 'Spearman' as const, hp: 60, shielded: false } };
    expect(garrisonHeadline(f, view, 'Epoch', false)).toBe('Epoch has fallen: you fight its Spearman garrison (60 health), and destroying it captures the city.');
    expect(garrisonHeadline(f, view, 'Epoch', true)).toBe('Epoch has fallen: this shot hits its Spearman garrison (60 health). Only a melee attack can capture the city.');
    expect(forecastWinLabel(f, true, false, false)).toBe('kill the garrison, capture');
    expect(forecastWinLabel(f, true, true, true)).toBe('destroy the garrison');
  });

  it('keeps ungarrisoned and field targets unchanged', () => {
    expect(garrisonHeadline(base, view, 'Epoch', false)).toBeNull();
    expect(forecastWinLabel(base, true, false, false)).toBe('capture it');
    expect(forecastWinLabel(base, true, true, true)).toBe('drop it to 0 health');
    expect(forecastWinLabel(base, false, false, false)).toBe('destroy it');
  });
});

it('shows a queued three-unit counterattack split and full cost when alone', () => {
  const attacker = view.units.find((u) => u.owner === view.me && u.military)!;
  const target = neighbors(attacker.pos)[0];
  const second = { ...attacker, id: attacker.id + 100 };
  const third = { ...attacker, id: attacker.id + 101 };
  view.units.push(second, third, { ...attacker, id: 999, owner: 1, pos: target });
  const f: BattleForecast = { attacker_strength: { total: 20, parts: [] }, defender_strength: { total: 20, parts: [] },
    win_chance: 0, loss_chance: 0, expected_damage_to_attacker: 21, expected_damage_to_defender: 30,
    attacker_hp: 100, defender_hp: 100, ranged: false };
  const orders: Order[] = [{ type: 'MoveUnit', unit: second.id, to: target }, { type: 'MoveUnit', unit: third.id, to: target }];
  const shared = forecastModel(f, false, view, attacker, target, orders);
  expect(shared.counterText).toBe('2 others also attacking: you take about −7 instead of −21.');
  expect(shared.taken).toBe(7);
  expect(shared.hpAfter).toBe(93);
  const alone = forecastModel(f, false, view, attacker, target, []);
  expect(alone.counterText).toBeNull();
  expect(alone.taken).toBe(21);
});

it('the headline follows the whole fight at a city, never the walls alone (T187 Lugano)', () => {
  const attacker = view.units.find((u) => u.owner === view.me && u.military)!;
  const target = neighbors(attacker.pos)[0];
  view.units.push({ ...attacker, id: 999, owner: 1, pos: target });
  view.cities.push({ ...city, id: city.id + 100, owner: 1, name: 'Tessitura', pos: target, hp: 5 });
  const f: BattleForecast = { attacker_strength: { total: 20, parts: [] }, defender_strength: { total: 12, parts: [] },
    win_chance: 100, loss_chance: 0, expected_damage_to_attacker: 21, expected_damage_to_defender: 5,
    attacker_hp: 100, defender_hp: 5, ranged: false,
    garrison: { unit: 999, owner: 1, kind: 'Spearman', hp: 100, shielded: true } };
  const held = { attacks: 1, city_hp: 5, city_hp_after: 0, capture_chance: 0, can_capture: true,
    garrison: { unit: 999, owner: 1, kind: 'Spearman' as const, hp: 100, hp_after: 61, destroyed_chance: 0 } };
  // The city drops (the old headline said Decisive) but the garrison under it holds: nothing is captured.
  const m = forecastModel(f, false, view, attacker, target, [], held);
  expect(m.win).toBe(0);
  expect(m.verdict).not.toBe('Decisive');
  expect(m.winLabel).toBe('capture the city');
  const taken = forecastModel(f, false, view, attacker, target, [], { ...held, capture_chance: 100, garrison: { ...held.garrison, hp_after: 0, destroyed_chance: 100 } });
  expect(taken.win).toBe(100);
  expect(taken.verdict).toBe('Decisive');
  // A shot alone cannot capture; with a garrison under the city the label is about the garrison.
  const shot = forecastModel({ ...f, ranged: true }, true, view, attacker, target, [], { ...held, can_capture: false });
  expect(shot.winLabel).toBe('destroy the garrison');
  // No pooled assault (the mock): the model still works and says nothing about it.
  expect(forecastModel(f, false, view, attacker, target, []).assault).toBeNull();
});

it('words the whole fight at a city from core numbers: city, garrison, capture, you', () => {
  const a = { attacks: 2, city_hp: 40, city_hp_after: 0, capture_chance: 0, can_capture: true,
    garrison: { unit: 7, owner: 1, kind: 'Spearman' as const, hp: 100, hp_after: 61, destroyed_chance: 0 } };
  const s = assaultSummary(a, 'Lugano', 'Spearman');
  expect(s.title).toBe('The whole fight at Lugano: 2 attacks this turn');
  expect(s.rows.map((r) => [r.key, r.hp, r.after])).toEqual([['city', 40, 0], ['garrison', 100, 61]]);
  expect(s.capture).toBe('Lugano is not captured this turn: its Spearman survives with about 61 health.');
  expect(s.captureTone).toBe('bad');
  const falls = assaultSummary({ ...a, capture_chance: 100, garrison: { ...a.garrison, hp_after: 0, destroyed_chance: 100 } }, 'Lugano', 'Spearman');
  expect(falls.capture).toBe('Lugano is captured this turn.');
  expect(falls.rows[1].note).toBe('destroyed');
  const maybe = assaultSummary({ ...a, capture_chance: 55 }, 'Lugano', 'Spearman');
  expect(maybe.capture).toBe('Lugano is captured this turn in 55% of outcomes; otherwise its Spearman survives with about 61 health.');
  const shot = assaultSummary({ ...a, can_capture: false, attacks: 1, garrison: null, city_hp_after: 12 }, 'Lugano', null);
  expect(shot.capture).toBe('A shot cannot capture a city. After this turn Lugano keeps about 12 health.');
  expect(shot.rows.map((r) => r.key)).toEqual(['city']);
});

it('ranged attacks never show a counterattack split', () => {
  const attacker = view.units.find((u) => u.owner === view.me && u.military)!;
  const target = neighbors(attacker.pos)[0];
  const ally = { ...attacker, id: attacker.id + 100 };
  view.units.push(ally, { ...attacker, id: 999, owner: 1, pos: target });
  const f: BattleForecast = { attacker_strength: { total: 20, parts: [] }, defender_strength: { total: 20, parts: [] },
    win_chance: 0, loss_chance: 0, expected_damage_to_attacker: 0, expected_damage_to_defender: 30,
    attacker_hp: 100, defender_hp: 100, ranged: true };
  expect(forecastModel(f, true, view, attacker, target, [{ type: 'MoveUnit', unit: ally.id, to: target }]).counterText).toBeNull();
});

describe('reinforcement note', () => {
  const own = (): UnitView => ({ ...view.units.find((u) => u.owner === view.me && u.military)!, kind: 'Spearman', ranged: 0, military: true });
  const enemyNear = (pos: UnitView['pos']): UnitView => ({ ...own(), id: 900, owner: 255, pos });

  it('says your unit gets there first when an enemy could attack your ungarrisoned city', () => {
    const unit = { ...own(), pos: neighbors(city.pos)[3] };
    view.units = [unit, enemyNear(neighbors(city.pos)[0])];
    const note = reinforcementNote(view, { ...unit, pos: neighbors(city.pos)[3] }, city.pos);
    expect(note).toContain(`If an enemy attacks ${city.name} this turn, your Spearman gets there first and becomes its garrison`);
    expect(note).toContain("HP absorbs the first hits");
    const far = neighbors(neighbors(city.pos)[3])[3];
    expect(reinforcementNote(view, { ...unit, pos: far }, city.pos)).toContain('more than one step');
  });

  it('is silent for ranged units, garrisoned cities, other tiles and a quiet front', () => {
    const from = { ...own(), pos: neighbors(city.pos)[3] };
    const unit = from;
    view.units = [unit, enemyNear(neighbors(city.pos)[0])];
    expect(reinforcementNote(view, { ...from, kind: 'Archer', ranged: 18 }, city.pos)).toBeNull();
    expect(reinforcementNote(view, from, neighbors(city.pos)[1])).toBeNull();
    view.units = [unit, { ...own(), id: 901, pos: city.pos }, enemyNear(neighbors(city.pos)[0])];
    expect(reinforcementNote(view, from, city.pos)).toBeNull();
    view.units = [unit];
    expect(reinforcementNote(view, from, city.pos)).toBeNull();
  });
});

describe('wallsLine', () => {
  const f = (walls: { tier: number; covered: number; percent: number }) => ({ walls }) as unknown as BattleForecast;
  it('shows both numbers when Siege would help, and names the missing tech when it only half helps', () => {
    expect(wallsLine(f({ tier: 1, covered: 0, percent: 50 }), 'Athens', 'Spearman', 1, 18)).toEqual({ text: 'Walls I halve this attack on Athens: 18 health, or 36 with Siege.', tone: 'plain' });
    expect(wallsLine(f({ tier: 1, covered: 0, percent: 50 }), 'Athens', 'Spearman', 0, 18)!.text).toBe('Walls I halve this attack on Athens.');
    expect(wallsLine(f({ tier: 2, covered: 1, percent: 75 }), 'Athens', 'Spearman', 1, 27)).toEqual({ text: 'Walls II: Siege covers up to level I, so 75% damage to Athens. Machinery covers level II.', tone: 'plain' });
    expect(wallsLine(f({ tier: 1, covered: 1, percent: 100 }), 'Athens', 'Spearman', 1, 36)!.tone).toBe('good');
    // Covered, the line says what is ignored; the red City fire row says the city still fires. Never "no Walls penalty" beside a red "Walls: ... fires first".
    expect(wallsLine(f({ tier: 1, covered: 1, percent: 100 }), 'Athens', 'Spearman', 1, 36)!.text).toBe('Walls I: Siege covers up to level I, so full damage to Athens.');
    expect(wallsLine(f({ tier: 3, covered: 3, percent: 100 }), 'Athens', 'Spearman', 3, 36)!.text).toBe('Walls III: Siege covers up to level III, so full damage to Athens.');
  });
  it('an Industrial unit says its Siege is built in, in core\'s words (designer, 2026-10-28)', () => {
    const builtIn = { walls: { tier: 1, covered: 4, percent: 100, built_in: true } } as unknown as BattleForecast;
    expect(wallsLine(builtIn, 'Athens', 'Rifleman', 0, 36)).toEqual({ text: 'Walls I: built-in Siege covers up to level IV, so full damage to Athens.', tone: 'good' });
    expect(BUILT_IN_SIEGE_NOTE).toBe('covers Walls through level IV in the Industrial era, one more level in each later era');
  });
});

describe('built-in Siege mirrors core data::siege_cover (designer, 2026-10-28)', () => {
  it('Industrial and later land units cover their own era tier; earlier ones need to buy it', () => {
    for (const kind of ['Rifleman', 'Cavalry', 'FieldCannon', 'Infantry', 'Tank', 'Artillery'] as const) {
      expect(builtInSiege(kind), kind).toBe(true);
      expect(canHaveSiege(kind), kind).toBe(false);
      expect(siegeCover(kind, false, []), kind).toBe(kind === 'Infantry' || kind === 'Tank' || kind === 'Artillery' ? 6 : 4);
      expect(wallDamagePercent(3, siegeCover(kind, false, [])), kind).toBe(100);
    }
    for (const kind of ['Spearman', 'Musketman', 'Arquebusier', 'Cuirassier'] as const) {
      expect(builtInSiege(kind), kind).toBe(false);
      expect(canHaveSiege(kind), kind).toBe(true);
      expect(siegeCover(kind, false, ['Mathematics', 'Machinery', 'Metallurgy']), kind).toBe(0);
      expect(siegeCover(kind, true, ['Mathematics', 'Machinery']), kind).toBe(2);
    }
    expect(builtInSiege('Bomber')).toBe(false);
    expect(siegeCover('Bomber', true, ['Metallurgy'])).toBe(0);
    expect(builtInSiege('Scout')).toBe(false);
  });
});

describe('occupation lock experiment', () => {
  it('warns before a unit enters a captured city and says what holds a locked unit', () => {
    const rival = view.players.find((p) => p.team !== view.team)!;
    const mover = view.units.find((u) => u.owner === view.me && u.military)!;
    const away = { ...mover, pos: { q: city.pos.q + 3, r: city.pos.r } };
    expect(occupationLockWarning(view, away, city)).toBeNull();
    city.occupation_lock = rival.id;
    expect(occupationLockWarning(view, away, city)).toBe(`Occupation: a unit that ends its move in ${city.name} stays as its garrison, holding the city down until Peace with ${ownerName(view, rival.id)}, and the city builds and buys nothing until then.`);
    expect(occupationLockWarning(view, { ...away, military: false }, city)).toBeNull();
    expect(occupationLockWarning(view, { ...away, pos: city.pos }, city)).toBeNull();
    const held = { ...mover, pos: city.pos, occupation_lock: rival.id };
    expect(occupationRefusal(view, held)).toBe(`${itemName({ kind: 'Unit', unit: held.kind }, view.players.find((p) => p.id === held.owner)?.faction)} is holding ${city.name} down and cannot leave until Peace with ${ownerName(view, rival.id)}`);
    expect(occupationLead(view, held)).toBe(`Garrisoning ${city.name}: holding the city down until Peace with ${ownerName(view, rival.id)}.`);
    expect(occupationNote(view, held)).toContain(`${city.name}'s people will not accept your rule while ${ownerName(view, rival.id)} still fights. This unit cannot leave or disband.`);
    expect(occupationTileLine(view, held)).toBe(`Garrison of ${city.name} · held here until Peace with ${ownerName(view, rival.id)}`);
    expect(occupationRefusal(view, { ...held, occupation_lock: null })).toBeNull();
  });

  it('a frozen city is never offered as a decision and says it is occupied', () => {
    const rival = view.players.find((p) => p.team !== view.team)!;
    city.detail!.producing = null;
    city.detail!.production_queue = [];
    expect(idleCitiesNeedingProduction(view, []).map((c) => c.id)).toContain(city.id);
    expect(cityProductionWarning(city, [], undefined, view)).not.toBeNull();
    city.occupation_lock = rival.id;
    expect(idleCitiesNeedingProduction(view, []).map((c) => c.id)).not.toContain(city.id);
    expect(cityProductionWarning(city, [], undefined, view)).toBeNull();
    expect(cityOccupiedLine(view, city)).toBe(`Occupied: ${city.name} builds and buys nothing until Peace with ${ownerName(view, rival.id)}`);
    expect(cityOccupiedLine(view, { ...city, owner: rival.id })).toBeNull();
  });

  it('a capture forecast also warns that the capturer is locked and the city frozen', () => {
    const rival = view.players.find((p) => p.team !== view.team)!;
    const attacker = view.units.find((u) => u.owner === view.me && u.military)!;
    const target = { q: city.pos.q + 4, r: city.pos.r };
    const enemy = { ...city, id: city.id + 100, owner: rival.id, name: 'Tessitura', pos: target, hp: 0, occupation_lock: rival.id };
    view.cities.push(enemy);
    const f: BattleForecast = { attacker_strength: { total: 20, parts: [] }, defender_strength: { total: 12, parts: [] },
      win_chance: 100, loss_chance: 0, expected_damage_to_attacker: 0, expected_damage_to_defender: 0, attacker_hp: 100, defender_hp: 0, ranged: false };
    const taken = { attacks: 1, city_hp: 0, city_hp_after: 0, capture_chance: 100, can_capture: true };
    expect(forecastModel(f, false, view, attacker, target, [], taken).lockWarning).toBe(occupationLockWarning(view, attacker, enemy));
    expect(forecastModel(f, false, view, attacker, target, [], taken).lockWarning).toContain(`until Peace with ${ownerName(view, rival.id)}`);
    expect(forecastModel(f, false, view, attacker, target, [], { ...taken, capture_chance: 0 }).lockWarning).toBeNull();
    expect(forecastModel(f, false, view, attacker, target, [], { ...taken, can_capture: false }).lockWarning).toBeNull();
    expect(forecastModel(f, false, view, attacker, target, []).lockWarning).toBeNull();
    enemy.occupation_lock = null as never;
    expect(forecastModel(f, false, view, attacker, target, [], taken).lockWarning).toBeNull();
  });
});

describe('the carried hit at a city', () => {
  it('splits the hit between the city and the garrison from core numbers', () => {
    const a = { attacks: 1, city_hp: 12, city_hp_after: 0, capture_chance: 0, can_capture: true,
      garrison: { unit: 7, owner: 1, kind: 'Spearman' as const, hp: 100, hp_after: 69, destroyed_chance: 0, carried: 31 } };
    const rows = assaultSummary(a, 'Lugano', 'Spearman').rows;
    expect(rows[0]).toMatchObject({ label: 'Lugano', hp: 12, after: 0, note: 'carries −31 to the garrison' });
    expect(rows[1].note).toBeUndefined();
    expect(assaultSummary({ ...a, garrison: { ...a.garrison, carried: 0 } }, 'Lugano', 'Spearman').rows[0].note).toBeUndefined();
  });
});
