// The 3D map scene (three.js): terrain, water, forests, roads, fields, towns, borders, units and the
// map's own overlays (hover, selection, route). It reads only the fogged PlayerView. The React view
// (Map3DView.tsx) owns input and HTML; this class owns the GPU side and the camera.
import * as THREE from 'three';
import { GLTFLoader, type GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import * as SkeletonUtils from 'three/examples/jsm/utils/SkeletonUtils.js';
import { RoomEnvironment } from 'three/examples/jsm/environments/RoomEnvironment.js';
import { buildCityKit, districtTiles } from './cityKit';
import { removeRockShards } from './removeRockShards';
import type { Hex, PlayerView, UnitView } from '../types';
import { fromOffset, toOffset } from '../hex';
import { neighborIndex } from '../map/grid';
import { ColdField } from './coldField';
import { RebuildTimer } from './rebuildPerf';
import { viewDelta } from './rebuildDelta';
import { refreshFogTiles } from './incrementalFog';
import { refreshTerrainTiles } from './incrementalTerrain';
import { refreshReliefTiles } from './incrementalRelief';
import { terrainNormals, refreshTerrainNormals } from './terrainNormals';
import { makeEdges, makeBorders } from './staticEdges';
import { refreshMemory } from './refreshMemory';
import { mapPlayerColor } from '../map/style';
import { mapUnitArtSlug, mapUnitMiniatureUrl } from '../unitArt';
import { BARBARIAN_COLOR } from '../theme';
import { createFlora, floraClearings, shadeRelief, FloraCache, type FloraRender } from './flora';
import { isBarbarian } from '../barbarians';
import { fadeOwn, ownMaterials } from './ownMaterials';
import { ARC, DEATH_MS, fxUnits, type CombatPlan, type ShotKind } from '../map/combatFx';
import { terrainMaterial } from './terrainMaterial';
import { addMemoryWash, remembered } from './memoryWash';
import {
  CORNERS, PX, R, ROW, hash2, heightAt, tileAt, tileX, tileZ, worldFrom,
  type Field, type World,
} from './world';
import { MapWater } from './water';
import { SELECT_HALO, SELECT_INK, selectionBreath, selectionBurst } from '../map/selectionMark';
import { MapAtmosphere, MapPost } from './post';

const PARCH = new THREE.Color('#d9c9a3');
const UP = new THREE.Vector3(0, 1, 0);
/** Lunges, recoils and flinches read larger on the 3D figures than on the flat tokens (a fifth of a hex was hard to see at working zoom). */
const LUNGE_3D = 1.6;
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
  /** What is selected (a unit or city id): a change plays the selection burst, even on the same tile. */
  selectedKey?: string;
  /** The planned route of the selected unit, tile by tile from its position. */
  route: Hex[];
}

/** A hex's centre in world units (the axial maths the 2D map uses, scaled to the 3D map). */
const hexCentre = (h: Hex) => ({ x: Math.sqrt(3) * (h.q + h.r / 2) * (36 / PX), z: 1.5 * h.r * (36 / PX) });

const tileIndexOf = (h: Hex, w: number, hgt: number) => {
  const [c, r] = toOffset(h);
  return c < 0 || r < 0 || c >= w || r >= hgt ? -1 : r * w + c;
};

/** Figures turn a little toward their spear side so the weapon reads from the map camera (units thread, 2026-11-25). */
const DISPLAY_YAW = 0.5;
/** One figure's mixer: `base` is the looping clip it rests in (idle, or walk while travelling). */
type Fig = { mx: THREE.AnimationMixer; idle: THREE.AnimationAction; base: THREE.AnimationAction; clips: THREE.AnimationClip[] };

