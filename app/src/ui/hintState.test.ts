import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { WORK_RADIUS } from '../data';
import type { Order, PlayerView } from '../types';
import { eligibleHints, hintCopy, HINT_IDS, HINT_KEY, readHintPreferences, useHintPreferences } from './hintState';

let client: FixtureClient;
let view: PlayerView;
beforeEach(async () => {
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'Switzerland', 'India', 'Ada', 'Bo'));
  view = client.view(0);
  useHintPreferences.setState({ enabled: true, seen: [], resetVersion: 0 });
});
afterEach(() => vi.unstubAllGlobals());

function startingCapital(player = 0) {
  return client.view(player);
}

describe('contextual first-game hints', () => {
  it('offers production first and withholds Ready until all decisions, including research, are planned', () => {
    expect(eligibleHints(view, [])[0]).toBe('production');
    expect(eligibleHints(view, [])).toContain('research');
    const units: Order[] = view.units.filter((u) => u.owner === view.me).map((u) => ({ type: 'SkipUnit', unit: u.id }));
    expect(eligibleHints(view, units)).not.toContain('ready');
    const city = view.cities.find((c) => c.owner === view.me)!;
    const complete: Order[] = [...units, { type: 'SetProduction', city: city.id, item: { kind: 'Building', building: 'Granary' } }, { type: 'SetResearch', tech: view.research.available[0].tech }];
    expect(eligibleHints(view, complete)).toContain('ready');
    expect(eligibleHints(view, complete, [{ index: complete.length - 1, message: 'Invalid research' }])).not.toContain('ready');
  });

  it('waits for an owned city, then skips production and research that are already planned', () => {
    view = startingCapital();
    const city = view.cities.find((c) => c.owner === view.me)!;
    expect(eligibleHints(view, []).slice(0, 2)).toEqual(['production', 'research']);
    const planned = eligibleHints(view, [
      { type: 'SetProduction', city: city.id, item: { kind: 'Unit', unit: 'Scout' } },
      { type: 'SetResearch', tech: 'BronzeWorking' },
    ]);
    expect(planned).not.toContain('production');
    expect(planned).not.toContain('research');
    expect(planned).not.toContain('ready'); // Units still need orders.
  });

  it('uses the actual player ID, including a remote player whose local seat index is zero', () => {
    view = startingCapital(1);
    expect(view.me).toBe(1);
    expect(eligibleHints(view, [])).toContain('production');
    expect(eligibleHints(client.view(0), [])).toContain('production');
    expect(eligibleHints(client.view(0), [])).toContain('research');
  });

  it('does not count allies, remembered cities or public scoreboard rivals as first contact', () => {
    const rivals = view.players.filter((p) => p.team !== view.team);
    view.units = view.units.filter((u) => !rivals.some((p) => p.id === u.owner));
    view.cities = [];
    view.players = view.players.map((p) => ({ ...p, cities: 8 }));
    expect(eligibleHints(view, [])).not.toContain('enemy');
    const city = startingCapital().cities.find((c) => c.owner === 0)!;
    view.cities = [{ ...city, owner: rivals[0].id, visible: false, detail: null }];
    expect(eligibleHints(view, [])).not.toContain('enemy');
    view.cities[0].visible = true;
    expect(eligibleHints(view, [])).toContain('enemy');
  });

  it('recognizes a visible enemy unit and stops teaching completed or eliminated games', () => {
    view.units.push({ ...view.units[0], id: 999, owner: view.players.find((p) => p.team !== view.team)!.id });
    expect(eligibleHints(view, [])).toContain('enemy');
    expect(eligibleHints({ ...view, finished: true }, [])).toEqual([]);
    view.players.find((p) => p.id === view.me)!.alive = false;
    expect(eligibleHints(view, [])).toEqual([]);
  });
});

describe('hint copy', () => {
  it('keeps every hint to a short action, with the rules in the guide', () => {
    for (const id of HINT_IDS) expect(hintCopy(id, view).body.length, id).toBeLessThanOrEqual(170);
  });

  it('says a sighted rival at Peace is at peace, and warns of war only when at war', () => {
    const rival = view.players.find((p) => p.team !== view.team)!;
    view.units.push({ ...view.units[0], id: 999, owner: rival.id });
    const relation = view.diplomacy.relations.find((r) => (r.a === view.team && r.b === rival.team) || (r.b === view.team && r.a === rival.team))!;
    relation.state = 'Peace';
    expect(hintCopy('enemy', view).body).toMatch(/^You are at Peace with [^()]+\./);
    expect(hintCopy('enemy', view).body).toContain('A rival can declare War and attack in the same turn.');
    expect(hintCopy('enemy', view).body).not.toContain('turn later');
    relation.state = 'War';
    expect(hintCopy('enemy', view).body).toContain('You are at War with them');
  });

  it('says a city works all owned tiles in range and growth claims land', () => {
    expect(hintCopy('production', view).body).toContain(`every owned tile within ${WORK_RADIUS} tiles`);
    expect(hintCopy('production', view).body).toContain('each time it grows it claims one more tile');
    expect(hintCopy('production', view).body).not.toContain('population works the best tiles');
  });
});

describe('hint preferences', () => {
  it('persists shown tips and disabling across reads, then resets and enables them', () => {
    const storage = new Map<string, string>();
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => storage.get(key) ?? null,
      setItem: (key: string, value: string) => storage.set(key, value),
    });
    useHintPreferences.getState().markSeen('production');
    useHintPreferences.getState().markSeen('production');
    useHintPreferences.getState().setEnabled(false);
    expect(readHintPreferences()).toEqual({ enabled: false, seen: ['production'] });
    useHintPreferences.getState().reset();
    expect(readHintPreferences()).toEqual({ enabled: true, seen: [] });
    expect(useHintPreferences.getState().resetVersion).toBe(1);
    expect(storage.has(HINT_KEY)).toBe(true);
  });

  it('recovers from malformed preferences and ignores unknown tip IDs', () => {
    for (const raw of ['broken', 'null', '[]', '{"seen": false}']) {
      vi.stubGlobal('localStorage', { getItem: () => raw });
      expect(readHintPreferences()).toEqual({ enabled: true, seen: [] });
    }
    vi.stubGlobal('localStorage', { getItem: () => JSON.stringify({ enabled: false, seen: [...HINT_IDS, 'future', 'production'] }) });
    expect(readHintPreferences()).toEqual({ enabled: false, seen: [...HINT_IDS] });
  });

  it('keeps working in memory when localStorage is blocked', () => {
    const blocked = () => { throw new Error('Storage denied'); };
    vi.stubGlobal('localStorage', { getItem: blocked, setItem: blocked });
    expect(readHintPreferences()).toEqual({ enabled: true, seen: [] });
    expect(() => useHintPreferences.getState().markSeen('enemy')).not.toThrow();
    expect(useHintPreferences.getState().seen).toEqual(['enemy']);
    useHintPreferences.getState().setEnabled(false);
    expect(useHintPreferences.getState().enabled).toBe(false);
  });
});
