// Barbarians never attack the Celts (core `rules::barbarians::spares`), so nothing automatic or advisory treats
// them as a threat to a Celtic viewer: alarms, danger marks, wake-ups, warnings and aircraft prompts all skip them.
// Attacks stay legal (`atWar`), so the Celts can still clear camps by giving the order.
import { describe, expect, it } from 'vitest';
import { addCity, addUnit, arenaView } from './test/arenaView';
import { atWar, barbariansSpareMe, isThreat } from './diplomacy';
import { enemyReach } from './enemyReach';
import { citiesUnderAttack } from './siege';
import { airState, aircraftNeedingOrders, pickerCandidates, spotsIn, type Hold } from './airWing';
import { eligibleHints } from './ui/hintState';
import { reinforcementNote, woundedNearEnemies } from './ui/forecastWarnings';
import { fromOffset, hexEq, neighbors } from './hex';
import { BARBARIAN } from './barbarians';
import type { Faction, UnitKind } from './types';

const HOME = fromOffset(8, 8);
const NEXT = fromOffset(9, 8);

/** A bare map with `faction` as you, your city at HOME and a barbarian Spearman beside it. */
async function scene(faction: Faction, kind: UnitKind = 'Spearman') {
  const v = await arenaView(faction);
  const city = addCity(v, 0, 'Bern', HOME);
  const barbarian = addUnit(v, 0, kind, NEXT, { owner: BARBARIAN });
  return { v, city, barbarian };
}

const CELTS: [Faction, boolean][] = [['Celts', false], ['Switzerland', true]];

describe('barbarians are hostile to the Celts but no threat to them', () => {
  it.each(CELTS)('%s', async (faction, threat) => {
    const { v } = await scene(faction);
    expect(atWar(v, BARBARIAN)).toBe(true);
    expect(barbariansSpareMe(v)).toBe(!threat);
    expect(isThreat(v, BARBARIAN)).toBe(threat);
    // Rivals at War stay a threat to everyone, the Celts included.
    expect(isThreat(v, 2)).toBe(true);
  });
});

describe('alarms and danger marks', () => {
  it.each(CELTS)('%s: a barbarian beside a city is an alarm only for non-Celts', async (faction, alarmed) => {
    const { v } = await scene(faction);
    expect(citiesUnderAttack(v, null).map((a) => a.kind)).toEqual(alarmed ? ['attack'] : []);
  });

  it.each(CELTS)('%s: a barbarian only counts in the enemy reach for non-Celts', async (faction, counts) => {
    const { v } = await scene(faction);
    expect(enemyReach(v).any).toBe(counts);
  });

  it.each(CELTS)('%s: a wounded unit beside a barbarian is flagged only for non-Celts', async (faction, flagged) => {
    const { v } = await scene(faction);
    addUnit(v, 0, 'Spearman', neighbors(NEXT).find((h) => !hexEq(h, HOME))!, { hp: 30 });
    expect(woundedNearEnemies(v).length).toBe(flagged ? 1 : 0);
  });

  it.each(CELTS)('%s: the reinforcement note needs a real threat', async (faction, noted) => {
    const { v } = await scene(faction);
    const me = addUnit(v, 0, 'Spearman', fromOffset(7, 8));
    expect(reinforcementNote(v, me, HOME) !== null).toBe(noted);
    // A rival Spearman at War beside the city always earns it.
    addUnit(v, 2, 'Spearman', fromOffset(8, 9));
    expect(reinforcementNote(v, me, HOME)).not.toBeNull();
  });

  it.each(CELTS)('%s: sighting a barbarian is a "rival in sight" hint only for non-Celts', async (faction, hinted) => {
    const v = await arenaView(faction);
    addUnit(v, 0, 'Spearman', NEXT, { owner: BARBARIAN });
    expect(eligibleHints(v, []).includes('enemy')).toBe(hinted);
  });
});

describe('aircraft', () => {
  function wing(faction: Faction) {
    return scene(faction).then(({ v, city }) => {
      v.aircraft = [{ id: 50, owner: 0, kind: 'Fighter', city: city.id, hp: 100 }];
      return { v, city, fighter: v.aircraft[0] };
    });
  }

  it.each(CELTS)('%s: a barbarian in range asks for a strike decision only for non-Celts', async (faction, asks) => {
    const { v, city, fighter } = await wing(faction);
    const state = airState(v, city, fighter, [], undefined);
    expect(state.kind).toBe(asks ? 'ready' : 'guard');
    // The plane still shows up as needing an answer (like any idle unit), whether or not it can strike.
    expect(aircraftNeedingOrders(v, [], undefined)).toHaveLength(1);
  });

  it('a Celtic plane asleep stays asleep with a barbarian in range, and the Strike picker still lists it', async () => {
    const { v, city, fighter } = await wing('Celts');
    const asleep: Hold = { turn: v.turn - 3, planes: [], asleep: [50] };
    expect(airState(v, city, fighter, [], asleep)).toMatchObject({ kind: 'guard', asleep: true });
    expect(aircraftNeedingOrders(v, [], asleep)).toEqual([]);
    // Manual strikes on barbarians stay available.
    expect(spotsIn(v, city, 4).hostile).toHaveLength(1);
    expect(pickerCandidates(v, city, fighter, 'strike').map((c) => c.at)).toEqual([NEXT]);
  });

  it('a rival at War still wakes a Celtic plane', async () => {
    const { v, city, fighter } = await wing('Celts');
    addUnit(v, 2, 'Spearman', fromOffset(8, 10));
    const asleep: Hold = { turn: v.turn - 3, planes: [], asleep: [50] };
    expect(airState(v, city, fighter, [], asleep).kind).toBe('ready');
  });
});
