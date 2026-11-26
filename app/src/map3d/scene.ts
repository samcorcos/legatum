// The 3D map scene (three.js): terrain, water, forests, roads, fields, towns, borders, units and the
// map's own overlays (hover, selection, route). It reads only the fogged PlayerView. The React view
// (Map3DView.tsx) owns input and HTML; this class owns the GPU side and the camera.
import * as THREE from 'three';
import { GLTFLoader, type GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import * as SkeletonUtils from 'three/examples/jsm/utils/SkeletonUtils.js';
import { RoomEnvironment } from 'three/examples/jsm/environments/RoomEnvironment.js';
import type { Hex, PlayerView, UnitView } from '../types';
import { fromOffset, toOffset } from '../hex';
import { neighborIndex } from '../map/grid';
import { mapPlayerColor } from '../map/style';
import { mapUnitArtSlug, mapUnitMiniatureUrl } from '../unitArt';
import { BARBARIAN_COLOR } from '../theme';
import { isBarbarian } from '../barbarians';
import {
  CORNERS, R, ROW, buildField, gridFor, hash2, heightAt, refreshFog, rnd, shapeChanged, tileAt, tileX, tileZ, worldFrom,
  type Field, type World,
} from './world';

const PARCH = new THREE.Color('#d9c9a3');
const UP = new THREE.Vector3(0, 1, 0);
/** Unit art with a finished 3D model (app/public/models/units/<file>.glb), keyed by the art slug the unit's painting
 *  uses (mapUnitArtSlug), so a model only stands in for its own painting: a Spartan or Aztec Spearman and a barbarian
 *  raider keep their own painted figures until they have models. Every other unit stands as its painted miniature.
 *  Licence and credits of each model: app/public/models/units/PROVENANCE-<file>.md. */
const MODELS: Partial<Record<string, string>> = { spearman: 'spearman' };
/** Close-up detail (unit contract v2): a model's LOD0 mesh draws only when the camera is closer than this, and only on
 *  the figures nearest the view centre (at most LOD0_FIGURES); every other figure draws its light LOD1. */
const LOD0_DIST = 4;
const LOD0_FIGURES = 16;
const MODEL_URL = (slug: string) => `${import.meta.env.BASE_URL}models/units/${slug}.glb`;

export interface Overlays {
  hover: Hex | null;
  selected: Hex | null;
  /** The planned route of the selected unit, tile by tile from its position. */
  route: Hex[];
}

const tileIndexOf = (h: Hex, w: number, hgt: number) => {
  const [c, r] = toOffset(h);
  return c < 0 || r < 0 || c >= w || r >= hgt ? -1 : r * w + c;
};

/** Figures turn a little toward their spear side so the weapon reads from the map camera (units thread, 2026-11-25). */
const DISPLAY_YAW = 0.5;

export class MapScene {
  readonly renderer: THREE.WebGLRenderer;
  readonly scene = new THREE.Scene();
  readonly camera = new THREE.PerspectiveCamera(32, 1, 0.05, 200);
  /** Camera target on the ground (world units) and distance from it. */
  readonly target = new THREE.Vector3();
  dist = 13;
  private sun = new THREE.DirectionalLight('#fff1d6', 2.4);
  private uniforms = { uTime: { value: 0 }, uParch: { value: PARCH } };
  private world: World | null = null;
  private field: Field | null = null;
  private terrain: THREE.Mesh | null = null;
  private static_ = new THREE.Group();
  private dynamic = new THREE.Group();
  private units = new THREE.Group();
  private hoverRing: THREE.Mesh;
  private selRing: THREE.Mesh;
  private route: THREE.Mesh;
  private mixers: THREE.AnimationMixer[] = [];
  private models = new Map<string, Promise<GLTF | null>>();
  private textures = new Map<string, THREE.Texture>();
  private view: PlayerView | null = null;
  private clock = new THREE.Clock();
  private ray = new THREE.Raycaster();
  private disposed = false;
  /** Bumped whenever something changed that needs a frame even if the camera is still. */
  private animating = true;

  constructor(canvas: HTMLCanvasElement) {
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.05;
    this.scene.background = PARCH.clone();
    this.scene.fog = new THREE.Fog(PARCH, 30, 70);
    this.scene.add(new THREE.HemisphereLight('#dfe9f2', '#6b5a3e', 1.1));
    const sun = this.sun;
    sun.castShadow = true;
    sun.shadow.mapSize.set(2048, 2048);
    sun.shadow.camera.near = 0.5; sun.shadow.camera.far = 40;
    sun.shadow.bias = -0.0006; sun.shadow.normalBias = 0.02;
    this.scene.add(sun, sun.target, this.static_, this.dynamic, this.units);
    // Rings are thin ground-hugging bands, not GL lines (WebGL draws lines 1 px wide whatever the zoom).
    this.hoverRing = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ color: '#fbf1cf', transparent: true, opacity: 0.85, depthTest: false, side: THREE.DoubleSide }));
    this.selRing = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ color: '#e2b13a', depthTest: false, side: THREE.DoubleSide }));
    this.route = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ color: '#f3dc8a', transparent: true, opacity: 0.9, depthTest: false, side: THREE.DoubleSide }));
    for (const o of [this.hoverRing, this.selRing, this.route]) { o.renderOrder = 6; o.visible = false; this.scene.add(o); }
  }

  dispose() {
    this.disposed = true;
    this.scene.traverse((o) => {
      const m = o as THREE.Mesh;
      m.geometry?.dispose();
      const mat = m.material as THREE.Material | THREE.Material[] | undefined;
      if (Array.isArray(mat)) mat.forEach((x) => x.dispose()); else mat?.dispose();
    });
    for (const t of this.textures.values()) t.dispose();
    this.env?.dispose();
    this.renderer.dispose();
  }

  resize(w: number, h: number) {
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / Math.max(1, h);
    this.camera.updateProjectionMatrix();
    this.animating = true;
  }

  // ---------------- world ----------------

  /** Build or update everything from a new view. Cheap when only fog, borders or units changed. */
  setView(view: PlayerView) {
    const next = worldFrom(view);
    const rebuild = !this.world || !this.field || shapeChanged(this.world, next);
    this.world = next;
    this.view = view;
    if (rebuild) {
      this.field = buildField(next, gridFor(next));
      this.buildTerrain();
    } else {
      refreshFog(next, this.field!);
      const attr = (this.terrain!.geometry.getAttribute('fogv') as THREE.BufferAttribute);
      (attr.array as Float32Array).set(this.field!.fog); attr.needsUpdate = true;
      this.updateWaterInfo();
    }
    this.buildStatic();
    this.buildUnits();
    this.animating = true;
  }

  private buildTerrain() {
    const f = this.field!;
    const { nx, nz } = f.grid;
    if (this.terrain) { this.scene.remove(this.terrain); this.terrain.geometry.dispose(); }
    const idx = new Uint32Array((nx - 1) * (nz - 1) * 6);
    let p = 0;
    for (let zi = 0; zi < nz - 1; zi++) for (let xi = 0; xi < nx - 1; xi++) {
      const a = zi * nx + xi, b = a + 1, c = a + nx, d = c + 1;
      idx[p++] = a; idx[p++] = c; idx[p++] = b; idx[p++] = b; idx[p++] = c; idx[p++] = d;
    }
    const g = new THREE.BufferGeometry();
    g.setAttribute('position', new THREE.BufferAttribute(f.pos, 3));
    g.setAttribute('color', new THREE.BufferAttribute(f.col, 3));
    g.setAttribute('fogv', new THREE.BufferAttribute(f.fog, 1));
    g.setIndex(new THREE.BufferAttribute(idx, 1));
    g.computeVertexNormals();
    const mat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.95, metalness: 0 });
    mat.onBeforeCompile = (sh) => {
      Object.assign(sh.uniforms, this.uniforms);
      sh.vertexShader = sh.vertexShader
        .replace('#include <common>', '#include <common>\nattribute float fogv; varying float vFog; varying vec3 vW;')
        .replace('#include <begin_vertex>', '#include <begin_vertex>\nvFog = fogv; vW = (modelMatrix * vec4(position, 1.0)).xyz;');
      sh.fragmentShader = sh.fragmentShader
        .replace('#include <common>', '#include <common>\nuniform vec3 uParch; varying float vFog; varying vec3 vW;')
        .replace('#include <dithering_fragment>', `#include <dithering_fragment>
          {
            vec2 p = vW.xz * 18.0; vec2 ip = floor(p), fp = fract(p); fp = fp * fp * (3.0 - 2.0 * fp);
            #define HH(q) fract(sin(dot(q, vec2(12.9898, 78.233))) * 43758.5453)
            float g1 = mix(mix(HH(ip), HH(ip + vec2(1, 0)), fp.x), mix(HH(ip + vec2(0, 1)), HH(ip + vec2(1, 1)), fp.x), fp.y);
            float st = sin(vW.x * 60.0 + sin(vW.z * 7.0) * 3.0) * 0.5 + 0.5;
            gl_FragColor.rgb *= 0.965 + 0.05 * g1 + 0.03 * st;
          }
          float seen = smoothstep(0.62, 0.95, vFog);
          float known = smoothstep(0.12, 0.42, vFog);
          vec3 c = gl_FragColor.rgb;
          float l = dot(c, vec3(0.299, 0.587, 0.114));
          c = mix(mix(vec3(l), c, 0.35) * 0.85 + uParch * 0.18, c, seen);
          float hatch = smoothstep(0.35, 0.5, abs(fract((vW.x + vW.z) * 9.0) - 0.5));
          c = mix(uParch * (0.93 + 0.07 * hatch), c, known);
          gl_FragColor.rgb = c;`);
    };
    this.terrain = new THREE.Mesh(g, mat);
    this.terrain.receiveShadow = true;
    this.terrain.castShadow = true;
    this.scene.add(this.terrain);
    this.buildWater();
  }

  private water: THREE.Mesh | null = null;
  private waterInfo: THREE.DataTexture | null = null;

  private updateWaterInfo() {
    const f = this.field!, tex = this.waterInfo!;
    const data = tex.image.data as Uint8Array;
    for (let k = 0; k < f.height.length; k++) {
      data[k * 4] = Math.max(0, Math.min(255, -f.height[k] * 400));
      data[k * 4 + 1] = Math.round(f.fog[k] * 255);
      data[k * 4 + 3] = 255;
    }
    tex.needsUpdate = true;
  }

  private buildWater() {
    const f = this.field!, { nx, nz, x0, z0, step } = f.grid;
    if (this.water) { this.scene.remove(this.water); this.water.geometry.dispose(); (this.water.material as THREE.Material).dispose(); this.waterInfo?.dispose(); }
    this.waterInfo = new THREE.DataTexture(new Uint8Array(nx * nz * 4), nx, nz, THREE.RGBAFormat);
    this.waterInfo.magFilter = THREE.LinearFilter; this.waterInfo.minFilter = THREE.LinearFilter;
    this.updateWaterInfo();
    const wdt = (nx - 1) * step, dep = (nz - 1) * step;
    const fog = this.scene.fog as THREE.Fog;
    const mat = new THREE.ShaderMaterial({
      transparent: true,
      fog: true,
      uniforms: {
        ...this.uniforms, uInfo: { value: this.waterInfo }, uBox: { value: new THREE.Vector4(x0, z0, wdt, dep) },
        fogColor: { value: fog.color }, fogNear: { value: fog.near }, fogFar: { value: fog.far },
      },
      vertexShader: `varying vec3 vW;
        #include <fog_pars_vertex>
        void main() { vec4 w = modelMatrix * vec4(position, 1.0); vW = w.xyz; vec4 mvPosition = viewMatrix * w; gl_Position = projectionMatrix * mvPosition;
        #include <fog_vertex>
        }`,
      fragmentShader: `uniform sampler2D uInfo; uniform vec4 uBox; uniform float uTime; uniform vec3 uParch; varying vec3 vW;
        #include <fog_pars_fragment>
        float h1(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
        float n1(vec2 p) { vec2 i = floor(p), f = fract(p); f = f * f * (3.0 - 2.0 * f); return mix(mix(h1(i), h1(i + vec2(1, 0)), f.x), mix(h1(i + vec2(0, 1)), h1(i + vec2(1, 1)), f.x), f.y); }
        void main() {
          vec4 inf = texture2D(uInfo, (vW.xz - uBox.xy) / uBox.zw);
          float depth = inf.r, fogv = inf.g;
          vec3 c = mix(vec3(0.36, 0.62, 0.60), vec3(0.10, 0.26, 0.34), smoothstep(0.02, 0.55, depth));
          float w1 = n1(vW.xz * 6.0 + vec2(uTime * 0.25, uTime * 0.18)), w2 = n1(vW.xz * 13.0 - vec2(uTime * 0.3, 0.0));
          c += smoothstep(0.72, 0.95, w1 * 0.6 + w2 * 0.4) * 0.10;
          float foam = (1.0 - smoothstep(0.0, 0.05, depth)) * (0.55 + 0.45 * sin(uTime * 1.6 + depth * 90.0 + w1 * 4.0));
          c = mix(c, vec3(0.93, 0.92, 0.86), clamp(foam, 0.0, 1.0) * 0.75);
          float a = mix(0.55, 0.92, smoothstep(0.0, 0.3, depth));
          float seen = smoothstep(0.62, 0.95, fogv), known = smoothstep(0.12, 0.42, fogv);
          float l = dot(c, vec3(0.299, 0.587, 0.114));
          c = mix(mix(vec3(l), c, 0.35) * 0.85 + uParch * 0.18, c, seen);
          c = mix(uParch, c, known); a = mix(1.0, a, known);
          gl_FragColor = vec4(c, a);
          #include <fog_fragment>
        }`,
    });
    this.water = new THREE.Mesh(new THREE.PlaneGeometry(wdt, dep).rotateX(-Math.PI / 2), mat);
    this.water.position.set(x0 + wdt / 2, 0, z0 + dep / 2);
    this.scene.add(this.water);
  }

  heightAt(x: number, z: number) { return this.field ? heightAt(this.field, x, z) : 0; }

  /** Grid, borders, forests, roads, fields and towns: everything that changes only when a turn resolves. */
  private buildStatic() {
    const wd = this.world!, view = this.view!;
    const { w, h } = wd;
    for (const o of [...this.static_.children]) { this.static_.remove(o); disposeTree(o); }
    const H = (x: number, z: number) => Math.max(0, this.heightAt(x, z));
    // Hex grid on known land, and borders on the owned side of each edge.
    const gp: number[] = [], bp: number[] = [], bc: number[] = [];
    for (let i = 0; i < w * h; i++) {
      if (!wd.fog[i]) continue;
      const cx = tileX(i, w), cz = tileZ(i, w);
      for (let e = 0; e < 6; e++) {
        const n = neighborIndex(i, e, w, h);
        const a = CORNERS[e], b = CORNERS[(e + 1) % 6];
        const ax = cx + a[0], az = cz + a[1], bx = cx + b[0], bz = cz + b[1];
        const land = wd.cls[i] > 1 || (n >= 0 && wd.cls[n] > 1);
        if (land && (n < 0 || n > i || !wd.fog[n])) {
          for (let k = 0; k < 4; k++) {
            const t0 = k / 4, t1 = (k + 1) / 4;
            const x0 = ax + (bx - ax) * t0, z0 = az + (bz - az) * t0, x1 = ax + (bx - ax) * t1, z1 = az + (bz - az) * t1;
            gp.push(x0, H(x0, z0) + 0.012, z0, x1, H(x1, z1) + 0.012, z1);
          }
        }
        const o = wd.owner[i];
        if (o >= 0 && (n < 0 || wd.owner[n] !== o)) {
          const c = new THREE.Color(mapPlayerColor(o));
          const P = (t: number, ins: number) => {
            const x = ax + (bx - ax) * t, z = az + (bz - az) * t;
            const xx = x + (cx - x) * ins / R, zz = z + (cz - z) * ins / R;
            return [xx, Math.max(0.005, this.heightAt(xx, zz)) + 0.02, zz];
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
    const gg = new THREE.BufferGeometry(); gg.setAttribute('position', new THREE.Float32BufferAttribute(gp, 3));
    const grid = new THREE.LineSegments(gg, new THREE.LineBasicMaterial({ color: '#2e2416', transparent: true, opacity: 0.16, depthWrite: false }));
    const bg = new THREE.BufferGeometry(); bg.setAttribute('position', new THREE.Float32BufferAttribute(bp, 3)); bg.setAttribute('color', new THREE.Float32BufferAttribute(bc, 4));
    const borders = new THREE.Mesh(bg, new THREE.MeshBasicMaterial({ vertexColors: true, transparent: true, depthWrite: false, side: THREE.DoubleSide }));
    borders.renderOrder = 2;
    this.static_.add(grid, borders);
    this.static_.add(...this.trees(), this.roads(), ...this.towns(view));
  }

  private trees(): THREE.Object3D[] {
    const wd = this.world!, { w, h } = wd;
    const pts: { x: number; y: number; z: number; s: number; pine: boolean; i: number }[] = [];
    const cityAt = new Set(this.view!.cities.map((c) => tileIndexOf(c.pos, w, h)));
    for (let i = 0; i < w * h; i++) {
      if (!wd.fog[i] || cityAt.has(i)) continue;
      const c = wd.cls[i];
      const forest = c === 4, sparse = (c === 2 || c === 3) && rnd(i, 3) < 0.35, hills = c === 5 && rnd(i, 4) < 0.5;
      if (!forest && !sparse && !hills) continue;
      const n = forest ? 14 : 3;
      for (let k = 0; k < n; k++) {
        const a = rnd(i, 100 + k) * Math.PI * 2, d = Math.sqrt(rnd(i, 200 + k)) * R * 0.82;
        const x = tileX(i, w) + Math.cos(a) * d, z = tileZ(i, w) + Math.sin(a) * d, y = this.heightAt(x, z);
        if (y < 0.02) continue;
        pts.push({ x, y, z, s: 0.75 + rnd(i, 300 + k) * 0.55, pine: rnd(i, 400 + k) < (wd.cls[i] === 7 ? 0.95 : 0.5), i });
      }
    }
    const out: THREE.Object3D[] = [];
    const pine = new THREE.ConeGeometry(0.045, 0.16, 7).translate(0, 0.1, 0);
    const trunk = new THREE.CylinderGeometry(0.008, 0.01, 0.04, 5).translate(0, 0.02, 0);
    const leafy = new THREE.IcosahedronGeometry(0.055, 1).scale(1, 0.85, 1).translate(0, 0.085, 0);
    const m4 = new THREE.Matrix4(), q = new THREE.Quaternion(), sc = new THREE.Vector3(), v3 = new THREE.Vector3(), grey = new THREE.Color();
    const mk = (geo: THREE.BufferGeometry, list: typeof pts, color: string, vary: number) => {
      if (!list.length) return;
      const im = new THREE.InstancedMesh(geo, new THREE.MeshStandardMaterial({ color: '#fff', roughness: 0.9, flatShading: true }), list.length);
      list.forEach((p, j) => {
        q.setFromAxisAngle(UP, p.x * 13.1);
        m4.compose(v3.set(p.x, p.y, p.z), q, sc.set(p.s, p.s, p.s)); im.setMatrixAt(j, m4);
        const c = new THREE.Color(color).multiplyScalar(1 - vary / 2 + hash2(j, 9) * vary);
        if (this.world!.fog[p.i] === 1) { const l = c.r * 0.3 + c.g * 0.59 + c.b * 0.11; c.lerp(grey.setRGB(l, l, l), 0.6); }
        im.setColorAt(j, c);
      });
      im.castShadow = true; im.receiveShadow = true; out.push(im);
    };
    const pines = pts.filter((p) => p.pine), leafs = pts.filter((p) => !p.pine);
    mk(pine, pines, '#3f5f34', 0.35); mk(trunk, pines, '#5a4632', 0.2); mk(leafy, leafs, '#5f7f3a', 0.4); mk(trunk.clone(), leafs, '#5a4632', 0.2);
    return out;
  }

  private roads(): THREE.Object3D {
    const wd = this.world!, view = this.view!, { w, h } = wd;
    const cityAt = new Set(view.cities.map((c) => tileIndexOf(c.pos, w, h)));
    const isRoad = (i: number) => wd.fog[i] > 0 && ((view.tiles[i]?.road ?? 0) > 0 || cityAt.has(i));
    const rp: number[] = [];
    for (let i = 0; i < w * h; i++) {
      if (!isRoad(i)) continue;
      for (let e = 0; e < 3; e++) {
        const n = neighborIndex(i, e, w, h);
        if (n < 0 || !isRoad(n)) continue;
        const x0 = tileX(i, w), z0 = tileZ(i, w), dx = tileX(n, w) - x0, dz = tileZ(n, w) - z0, L = Math.hypot(dx, dz);
        const px = (-dz / L) * 0.025, pz = (dx / L) * 0.025;
        for (let k = 0; k < 8; k++) {
          const P = (t: number, s: number) => { const x = x0 + dx * t + px * s, z = z0 + dz * t + pz * s; return [x, this.heightAt(x, z) + 0.008, z]; };
          const a = P(k / 8, 1), b = P(k / 8, -1), c = P((k + 1) / 8, 1), d = P((k + 1) / 8, -1);
          rp.push(...a, ...b, ...c, ...c, ...b, ...d);
        }
      }
    }
    const g = new THREE.BufferGeometry(); g.setAttribute('position', new THREE.Float32BufferAttribute(rp, 3)); g.computeVertexNormals();
    const roads = new THREE.Mesh(g, new THREE.MeshStandardMaterial({ color: '#9b8461', roughness: 1, polygonOffset: true, polygonOffsetFactor: -2 }));
    roads.receiveShadow = true;
    return roads;
  }

  /** Stand-in towns until the culture kits land: houses by population, a keep (capitals taller), walls. */
  private towns(view: PlayerView): THREE.Object3D[] {
    const wd = this.world!, { w, h } = wd;
    type P = { x: number; y: number; z: number; ry: number; s: number; o?: number; len?: number };
    const houses: P[] = [], towers: P[] = [], walls: P[] = [], fields: P[] = [];
    for (let i = 0; i < w * h; i++) {
      const t = view.tiles[i];
      if (!wd.fog[i] || !t?.improvement || !(wd.cls[i] === 2 || wd.cls[i] === 3 || wd.cls[i] === 6)) continue;
      const rot = rnd(i, 5) * Math.PI;
      for (let k = -3; k <= 3; k++) {
        const ox = -Math.sin(rot) * k * 0.06, oz = Math.cos(rot) * k * 0.06;
        const x = tileX(i, w) + ox, z = tileZ(i, w) + oz;
        fields.push({ x, y: this.heightAt(x, z) + 0.004, z, ry: rot, s: 1, o: (k + 3 + i) % 4 });
      }
    }
    for (const c of view.cities) {
      const i = tileIndexOf(c.pos, w, h);
      if (i < 0 || !wd.fog[i]) continue;
      const x0 = tileX(i, w), z0 = tileZ(i, w);
      const n = Math.min(26, 10 + c.population * 1.5) | 0;
      for (let k = 0; k < n; k++) {
        const a = k * 2.399 + rnd(i, k) * 0.6, d = 0.08 + Math.sqrt(k / n) * 0.33;
        const x = x0 + Math.cos(a) * d, z = z0 + Math.sin(a) * d;
        houses.push({ x, y: this.heightAt(x, z), z, ry: Math.round(rnd(i, 50 + k) * 2) * (Math.PI / 6) + 0.1, s: 0.8 + rnd(i, 60 + k) * 0.5 });
      }
      towers.push({ x: x0, y: this.heightAt(x0, z0), z: z0, ry: 0, s: c.is_capital ? 1.25 : 0.85, o: c.owner });
      if (c.walls) for (let e = 0; e < 6; e++) {
        const a = CORNERS[e], b = CORNERS[(e + 1) % 6], f = 0.78;
        const mx = x0 + ((a[0] + b[0]) / 2) * f, mz = z0 + ((a[1] + b[1]) / 2) * f;
        walls.push({ x: mx, y: this.heightAt(mx, mz), z: mz, ry: -Math.atan2(b[1] - a[1], b[0] - a[0]), s: 1, len: R * f });
      }
    }
    const out: THREE.Object3D[] = [];
    const m4 = new THREE.Matrix4(), q = new THREE.Quaternion(), v = new THREE.Vector3(), sv = new THREE.Vector3();
    const inst = (geo: THREE.BufferGeometry, list: P[], color: (p: P, j: number) => THREE.Color, scale?: (p: P) => THREE.Vector3, shadow = true) => {
      if (!list.length) return;
      const im = new THREE.InstancedMesh(geo, new THREE.MeshStandardMaterial({ color: '#fff', roughness: 0.85 }), list.length);
      list.forEach((p, j) => {
        q.setFromAxisAngle(UP, p.ry);
        m4.compose(v.set(p.x, p.y, p.z), q, scale ? scale(p) : sv.set(p.s, p.s, p.s));
        im.setMatrixAt(j, m4); im.setColorAt(j, color(p, j));
      });
      im.castShadow = shadow; im.receiveShadow = true; out.push(im);
    };
    const plaster = ['#e7dcc4', '#dccfae', '#efe6d2', '#d2c3a0'].map((c) => new THREE.Color(c));
    const tiles = ['#a4553a', '#b4643f', '#93492f', '#b8734e'].map((c) => new THREE.Color(c));
    const crops = ['#c9b35a', '#8fa04a', '#b7984f', '#a7b25c'].map((c) => new THREE.Color(c));
    inst(new THREE.BoxGeometry(0.068, 0.05, 0.085).translate(0, 0.025, 0), houses, (_p, j) => plaster[j % 4]);
    inst(new THREE.CylinderGeometry(0.0001, 0.056, 0.04, 4, 1).rotateY(Math.PI / 4).scale(0.85, 1, 1.05).translate(0, 0.069, 0), houses.map((p) => ({ ...p, y: p.y + (p.s - 1) * 0.03 })), (_p, j) => tiles[(j * 7) % 4]);
    inst(new THREE.CylinderGeometry(0.032, 0.038, 0.16, 8).translate(0, 0.08, 0), towers, () => new THREE.Color('#cfc3a8'));
    inst(new THREE.ConeGeometry(0.045, 0.06, 8).translate(0, 0.19, 0), towers, (p) => new THREE.Color(mapPlayerColor(p.o)).multiplyScalar(0.8));
    inst(new THREE.BoxGeometry(1, 0.05, 0.022).translate(0, 0.025, 0), walls, () => new THREE.Color('#b9ad94'), (p) => sv.set(p.len!, 1, 1));
    inst(new THREE.BoxGeometry(0.46, 0.012, 0.045), fields, (p) => crops[p.o!], undefined, false);
    return out;
  }

  // ---------------- units ----------------

  private model(slug: string): Promise<GLTF | null> {
    let p = this.models.get(slug);
    if (!p) {
      p = new GLTFLoader().loadAsync(MODEL_URL(slug)).catch(() => null);
      this.models.set(slug, p);
    }
    return p;
  }

  private texture(url: string): THREE.Texture {
    let t = this.textures.get(url);
    if (!t) {
      t = new THREE.TextureLoader().load(url, () => { this.animating = true; });
      t.colorSpace = THREE.SRGBColorSpace;
      t.anisotropy = 4;
      this.textures.set(url, t);
    }
    return t;
  }

  private unitSeq = 0;
  private env: THREE.Texture | null = null;
  /** A soft studio light for model materials only (the terrain keeps its own lighting). */
  private unitEnv(): THREE.Texture {
    if (!this.env) {
      const pm = new THREE.PMREMGenerator(this.renderer);
      this.env = pm.fromScene(new RoomEnvironment(), 0.04).texture;
      pm.dispose();
    }
    return this.env;
  }
  /** Each drawn unit's group, by unit id (the marks overlay asks where its head is). */
  private unitGroups = new Map<number, THREE.Group>();
  private squads: { fig: THREE.Object3D; lod0: THREE.Object3D[]; lod1: THREE.Object3D[]; scale: number; x: number; z: number }[] = [];
  /** Reduced motion or the Animations switch off (ui/motion.ts motionStill): models hold their pose. */
  still = false;

  /** Picks each model figure's mesh: LOD0 for the few figures nearest the view centre when the camera is close, else LOD1. */
  private updateLod() {
    const p = new THREE.Vector3();
    const near = this.dist < LOD0_DIST
      ? new Set(this.squads
        .map((s) => ({ s, d: s.fig.getWorldPosition(p).distanceToSquared(this.target) }))
        .sort((a, b) => a.d - b.d).slice(0, LOD0_FIGURES).map((x) => x.s))
      : new Set();
    // A squad of small figures reads smaller than one painted miniature, so it grows as the camera pulls back.
    const grow = 1 + 0.5 * THREE.MathUtils.smoothstep(this.dist, 3, 13);
    for (const s of this.squads) {
      s.fig.scale.setScalar(s.scale * grow);
      s.fig.position.set(s.x * grow, 0, s.z * grow);
      if (!s.lod0.length || !s.lod1.length) continue; // a model without a LOD1 draws its LOD0 everywhere
      const hi = near.has(s);
      for (const m of s.lod0) m.visible = hi;
      for (const m of s.lod1) m.visible = !hi;
    }
  }
  private buildUnits() {
    const view = this.view!, wd = this.world!;
    const seq = ++this.unitSeq;
    for (const o of [...this.units.children]) this.units.remove(o);
    this.mixers = [];
    this.squads = [];
    this.unitGroups.clear();
    const byTile = new Map<number, UnitView[]>();
    for (const u of view.units) {
      const i = tileIndexOf(u.pos, wd.w, wd.h);
      if (i < 0 || wd.fog[i] !== 2) continue;
      let list = byTile.get(i);
      if (!list) byTile.set(i, (list = []));
      list.push(u);
    }
    const cityAt = new Set(view.cities.map((c) => tileIndexOf(c.pos, wd.w, wd.h)));
    const disc = new THREE.CylinderGeometry(0.11, 0.115, 0.01, 32);
    const factionOf = new Map(view.players.map((pl) => [pl.id, pl.faction] as const));
    for (const [i, list] of byTile) {
      list.forEach((u, k) => {
        const g = new THREE.Group();
        g.userData.unit = u.id;
        this.unitGroups.set(u.id, g);
        const barbarian = isBarbarian(u.owner);
        const team = new THREE.Color(barbarian ? BARBARIAN_COLOR : mapPlayerColor(u.owner));
        const ring = new THREE.Mesh(disc, new THREE.MeshStandardMaterial({ color: team, roughness: 0.55, metalness: 0.15, transparent: true, opacity: 0.85 }));
        ring.position.y = 0.004; ring.receiveShadow = true; g.add(ring);
        const off = list.length > 1 ? (k - (list.length - 1) / 2) * 0.24 : 0;
        const x = tileX(i, wd.w) + off, z = tileZ(i, wd.w) + (cityAt.has(i) ? 0.2 : 0.05);
        g.position.set(x, Math.max(0, this.heightAt(x, z)), z);
        this.units.add(g);
        const slug = MODELS[mapUnitArtSlug(u, factionOf, barbarian)];
        if (slug) {
          void this.model(slug).then((gltf) => {
            if (!gltf || this.disposed || seq !== this.unitSeq) return;
            this.addSquad(g, gltf, team, i * 7 + k);
            this.updateLod();
            this.animating = true;
          });
        } else {
          const url = mapUnitMiniatureUrl(u, factionOf, barbarian);
          if (url) {
            const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: this.texture(url), transparent: true, alphaTest: 0.08 }));
            sp.scale.set(0.3, 0.3, 1);
            sp.center.set(0.5, 0.06);
            sp.position.y = 0.0;
            g.add(sp);
          }
        }
      });
    }
  }

  /** Unit contract: COLOR_0's red channel is the owner-colour mask (crest, shield rim, cloak trim), not a colour.
   *  Masked texels are painted neutral and are multiplied by the team colour, so their light and dark survive.
   *  One material per (source material, team), shared by every figure. */
  private teamMats = new Map<string, THREE.Material>();
  private teamMaterial(src: THREE.MeshStandardMaterial, team: THREE.Color): THREE.Material {
    const key = `${src.uuid}:${team.getHexString()}`;
    const hit = this.teamMats.get(key);
    if (hit) return hit;
    const mat = src.clone();
    mat.vertexColors = true;
    const uTeam = { value: team.clone() };
    mat.onBeforeCompile = (sh) => {
      sh.uniforms.uTeam = uTeam;
      sh.fragmentShader = sh.fragmentShader
        .replace('#include <common>', '#include <common>\nuniform vec3 uTeam;')
        // The contract's operation: masked texels are painted neutral, so multiplying by the team colour tints them and keeps their shading.
        .replace('#include <color_fragment>', 'diffuseColor.rgb *= mix(vec3(1.0), uTeam, clamp(vColor.r, 0.0, 1.0));');
    };
    mat.customProgramCacheKey = () => 'team-mask';
    this.teamMats.set(key, mat);
    return mat;
  }

  private addSquad(g: THREE.Group, gltf: GLTF, team: THREE.Color, seed: number) {
    const box = new THREE.Box3().setFromObject(gltf.scene);
    // A figure stands about as tall as a painted miniature (0.3 including its base), so models and sprites read at one size.
    const scale = 0.26 / Math.max(0.001, box.max.y - box.min.y);
    const idle = gltf.animations.find((a) => a.name === 'idle') ?? gltf.animations[0];
    const SQUAD: [number, number][] = [[0, 0.045], [-0.05, -0.03], [0.05, -0.03]];
    SQUAD.forEach(([sx, sz], f) => {
      const fig = SkeletonUtils.clone(gltf.scene) as THREE.Object3D;
      fig.scale.setScalar(scale);
      fig.position.set(sx, 0, sz);
      fig.rotation.y = DISPLAY_YAW + (hash2(seed, f) - 0.5) * 0.3;
      const lod0: THREE.Object3D[] = [], lod1: THREE.Object3D[] = [];
      fig.traverse((o) => {
        const m = o as THREE.Mesh;
        if (!m.isMesh) return;
        if (/_LOD0$/.test(m.name)) lod0.push(m);
        else if (/_LOD1$/.test(m.name)) lod1.push(m);
        m.castShadow = true; m.frustumCulled = false;
        if (Array.isArray(m.material)) return;
        if (m.geometry.getAttribute('color')) m.material = this.teamMaterial(m.material as THREE.MeshStandardMaterial, team);
        else if (/crest|aspis/i.test(m.name)) {
          // A model without the COLOR_0 mask: colour its crest and shield meshes whole.
          const mat = (m.material as THREE.MeshStandardMaterial).clone();
          mat.color = team.clone().lerp(new THREE.Color('#ffffff'), 0.1);
          m.material = mat;
        }
        // Metal (bronze, the spear head) reflects its surroundings; with nothing to reflect it renders black.
        const std = m.material as THREE.MeshStandardMaterial;
        if (std.isMeshStandardMaterial && !std.envMap) { std.envMap = this.unitEnv(); std.envMapIntensity = 0.7; }
      });
      g.add(fig);
      this.squads.push({ fig, lod0, lod1, scale, x: sx, z: sz });
      if (idle) {
        const mx = new THREE.AnimationMixer(fig);
        const a = mx.clipAction(idle); a.play(); a.time = hash2(seed, f + 9) * idle.duration;
        mx.update(0); // pose it now, so a still map shows the idle pose, not the bind pose
        this.mixers.push(mx);
      }
    });
  }

  // ---------------- overlays ----------------

  setOverlays(o: Overlays) {
    if (!this.world) return;
    const { w, h } = this.world;
    const ring = (mesh: THREE.Mesh, hex: Hex | null, lift: number, inner: number, outer: number) => {
      const i = hex ? tileIndexOf(hex, w, h) : -1;
      mesh.visible = i >= 0;
      if (i < 0) return;
      const cx = tileX(i, w), cz = tileZ(i, w);
      const at = (e: number, t: number, s: number) => {
        const a = CORNERS[e], b = CORNERS[(e + 1) % 6];
        const x = cx + (a[0] + (b[0] - a[0]) * t) * s, z = cz + (a[1] + (b[1] - a[1]) * t) * s;
        return [x, Math.max(0, this.heightAt(x, z)) + lift, z];
      };
      const p: number[] = [];
      for (let e = 0; e < 6; e++) for (let k = 0; k < 4; k++) {
        const o0 = at(e, k / 4, outer), i0 = at(e, k / 4, inner), o1 = at(e, (k + 1) / 4, outer), i1 = at(e, (k + 1) / 4, inner);
        p.push(...o0, ...i0, ...o1, ...o1, ...i0, ...i1);
      }
      mesh.geometry.dispose();
      mesh.geometry = new THREE.BufferGeometry().setAttribute('position', new THREE.Float32BufferAttribute(p, 3));
    };
    ring(this.hoverRing, o.hover, 0.03, 0.91, 0.95);
    ring(this.selRing, o.selected, 0.04, 0.895, 0.95);
    // Route: a flat gilt ribbon from tile centre to tile centre, following the ground.
    const pts = o.route.map((hx) => tileIndexOf(hx, w, h)).filter((i) => i >= 0);
    this.route.visible = pts.length > 1;
    if (pts.length > 1) {
      const rp: number[] = [];
      for (let s = 0; s + 1 < pts.length; s++) {
        const x0 = tileX(pts[s], w), z0 = tileZ(pts[s], w), dx = tileX(pts[s + 1], w) - x0, dz = tileZ(pts[s + 1], w) - z0, L = Math.hypot(dx, dz);
        const px = (-dz / L) * 0.035, pz = (dx / L) * 0.035;
        for (let k = 0; k < 8; k++) {
          const P = (t: number, sg: number) => { const x = x0 + dx * t + px * sg, z = z0 + dz * t + pz * sg; return [x, Math.max(0, this.heightAt(x, z)) + 0.03, z]; };
          const a = P(k / 8, 1), b = P(k / 8, -1), c = P((k + 1) / 8, 1), d = P((k + 1) / 8, -1);
          rp.push(...a, ...b, ...c, ...c, ...b, ...d);
        }
      }
      this.route.geometry.dispose();
      this.route.geometry = new THREE.BufferGeometry().setAttribute('position', new THREE.Float32BufferAttribute(rp, 3));
    }
    this.animating = true;
  }

  // ---------------- camera, picking, projection ----------------

  /** Radians from vertical: a working view up close, flattening toward a map view far out (Civ 6). */
  private tilt() { return THREE.MathUtils.lerp(0.78, 0.32, THREE.MathUtils.smoothstep(this.dist, 3, 26)); }

  placeCamera() {
    const t = this.tilt();
    this.camera.position.set(this.target.x, this.target.y + this.dist * Math.cos(t), this.target.z + this.dist * Math.sin(t));
    this.camera.lookAt(this.target);
    this.camera.updateMatrixWorld();
    this.sun.position.set(this.target.x - 6, 10, this.target.z - 5);
    this.sun.target.position.copy(this.target);
    const sh = Math.min(16, Math.max(5, this.dist * 1.1));
    Object.assign(this.sun.shadow.camera, { left: -sh, right: sh, top: sh, bottom: -sh });
    this.sun.shadow.camera.updateProjectionMatrix();
    this.updateLod();
    this.animating = true;
  }

  clampTarget() {
    if (!this.world) return;
    this.target.x = THREE.MathUtils.clamp(this.target.x, 0, this.world.w);
    this.target.z = THREE.MathUtils.clamp(this.target.z, 0, (this.world.h - 1) * ROW);
  }

  private ndc(sx: number, sy: number) {
    const el = this.renderer.domElement;
    return new THREE.Vector2((sx / el.clientWidth) * 2 - 1, -(sy / el.clientHeight) * 2 + 1);
  }

  /** The ground point under a screen point (CSS px in the canvas), on the sea-level plane. */
  groundAt(sx: number, sy: number): THREE.Vector3 | null {
    this.ray.setFromCamera(this.ndc(sx, sy), this.camera);
    const g = new THREE.Vector3();
    return this.ray.ray.intersectPlane(new THREE.Plane(UP, 0), g) ? g : null;
  }

  /** The tile under a screen point: the terrain surface where it is hit, else sea level. */
  pick(sx: number, sy: number): Hex | null {
    if (!this.world || !this.terrain) return null;
    this.ray.setFromCamera(this.ndc(sx, sy), this.camera);
    const hit = this.ray.intersectObject(this.terrain, false)[0];
    let p: THREE.Vector3 | null = hit && hit.point.y >= 0 ? hit.point : null;
    if (!p) p = this.groundAt(sx, sy);
    if (!p) return null;
    const { w, h } = this.world;
    if (p.x < -0.6 || p.z < -0.6 || p.x > w + 0.2 || p.z > (h - 1) * ROW + 0.6) return null;
    const i = tileAt(p.x, p.z, w, h);
    return fromOffset(i % w, Math.floor(i / w));
  }

  /** A tile's centre on screen (CSS px in the canvas) and its apparent radius. */
  project(hex: Hex, lift = 0): { x: number; y: number; radius: number } | null {
    if (!this.world) return null;
    const { w, h } = this.world;
    const i = tileIndexOf(hex, w, h);
    if (i < 0) return null;
    const x = tileX(i, w), z = tileZ(i, w), y = Math.max(0, this.heightAt(x, z)) + lift;
    const el = this.renderer.domElement;
    const v = new THREE.Vector3(x, y, z).project(this.camera);
    if (v.z > 1) return null;
    const e = new THREE.Vector3(x + R, y, z).project(this.camera);
    const W = el.clientWidth, Hh = el.clientHeight;
    const sx = (v.x + 1) / 2 * W, sy = (1 - v.y) / 2 * Hh;
    return { x: sx, y: sy, radius: Math.abs((e.x - v.x) / 2 * W) };
  }

  /** The top of a drawn unit's figure on screen (CSS px in the canvas), or null when it isn't drawn or is off camera. */
  unitHead(id: number): { x: number; y: number; radius: number } | null {
    const g = this.unitGroups.get(id);
    if (!g) return null;
    const grow = 1 + 0.5 * THREE.MathUtils.smoothstep(this.dist, 3, 13);
    const v = new THREE.Vector3(g.position.x, g.position.y + 0.3 * grow, g.position.z).project(this.camera);
    if (v.z > 1) return null;
    const e = new THREE.Vector3(g.position.x + R, g.position.y, g.position.z).project(this.camera);
    const el = this.renderer.domElement, W = el.clientWidth, Hh = el.clientHeight;
    return { x: (v.x + 1) / 2 * W, y: (1 - v.y) / 2 * Hh, radius: Math.abs((e.x - v.x) / 2 * W) };
  }

  centerOn(hex: Hex) {
    if (!this.world) return;
    const i = tileIndexOf(hex, this.world.w, this.world.h);
    if (i < 0) return;
    this.target.set(tileX(i, this.world.w), 0, tileZ(i, this.world.w));
    this.placeCamera();
  }

  /** Render one frame. Returns false when nothing moved (the caller may skip frames). */
  frame(): boolean {
    const dt = Math.min(0.05, this.clock.getDelta());
    this.uniforms.uTime.value += dt;
    if (!this.still) for (const m of this.mixers) m.update(dt);
    this.renderer.render(this.scene, this.camera);
    const was = this.animating;
    this.animating = false;
    return was;
  }
}

function disposeTree(o: THREE.Object3D) {
  o.traverse((x) => {
    const m = x as THREE.Mesh;
    m.geometry?.dispose();
    const mat = m.material as THREE.Material | THREE.Material[] | undefined;
    if (Array.isArray(mat)) mat.forEach((y) => y.dispose()); else mat?.dispose();
  });
}
