import { describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { createFlora, FloraCache, sculptRelief } from './flora';
import { buildField, gridFor, heightAt, tileAt, tileX, tileZ, worldFromArrays } from './world';

describe('procedural flora', () => {
  it('is repeatable and leaves space around a unit base', () => {
    const wd = worldFromArrays(1, 1, new Int8Array([4]), new Uint8Array([2]));
    const unit = { x: tileX(0, 1), z: tileZ(0, 1) + 0.05 };
    const make = () => createFlora(wd, () => 0.08, { cityTiles: new Set<number>(), units: [unit] });
    const positions = (render: ReturnType<typeof createFlora>) => render.group.children
      .filter((o): o is THREE.InstancedMesh => o instanceof THREE.InstancedMesh && o.count > 0)
      .slice(0, 1).flatMap((mesh) => {
        const matrix = new THREE.Matrix4(), point = new THREE.Vector3();
        return Array.from({ length: mesh.count }, (_, i) => {
          mesh.getMatrixAt(i, matrix); point.setFromMatrixPosition(matrix);
          return [point.x, point.z];
        });
      });
    const a = positions(make()), b = positions(make());
    expect(a.length).toBeGreaterThan(0);
    expect(a).toEqual(b);
    for (const [x, z] of a) expect(Math.hypot(x - unit.x, z - unit.z)).toBeGreaterThanOrEqual(0.18);
  });

  it('sculpts known mountain ground without changing unexplored ground', () => {
    const wd = worldFromArrays(2, 1, new Int8Array([8, -1]), new Uint8Array([2, 0]));
    const field = buildField(wd, gridFor(wd, 0.14));
    const original = field.height.slice();
    sculptRelief(wd, field);
    let changed = 0;
    for (let k = 0; k < field.height.length; k++) {
      const x = field.grid.x0 + (k % field.grid.nx) * field.grid.step;
      const z = field.grid.z0 + Math.floor(k / field.grid.nx) * field.grid.step;
      const i = tileAt(x, z, wd.w, wd.h);
      if (i === 1) expect(field.height[k]).toBe(original[k]);
      else if (field.height[k] !== original[k]) changed++;
      expect(field.pos[k * 3 + 1]).toBe(field.height[k]);
    }
    expect(changed).toBeGreaterThan(0);
  });

  it('raises a broad hill without adding geometry to its tile', () => {
    const wd = worldFromArrays(1, 1, new Int8Array([5]), new Uint8Array([2]));
    const field = buildField(wd, gridFor(wd, 0.1));
    const original = field.height.slice();
    sculptRelief(wd, field);
    expect(field.height.some((y, k) => y > original[k] + 0.1)).toBe(true);
  });

  it('keeps cached forest placements equal to a full rebuild after a reveal', () => {
    const cls = new Int8Array(12).fill(4), fog = new Uint8Array(12).fill(2);
    cls[5] = -1; fog[5] = 0;
    const before = worldFromArrays(4, 3, cls, fog);
    const beforeField = buildField(before); sculptRelief(before, beforeField);
    const clear = { cityTiles: new Set<number>(), units: [] };
    const cache = new FloraCache();
    createFlora(before, (x, z) => heightAt(beforeField, x, z), clear, cache);
    cls[5] = 8; fog[5] = 2;
    const after = worldFromArrays(4, 3, cls, fog);
    const field = buildField(after); sculptRelief(after, field);
    cache.invalidate(after, [5]);
    const make = (cached?: FloraCache) => createFlora(after, (x, z) => heightAt(field, x, z), clear, cached).group.children.map((o) => {
      const m = o as THREE.Mesh;
      return [m.geometry.getAttribute('position').array, m.geometry.getAttribute('instanceMemory')?.array,
        m.geometry.getAttribute('vertexMemory')?.array, (m as THREE.InstancedMesh).instanceMatrix?.array];
    });
    expect(make(cache)).toEqual(make());
  });
});
