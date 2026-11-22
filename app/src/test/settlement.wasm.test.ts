// Full structural equality catches rounding, tie order, fog and explanation drift.
import { beforeAll, expect, it } from 'vitest';
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { knownLegalSite, settlementSites, siteBlocker } from './../settlement';
import { distance, hexAt, tileIndex, within } from '../hex';
import type { PlayerView, UnitView } from '../types';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let Game: { settlement_sites(view: string, unit: number, reserved: string): string; settlement_site(view: string, unit: number, q: number, r: number, reserved: string): string; site_blocker(view: string, q: number, r: number): string; new(setup: string): { view(p: number): string; free(): void }; standard_setup(...args: string[]): string };
beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js';
  const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  Game = mod.Game;
});
it.skipIf(!available)('lists legal fogged sites with compact scores and on-demand detail', () => {
  const faction = 'Germany';
  const game = new Game(Game.standard_setup('42', faction, 'India', 'A', 'B'));
  const v: PlayerView = JSON.parse(game.view(0));
  const u = v.units.find((u) => u.owner === v.me && u.kind === 'Spearman')!;
  u.kind = 'Settler'; u.military = false;
  for (const variant of [0, 1, 2]) {
    if (variant) {
      v.tiles.forEach((t, i) => { t.explored = i % 11 !== 0; t.visible = i % 3 === 0; t.terrain = ['Forest', 'Grassland', 'Hills', 'Plains', 'Coast', 'Mountains'][i % 6] as typeof t.terrain; t.owner = i % 23 === 0 ? 2 : i % 5 === 0 ? v.me : null; });
      const i = v.tiles.findIndex((t) => t.explored && t.owner === null && t.terrain === 'Grassland');
      u.pos = hexAt(i, v.width); u.goto = hexAt(i + 2, v.width); u.max_moves = variant;
      v.failed_city_sites = [hexAt(i + 6, v.width)];
      v.units.push({ ...u, id: 900 + variant, owner: 2, kind: 'Archer', military: true, ranged: 15, range: 2, pos: hexAt(i + 3, v.width) } as UnitView);
      if (variant === 2) v.cities.push({ owner: v.me, id: 100, name: 'Home', population: 2, hp: 200, max_hp: 200, strength: 20, walls: false, citadel: false, great_works: [], visible: true, detail: null, pos: hexAt(i + 10, v.width), is_capital: true });
    }
    const reserved = variant === 2 ? [hexAt(100, v.width)] : [];
    const expected = settlementSites(v, u, reserved);
    const scores = JSON.parse(Game.settlement_sites(JSON.stringify(v), u.id, JSON.stringify(reserved)));
    expect(scores).toEqual(expected.map(({ pos, score, quality, band, border_risk, rank, travel_turns }) => ({ pos, score, quality, band, border_risk, rank, travel_turns })));
    // The lens tints exactly the legal sites (core's rule, the mirror's rule and the list agree), and every other explored tile has a reason.
    const legal = v.tiles.flatMap((_, i) => knownLegalSite(v, hexAt(i, v.width)) ? [tileIndex(hexAt(i, v.width), v.width, v.height)] : []);
    expect(scores.map((s: { pos: { q: number; r: number } }) => tileIndex(s.pos, v.width, v.height)).sort((a: number, b: number) => a - b)).toEqual(legal);
    for (let i = 0; i < v.tiles.length; i += 7) {
      const pos = hexAt(i, v.width);
      expect(JSON.parse(Game.site_blocker(JSON.stringify(v), pos.q, pos.r))).toEqual(siteBlocker(v, pos));
      expect(siteBlocker(v, pos) === null).toBe(knownLegalSite(v, pos));
    }
    // Full explanations are fetched only for inspected sites, including reserved spacing.
    for (const site of [expected[0], expected.at(-1), expected.find((s) => s.rank === 1)].filter((s) => !!s)) {
      expect(JSON.parse(Game.settlement_site(JSON.stringify(v), u.id, site.pos.q, site.pos.r, JSON.stringify(reserved))))
        .toEqual({ ...site, rank: null });
    }
    expect(JSON.stringify(scores).length).toBeLessThan(JSON.stringify(expected).length / 5);
  }
  game.free();
}, 120_000); // ~10 s alone; 54 s seen on a loaded shared host.

it.skipIf(!available)('accepts a 3-hex city gap for India and blocks it for other factions in the Settle lens', () => {
  for (const faction of ['India', 'Athens'] as const) {
    const game = new Game(Game.standard_setup('42', faction, 'Byzantium', 'A', 'B'));
    try {
      const v: PlayerView = JSON.parse(game.view(0));
      const u = v.units.find((u) => u.owner === v.me && u.kind === 'Spearman')!;
      u.kind = 'Settler'; u.military = false;
      const home = { ...u.pos };
      const city = { owner: v.me, id: 100, name: 'Home', population: 2, hp: 200, max_hp: 200, strength: 20, walls: false, citadel: false, great_works: [], visible: true, detail: null, pos: home, is_capital: true };
      for (const t of v.tiles) { t.explored = true; t.visible = true; t.terrain = 'Grassland'; t.owner = null; }
      const pos = within(home, 3).find((h) => distance(h, home) === 3 && tileIndex(h, v.width, v.height) >= 0)!;
      u.pos = pos; v.units = [u]; v.cities = [city]; v.city_gap_blocked = [];
      const legal = knownLegalSite(v, pos);
      expect(JSON.parse(Game.site_blocker(JSON.stringify(v), pos.q, pos.r))).toEqual(siteBlocker(v, pos));
      expect(legal).toBe(faction === 'India');
    } finally { game.free(); }
  }
});

