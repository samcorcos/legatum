// Procedural flora and exposed stone for the opt-in 3D map. All positions are derived from the
// fogged PlayerView's world; no asset download, random stream, or privileged game state is used.
import * as THREE from 'three';
import type { PlayerView } from '../types';
import { toOffset } from '../hex';
import { neighborIndex } from '../map/grid';
import { sculptRanges } from './ridgeRanges';
import { addMemoryWash, remembered } from './memoryWash';
import { loadTreeModel, TREE_MODELS, type TreeModel } from './treeModels';
import { fbm, hash2, R, rnd, tileAt, tileX, tileZ, type Field, type World } from './world';

type Point = { x: number; y: number; z: number; scale: number; yaw: number; fog: number; seed: number; tile: number };
type CachedPoint = { model: TreeModel; point: Point };
export class FloraCache {
  private points = new Map<number, { key: string; values: CachedPoint[] }>();
  clear(): void { this.points.clear(); }
  get(tile: number, key: string): CachedPoint[] | null {
    const entry = this.points.get(tile);
    return entry?.key === key ? entry.values : null;
  }
  set(tile: number, key: string, values: CachedPoint[]): void { this.points.set(tile, { key, values }); }
  invalidate(world: World, tiles: readonly number[]): void {
    for (const i of this.points.keys()) {
      const x = tileX(i, world.w), z = tileZ(i, world.w);
      if (tiles.some((j) => (tileX(j, world.w) - x) ** 2 + (tileZ(j, world.w) - z) ** 2 <= 3 ** 2)) this.points.delete(i);
    }
  }
}
type Species = 'broadleaf' | 'conifer' | 'olive' | 'snow';
const UP = new THREE.Vector3(0, 1, 0);

/** City tiles and a small space around each visible unit stay readable. */
export function floraClearings(view: PlayerView): { cityTiles: Set<number>; units: { x: number; z: number }[] } {
  const index = (pos: { q: number; r: number }) => {
    const [c, r] = toOffset(pos);
    return c < 0 || r < 0 || c >= view.width || r >= view.height ? -1 : r * view.width + c;
  };
  const cityTiles = new Set(view.cities.map((c) => index(c.pos)));
  const unitsByTile = new Map<number, number>();
  for (const u of view.units) {
    const i = index(u.pos);
    if (i >= 0) unitsByTile.set(i, (unitsByTile.get(i) ?? 0) + 1);
  }
  const units: { x: number; z: number }[] = [];
  for (const [i, count] of unitsByTile) for (let k = 0; k < count; k++) {
    const off = count > 1 ? (k - (count - 1) / 2) * 0.24 : 0;
    units.push({ x: tileX(i, view.width) + off, z: tileZ(i, view.width) + (cityTiles.has(i) ? 0.2 : 0.05) });
  }
  return { cityTiles, units };
}

function climate(wd: World, i: number): Species {
  let dry = 0, cold = 0;
  for (let e = -1; e < 6; e++) {
    const n = e < 0 ? i : neighborIndex(i, e, wd.w, wd.h);
    if (n < 0 || !wd.fog[n]) continue;
    dry += wd.cls[n] === 6 ? 2 : wd.cls[n] === 3 || wd.cls[n] === 5 ? 0.5 : 0;
    cold += wd.cls[n] === 7 ? 2 : wd.cls[n] === 8 ? 0.35 : 0;
  }
  if (cold >= 2) return 'snow';
  if (wd.cls[i] === 5) return 'conifer';
  if ((wd.cls[i] === 3 || wd.cls[i] === 4) && dry >= 2.5) return 'olive';
  return rnd(i, 81) < 0.24 ? 'conifer' : 'broadleaf';
}

