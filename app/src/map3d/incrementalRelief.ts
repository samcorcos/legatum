import { sculptRelief } from './flora';
import { tileX, tileZ, type Field, type World } from './world';

/** Re-sculpt only the neighbourhood whose ridge graph changed on a small reveal. */
export function refreshReliefTiles(world: World, base: Field, field: Field, changed: readonly number[]): number[] {
  const grid = field.grid;
  const pad = 2.2; // one neighbour-wide ridge, its flank and the terrain blend
  let minX = Infinity, maxX = -Infinity, minZ = Infinity, maxZ = -Infinity;
  for (const tile of changed) {
    const x = tileX(tile, world.w), z = tileZ(tile, world.w);
    minX = Math.min(minX, x - pad); maxX = Math.max(maxX, x + pad);
    minZ = Math.min(minZ, z - pad); maxZ = Math.max(maxZ, z + pad);
  }
  const xa = Math.max(0, Math.floor((minX - grid.x0) / grid.step));
  const xb = Math.min(grid.nx - 1, Math.ceil((maxX - grid.x0) / grid.step));
  const za = Math.max(0, Math.floor((minZ - grid.z0) / grid.step));
  const zb = Math.min(grid.nz - 1, Math.ceil((maxZ - grid.z0) / grid.step));
  const nx = xb - xa + 1, nz = zb - za + 1, count = nx * nz;
  const sub: Field = {
    grid: { x0: grid.x0 + xa * grid.step, z0: grid.z0 + za * grid.step, step: grid.step, nx, nz },
    pos: new Float32Array(count * 3), col: new Float32Array(count * 3), height: new Float32Array(count),
    fog: new Float32Array(count), shore: new Uint8Array(count), materials: new Uint8Array(count * 4),
  };
  for (let z = 0; z < nz; z++) for (let x = 0; x < nx; x++) {
    const k = (za + z) * grid.nx + xa + x, j = z * nx + x;
    for (let c = 0; c < 3; c++) { sub.pos[j * 3 + c] = base.pos[k * 3 + c]; sub.col[j * 3 + c] = base.col[k * 3 + c]; }
    sub.height[j] = base.height[k];
  }
  sculptRelief(world, sub);
  const dirty: number[] = [];
  for (let z = 0; z < nz; z++) for (let x = 0; x < nx; x++) {
    const k = (za + z) * grid.nx + xa + x, j = z * nx + x;
    if (field.height[k] !== sub.height[j]) dirty.push(k);
    field.height[k] = sub.height[j];
    for (let c = 0; c < 3; c++) { field.pos[k * 3 + c] = sub.pos[j * 3 + c]; field.col[k * 3 + c] = sub.col[j * 3 + c]; }
  }
  return dirty;
}