it.skipIf(!available)('flags a site near an unseen city inferred from visible borders, and never recommends it, until recon rules it out', () => {
  const game = new Game(Game.standard_setup('42', 'Athens', 'Byzantium', 'A', 'B'));
  try {
    const v: PlayerView = JSON.parse(game.view(0));
    const u = v.units.find((u) => u.owner === v.me && u.kind === 'Spearman')!;
    u.kind = 'Settler'; u.military = false;
    u.pos = { q: 5, r: 6 };
    v.units = [u]; v.cities = []; v.failed_city_sites = [];
    for (const t of v.tiles) { t.explored = true; t.visible = true; t.terrain = 'Grassland'; t.owner = null; }
    const center = v.tiles[tileIndex({ q: 8, r: 6 }, v.width, v.height)];
    const border = v.tiles[tileIndex({ q: 7, r: 6 }, v.width, v.height)];
    center.visible = false;
    const inspect = () => JSON.parse(Game.settlement_site(JSON.stringify(v), u.id, u.pos.q, u.pos.r, '[]'));
    expect(inspect().border_risk).toBe(false);
    border.owner = 1;
    expect(inspect()).toMatchObject({ border_risk: true, rank: null });
    expect(inspect()).toEqual({ ...settlementSites(v, u, [], u.pos)[0], rank: null });
    center.visible = true;
    expect(inspect().border_risk).toBe(false);
    expect(inspect()).toEqual(settlementSites(v, u, [], u.pos)[0]);
  } finally { game.free(); }
});

it.skipIf(!available)('scores Mongols sites like everyone else: no spaced-network adjustment, with or without reserved sites', () => {
  const game = new Game(Game.standard_setup('42', 'Mongols', 'India', 'A', 'B'));
  try {
    const v: PlayerView = JSON.parse(game.view(0));
    const u = v.units.find((u) => u.owner === v.me && u.kind === 'Spearman')!;
    u.kind = 'Settler'; u.military = false;
    for (const t of v.tiles) { t.explored = true; t.visible = true; t.terrain = 'Grassland'; t.owner = null; }
    const positions = [{ q: 4, r: 4 }, { q: 8, r: 4 }, { q: 4, r: 8 }];
    v.units = [u]; u.pos = { q: 16, r: 4 };
    v.cities = positions.map((pos, id) => ({ owner: v.me, id, name: 'Road', population: 2, hp: 200, max_hp: 200, strength: 20, walls: false, citadel: false, great_works: [], visible: true, detail: null, pos, is_capital: id === 0 }));
    const inspect = (pos: typeof u.pos, reserved: typeof positions = []) => {
      const expected = settlementSites(v, u, reserved, pos)[0];
      expect(JSON.parse(Game.settlement_site(JSON.stringify(v), u.id, pos.q, pos.r, JSON.stringify(reserved)))).toEqual(expected);
      return expected;
    };
    const plain = inspect(u.pos);
    expect(plain.breakdown.some((p) => p.label.startsWith('Mongols'))).toBe(false);
    const withReserved = inspect(u.pos, [{ q: 20, r: 4 }]);
    expect(withReserved.breakdown.some((p) => p.label.startsWith('Mongols'))).toBe(false);
    expect(withReserved.score).toBe(plain.score);
  } finally { game.free(); }
});

it.skipIf(!available)('matches Peace borders, neutral blockers, War threats and treaty egress', () => {
  const game = new Game(Game.standard_setup('42', 'India', 'Byzantium', 'A', 'B'));
  try {
    const v: PlayerView = JSON.parse(game.view(0));
    const u = v.units.find((u) => u.owner === v.me && u.kind === 'Spearman')!;
    u.kind = 'Settler'; u.military = false;
    const at = (col: number) => hexAt(4 * v.width + col, v.width);
    const tile = (col: number) => v.tiles[tileIndex(at(col), v.width, v.height)];
    u.pos = at(2); u.max_moves = 2; u.goto = null;
    v.units = [u]; v.cities = []; v.failed_city_sites = [];
    for (const t of v.tiles) { t.explored = true; t.visible = true; t.terrain = 'Mountains'; t.owner = null; }
    for (let col = 2; col <= 6; col++) tile(col).terrain = 'Grassland';
    // The standard setup's second human is a permanent teammate, not a rival.
    const { id: rival, team } = v.players.find((p) => p.team !== v.team)!;
    const relation = v.diplomacy.relations.find((r) => [r.a, r.b].includes(v.team) && [r.a, r.b].includes(team))!;
    const inspect = (col: number) => {
      const pos = at(col);
      const actual = JSON.parse(Game.settlement_site(JSON.stringify(v), u.id, pos.q, pos.r, '[]'));
      expect(actual).toEqual(settlementSites(v, u, [], pos)[0] ?? null);
      return actual;
    };
    for (const state of ['Peace'] as const) {
      relation.state = state;
      tile(3).owner = rival;
      expect(inspect(4).travel_turns).toBeNull();
      tile(3).owner = null;
      v.units = [u, { ...u, id: 900, owner: rival, kind: 'Archer', military: true, ranged: 15, range: 2, pos: at(3) }];
      expect(inspect(4)).toMatchObject({ travel_turns: null, threats: 0 });
      v.units = [u];
    }
    relation.state = 'War'; tile(3).owner = rival;
    expect(inspect(4).travel_turns).toBe(1);
    v.units.push({ ...u, id: 900, owner: rival, kind: 'Archer', military: true, ranged: 15, range: 2, pos: at(5) });
    expect(inspect(4).threats).toBeGreaterThan(0);
    relation.state = 'Peace'; v.units = [u]; u.pos = at(4);
    for (let col = 3; col <= 6; col++) tile(col).owner = rival;
    expect(inspect(2).travel_turns).toBe(1);
  } finally { game.free(); }
});
