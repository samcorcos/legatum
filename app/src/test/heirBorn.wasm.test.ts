// The Heir's birth and public position against the real engine: every empire is told, the rival sees him in fog, and the app's
// report line, moment card, minimap dot and map tag read the same facts.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { fromOffset, within } from '../hex';
import { tileIndex } from '../hex';
import { heirHoverLine, heirTag } from '../heir';
import { contextFromViews, formatEvents } from '../format';
import { momentsForTurn } from '../celebrate/moments';
import { minimapUnits } from '../map/minimapView';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, meetAll } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function city(s: MState, owner: number, col: number, row: number, extra: Partial<MCity> = {}) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: owner === 0 ? 'Constantinople' : 'Pella', pos: fromOffset(col, row),
    population: 3, food: 0, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0, ...extra,
  };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  return c;
}

describe.skipIf(!available)('the Heir is born in public', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('tells every empire, shows the Heir to the rival in fog, and reads cleanly for both', () => {
    const setup = makeStandardSetup('55013', 'Byzantium', 'Sparta', 'AI 1', 'AI 2');
    setup.width = setup.height = 24;
    setup.players = setup.players.slice(0, 2).map((p, team) => ({ ...p, team, is_human: true }));
    const s = createState(setup);
    s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
    s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
    for (const r of s.diplomacy?.relations ?? []) r.state = 'Peace';
    meetAll(s);
    city(s, 0, 4, 4, { producing: { kind: 'Unit', unit: 'HeirToTheThrone' }, prod: 1000 });
    city(s, 1, 19, 19);
    refreshVision(s);
    const core = makeCore(); core.load(coreFixture(s));
    const b0 = core.view(0), b1 = core.view(1);
    const result = core.endTurn({});
    const a0 = core.view(0), a1 = core.view(1);
    const born = result.events.find((e) => e.type === 'HeirBorn');
    expect(born).toMatchObject({ type: 'HeirBorn', owner: 0, city_name: 'Constantinople' });

    // The rival never saw that tile, yet the Heir is in its view, current, and held by Byzantium.
    const heir = a1.units.find((u) => u.kind === 'HeirToTheThrone')!;
    expect(heir).toBeDefined();
    expect(heir.heir_of).toBe(0);
    expect(a1.tiles[tileIndex(heir.pos, a1.width, a1.height)].visible).toBe(false);
    expect(minimapUnits(a1).some((u) => u.heir && u.hex.q === heir.pos.q && u.hex.r === heir.pos.r)).toBe(true);
    expect(heirTag(a1, heir)).toBe('Heir to the Throne');
    expect(heirTag(a0, a0.units.find((u) => u.kind === 'HeirToTheThrone')!)).toBe('Your Heir');
    expect(heirHoverLine(a1, heir, 2)).toBe('Be at War, then capture him to score 2 VP');

    const rival = formatEvents(result.events, contextFromViews(b1, a1)).map((l) => l.text);
    expect(rival).toContain('Byzantium\'s Heir was born in Constantinople: be at War, then capture him to score 2 VP');
    const own = formatEvents(result.events, contextFromViews(b0, a0)).map((l) => l.text);
    expect(own).toContain('Your Heir was born in Constantinople: hold him to score 2 VP, and keep him safe');
    expect(own.join(' ')).not.toMatch(/trained the Heir/);

    // One card each: the rival's names the city and offers the map; Byzantium's is the Heir card with the city in its kicker.
    const theirs = momentsForTurn(result.events, 1, b1, a1);
    expect(theirs.filter((m) => m.title === 'Heir born')).toHaveLength(1);
    expect(theirs[0].effect).toContain('Born in Constantinople');
    expect(theirs[0].at).toEqual(born!.type === 'HeirBorn' ? born!.at : null);
    const mine = momentsForTurn(result.events, 0, b0, a0);
    expect(mine.filter((m) => m.title === 'The Heir lives')).toHaveLength(1);
    expect(mine[0].kicker).toBe('Your Heir is born in Constantinople');
  });
});