/** Low-poly trunk and clustered crowns, with separate fine and distant silhouettes. */
const treeTemplates = new Map<string, THREE.BufferGeometry>();
function treeGeometry(species: Species, far: boolean): THREE.BufferGeometry {
  const key = `${species}-${far}`;
  const cached = treeTemplates.get(key);
  if (cached) {
    const geometry = new THREE.BufferGeometry();
    for (const name of Object.keys(cached.attributes)) geometry.setAttribute(name, cached.getAttribute(name));
    geometry.boundingSphere = cached.boundingSphere?.clone() ?? null;
    return geometry;
  }
  const pos: number[] = [], col: number[] = [];
  const add = (a: number[], b: number[], c: number[], color: [number, number, number]) => {
    pos.push(...a, ...b, ...c);
    for (let k = 0; k < 3; k++) col.push(...color);
  };
  const ring = (radius: number, y: number, sides: number, phase = 0) => Array.from({ length: sides }, (_, j) => {
    const a = j * Math.PI * 2 / sides + phase;
    return [Math.cos(a) * radius, y, Math.sin(a) * radius];
  });
  const bark: [number, number, number] = species === 'olive' ? [0.43, 0.39, 0.27] : [0.34, 0.27, 0.18];
  const sides = far ? 4 : 6, bottom = ring(0.013, 0, sides), top = ring(0.009, species === 'conifer' || species === 'snow' ? 0.17 : 0.14, sides);
  for (let j = 0; j < sides; j++) { const n = (j + 1) % sides; add(bottom[j], bottom[n], top[j], bark); add(top[j], bottom[n], top[n], bark); }
  const leaf: [number, number, number] = species === 'snow' ? [0.56, 0.65, 0.62]
    : species === 'conifer' ? [0.17, 0.31, 0.22] : species === 'olive' ? [0.38, 0.43, 0.28] : [0.30, 0.47, 0.23];
  const crown = (cx: number, cy: number, cz: number, rx: number, ry: number, rz: number, palette: [number, number, number]) => {
    const N = far ? 5 : 7;
    const mid = ring(1, 0, N, 0.12).map(([x, , z]) => [cx + x * rx, cy, cz + z * rz]);
    const peak = [cx, cy + ry, cz], base = [cx, cy - ry * 0.65, cz];
    for (let j = 0; j < N; j++) {
      const n = (j + 1) % N, shade = 0.82 + (j % 3) * 0.09;
      const c: [number, number, number] = [palette[0] * shade, palette[1] * shade, palette[2] * shade];
      if (species === 'conifer' || species === 'snow') {
        add(mid[j], mid[n], peak, c); add(base, mid[n], mid[j], c);
      } else {
        const angle = (j * Math.PI * 2 / N) + 0.12;
        const nextAngle = (n * Math.PI * 2 / N) + 0.12;
        const point = (a: number, r: number, y: number) => [cx + Math.cos(a) * rx * r, cy + ry * y, cz + Math.sin(a) * rz * r];
        const upper = point(angle, 0.72, 0.65), upperNext = point(nextAngle, 0.72, 0.65);
        const lower = point(angle, 0.82, -0.55), lowerNext = point(nextAngle, 0.82, -0.55);
        const light: [number, number, number] = [c[0] * 1.13, c[1] * 1.13, c[2] * 1.13];
        const dark: [number, number, number] = [c[0] * 0.78, c[1] * 0.78, c[2] * 0.78];
        add(upper, upperNext, peak, light);
        add(mid[j], mid[n], upper, c); add(mid[n], upperNext, upper, c);
        add(lower, lowerNext, mid[j], dark); add(lowerNext, mid[n], mid[j], dark);
        add(base, lowerNext, lower, dark);
      }
    }
  };
  if (species === 'conifer' || species === 'snow') {
    const tiers = far ? 2 : 3;
    for (let t = 0; t < tiers; t++) {
      const y = 0.105 + t * 0.063, r = 0.067 - t * 0.012;
      crown(0, y, 0, r, 0.056, r, leaf);
    }
    if (species === 'snow') crown(0, 0.24, 0, 0.018, 0.022, 0.018, [0.84, 0.88, 0.84]);
  } else if (species === 'olive') {
    crown(-0.027, 0.16, 0, 0.06, 0.047, 0.052, leaf);
    crown(0.03, 0.17, 0.005, 0.06, 0.045, 0.053, leaf);
    if (!far) crown(0, 0.195, -0.02, 0.05, 0.039, 0.047, [0.43, 0.48, 0.29]);
  } else {
    crown(0, 0.18, 0, 0.108, 0.072, 0.102, leaf);
    if (!far) {
      crown(-0.052, 0.16, 0.019, 0.074, 0.052, 0.068, [0.25, 0.42, 0.20]);
      crown(0.051, 0.166, -0.021, 0.072, 0.053, 0.069, [0.36, 0.51, 0.25]);
    }
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
  g.setAttribute('color', new THREE.Float32BufferAttribute(col, 3));
  g.computeVertexNormals();
  g.computeBoundingSphere();
  treeTemplates.set(key, g);
  const geometry = new THREE.BufferGeometry();
  for (const name of Object.keys(g.attributes)) geometry.setAttribute(name, g.getAttribute(name));
  geometry.boundingSphere = g.boundingSphere?.clone() ?? null;
  return geometry;
}

function rockGeometry(wd: World, H: (x: number, z: number) => number): THREE.BufferGeometry {
  const p: number[] = [], c: number[] = [], memory: number[] = [], memoryTiles: number[] = [];
  const tri = (a: number[], b: number[], d: number[], color: [number, number, number]) => {
    p.push(...a, ...b, ...d); for (let q = 0; q < 3; q++) { c.push(...color); memory.push(rockMemory); memoryTiles.push(rockTile); }
  };
  let rockMemory = 0, rockTile = -1;
  for (let i = 0; i < wd.cls.length; i++) {
    if (!wd.fog[i] || (wd.cls[i] !== 8 && wd.cls[i] !== 5)) continue;
    const mountain = wd.cls[i] === 8, count = mountain ? 7 : 3;
    for (let k = 0; k < count; k++) {
      const a = rnd(i, k + 600) * Math.PI * 2, d = mountain ? 0.29 + rnd(i, k + 620) * 0.27 : 0.12 + rnd(i, k + 620) * 0.23;
      const x = tileX(i, wd.w) + Math.cos(a) * d, z = tileZ(i, wd.w) + Math.sin(a) * d;
      const radius = mountain ? 0.018 + rnd(i, k + 640) * 0.024 : 0.028 + rnd(i, k + 640) * 0.026;
      const h = H(x, z), rise = mountain ? 0.012 + rnd(i, k + 660) * 0.025 : 0.012 + rnd(i, k + 660) * 0.025;
      rockMemory = remembered(wd.fog[i]);
      rockTile = i;
      const stone: [number, number, number] = mountain ? [0.45, 0.42, 0.38] : [0.51, 0.47, 0.38];
      const cap: [number, number, number] = mountain && h + rise > 0.75 ? [0.78, 0.79, 0.76] : [stone[0] * 1.08, stone[1] * 1.07, stone[2] * 1.05];
      const verts = Array.from({ length: 5 }, (_, j) => {
        const t = a + j * Math.PI * 2 / 5;
        const xx = x + Math.cos(t) * radius, zz = z + Math.sin(t) * radius;
        return [xx, H(xx, zz) - 0.012, zz];
      });
      const top = [x + radius * 0.38, h + rise, z - radius * 0.21];
      for (let j = 0; j < 5; j++) tri(verts[j], verts[(j + 1) % 5], top, j < 2 ? cap : stone);
    }
  }
  const g = new THREE.BufferGeometry(); g.setAttribute('position', new THREE.Float32BufferAttribute(p, 3)); g.setAttribute('color', new THREE.Float32BufferAttribute(c, 3)); g.setAttribute('vertexMemory', new THREE.Float32BufferAttribute(memory, 1)); g.userData.memoryTiles = Int16Array.from(memoryTiles); g.computeVertexNormals(); return g;
}

/** Broaden the existing mountain spikes into connected, asymmetric ridges in the terrain grid. */
export function sculptRelief(wd: World, field: Field): void {
  const { grid, height, pos, col } = field;
  const n = height.length, crest = new Float32Array(n), influence = new Float32Array(n), hillRise = new Float32Array(n);
  for (let i = 0; i < wd.cls.length; i++) {
    if (wd.cls[i] !== 5 || !wd.fog[i]) continue;
    const cx = tileX(i, wd.w), cz = tileZ(i, wd.w);
    const xmin = Math.max(0, Math.floor((cx - 0.55 - grid.x0) / grid.step));
    const xmax = Math.min(grid.nx - 1, Math.ceil((cx + 0.55 - grid.x0) / grid.step));
    const zmin = Math.max(0, Math.floor((cz - 0.55 - grid.z0) / grid.step));
    const zmax = Math.min(grid.nz - 1, Math.ceil((cz + 0.55 - grid.z0) / grid.step));
    const angle = rnd(i, 890) * Math.PI * 2, ca = Math.cos(angle), sa = Math.sin(angle);
    for (let zi = zmin; zi <= zmax; zi++) for (let xi = xmin; xi <= xmax; xi++) {
      const x = grid.x0 + xi * grid.step, z = grid.z0 + zi * grid.step;
      const dx = x - cx, dz = z - cz;
      const radial2 = dx * dx + dz * dz;
      if (radial2 > 0.55 ** 2 || (radial2 > 0.48 ** 2 && !wd.fog[tileAt(x, z, wd.w, wd.h)])) continue;
      const u = dx * ca + dz * sa, v = -dx * sa + dz * ca;
      const broad = Math.exp(-((u + 0.11) ** 2 / 0.12 + v * v / 0.085));
      const lobe = Math.exp(-((u - 0.19) ** 2 / 0.055 + (v - 0.08) ** 2 / 0.065));
      const k = zi * grid.nx + xi;
      hillRise[k] = Math.max(hillRise[k], 0.15 * broad + 0.07 * lobe);
    }
  }
  sculptRanges(wd, field, crest, influence);
  for (let k = 0; k < n; k++) {
    const t = influence[k]; if (t <= 0 && hillRise[k] <= 0) continue;
    const y = (height[k] + hillRise[k]) * (1 - t) + crest[k] * t;
    height[k] = y; pos[k * 3 + 1] = y;
    const stone = y > 0.88 ? [0.69, 0.70, 0.68] : y > 0.48 ? [0.53, 0.51, 0.48] : [0.43, 0.40, 0.35];
    const mix = t * 0.88;
    for (let channel = 0; channel < 3; channel++) col[k * 3 + channel] = col[k * 3 + channel] * (1 - mix) + stone[channel] * mix;
  }
}

export interface FloraRender {
  group: THREE.Group;
  profile: { placementsMs: number; instancesMs: number; floorMs: number; rocksMs: number };
  update(time: number, still: boolean, distance: number): void;
  clearUnits(units: { x: number; z: number }[]): void;
}

/** Colour the existing continuous terrain height field as layered stone above the foothills. */
export function shadeRelief(fragmentShader: string): string {
  return fragmentShader.replace('float seen = smoothstep(0.62, 0.95, vFog);', `
    float reliefRock = smoothstep(0.31, 0.49, vW.y);
    float reliefSnow = smoothstep(0.87, 1.04, vW.y + sin(vW.x * 17.0 + vW.z * 11.0) * 0.025);
    vec3 reliefNormal = normalize(vTerrainNormal);
    float reliefFace = dot(reliefNormal, normalize(vec3(-0.5, 0.8, 0.35)));
    float reliefFault = terrainNoise(vW.xz * 7.0 + vec2(vW.y * 4.0, -vW.y * 5.0));
    float reliefSeam = 1.0 - smoothstep(0.025, 0.10, abs(reliefFault - 0.5));
    gl_FragColor.rgb *= 1.0 + reliefRock * (1.0 - reliefSnow) * ((reliefFace - 0.55) * 0.22 - reliefSeam * 0.09);
    gl_FragColor.rgb = mix(gl_FragColor.rgb, vec3(0.90, 0.91, 0.89), reliefSnow * 0.48);
    float seen = smoothstep(0.62, 0.95, vFog);`);
}

export function createFlora(wd: World, H: (x: number, z: number) => number, clear: ReturnType<typeof floraClearings>, cache?: FloraCache): FloraRender {
  const started = performance.now();
  const byModel = Object.fromEntries(TREE_MODELS.map((model) => [model, [] as Point[]])) as Record<TreeModel, Point[]>;
  const floor: number[] = [], floorColors: number[] = [], floorMemory: number[] = [], floorTiles: number[] = [];
  const addFloor = (x: number, z: number, radius: number, memory: boolean, tile: number) => {
    const y = Math.max(0, H(x, z)) + 0.007, n = 8, color = [0.10, 0.17, 0.075];
    for (let j = 0; j < n; j++) {
      const a = j * Math.PI * 2 / n, b = (j + 1) * Math.PI * 2 / n;
      floor.push(x, y, z, x + Math.cos(a) * radius, Math.max(0, H(x + Math.cos(a) * radius, z + Math.sin(a) * radius)) + 0.008, z + Math.sin(a) * radius,
        x + Math.cos(b) * radius, Math.max(0, H(x + Math.cos(b) * radius, z + Math.sin(b) * radius)) + 0.008, z + Math.sin(b) * radius);
      for (let v = 0; v < 3; v++) { floorColors.push(...color); floorMemory.push(memory ? 1 : 0); floorTiles.push(tile); }
    }
  };
  for (let i = 0; i < wd.cls.length; i++) {
    if (!wd.fog[i] || clear.cityTiles.has(i)) continue;
    const kind = wd.cls[i], forest = kind === 4;
    if (!forest && !((kind === 2 || kind === 3 || kind === 7) && rnd(i, 3) < 0.26)) continue;
    const cx = tileX(i, wd.w), cz = tileZ(i, wd.w);
    if (forest) addFloor(cx, cz, 0.37, wd.fog[i] === 1, i);
    const key = [kind, ...Array.from({ length: 6 }, (_, e) => {
      const n = neighborIndex(i, e, wd.w, wd.h);
      return n < 0 ? -1 : wd.cls[n] * 4 + Number(wd.fog[n] > 0) * 2 + Number(clear.cityTiles.has(n));
    })].join(',');
    const cached = cache?.get(i, key);
    if (cached) {
      for (const { model, point } of cached) { point.fog = wd.fog[i]; byModel[model].push(point); }
      continue;
    }
    const tilePoints: CachedPoint[] = [];
    const base = climate(wd, i);
    // A world-space jittered lattice continues across connected forest tiles. The
    // density noise has no tile seed, so neighbouring crowns form one canopy.
    const candidates: { x: number; z: number; seed: number }[] = [];
    if (forest) {
      const spacing = 0.115;
      for (let gz = Math.floor((cz - R) / spacing); gz <= Math.ceil((cz + R) / spacing); gz++)
        for (let gx = Math.floor((cx - R) / spacing); gx <= Math.ceil((cx + R) / spacing); gx++) {
          const seed = hash2(gx, gz), x = (gx + 0.5 + (seed - 0.5) * 0.72) * spacing;
          const z = (gz + 0.5 + (hash2(gx + 91, gz - 37) - 0.5) * 0.72) * spacing;
          if (tileAt(x, z, wd.w, wd.h) !== i) continue;
          let edge = 1;
          for (let e = 0; e < 6; e++) {
            const n = neighborIndex(i, e, wd.w, wd.h);
            if (n >= 0 && wd.cls[n] === 4 && wd.fog[n] && !clear.cityTiles.has(n)) continue;
            if (n < 0) continue;
            const dx = tileX(n, wd.w) - cx, dz = tileZ(n, wd.w) - cz;
            const fromBorder = 0.5 - ((x - cx) * dx + (z - cz) * dz) / Math.hypot(dx, dz);
            edge = Math.min(edge, THREE.MathUtils.smoothstep(fromBorder, 0.06, 0.19));
          }
          const density = 0.68 + (fbm(x * 2.4 + 31, z * 2.4 - 17, 3) - 0.5) * 0.65;
          if (hash2(gx - 63, gz + 18) > density * edge) continue;
          candidates.push({ x, z, seed });
        }
    } else {
      for (let k = 0; k < 2 + Math.floor(rnd(i, 5) * 2); k++) {
        const a = rnd(i, 200 + k) * Math.PI * 2, d = 0.12 + rnd(i, 300 + k) * 0.17;
        candidates.push({ x: tileX(i, wd.w) + Math.cos(a) * d,
          z: tileZ(i, wd.w) + Math.sin(a) * d, seed: rnd(i, 700 + k) });
      }
    }
    for (let k = 0; k < candidates.length; k++) {
      const { x, z, seed } = candidates[k];
      const y = H(x, z); if (y < 0.025) continue;
      const species = base === 'broadleaf' && hash2(Math.floor(x * 43), Math.floor(z * 43)) < 0.18 ? 'conifer' : base;
      const variants = species === 'conifer' || species === 'snow' ? TREE_MODELS.slice(4) : TREE_MODELS.slice(0, 4);
      const model = variants[Math.min(variants.length - 1, Math.floor(hash2(Math.floor(x * 97), Math.floor(z * 97)) * variants.length))];
      const point = { x, y, z, scale: forest ? 0.46 + seed * 0.28 : 0.67 + seed * 0.3,
        yaw: seed * Math.PI * 2, fog: wd.fog[i], seed, tile: i };
      byModel[model].push(point);
      tilePoints.push({ model, point });
    }
    cache?.set(i, key, tilePoints);
  }
  const placementsMs = performance.now() - started;
  const group = new THREE.Group(), close: THREE.InstancedMesh[] = [], distant: THREE.InstancedMesh[] = [];
  const placements: { mesh: THREE.InstancedMesh; points: Point[]; hidden: boolean[] }[] = [];
  const wind = { value: 0 }, strength = { value: 0 };
  for (const model of TREE_MODELS) {
    const points = byModel[model]; if (!points.length) continue;
    const species: Species = model.includes('pine') ? 'conifer' : 'broadleaf';
    for (const far of [false, true]) {
      const mat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.96, side: THREE.DoubleSide, flatShading: true });
      mat.onBeforeCompile = (shader) => {
        addMemoryWash(shader);
        shader.uniforms.uFloraTime = wind; shader.uniforms.uFloraWind = strength;
        shader.vertexShader = shader.vertexShader.replace('#include <common>', '#include <common>\nuniform float uFloraTime; uniform float uFloraWind;')
          .replace('#include <begin_vertex>', `#include <begin_vertex>
            float sway = max(0.0, position.y - 0.075) * uFloraWind;
            transformed.x += sway * sin(uFloraTime * 1.05 + instanceMatrix[3].x * 3.7 + instanceMatrix[3].z * 2.1 + position.y * 9.0) * 0.11;
            transformed.z += sway * cos(uFloraTime * 0.83 + instanceMatrix[3].z * 3.3 + position.y * 7.0) * 0.08;`);
      };
      mat.customProgramCacheKey = () => 'legatum-flora-wind-memory-v2';
      const mesh = new THREE.InstancedMesh(treeGeometry(species, far), mat, points.length);
      mesh.userData.memoryTiles = Int16Array.from(points, (point) => point.tile);
      mesh.geometry.setAttribute('instanceMemory', new THREE.InstancedBufferAttribute(new Float32Array(points.map((point) => remembered(point.fog))), 1));
      const hidden: boolean[] = [];
      const matrices = mesh.instanceMatrix.array as Float32Array;
      const colors = new Float32Array(points.length * 3);
      points.forEach((point, j) => {
        hidden[j] = clear.units.some((u) => (u.x - point.x) ** 2 + (u.z - point.z) ** 2 < 0.25 ** 2);
        const scale = hidden[j] ? 0 : point.scale;
        const half = point.yaw / 2, y = Math.sin(half), w = Math.cos(half);
        const yy = y * (y + y), wy = w * (y + y), off = j * 16;
        matrices[off] = (1 - yy) * scale; matrices[off + 1] = 0; matrices[off + 2] = -wy * scale; matrices[off + 3] = 0;
        matrices[off + 4] = 0; matrices[off + 5] = scale; matrices[off + 6] = 0; matrices[off + 7] = 0;
        matrices[off + 8] = wy * scale; matrices[off + 9] = 0; matrices[off + 10] = (1 - yy) * scale; matrices[off + 11] = 0;
        matrices[off + 12] = point.x; matrices[off + 13] = point.y; matrices[off + 14] = point.z; matrices[off + 15] = 1;
        const tint = 0.84 + point.seed * 0.28;
        colors[j * 3] = tint; colors[j * 3 + 1] = tint; colors[j * 3 + 2] = tint;
      });
      mesh.instanceColor = new THREE.InstancedBufferAttribute(colors, 3);
      mesh.instanceMatrix.needsUpdate = true;
      mesh.castShadow = !far; mesh.receiveShadow = true;
      mesh.frustumCulled = false;
      group.add(mesh); (far ? distant : close).push(mesh);
      placements.push({ mesh, points, hidden });
      if (typeof document !== 'undefined') {
        void loadTreeModel(model).then((source) => {
          if (!group.parent) return;
          const geometry = source.clone();
          geometry.setAttribute('instanceMemory', (mesh.geometry.getAttribute('instanceMemory') as THREE.InstancedBufferAttribute).clone());
          mesh.geometry.dispose();
          mesh.geometry = geometry;
        }).catch(() => { /* Procedural silhouette stays usable if an asset fails to load. */ });
      }
    }
  }
  const instancesMs = performance.now() - started - placementsMs;
  if (floor.length) {
    const g = new THREE.BufferGeometry(); g.setAttribute('position', new THREE.Float32BufferAttribute(floor, 3)); g.setAttribute('color', new THREE.Float32BufferAttribute(floorColors, 3)); g.setAttribute('vertexMemory', new THREE.Float32BufferAttribute(floorMemory, 1)); g.userData.memoryTiles = Int16Array.from(floorTiles); g.computeVertexNormals();
    const m = new THREE.Mesh(g, new THREE.MeshBasicMaterial({ vertexColors: true, transparent: true, opacity: 0.23, depthWrite: false, side: THREE.DoubleSide }));
    m.userData.memoryTiles = g.userData.memoryTiles;
    m.material.onBeforeCompile = (shader) => addMemoryWash(shader, 'vertexMemory');
    m.material.customProgramCacheKey = () => 'flora-floor-memory-v1';
    m.renderOrder = 1; group.add(m);
  }
  const floorMs = performance.now() - started - placementsMs - instancesMs;
  const rocks = rockGeometry(wd, H);
  if (rocks.getAttribute('position').count) { const mesh = new THREE.Mesh(rocks, new THREE.MeshStandardMaterial({ vertexColors: true, flatShading: true, roughness: 1, side: THREE.DoubleSide })); mesh.userData.memoryTiles = rocks.userData.memoryTiles; mesh.material.onBeforeCompile = (shader) => addMemoryWash(shader, 'vertexMemory'); mesh.material.customProgramCacheKey = () => 'flora-rock-memory-v1'; mesh.castShadow = true; mesh.receiveShadow = true; group.add(mesh); }
  else rocks.dispose();
  const rocksMs = performance.now() - started - placementsMs - instancesMs - floorMs;
  return { group, profile: { placementsMs, instancesMs, floorMs, rocksMs }, clearUnits(units) {
    const m = new THREE.Matrix4(), q = new THREE.Quaternion(), v = new THREE.Vector3(), s = new THREE.Vector3();
    for (const { mesh, points, hidden } of placements) {
      let changed = false;
      points.forEach((point, j) => {
        const hide = units.some((u) => (u.x - point.x) ** 2 + (u.z - point.z) ** 2 < 0.25 ** 2);
        if (hide === hidden[j]) return;
        hidden[j] = hide; changed = true;
        q.setFromAxisAngle(UP, point.yaw);
        m.compose(v.set(point.x, point.y, point.z), q, s.setScalar(hide ? 0 : point.scale));
        mesh.setMatrixAt(j, m);
      });
      if (changed) mesh.instanceMatrix.needsUpdate = true;
    }
  }, update(time, still, distance) {
    wind.value = time; strength.value = still ? 0 : 1;
    const near = distance < 8.5;
    for (const mesh of close) mesh.visible = near;
    for (const mesh of distant) mesh.visible = !near;
  } };
}
