import { describe, expect, it } from 'vitest';
import type { BuildingKind, GreatWorkView, PlayerView, TileView, VpSource } from '../types';
import { scoreCheckFor } from './scoreCheckCards';

function makeView(opts: {
  known?: string[];
  greatWorks?: GreatWorkView[];
  vpSources?: [VpSource, number][];
  tiles?: TileView[];
} = {}): PlayerView {
  const vpSources = opts.vpSources ?? [];
  const vp = vpSources.reduce((n, [, v]) => n + v, 0);
  return {
    me: 0,
    team: 0,
    turn: 42,
    turn_limit: 500,
    players: [
      {
        id: 0,
        team: 0,
        name: 'Ada',
        faction: 'India',
        alive: true,
        era: 2,
        vp,
        vp_sources: vpSources,
        capitals_taken: [],
        capitals_sacked: [],
        faction_goal_done: false,
        gold: 100,
        economy: 40,
        army: 40,
        advancement: 2000,
        power: 100,
        lead_percent: 0,
        decadence_percent: 0,
      },
      {
        id: 1,
        team: 1,
        name: 'Bo',
        faction: 'Mongols',
        alive: true,
        era: 2,
        vp: 1,
        vp_sources: [[{ kind: 'EraObjective', era: 2 }, 1]],
        capitals_taken: [],
        capitals_sacked: [],
        faction_goal_done: false,
        gold: 100,
        economy: 40,
        army: 40,
        advancement: 2000,
        power: 100,
        lead_percent: 0,
        decadence_percent: 0,
      },
    ],
    standings: [
      { team: 0, members: [0], vp, target: 10, victory_hold: null },
      { team: 1, members: [1], vp: 1, target: 10, victory_hold: null },
    ],
    diplomacy: { met: [1], relations: [] },
    economy: { gold: 100 },
    research: { known: opts.known ?? ['Masonry', 'Philosophy'], active: null, progress: 0 },
    eras: [
      {
        era: 2,
        name: 'Classical',
        entered_by: [0, 1],
        scored_by: [{ player: 1, slot: 0, turn: 40 }],
        objectives: [{ slot: 0, family: 'Fight', text: 'Veterans: earn 4 promotions', target: 4, objective: { kind: 'Veterans', n: 4 } }],
      },
    ],
    objective_progress: [{ era: 2, slot: 0, current: 0, target: 4, entered: true, era_scored: false, ready: false }],
    cities: [],
    tiles: opts.tiles ?? [],
    great_works: opts.greatWorks ?? [],
  } as unknown as PlayerView;
}

const wonderLineOf = (view: PlayerView) => scoreCheckFor(view, 2, 0).lines.find((l) => l.key === 'wonders')!;

describe('scoreCheckFor wonderLine (one built world wonder per empire)', () => {
  it('shows at most one open wonder box and uses "or" when multiple unbuilt wonders are researched', () => {
    const line = wonderLineOf(makeView({ known: ['Masonry', 'Philosophy'] }));
    expect(line.boxes).toHaveLength(1);
    expect(line.boxes[0]).toMatchObject({ state: 'open', span: 2 });
    expect(line.open).toBe(2);
    expect(line.how).toBe('Build the Pyramids or Great Library');
    expect(line.detail).toContain('one world wonder');
  });

  it('shows a single open box for the wonder currently under construction', () => {
    const line = wonderLineOf(makeView({
      known: ['Masonry', 'Philosophy'],
      greatWorks: [{ player: 0, completed: null, building: 'Pyramids', city: 1, progress: 120, cost: 360, production_per_turn: 48, turns: 5, blocker: null }],
    }));
    expect(line.boxes).toHaveLength(1);
    expect(line.boxes[0]).toMatchObject({ state: 'open', span: 2 });
    expect(line.open).toBe(2);
    expect(line.how).toBe('Building the Pyramids: 5 turns left');
  });

  it('shows no open buildable wonders once the empire has completed its world wonder', () => {
    const line = wonderLineOf(makeView({
      known: ['Masonry', 'Philosophy'],
      vpSources: [[{ kind: 'GreatWork', of: 0 }, 2]],
      greatWorks: [{ player: 0, completed: 'GreatLibrary', building: null, city: null, progress: 0, cost: 400, production_per_turn: 0, turns: null, blocker: null }],
    }));
    expect(line.boxes).toHaveLength(1);
    expect(line.boxes[0]).toMatchObject({ state: 'held', span: 2 });
    expect(line.open).toBe(0);
    expect(line.how).toBe('Capture a rival’s wonder');
    expect(line.detail).toContain('Great Library');
    expect(line.detail).toContain('only one');
  });

  it('still forbids building another wonder if the empire completed one and later lost its city', () => {
    const line = wonderLineOf(makeView({
      known: ['Masonry', 'Philosophy'],
      vpSources: [],
      greatWorks: [{ player: 0, completed: 'GreatLibrary', building: null, city: null, progress: 0, cost: 400, production_per_turn: 0, turns: null, blocker: null }],
    }));
    expect(line.boxes).toHaveLength(0);
    expect(line.open).toBe(0);
    expect(line.how).toBe('Capture a rival’s wonder');
    expect(line.detail).toContain('only one world wonder');
  });

  it('still allows building one world wonder when holding only a captured rival wonder', () => {
    const capturedTile = {
      owner: 0,
      landmark: { building: 'Pyramids' as BuildingKind, of: 1 },
    } as unknown as TileView;
    const line = wonderLineOf(makeView({
      known: ['Masonry', 'Philosophy'],
      vpSources: [[{ kind: 'GreatWork', of: 1 }, 2]],
      greatWorks: [
        { player: 0, completed: null, building: null, city: null, progress: 0, cost: 0, production_per_turn: 0, turns: null, blocker: null },
        { player: 1, completed: 'Pyramids', building: null, city: null, progress: 0, cost: 360, production_per_turn: 0, turns: null, blocker: null },
      ],
      tiles: [capturedTile],
    }));
    expect(line.boxes.map((b) => b.state)).toEqual(['held', 'open']);
    expect(line.open).toBe(2);
    expect(line.how).toBe('Build the Great Library');
    expect(line.detail).toContain('Pyramids');
    expect(line.detail).toContain('still build one world wonder');
  });
});
