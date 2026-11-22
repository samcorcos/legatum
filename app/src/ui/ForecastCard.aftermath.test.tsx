// The attack forecast covers the aftermath and its own assumption about fortification
// (p881 horseman, 2026-10-11).
// @vitest-environment happy-dom
import { act, type ReactElement } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ForecastCard, ForecastSummary, forecastModel } from './ForecastCard';
import { useGame } from '../store';
import { classBonusStrength, exactOdds, unitAttackStrength, unitDefenseStrength, addFightClassBonus } from '../combat';
import { unitStatsFor } from '../data';
import { addCity, addUnit, arenaView } from '../test/arenaView';
import type { BattleForecast, UnitView } from '../types';

const at = (q: number, r: number) => ({ q, r });

/** The forecast the core would give: Horseman charging a fortified Archer on plains. */
function charge(horse: UnitView, archer: UnitView): BattleForecast {
  const atk = unitAttackStrength(horse.kind, horse.hp, 'Sparta', 0);
  addFightClassBonus(atk, horse.kind, archer.kind, true, false, 'Sparta');
  const def = unitDefenseStrength(archer.kind, archer.hp, 'Plains', archer.fortify, 'India', 0);
  addFightClassBonus(def, archer.kind, horse.kind, false, false, 'India');
  const odds = exactOdds(atk.total, def.total, horse.hp, archer.hp, false);
  return {
    attacker_strength: atk, defender_strength: def, win_chance: odds.win, loss_chance: odds.loss,
    expected_damage_to_attacker: odds.expToAtk, expected_damage_to_defender: odds.expToDef,
    attacker_hp: horse.hp, defender_hp: archer.hp, ranged: false,
  };
}

async function scene() {
  const v = await arenaView('Sparta');
  const horse = addUnit(v, 0, 'Horseman', at(8, 5));
  const archer = addUnit(v, 2, 'Ranger', at(10, 5), { fortify: 6 });
  addCity(v, 2, 'Nara', at(12, 5));
  addUnit(v, 2, 'Soldier', at(10, 7));
  addUnit(v, 2, 'Soldier', at(11, 6));
  return { v, horse, archer, target: archer.pos };
}

