/** BufferGeometry.computeVertexNormals for the regular terrain grid, without per-face Vector3 work. */
export function terrainNormals(pos: Float32Array, nx: number, nz: number): Float32Array {
  const normals = new Float32Array(pos.length);
  const tri = (ia: number, ib: number, ic: number) => {
    const a = ia * 3, b = ib * 3, c = ic * 3;
    const cbx = pos[c] - pos[b], cby = pos[c + 1] - pos[b + 1], cbz = pos[c + 2] - pos[b + 2];
    const abx = pos[a] - pos[b], aby = pos[a + 1] - pos[b + 1], abz = pos[a + 2] - pos[b + 2];
    const x = cby * abz - cbz * aby;
    const y = cbz * abx - cbx * abz;
    const z = cbx * aby - cby * abx;
    normals[a] += x; normals[a + 1] += y; normals[a + 2] += z;
    normals[b] += x; normals[b + 1] += y; normals[b + 2] += z;
    normals[c] += x; normals[c + 1] += y; normals[c + 2] += z;
  };
  for (let zi = 0; zi < nz - 1; zi++) for (let xi = 0; xi < nx - 1; xi++) {
    const a = zi * nx + xi, b = a + 1, c = a + nx, d = c + 1;
    tri(a, c, b); tri(b, c, d);
  }
  for (let k = 0; k < normals.length; k += 3) {
    const len = Math.sqrt(normals[k] * normals[k] + normals[k + 1] * normals[k + 1] + normals[k + 2] * normals[k + 2]);
    if (len) { normals[k] /= len; normals[k + 1] /= len; normals[k + 2] /= len; }
  }
  return normals;
}

/** Recalculate normals touched by changed terrain vertices, in the same cell order as a full pass. */
export function refreshTerrainNormals(pos: Float32Array, normals: Float32Array, nx: number, nz: number, dirty: readonly number[]): void {
  const affected = new Set<number>();
  for (const k of dirty) {
    const x = k % nx, z = (k - x) / nx;
    for (let dz = -1; dz <= 1; dz++) for (let dx = -1; dx <= 1; dx++) {
      const xx = x + dx, zz = z + dz;
      if (xx >= 0 && xx < nx && zz >= 0 && zz < nz) affected.add(zz * nx + xx);
    }
  }
  for (const k of affected) {
    const x = k % nx, z = (k - x) / nx, v = k * 3;
    normals[v] = 0; normals[v + 1] = 0; normals[v + 2] = 0;
    for (let cz = Math.max(0, z - 1); cz <= Math.min(nz - 2, z); cz++) for (let cx = Math.max(0, x - 1); cx <= Math.min(nx - 2, x); cx++) {
      const a = cz * nx + cx, b = a + 1, c = a + nx, d = c + 1;
      const add = (ia: number, ib: number, ic: number) => {
        if (k !== ia && k !== ib && k !== ic) return;
        const A = ia * 3, B = ib * 3, C = ic * 3;
        const cbx = pos[C] - pos[B], cby = pos[C + 1] - pos[B + 1], cbz = pos[C + 2] - pos[B + 2];
        const abx = pos[A] - pos[B], aby = pos[A + 1] - pos[B + 1], abz = pos[A + 2] - pos[B + 2];
        normals[v] += cby * abz - cbz * aby;
        normals[v + 1] += cbz * abx - cbx * abz;
        normals[v + 2] += cbx * aby - cby * abx;
      };
      add(a, c, b); add(b, c, d);
    }
    const len = Math.sqrt(normals[v] * normals[v] + normals[v + 1] * normals[v + 1] + normals[v + 2] * normals[v + 2]);
    if (len) { normals[v] /= len; normals[v + 1] /= len; normals[v + 2] /= len; }
  }
}
