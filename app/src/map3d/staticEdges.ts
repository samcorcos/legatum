import * as THREE from 'three';
import { neighborIndex } from '../map/grid';
import { mapPlayerColor } from '../map/style';
import { CORNERS, R, tileX, tileZ, type World } from './world';

/** Grid and owner-border geometry in the same tile/edge order as the original scene builder. */
function geometry(wd: World, height: (x: number, z: number) => number, includeGrid: boolean) {
  const { w, h } = wd;
  const gp: number[] = [], bp: number[] = [], bc: number[] = [];
  const ownerColors = new Map<number, THREE.Color>();
  for (let i = 0; i < w * h; i++) {
    if (!wd.fog[i]) continue;
    const cx = tileX(i, w), cz = tileZ(i, w);
    for (let e = 0; e < 6; e++) {
      const n = neighborIndex(i, e, w, h);
      const a = CORNERS[e], b = CORNERS[(e + 1) % 6];
      const ax = cx + a[0], az = cz + a[1], bx = cx + b[0], bz = cz + b[1];
      const land = wd.cls[i] > 1 || (n >= 0 && wd.cls[n] > 1);
      if (includeGrid && land && (n < 0 || n > i || !wd.fog[n])) {
        for (let k = 0; k < 4; k++) {
          const t0 = k / 4, t1 = (k + 1) / 4;
          const x0 = ax + (bx - ax) * t0, z0 = az + (bz - az) * t0, x1 = ax + (bx - ax) * t1, z1 = az + (bz - az) * t1;
          gp.push(x0, Math.max(0, height(x0, z0)) + 0.012, z0, x1, Math.max(0, height(x1, z1)) + 0.012, z1);
        }
      }
      const o = wd.owner[i];
      if (o >= 0 && (n < 0 || wd.owner[n] !== o)) {
        let c = ownerColors.get(o);
        if (!c) { c = new THREE.Color(mapPlayerColor(o)); ownerColors.set(o, c); }
        const P = (t: number, ins: number) => {
          const x = ax + (bx - ax) * t, z = az + (bz - az) * t;
          const xx = x + (cx - x) * ins / R, zz = z + (cz - z) * ins / R;
          return [xx, Math.max(0.005, height(xx, zz)) + 0.02, zz];
        };
        for (let k = 0; k < 5; k++) {
          const t0 = k / 5, t1 = (k + 1) / 5;
          const p0 = P(t0, 0.01), p1 = P(t1, 0.01), q0 = P(t0, 0.11), q1 = P(t1, 0.11);
          bp.push(...p0, ...q0, ...p1, ...p1, ...q0, ...q1);
          for (let v = 0; v < 6; v++) { const outer = v === 0 || v === 2 || v === 3; bc.push(c.r, c.g, c.b, outer ? 1 : 0); }
        }
      }
    }
  }
  return { gp, bp, bc };
}

export function makeBorders(wd: World, height: (x: number, z: number) => number): THREE.Mesh {
  const { bp, bc } = geometry(wd, height, false);
  const bg = new THREE.BufferGeometry(); bg.setAttribute('position', new THREE.Float32BufferAttribute(bp, 3)); bg.setAttribute('color', new THREE.Float32BufferAttribute(bc, 4));
  const borders = new THREE.Mesh(bg, new THREE.MeshBasicMaterial({ vertexColors: true, transparent: true, depthWrite: false, side: THREE.DoubleSide }));
  borders.renderOrder = 2;
  return borders;
}

export function makeEdges(wd: World, height: (x: number, z: number) => number): { grid: THREE.LineSegments; borders: THREE.Mesh } {
  const { gp, bp, bc } = geometry(wd, height, true);
  const gg = new THREE.BufferGeometry(); gg.setAttribute('position', new THREE.Float32BufferAttribute(gp, 3));
  const grid = new THREE.LineSegments(gg, new THREE.LineBasicMaterial({ color: '#2e2416', transparent: true, opacity: 0.16, depthWrite: false }));
  const bg = new THREE.BufferGeometry(); bg.setAttribute('position', new THREE.Float32BufferAttribute(bp, 3)); bg.setAttribute('color', new THREE.Float32BufferAttribute(bc, 4));
  const borders = new THREE.Mesh(bg, new THREE.MeshBasicMaterial({ vertexColors: true, transparent: true, depthWrite: false, side: THREE.DoubleSide }));
  borders.renderOrder = 2;
  return { grid, borders };
}