export class MapScene {
  readonly renderer: THREE.WebGLRenderer;
  readonly scene = new THREE.Scene();
  readonly camera = new THREE.PerspectiveCamera(32, 1, 0.05, 200);
  /** Camera target on the ground (world units) and distance from it. */
  readonly target = new THREE.Vector3();
  dist = 13;
  private sun = new THREE.DirectionalLight('#fff1d6', 2.4);
  private water: MapWater;
  private atmosphere: MapAtmosphere;
  private post: MapPost;
  private uniforms = { uTime: { value: 0 }, uParch: { value: PARCH } };
  private world: World | null = null;
  private field: Field | null = null;
  private baseField: Field | null = null;
  private terrain: THREE.Mesh | null = null;
  private static_ = new THREE.Group();
  private roadMesh: THREE.Object3D | null = null;
  private cityMeshes: THREE.Object3D[] = [];
  private dynamic = new THREE.Group();
  private flora: FloraRender | null = null;
  private floraCache = new FloraCache();
  private floraKey = '';
  /** Time spent rebuilding this lane's procedural meshes, separate from the existing terrain rebuild. */
  floraBuildMs = 0;
  lastBuildProfile: Record<string, number> = {};
  private staticProfile: Record<string, number> = {};
  private coldField = new ColdField((field, base, workerMs, complete) => this.finishCold(field, base, workerMs, complete));
  private borderMesh: THREE.Mesh | null = null;
  private lensInput: { index: number; color: string; alpha: number }[] = [];
  private overlayInput: Overlays | null = null;
  private units = new THREE.Group();
  private hoverRing: THREE.Mesh;
  private selRing: THREE.Mesh;
  /** The selection mark's dark halo, its breathing wash, and the ring that bursts out when the selection changes (map/selectionMark.ts). */
  private selHalo: THREE.Mesh;
  private selFill: THREE.Mesh;
  private selBurst: THREE.Mesh;
  private selKey = '';
  private selAt = -Infinity;
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
    this.atmosphere = new MapAtmosphere(this.scene, this.renderer, this.sun, PARCH);
    this.water = new MapWater(this.scene, this.uniforms);
    this.post = new MapPost(this.renderer);
    const sun = this.sun;
    sun.castShadow = true;
    sun.shadow.mapSize.set(2048, 2048);
    sun.shadow.camera.near = 0.5; sun.shadow.camera.far = 40;
    sun.shadow.bias = -0.0006; sun.shadow.normalBias = 0.02;
    this.scene.add(sun, sun.target, this.static_, this.dynamic, this.units);
    // Rings are thin ground-hugging bands, not GL lines (WebGL draws lines 1 px wide whatever the zoom).
    this.hoverRing = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ color: '#fbf1cf', transparent: true, opacity: 0.85, depthTest: false, side: THREE.DoubleSide }));
    this.selRing = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ color: SELECT_INK, transparent: true, depthTest: false, side: THREE.DoubleSide }));
    this.selHalo = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ color: SELECT_HALO, transparent: true, opacity: 0.85, depthTest: false, side: THREE.DoubleSide }));
    this.selFill = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ color: SELECT_INK, transparent: true, opacity: 0.2, depthTest: false, depthWrite: false, side: THREE.DoubleSide }));
    // The burst: a flat hexagonal band round the origin, placed on the tile and scaled out.
    const band: number[] = [];
    for (let e = 0; e < 6; e++) {
      const a = CORNERS[e], b = CORNERS[(e + 1) % 6], k = 0.9;
      band.push(a[0], 0, a[1], a[0] * k, 0, a[1] * k, b[0], 0, b[1], b[0], 0, b[1], a[0] * k, 0, a[1] * k, b[0] * k, 0, b[1] * k);
    }
    this.selBurst = new THREE.Mesh(new THREE.BufferGeometry().setAttribute('position', new THREE.Float32BufferAttribute(band, 3)),
      new THREE.MeshBasicMaterial({ color: SELECT_INK, transparent: true, depthTest: false, depthWrite: false, side: THREE.DoubleSide }));
    this.selFill.renderOrder = 5; this.selHalo.renderOrder = 5;
    for (const o of [this.selFill, this.selHalo, this.selBurst]) { o.visible = false; this.scene.add(o); }
    this.selBurst.renderOrder = 6;
    this.route = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial({ color: '#f3dc8a', transparent: true, opacity: 0.9, depthTest: false, side: THREE.DoubleSide }));
    for (const o of [this.hoverRing, this.selRing, this.route]) { o.renderOrder = 6; o.visible = false; this.scene.add(o); }
  }

  dispose() {
    this.disposed = true;
    this.coldField.dispose();
    this.scene.traverse((o) => {
      const m = o as THREE.Mesh;
      m.geometry?.dispose();
      const mat = m.material as THREE.Material | THREE.Material[] | undefined;
      if (Array.isArray(mat)) mat.forEach((x) => x.dispose()); else mat?.dispose();
    });
    for (const t of this.textures.values()) t.dispose();
    this.env?.dispose();
    this.water.dispose();
    this.atmosphere.dispose();
    this.post.dispose();
    this.renderer.dispose();
  }

  resize(w: number, h: number) {
    this.renderer.setSize(w, h, false);
    this.post.resize(w, h);
    this.camera.aspect = w / Math.max(1, h);
    this.camera.updateProjectionMatrix();
    this.animating = true;
  }

  // ---------------- world ----------------

  forceRebuild() {
    this.coldField.cancel();
    this.world = null;
    this.field = null;
    this.baseField = null;
    this.view = null;
    this.floraKey = '';
    this.floraCache.clear();
  }

  whenReady(): Promise<number> { return this.coldField.whenReady(); }

  private finishCold(field: Field, base: Field, workerMs: number, complete: () => number | null): void {
    if (this.disposed) return;
    const applyStart = performance.now();
    this.field = field;
    this.baseField = base;
    base.fog = field.fog;
    base.shore = field.shore;
    base.materials = field.materials;
    const timer = new RebuildTimer();
    this.buildTerrain(); timer.mark('terrainMs');
    this.buildStatic(true); timer.mark('staticMs');
    this.lastBuildProfile.floraMs = this.floraBuildMs;
    this.buildUnits(); timer.mark('unitsMs');
    if (this.lensInput.length) this.setLensWash(this.lensInput);
    if (this.overlayInput) this.setOverlays(this.overlayInput);
    this.animating = true;
    Object.assign(this.lastBuildProfile, timer.spans, this.staticProfile, {
      workerMs: Math.round(workerMs * 10) / 10,
      applyMs: Math.round((performance.now() - applyStart) * 10) / 10,
    });
    requestAnimationFrame(() => {
      if (this.disposed) return;
      this.frame();
      const wallMs = complete();
      if (wallMs !== null) this.lastBuildProfile.wallToVisibleMs = wallMs;
    });
  }

  /** Build or update everything from the fogged PlayerView. */
  setView(view: PlayerView) {
    if (view === this.view && this.coldField.pending) return;
    if (view === this.view && this.world && this.field) {
      this.lastBuildProfile = { sameView: 0 };
      return;
    }
    const timer = new RebuildTimer();
    const next = worldFrom(view);
    const previous = this.world;
    const priorView = this.view;
    timer.mark('worldMs');
    const sameDimensions = this.world?.w === next.w && this.world?.h === next.h;
    const changedTerrain: number[] = [];
    if (sameDimensions) for (let i = 0; i < next.cls.length; i++) if (this.world!.cls[i] !== next.cls[i]) changedTerrain.push(i);
    const rebuild = !this.world || !this.field || !sameDimensions || changedTerrain.length > 16;
    if (rebuild) this.floraCache.clear();
    else if (changedTerrain.length) this.floraCache.invalidate(next, changedTerrain);
    const delta = !rebuild && this.view ? viewDelta(this.view, view, this.world!, next) : null;
    this.world = next;
    this.view = view;
    if (rebuild) {
      this.field = null;
      this.lastBuildProfile = timer.spans;
      this.coldField.request(next);
      timer.mark('dispatchMs');
      this.lastBuildProfile.dispatchMs = timer.spans.dispatchMs;
      return;
    }
    if (changedTerrain.length) {
      // Resample the changed base vertices, then apply connected ridges to that base.
      // Re-sculpting an already sculpted field would double the mountain height.
      refreshTerrainTiles(next, this.baseField!, changedTerrain);
      const f = this.field!, base = this.baseField!;
      const dirty = refreshReliefTiles(next, base, f, changedTerrain);
      refreshTerrainNormals(f.pos, (this.terrain!.geometry.getAttribute('normal') as THREE.BufferAttribute).array as Float32Array, f.grid.nx, f.grid.nz, dirty);
      for (const name of ['position', 'color', 'normal', 'shorev', 'terrainWeights']) (this.terrain!.geometry.getAttribute(name) as THREE.BufferAttribute).needsUpdate = true;
      this.water.update(f, dirty);
    }
    if (delta!.fogTiles.length) {
      const dirty = refreshFogTiles(next, this.field!, delta!.fogTiles);
      (this.terrain!.geometry.getAttribute('fogv') as THREE.BufferAttribute).needsUpdate = true;
      this.water.update(this.field!, dirty);
      refreshMemory(this.static_, next);
    }
    timer.mark('fieldMs');
    this.staticProfile = {};
    this.floraBuildMs = 0;
    if (changedTerrain.length || delta!.staticChanged || delta!.fogTiles.some((i) => !previous!.fog[i] !== !next.fog[i])) {
      const revealed = delta!.fogTiles.filter((i) => !previous!.fog[i] !== !next.fog[i]);
      const roadNear = revealed.some((i) => (view.tiles[i]?.road ?? 0) > 0 || view.cities.some((c) => tileIndexOf(c.pos, next.w, next.h) === i));
      const cityNear = view.cities.some((c, index) => {
        const j = tileIndexOf(c.pos, next.w, next.h);
        return revealed.includes(j) || (priorView && priorView.cities[index]
          && districtTiles(priorView, previous!, priorView.cities[index]).join(',') !== districtTiles(view, next, c).join(','));
      });
      this.buildStatic(false, !delta!.staticChanged && !roadNear, !delta!.staticChanged && !cityNear);
    }
    else if (delta!.ownersChanged) {
      this.rebuildBorders();
      if (view.cities.some((c) => c.population >= 4 && this.cityOwnerChangedNear(c, previous!, next))) this.rebuildCities();
    }
    timer.mark('staticMs');
    this.lastBuildProfile = { ...timer.spans, ...this.staticProfile };
    this.lastBuildProfile.floraMs = this.floraBuildMs;
    this.buildUnits(); timer.mark('unitsMs');
    this.lastBuildProfile.unitsMs = timer.spans.unitsMs;
    this.animating = true;
  }

  private buildTerrain() {
    const f = this.field!;
    const { nx, nz } = f.grid;
    if (this.terrain) { this.scene.remove(this.terrain); this.terrain.geometry.dispose(); (this.terrain.material as THREE.Material).dispose(); }
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
    g.setAttribute('shorev', new THREE.Uint8BufferAttribute(f.shore, 1, true));
    g.setAttribute('terrainWeights', new THREE.Uint8BufferAttribute(f.materials, 4, true));
    g.setIndex(new THREE.BufferAttribute(idx, 1));
    g.setAttribute('normal', new THREE.BufferAttribute(terrainNormals(f.pos, nx, nz), 3));
    const mat = terrainMaterial(PARCH);
    const compileTerrain = mat.onBeforeCompile;
    mat.onBeforeCompile = (shader, renderer) => {
      compileTerrain(shader, renderer);
      shader.fragmentShader = shadeRelief(shader.fragmentShader);
    };
    this.terrain = new THREE.Mesh(g, mat);
    this.terrain.receiveShadow = true;
    this.terrain.castShadow = true;
    this.scene.add(this.terrain);
    this.water.rebuild(f);
  }

  private cityOwnerChangedNear(city: PlayerView['cities'][number], before: World, after: World): boolean {
    const i = tileIndexOf(city.pos, after.w, after.h);
    if (i < 0) return false;
    return Array.from({ length: 6 }, (_, e) => neighborIndex(i, e, after.w, after.h))
      .some((j) => j >= 0 && before.owner[j] !== after.owner[j]);
  }

  private rebuildCities() {
    for (const mesh of this.cityMeshes) { this.static_.remove(mesh); disposeTree(mesh); }
    this.cityMeshes = buildCityKit(this.view!, this.world!, (x, z) => this.heightAt(x, z));
    this.static_.add(...this.cityMeshes);
  }

  heightAt(x: number, z: number) { return this.field ? heightAt(this.field, x, z) : 0; }

  /** Grid, borders, forests, roads, fields and towns: everything that changes only when a turn resolves. */
  private buildStatic(rebuild: boolean, keepRoad = false, keepCities = false) {
    const started = performance.now();
    const wd = this.world!, view = this.view!;
    const clear = floraClearings(view);
    // The forest depends on known tiles and city clearings. Unit clearings update its
    // matrices in place, so an ordinary view refresh does not rebuild the instances.
    const floraKey = `${Array.from(wd.fog, (f) => Number(f > 0)).join('')}/${wd.cls.join(',')}/${[...clear.cityTiles].sort((a, b) => a - b).join(',')}`;
    const reuseFlora = !rebuild && this.flora !== null && this.floraKey === floraKey;
    const reuseRoad = !rebuild && keepRoad && this.roadMesh !== null;
    const reuseCities = !rebuild && keepCities && this.cityMeshes.length > 0;
    const retained = new Set<THREE.Object3D>([
      ...(reuseFlora ? [this.flora!.group] : []),
      ...(reuseRoad ? [this.roadMesh!] : []),
      ...(reuseCities ? this.cityMeshes : []),
    ]);
    for (const o of [...this.static_.children]) {
      this.static_.remove(o);
      if (!retained.has(o)) disposeTree(o);
    }
    const H = (x: number, z: number) => Math.max(0, this.heightAt(x, z));
    const { grid, borders } = makeEdges(wd, (x, z) => this.heightAt(x, z));
    this.borderMesh = borders;
    this.static_.add(grid, borders);
    const edgesMs = performance.now() - started;
    if (!reuseFlora) {
      const floraStart = performance.now();
      this.flora = createFlora(wd, H, clear, this.floraCache);
      this.floraKey = floraKey;
      this.floraBuildMs = performance.now() - floraStart;
    } else {
      this.flora!.clearUnits(clear.units);
      this.floraBuildMs = 0;
    }
    removeRockShards(this.flora!.group);
    this.flora!.update(this.uniforms.uTime.value, this.still, this.dist);
    const floraMs = performance.now() - started - edgesMs;
    this.roadMesh = reuseRoad ? this.roadMesh! : this.roads();
    const roadsMs = performance.now() - started - edgesMs - floraMs;
    this.cityMeshes = reuseCities ? this.cityMeshes : buildCityKit(view, wd, (x, z) => this.heightAt(x, z));
    this.static_.add(this.flora!.group, this.roadMesh, ...this.cityMeshes);
    this.staticProfile = { edgesMs, floraMs, roadsMs, citiesMs: performance.now() - started - edgesMs - floraMs - roadsMs,
      ...(!reuseFlora ? {
        floraPlacementsMs: this.flora!.profile.placementsMs,
        floraInstancesMs: this.flora!.profile.instancesMs,
        floraFloorMs: this.flora!.profile.floorMs,
        floraRocksMs: this.flora!.profile.rocksMs,
      } : {}) };

  }

  private rebuildBorders() {
    if (this.borderMesh) { this.static_.remove(this.borderMesh); disposeTree(this.borderMesh); }
    this.borderMesh = makeBorders(this.world!, (x, z) => this.heightAt(x, z));
    this.static_.add(this.borderMesh);
  }

  private roads(): THREE.Object3D {
    const wd = this.world!, view = this.view!, { w, h } = wd;
    const cityAt = new Set(view.cities.map((c) => tileIndexOf(c.pos, w, h)));
    const isRoad = (i: number) => wd.fog[i] > 0 && ((view.tiles[i]?.road ?? 0) > 0 || cityAt.has(i));
    const rp: number[] = [], rm: number[] = [], from: number[] = [], to: number[] = [], blend: number[] = [];
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
          const memory = remembered(wd.fog[i]) * (1 - k / 8) + remembered(wd.fog[n]) * (k / 8);
          const nextMemory = remembered(wd.fog[i]) * (1 - (k + 1) / 8) + remembered(wd.fog[n]) * ((k + 1) / 8);
          rm.push(memory, memory, nextMemory, nextMemory, memory, nextMemory);
          from.push(i, i, i, i, i, i);
          to.push(n, n, n, n, n, n);
          blend.push(k / 8, k / 8, (k + 1) / 8, (k + 1) / 8, k / 8, (k + 1) / 8);
        }
      }
    }
    const g = new THREE.BufferGeometry(); g.setAttribute('position', new THREE.Float32BufferAttribute(rp, 3)); g.setAttribute('vertexMemory', new THREE.Float32BufferAttribute(rm, 1)); g.computeVertexNormals();
    const roads = new THREE.Mesh(g, new THREE.MeshStandardMaterial({ color: '#9b8461', roughness: 1, polygonOffset: true, polygonOffsetFactor: -2 }));
    roads.userData.memoryRoad = { from: Int16Array.from(from), to: Int16Array.from(to), blend: Float32Array.from(blend) };
    roads.material.onBeforeCompile = (shader) => addMemoryWash(shader, 'vertexMemory');
    roads.material.customProgramCacheKey = () => 'map-roads-memory-v1';
    roads.receiveShadow = true;
    return roads;
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
    this.settling.clear();
    const byTile = new Map<number, UnitView[]>();
    for (const u of view.units) {
      const i = tileIndexOf(u.pos, wd.w, wd.h);
      if (i < 0 || wd.fog[i] !== 2) continue;
      let list = byTile.get(i);
      if (!list) byTile.set(i, (list = []));
      list.push(u);
    }
    const cityAt = new Set(view.cities.map((c) => tileIndexOf(c.pos, wd.w, wd.h)));
    for (const [i, list] of byTile) {
      list.forEach((u, k) => {
        const off = list.length > 1 ? (k - (list.length - 1) / 2) * 0.24 : 0;
        const x = tileX(i, wd.w) + off, z = tileZ(i, wd.w) + (cityAt.has(i) ? 0.2 : 0.05);
        const g = this.unitFigure(u, i * 7 + k, seq);
        g.position.set(x, Math.max(0, this.heightAt(x, z)), z);
        g.userData.home = g.position.clone();
        this.unitGroups.set(u.id, g);
        this.units.add(g);
      });
    }
  }

  private disc = new THREE.CylinderGeometry(0.11, 0.115, 0.01, 32);
  /** One unit's figure on its owner disc: the unit's 3D model as a squad of three, else its painted miniature. */
  private unitFigure(u: UnitView, seed: number, seq: number): THREE.Group {
    const view = this.view!;
    const g = new THREE.Group();
    g.userData.unit = u.id;
    const barbarian = isBarbarian(u.owner);
    const team = new THREE.Color(barbarian ? BARBARIAN_COLOR : mapPlayerColor(u.owner));
    const ring = new THREE.Mesh(this.disc, new THREE.MeshStandardMaterial({ color: team, roughness: 0.55, metalness: 0.15, transparent: true, opacity: 0.85 }));
    ring.position.y = 0.004; ring.receiveShadow = true; g.add(ring);
    const factionOf = new Map(view.players.map((pl) => [pl.id, pl.faction] as const));
    const slug = MODELS[mapUnitArtSlug(u, factionOf, barbarian)];
    if (slug) {
      void this.model(slug).then((gltf) => {
        if (!gltf || this.disposed || seq !== this.unitSeq) return;
        this.addSquad(g, gltf, team, seed);
        this.updateLod();
        this.animating = true;
      });
    } else {
      const url = mapUnitMiniatureUrl(u, factionOf, barbarian);
      if (url) {
        const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: this.texture(url), transparent: true, alphaTest: 0.08 }));
        sp.scale.set(0.3, 0.3, 1);
        sp.center.set(0.5, 0.06);
        g.add(sp);
      }
    }
    return g;
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
        const figs = (g.userData.figs ??= []) as Fig[];
        figs.push({ mx, idle: a, base: a, clips: gltf.animations });
      }
    });
  }

  // ---------------- overlays ----------------

  private lensMesh: THREE.Mesh | null = null;
  /** The active lens's wash on the ground: one hex per tile, following the terrain, under units and banners. */
  setLensWash(tiles: { index: number; color: string; alpha: number }[]) {
    this.lensInput = tiles;
    if (this.lensMesh) { this.scene.remove(this.lensMesh); this.lensMesh.geometry.dispose(); (this.lensMesh.material as THREE.Material).dispose(); this.lensMesh = null; }
    if (!this.world || !tiles.length) { this.animating = true; return; }
    const { w } = this.world;
    const pos: number[] = [], col: number[] = [];
    const c = new THREE.Color();
    const S = 0.93, RINGS = 3;
    for (const t of tiles) {
      const cx = tileX(t.index, w), cz = tileZ(t.index, w);
      c.set(t.color);
      // Each of the six triangles is split into rings so the wash follows hills instead of cutting through them.
      for (let e = 0; e < 6; e++) {
        const a = CORNERS[e], b = CORNERS[(e + 1) % 6];
        const P = (u: number, v: number) => {
          const x = cx + (a[0] * u + b[0] * v) * S, z = cz + (a[1] * u + b[1] * v) * S;
          return [x, Math.max(0, this.heightAt(x, z)) + 0.014, z];
        };
        for (let i = 0; i < RINGS; i++) for (let j = 0; j < RINGS - i; j++) {
          const f = (k: number) => k / RINGS;
          const tri = (p: number[][]) => { for (const q of p) { pos.push(...q); col.push(c.r, c.g, c.b, t.alpha); } };
          tri([P(f(i), f(j)), P(f(i + 1), f(j)), P(f(i), f(j + 1))]);
          if (i + j + 1 < RINGS) tri([P(f(i + 1), f(j)), P(f(i + 1), f(j + 1)), P(f(i), f(j + 1))]);
        }
      }
    }
    const g = new THREE.BufferGeometry();
    g.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
    g.setAttribute('color', new THREE.Float32BufferAttribute(col, 4));
    this.lensMesh = new THREE.Mesh(g, new THREE.MeshBasicMaterial({ vertexColors: true, transparent: true, depthWrite: false, side: THREE.DoubleSide }));
    this.lensMesh.renderOrder = 3;
    this.scene.add(this.lensMesh);
    this.animating = true;
  }

  /** The apparent hex radius in px at the camera's target. */
  hexPx(): number {
    const v = this.target.clone().project(this.camera), e = this.target.clone().add(new THREE.Vector3(R, 0, 0)).project(this.camera);
    return Math.abs((e.x - v.x) / 2 * this.renderer.domElement.clientWidth);
  }

  /** A 2D-map world point (px) on screen, on the 3D ground: lets the 2D map's own drawing code draw on the 3D map. */
  projectFlat(x2: number, y2: number, lift = 0.01): { x: number; y: number } | null {
    if (!this.world) return null;
    const x = x2 / PX, z = y2 / PX;
    const v = new THREE.Vector3(x, Math.max(0, this.heightAt(x, z)) + lift, z).project(this.camera);
    if (v.z > 1) return null;
    const el = this.renderer.domElement;
    return { x: (v.x + 1) / 2 * el.clientWidth, y: (1 - v.y) / 2 * el.clientHeight };
  }

  /** A tile's centre on screen by tile index (the lens chips and badges use it). */
  projectIndex(i: number, lift = 0) {
    if (!this.world) return null;
    const { w } = this.world;
    const c = Math.floor(i % w), r = Math.floor(i / w);
    return this.project(fromOffset(c, r), lift);
  }

  setOverlays(o: Overlays) {
    this.overlayInput = o;
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
    ring(this.selHalo, o.selected, 0.038, 0.85, 0.985);
    ring(this.selRing, o.selected, 0.04, 0.885, 0.955);
    ring(this.selFill, o.selected, 0.036, 0, 0.87);
    const key = o.selected ? o.selectedKey ?? `${o.selected.q},${o.selected.r}` : '';
    if (key !== this.selKey) { this.selKey = key; this.selAt = key ? performance.now() : -Infinity; }
    const si = o.selected ? tileIndexOf(o.selected, w, h) : -1;
    if (si >= 0) {
      const cx = tileX(si, w), cz = tileZ(si, w);
      this.selBurst.position.set(cx, Math.max(0, this.heightAt(cx, cz)) + 0.05, cz);
    }
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
    this.sun.position.set(this.target.x - 8, 8, this.target.z - 6);
    this.sun.target.position.copy(this.target);
    const sh = Math.min(16, Math.max(3.8, this.dist * 0.92));
    Object.assign(this.sun.shadow.camera, { left: -sh, right: sh, top: sh, bottom: -sh });
    this.sun.shadow.camera.updateProjectionMatrix();
    this.updateLod();
    this.flora?.update(this.uniforms.uTime.value, this.still, this.dist);
    this.atmosphere.update(this.dist);
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

  /** Where a unit's figure stands on screen (the centre of its base disc). */
  unitFoot(id: number): { x: number; y: number; radius: number } | null {
    const g = this.unitGroups.get(id);
    if (!g) return null;
    const v = g.position.clone().project(this.camera);
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

  // ---------------- turn playback: moves, the fighting (map/combatFx.ts plan), the fallen ----------------

  private anim: { start: number; duration: number; paths: Map<number, Hex[]>; plan: CombatPlan | null; ghosts: Map<number, THREE.Group>; fired: Set<string>; visualEnd: number; last?: number } | null = null;
  private shotPool: THREE.Mesh[] = [];
  /** Units turning back to their resting facing after a move. */
  private settling = new Set<THREE.Object3D>();

  /** Play the turn just resolved: the same plan and clock as the 2D map. Null (or a still map) shows units where they ended. */
  setAnim(a: { start: number; duration: number; paths: Map<number, Hex[]>; plan?: CombatPlan | null } | null) {
    this.endAnim();
    if (a && !this.still) this.settling.clear(); // the new playback steers every facing itself
    if (!a || this.still || !this.world) return;
    const ghosts = new Map<number, THREE.Group>();
    for (const [id, u] of a.plan?.ghosts ?? []) {
      const g = this.unitFigure(u, id * 13, this.unitSeq);
      const i = tileIndexOf(u.pos, this.world.w, this.world.h);
      if (i < 0) continue;
      g.position.set(tileX(i, this.world.w), Math.max(0, this.heightAt(tileX(i, this.world.w), tileZ(i, this.world.w))), tileZ(i, this.world.w) + 0.05);
      ghosts.set(id, g);
      this.units.add(g);
      // Its own transparent materials from the start: the fade never touches a material live figures share, and never
      // recompiles a shader mid-playback. A squad still loading is given its own when the fall begins.
      ownMaterials(g);
    }
    // The figures are done once the last move, blow, missile and fall are (the plan's tail is for the 2D map's damage cues).
    const p = a.plan;
    const visualEnd = p ? Math.max(0, ...[...p.moves.values()].flat().map((m) => m.t1), ...p.clashes.map((c) => c.t + p.stepMs),
      ...p.shots.map((x) => x.launch + x.flight + 120), ...p.deaths.map((d) => d.t + DEATH_MS)) + 150 : 0;
    this.anim = { start: a.start, duration: a.duration, paths: a.paths, plan: p ?? null, ghosts, fired: new Set(), visualEnd };
  }

  private endAnim() {
    const a = this.anim;
    if (!a) return;
    for (const g of a.ghosts.values()) {
      this.units.remove(g);
      // The ghost's own material copies go with it (the shared ones stay with the live figures).
      g.traverse((o) => { const m = (o as THREE.Mesh).material; for (const x of Array.isArray(m) ? m : m ? [m] : []) if (x.userData.ownMaterial) x.dispose(); });
    }
    for (const g of this.unitGroups.values()) {
      const home = g.userData.home as THREE.Vector3 | undefined;
      if (home) g.position.copy(home);
      // Upright again, but the facing eases back to rest in frame() rather than snapping.
      g.rotation.x = 0; g.rotation.z = 0;
      if (Math.abs(g.rotation.y) > 1e-3) this.settling.add(g); else g.rotation.y = 0;
      this.setWalk(g, false);
      delete g.userData.prevBase; delete g.userData.walkUntil; delete g.userData.heading;
    }
    for (const m of this.shotPool) m.visible = false;
    this.anim = null;
    this.animating = true;
  }

  /** Play a clip once on every figure of a unit, then back to idle (death holds its last pose). */
  private playClip(g: THREE.Object3D | undefined, name: 'attack' | 'hit' | 'death') {
    for (const f of (g?.userData.figs ?? []) as Fig[]) {
      const clip = f.clips.find((c) => c.name === name);
      if (!clip) continue;
      const a = f.mx.clipAction(clip);
      a.reset(); a.setLoop(THREE.LoopOnce, 1); a.clampWhenFinished = true;
      a.crossFadeFrom(f.base, 0.08, false); a.play();
      if (name !== 'death') {
        const back = (e: { action: THREE.AnimationAction }) => {
          if (e.action !== a) return;
          f.mx.removeEventListener('finished', back);
          f.base.reset(); f.base.play(); f.base.crossFadeFrom(a, 0.15, false);
        };
        f.mx.addEventListener('finished', back);
      }
    }
  }

  /** Walk while a unit travels, idle when it stops: the base clip that attack and hit return to. */
  private setWalk(g: THREE.Object3D, on: boolean) {
    if (!!g.userData.walking === on) return;
    g.userData.walking = on;
    for (const f of (g.userData.figs ?? []) as Fig[]) {
      const clip = f.clips.find((c) => c.name === 'walk');
      if (!clip) continue;
      const walk = f.mx.clipAction(clip);
      const next = on ? walk : f.idle, prev = f.base;
      if (next === prev) continue;
      next.reset(); next.setLoop(THREE.LoopRepeat, Infinity);
      if (on) next.time = Math.random() * clip.duration; // figures of a squad out of step
      next.play(); next.crossFadeFrom(prev, 0.15, false);
      f.base = next;
    }
  }

  /** Track a unit's travel (not its lunges): walk and face the way it goes while it moves, then settle back. */
  private stepWalk(g: THREE.Object3D, x: number, z: number, t: number, dt: number) {
    const prev = g.userData.prevBase as { x: number; z: number } | undefined;
    g.userData.prevBase = { x, z };
    if (prev) {
      const dx = x - prev.x, dz = z - prev.z;
      if (Math.hypot(dx, dz) > 0.0004) { g.userData.walkUntil = t + 90; g.userData.heading = Math.atan2(dx, dz) - DISPLAY_YAW; }
    }
    const on = t < ((g.userData.walkUntil as number | undefined) ?? -Infinity);
    this.setWalk(g, on);
    const want = on ? ((g.userData.heading as number | undefined) ?? 0) : 0;
    let d = want - g.rotation.y;
    d = Math.atan2(Math.sin(d), Math.cos(d));
    g.rotation.y += d * Math.min(1, dt * 12);
  }

  private shotMesh(k: number, kind: ShotKind): THREE.Mesh {
    let m = this.shotPool[k];
    if (!m) {
      m = new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshStandardMaterial({ color: '#6e5536', roughness: 0.7 }));
      m.castShadow = true;
      this.shotPool[k] = m;
      this.scene.add(m);
    }
    const want = kind === 'arrows' || kind === 'bolts' ? 'shaft' : 'ball';
    if (m.userData.shape !== want) {
      m.geometry.dispose();
      // A tapered shaft, point forward: thin and short beside the figures (a stick read as a pole at close zoom).
      m.geometry = want === 'shaft' ? new THREE.CylinderGeometry(0.0006, 0.0018, 0.042, 5).rotateX(Math.PI / 2) : new THREE.SphereGeometry(kind === 'stone' ? 0.016 : 0.01, 8, 6);
      m.userData.shape = want;
    }
    m.visible = true;
    return m;
  }

  private stepAnim(now: number) {
    const a = this.anim;
    if (!a || !this.world) return;
    const t = now - a.start;
    const dt = Math.min(0.05, Math.max(0, (now - (a.last ?? now)) / 1000));
    a.last = now;
    const plan = a.plan;
    if (plan ? t > a.visualEnd : t > a.duration + 50) { this.endAnim(); return; }
    this.animating = true;
    const place = (g: THREE.Object3D, x: number, z: number) => g.position.set(x, Math.max(0, this.heightAt(x, z)), z);
    if (!plan) {
      // A plain glide (no fighting): every moving unit eases along its path.
      const k = Math.min(1, Math.max(0, t / a.duration)), e = k < 0.5 ? 2 * k * k : 1 - Math.pow(-2 * k + 2, 2) / 2;
      for (const [id, path] of a.paths) {
        const g = this.unitGroups.get(id);
        if (!g || path.length < 2) continue;
        const f = e * (path.length - 1), j = Math.min(path.length - 2, Math.floor(f)), r = f - j;
        const p0 = hexCentre(path[j]), p1 = hexCentre(path[j + 1]);
        const x = p0.x + (p1.x - p0.x) * r, z = p0.z + (p1.z - p0.z) * r + 0.05;
        place(g, x, z);
        this.stepWalk(g, x, z, t, dt);
      }
      return;
    }
    const fx = fxUnits(plan, t);
    for (const [id, p] of fx.pos) {
      const g = this.unitGroups.get(id) ?? a.ghosts.get(id);
      if (!g) continue;
      const s = fx.shift.get(id) ?? { x: 0, y: 0 };
      place(g, (p.x + s.x * LUNGE_3D) / PX, (p.y + s.y * LUNGE_3D) / PX + 0.05);
      if (!a.ghosts.has(id)) this.stepWalk(g, p.x / PX, p.y / PX, t, dt);
    }
    for (const [id, s] of fx.shift) {
      if (fx.pos.has(id)) continue;
      const g = this.unitGroups.get(id) ?? a.ghosts.get(id);
      const home = g?.userData.home as THREE.Vector3 | undefined;
      if (g && home) place(g, home.x + s.x * LUNGE_3D / PX, home.z + s.y * LUNGE_3D / PX);
    }
    // Clips on their beats: the attacker strikes, the defender reels, the fallen fall.
    for (const c of plan.clashes) {
      const key = `c${c.attacker}:${c.t}`;
      if (t >= c.t - 60 && !a.fired.has(key)) {
        a.fired.add(key);
        this.playClip(c.attacker != null ? this.unitGroups.get(c.attacker) ?? a.ghosts.get(c.attacker) : undefined, 'attack');
        if (c.hit && c.defender != null) this.playClip(this.unitGroups.get(c.defender) ?? a.ghosts.get(c.defender), 'hit');
      }
    }
    for (const sh of plan.shots) {
      const key = `s${sh.shooter}:${sh.launch}:${sh.seed}`;
      if (t >= sh.launch && sh.shooter != null && !a.fired.has(key)) { a.fired.add(key); this.playClip(this.unitGroups.get(sh.shooter), 'attack'); }
    }
    const falling = new Set(fx.ghosts.map((gh) => gh.unit.id));
    for (const d of plan.deaths) {
      const g = a.ghosts.get(d.unit.id);
      if (!g) continue;
      const key = `d${d.unit.id}`;
      if (t >= d.t && !a.fired.has(key)) { a.fired.add(key); this.playClip(g, 'death'); }
      // Gone once its fall is over.
      g.visible = falling.has(d.unit.id) || t < d.t;
    }
    for (const gh of fx.ghosts) {
      const g = a.ghosts.get(gh.unit.id);
      if (!g) continue;
      // A figure without a death clip (a painted miniature) topples and fades instead.
      if (!(g.userData.figs as unknown[] | undefined)?.length) g.rotation.z = gh.tilt * 0.8;
      place(g, gh.x / PX, gh.y / PX + 0.05);
      g.position.y -= gh.sink * 0.004;
      ownMaterials(g);
      fadeOwn(g, gh.alpha);
    }
    // Missiles in flight: an arc from the shooter to the target.
    let k = 0;
    for (const sh of plan.shots) {
      const u = (t - sh.launch) / sh.flight;
      if (u < 0 || u > 1) continue;
      const fx0 = sh.from.x / PX, fz0 = sh.from.y / PX, fx1 = sh.to.x / PX, fz1 = sh.to.y / PX;
      const dist = Math.hypot(fx1 - fx0, fz1 - fz0);
      const y0 = Math.max(0, this.heightAt(fx0, fz0)) + 0.14, y1 = Math.max(0, this.heightAt(fx1, fz1)) + 0.08;
      const arc = ARC[sh.kind] * dist * 4 * u * (1 - u);
      // Arrows fly as a small volley: a few shafts side by side.
      const n = sh.kind === 'arrows' ? 4 : 1;
      for (let j = 0; j < n; j++) {
        const m = this.shotMesh(k++, sh.kind);
        const off = (j - (n - 1) / 2) * 0.018;
        const px = fx0 + (fx1 - fx0) * u + off, pz = fz0 + (fz1 - fz0) * u + off * 0.5, py = y0 + (y1 - y0) * u + arc;
        m.position.set(px, py, pz);
        const u2 = Math.min(1, u + 0.02);
        m.lookAt(fx0 + (fx1 - fx0) * u2 + off, y0 + (y1 - y0) * u2 + ARC[sh.kind] * dist * 4 * u2 * (1 - u2), fz0 + (fz1 - fz0) * u2 + off * 0.5);
      }
    }
    for (; k < this.shotPool.length; k++) this.shotPool[k].visible = false;
  }

  /** The selection mark's breathing and its burst (held still with motion off). */
  private stepSelection(now: number) {
    if (!this.selRing.visible) { this.selBurst.visible = false; return; }
    const pulse = selectionBreath(this.uniforms.uTime.value * 1000, this.still);
    (this.selFill.material as THREE.MeshBasicMaterial).opacity = 0.14 + 0.18 * pulse;
    (this.selRing.material as THREE.MeshBasicMaterial).opacity = 0.8 + 0.2 * pulse;
    const burst = this.still ? null : selectionBurst(now - this.selAt);
    this.selBurst.visible = !!burst;
    if (burst) {
      this.selBurst.scale.setScalar(0.97 * burst.scale);
      (this.selBurst.material as THREE.MeshBasicMaterial).opacity = burst.alpha;
      this.animating = true;
    }
  }

  /** Render one frame. Returns false when nothing moved (the caller may skip frames). */
  frame(): boolean {
    const dt = Math.min(0.05, this.clock.getDelta());
    this.uniforms.uTime.value += dt;
    this.stepAnim(performance.now());
    this.flora?.update(this.uniforms.uTime.value, this.still, this.dist);
    for (const g of this.settling) {
      g.rotation.y += -g.rotation.y * Math.min(1, dt * 12);
      if (Math.abs(g.rotation.y) < 2e-3) { g.rotation.y = 0; this.settling.delete(g); }
      this.animating = true;
    }
    if (!this.still) for (const m of this.mixers) m.update(dt);
    this.stepSelection(performance.now());
    this.post.render(this.scene, this.camera);
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
