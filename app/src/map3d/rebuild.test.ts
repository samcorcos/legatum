import { describe, expect, it } from 'vitest';
import type { PlayerView } from '../types';
import { buildField, gridFor, refreshFog, ROW, tileAt, tileX, tileZ, worldFrom, worldFromArrays } from './world';
import { refreshFogTiles } from './incrementalFog';
import { TerrainLookup } from './terrainLookup';
import { refreshTerrainTiles } from './incrementalTerrain';
import { terrainNormals, refreshTerrainNormals } from './terrainNormals';
import { sculptRelief } from './flora';
import { refreshReliefTiles } from './incrementalRelief';

describe('3D rebuild maths', () => {
  it('rebuilds the same field from transferred fogged tile arrays', () => {
    const w = 9, h = 7;
    const terrains = ['Mountains', 'Hills', 'Forest', 'Ocean', 'Grassland'];
    const tiles = Array.from({ length: w * h }, (_, i) => ({ explored: i % 6 !== 0, visible: i % 4 === 0, terrain: terrains[i % terrains.length], owner: null }));
    const world = worldFrom({ width: w, height: h, tiles } as unknown as PlayerView);
    const transferred = worldFromArrays(w, h, world.cls.slice(), world.fog.slice());
    expect(buildField(transferred)).toEqual(buildField(world));
  });

  it('keeps the same nearest tile as the previous nine-candidate search', () => {
    const w = 48, h = 32;
    const lookup = new TerrainLookup(w, h);
    const oldTileAt = (x: number, z: number) => {
      const r0 = Math.round(z / ROW);
      let best = 0, bd = Infinity;
      for (let dr = -1; dr <= 1; dr++) {
        const r = Math.min(h - 1, Math.max(0, r0 + dr));
        const c0 = Math.round(x - 0.5 * (r & 1));
        for (let dc = -1; dc <= 1; dc++) {
          const c = Math.min(w - 1, Math.max(0, c0 + dc));
          const i = r * w + c, dx = tileX(i, w) - x, dz = tileZ(i, w) - z, d = dx * dx + dz * dz;
          if (d < bd) { bd = d; best = i; }
        }
      }
      return best;
    };
    for (let zi = 0; zi < 400; zi++) for (let xi = 0; xi < 700; xi++) {
      const x = -0.6 + xi * 0.07, z = -0.6 + zi * 0.07;
      expect(tileAt(x, z, w, h)).toBe(oldTileAt(x, z));
      expect(lookup.nearest(x, z)).toBe(oldTileAt(x, z));
    }
  });

  it('matches a full fog refresh when one tile changes visibility', () => {
    const w = 8, h = 6;
    const tiles = Array.from({ length: w * h }, (_, i) => ({ explored: true, visible: i % 3 === 0, terrain: i % 9 === 0 ? 'Hills' : 'Grassland', owner: null }));
    const view = { width: w, height: h, tiles } as unknown as PlayerView;
    const wd = worldFrom(view);
    const original = buildField(wd, gridFor(wd));
    const expected = buildField(wd, gridFor(wd));
    wd.fog[20] = wd.fog[20] === 1 ? 2 : 1;
    refreshFog(wd, expected);
    const dirty = refreshFogTiles(wd, original, [20]);
    expect(dirty.length).toBeGreaterThan(0);
    expect(original.fog).toEqual(expected.fog);
  });

  it('matches a full field and normal rebuild when a mountain is revealed', () => {
    const w = 8, h = 6;
    const tiles = Array.from({ length: w * h }, (_, i) => ({ explored: i !== 20, visible: i % 3 === 0, terrain: i === 20 ? null : 'Grassland', owner: null }));
    const before = worldFrom({ width: w, height: h, tiles } as unknown as PlayerView);
    const field = buildField(before);
    const normals = terrainNormals(field.pos, field.grid.nx, field.grid.nz);
    tiles[20] = { explored: true, visible: true, terrain: 'Mountains', owner: null };
    const after = worldFrom({ width: w, height: h, tiles } as unknown as PlayerView);
    const expected = buildField(after);
    const dirty = refreshTerrainTiles(after, field, [20]);
    refreshFogTiles(after, field, [20]);
    refreshTerrainNormals(field.pos, normals, field.grid.nx, field.grid.nz, dirty);
    expect(field.pos).toEqual(expected.pos);
    expect(field.col).toEqual(expected.col);
    expect(field.fog).toEqual(expected.fog);
    expect(field.height).toEqual(expected.height);
    expect(normals).toEqual(terrainNormals(expected.pos, expected.grid.nx, expected.grid.nz));
  });

  it('keeps connected ridges identical after a one-tile reveal', () => {
    const w = 8, h = 6;
    const tiles = Array.from({ length: w * h }, (_, i) => ({ explored: i !== 20, visible: i !== 20, terrain: i === 20 || i === 21 ? 'Mountains' : 'Grassland', owner: null }));
    const before = worldFrom({ width: w, height: h, tiles } as unknown as PlayerView);
    const base = buildField(before), field = { ...base, pos: base.pos.slice(), col: base.col.slice(), height: base.height.slice() };
    sculptRelief(before, field);
    tiles[20] = { explored: true, visible: true, terrain: 'Mountains', owner: null };
    const after = worldFrom({ width: w, height: h, tiles } as unknown as PlayerView);
    refreshTerrainTiles(after, base, [20]);
    refreshReliefTiles(after, base, field, [20]);
    const expected = buildField(after);
    sculptRelief(after, expected);
    expect(field.pos).toEqual(expected.pos);
    expect(field.col).toEqual(expected.col);
    expect(field.height).toEqual(expected.height);
  });
});