let container: HTMLDivElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div'));
  root = createRoot(container);
  useGame.setState({ orders: [[]], seat: 0 });
});
afterEach(() => {
  act(() => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
});
const render = (ui: ReactElement) => act(() => root.render(ui));
const byTestId = (id: string) => container.querySelector(`[data-testid="${id}"]`) as HTMLElement;

describe('forecastModel', () => {
  it('carries the aftermath of a winning charge', async () => {
    const { v, horse, archer, target } = await scene();
    const m = forecastModel(charge(horse, archer), false, v, horse, target, []);
    expect(m.aftermath?.risk).toBe('Likely lost');
    expect(m.aftermath?.hp).toBe(100);
  });

  it('says it assumes fortification, with the kill chance if the Archer fires first', async () => {
    const { v, horse, archer, target } = await scene();
    const f = charge(horse, archer);
    const m = forecastModel(f, false, v, horse, target, []);
    expect(m.fortify).toEqual({ bonus: 6, win: expect.any(Number), shoots: true });
    expect(m.fortify!.win).toBeGreaterThan(m.win);
    const loose = exactOdds(f.attacker_strength.total, f.defender_strength.total - 6, f.attacker_hp, f.defender_hp, false);
    expect(m.fortify!.win).toBe(loose.win);
  });

  it('has neither for a ranged shot or an unfortified defender', async () => {
    const { v, horse, archer, target } = await scene();
    archer.fortify = 0;
    const m = forecastModel(charge(horse, archer), false, v, horse, target, []);
    expect(m.fortify).toBeNull();
    const shot = forecastModel({ ...charge(horse, archer), ranged: true }, true, v, horse, target, []);
    expect(shot.aftermath).toBeNull();
    expect(shot.fortify).toBeNull();
  });
});

describe('ForecastCard', () => {
  it('shows the held point for capturing a rival original capital, even one taken before', async () => {
    const { v, horse, target } = await scene();
    const city = addCity(v, 2, 'Nara', target);
    city.original_capital = true;
    city.original_capital_of = 2;
    const forecast = charge(horse, addUnit(v, 2, 'Soldier', at(target.q + 1, target.r)));
    render(<ForecastCard forecast={forecast} ranged={false} view={v} unit={horse} target={city.pos} />);
    expect(byTestId('forecast-capital-vp').querySelector(':scope > span:last-child')!.textContent).toBe('+1 VP while you hold Nara');
    expect(byTestId('forecast-capital-vp').querySelector('.vh-seal')).not.toBeNull();
    // A point scored on an earlier capture does not stop the next one: the point follows the city.
    v.players.find((p) => p.id === v.me)!.vp_sources.push([{ kind: 'CapitalTaken', capital: city.id }, 1]);
    render(<ForecastCard forecast={forecast} ranged={false} view={v} unit={horse} target={city.pos} />);
    expect(byTestId('forecast-capital-vp')).not.toBeNull();
  });

  it('shows the Gold a Celtic city attack would sack', async () => {
    const { v, horse, target } = await scene();
    v.players.find((p) => p.id === v.me)!.faction = 'Celts';
    const city = addCity(v, 2, 'Nara', target);
    const forecast = { ...charge(horse, addUnit(v, 2, 'Soldier', at(target.q + 1, target.r))), sack_gold: 180 };
    render(<ForecastCard forecast={forecast} ranged={false} view={v} unit={horse} target={city.pos} />);
    expect(byTestId('forecast-sack').textContent).toBe('Sack: +180 Gold (60 per population)');
  });
  it('puts a severe aftermath directly under the header and states the fortification assumption', async () => {
    const { v, horse, archer, target } = await scene();
    render(<ForecastCard forecast={charge(horse, archer)} ranged={false} view={v} unit={horse} target={target} />);
    const line = byTestId('forecast-aftermath');
    expect(line.getAttribute('data-risk')).toBe('Likely lost');
    expect(line.textContent).toMatch(/Nara's walls \(about −\d+\) and 2 Spearmen \(about −\d+ each, \+\d+ vs mounted\) can reach it\. Likely lost\.$/);
    expect(line.textContent).toContain(`+${classBonusStrength(unitStatsFor('India', 'Soldier').strength)} vs mounted`);
    const card = container.querySelector('.forecast')!;
    // Header, then the aftermath, before the sides.
    expect(card.children[0].className).toContain('forecast-head');
    expect(card.children[1]).toBe(line);
    expect(byTestId('forecast-fortify').textContent).toMatch(/^Assumes the Archer stays fortified \(\+6 Defense\)\. Firing loses it: chance above\.$/);
  });

  it('shows a Safe aftermath below the sides, quietly', async () => {
    const { v, horse, archer, target } = await scene();
    v.units = v.units.filter((u) => u.kind !== 'Soldier');
    v.cities = [];
    render(<ForecastCard forecast={charge(horse, archer)} ranged={false} view={v} unit={horse} target={target} />);
    const line = byTestId('forecast-aftermath');
    expect(line.getAttribute('data-risk')).toBe('Safe');
    expect(line.className).not.toContain('severe');
    expect(container.querySelector('.forecast')!.children[1]).not.toBe(line);
  });

  it('the touch summary carries both lines too', async () => {
    const { v, horse, archer, target } = await scene();
    render(<ForecastSummary forecast={charge(horse, archer)} ranged={false} view={v} unit={horse} target={target} />);
    expect(byTestId('forecast-aftermath').textContent).toContain('Likely lost');
    expect(byTestId('forecast-fortify').textContent).toContain('stays fortified');
  });
});
