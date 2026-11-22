// The Keshig's forecast shows both options for the same target, each with its key, and never picks silently.
// @vitest-environment happy-dom
import { act, type ReactElement } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { DualForecastCard, classLines } from './DualForecastCard';
import { ForecastCard } from './ForecastCard';
import { useGame } from '../store';
import { MELEE_VS_MOUNTED_LABEL, MOUNTED_VS_RANGED_LABEL, addFightClassBonus, classBonusStrength, exactOdds, unitAttackStrength, unitDefenseStrength, unitRangedStrength } from '../combat';
import { unitStatsFor } from '../data';
import { addCity, addUnit, arenaView } from '../test/arenaView';
import type { BattleForecast, UnitKind, UnitView } from '../types';

const at = (q: number, r: number) => ({ q, r });

/** The forecast the core gives for a Keshig against `target`: a shot (fights as ranged) or a charge (as mounted). */
function forecast(keshig: UnitView, target: UnitView, shot: boolean): BattleForecast {
  const atk = shot ? unitRangedStrength(keshig.kind, keshig.hp, 'Mongols', 0) : unitAttackStrength(keshig.kind, keshig.hp, 'Mongols', 0);
  addFightClassBonus(atk, keshig.kind, target.kind as UnitKind, true, shot, 'Mongols');
  const def = unitDefenseStrength(target.kind as UnitKind, target.hp, 'Plains', target.fortify, 'Byzantium', 0);
  addFightClassBonus(def, target.kind as UnitKind, keshig.kind, false, shot, 'Byzantium');
  const odds = exactOdds(atk.total, def.total, keshig.hp, target.hp, shot);
  return {
    attacker_strength: atk, defender_strength: def, win_chance: odds.win, loss_chance: shot ? 0 : odds.loss,
    expected_damage_to_attacker: shot ? 0 : odds.expToAtk, expected_damage_to_defender: odds.expToDef,
    attacker_hp: keshig.hp, defender_hp: target.hp, ranged: shot,
  };
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

async function scene(targetKind: UnitKind = 'Spearman') {
  const v = await arenaView('Mongols');
  const keshig = addUnit(v, 0, 'Cataphract', at(8, 5));
  const target = addUnit(v, 2, targetKind, at(9, 5));
  return { v, keshig, target };
}

describe('the shoot-or-charge forecast', () => {
  it('shows the shot and the charge, each with the key that orders it', async () => {
    const { v, keshig, target } = await scene();
    render(<DualForecastCard shot={forecast(keshig, target, true)} charge={forecast(keshig, target, false)} view={v} unit={keshig} target={target.pos} />);
    const shot = byTestId('forecast-shot'), charge = byTestId('forecast-charge');
    expect(shot.textContent).toContain('Shoot');
    expect(shot.textContent).toContain('R, then click');
    expect(shot.textContent).toContain('no return fire');
    expect(charge.textContent).toContain('Charge');
    expect(charge.textContent).toContain('Right-click');
    expect(byTestId('forecast-dual').textContent).toContain('One order a turn');
    expect(byTestId('forecast-dual').textContent).toContain('never both');
  });

  it('gives a spear line no class bonus against the shot, and the bonus against the charge', async () => {
    const { v, keshig, target } = await scene('Spearman');
    const shot = forecast(keshig, target, true), charge = forecast(keshig, target, false);
    render(<DualForecastCard shot={shot} charge={charge} view={v} unit={keshig} target={target.pos} />);
    expect(byTestId('forecast-shot-class').textContent).toBe('No class bonus for either side.');
    const spear = classBonusStrength(unitStatsFor('Byzantium', 'Spearman').strength);
    expect(byTestId('forecast-charge-class').textContent).toContain(`${MELEE_VS_MOUNTED_LABEL}: +${spear} for the Spearman`);
    expect(classLines(shot, 'Keshig', 'Spearman')).toEqual([]);
    // So the shot is the stronger fight against the spear line.
    expect(shot.defender_strength.total).toBeLessThan(charge.defender_strength.total);
  });

  it('gives the Keshig the mounted bonus against an archer only when it charges', async () => {
    const { v, keshig, target } = await scene('Archer');
    const shot = forecast(keshig, target, true), charge = forecast(keshig, target, false);
    render(<DualForecastCard shot={shot} charge={charge} view={v} unit={keshig} target={target.pos} />);
    const rider = classBonusStrength(unitStatsFor('Mongols', keshig.kind).strength);
    expect(byTestId('forecast-charge-class').textContent).toContain(`${MOUNTED_VS_RANGED_LABEL}: +${rider} for your Keshig`);
    expect(byTestId('forecast-shot-class').textContent).toBe('No class bonus for either side.');
  });

  it('says a shot cannot take a city and a charge can', async () => {
    const { v, keshig } = await scene();
    const city = addCity(v, 2, 'Nara', at(9, 6), false);
    const target = addUnit(v, 2, 'Spearman', city.pos);
    render(<DualForecastCard shot={forecast(keshig, target, true)} charge={forecast(keshig, target, false)} view={v} unit={keshig} target={city.pos} />);
    expect(byTestId('forecast-shot').textContent).toContain("A shot can't take a city");
    expect(byTestId('forecast-charge').textContent).not.toContain("A shot can't take a city");
  });

  it('a single card says why there is no shot, beside the charge', async () => {
    const { v, keshig, target } = await scene();
    render(<ForecastCard forecast={forecast(keshig, target, false)} ranged={false} view={v} unit={keshig} target={target.pos} bowNote="No shot from here: this unit shoots only at a target next to it, and the target is 3 tiles away. Right-click to charge it." />);
    expect(byTestId('forecast-shot-why').textContent).toContain('Right-click to charge it');
  });
});
