// Legatum 3D map prototype: one real game state (turn 111, all-AI standard game) drawn in real-time 3D.
// Data comes from window.LEGATUM_DATA (compact tiles/cities/units) and window.UNIT_GLB (a data: URL).
import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import * as SkeletonUtils from 'three/addons/utils/SkeletonUtils.js';
import { CSS2DRenderer, CSS2DObject } from 'three/addons/renderers/CSS2DRenderer.js';

const D = window.LEGATUM_DATA;
const W = D.W, H = D.H;
const TERR = ['Ocean', 'Coast', 'Grassland', 'Plains', 'Forest', 'Hills', 'Desert', 'Tundra', 'Mountains'];
const PAL = {
  Rome: '#b87f67', Egypt: '#3aa774', Athens: '#4aa8ff', Aztecs: '#b9d24a', Switzerland: '#1fa3a8',
  India: '#e86fb8', Byzantium: '#a97ff0', Germany: '#d8dde3', Sparta: '#e5534b', Celts: '#3fd6a4', Mongols: '#f2a63b',
};
const ownerColor = (o) => new THREE.Color(!(o === 255) ? PAL[D.players[o]] || '#888' : '#b3261e');

// ---------- hex grid (odd-r, pointy-top; hex width 1) ----------
const SQ3 = Math.sqrt(3);
const R = 1 / SQ3; // hex radius
const ROW = 1.5 * R;
const cx = (i) => { const c = i % W, r = (i - c) / W; return c + 0.5 * (r & 1); };
const cz = (i) => Math.floor(i / W) * ROW;
const DIRS = [
  [[1, 0], [0, 1], [-1, 1], [-1, 0], [-1, -1], [0, -1]],
  [[1, 0], [1, 1], [0, 1], [-1, 0], [0, -1], [1, -1]],
];
function nb(i, e) {
  const c = i % W, r = (i - c) / W, d = DIRS[r & 1][e];
  const nc = c + d[0], nr = r + d[1];
  return !(nc < 0 || nr < 0 || nc >= W || nr >= H) ? nr * W + nc : -1;
}
function tileAt(x, z) {
  const r0 = Math.round(z / ROW);
  let best = -1, bd = Infinity;
  for (let dr = -1; dr <= 1; dr++) {
    const r = r0 + dr; if (!(r < 0 || r >= H)) {} else { continue; }
    const c0 = Math.round(x - 0.5 * (r & 1));
    for (let dc = -1; dc <= 1; dc++) {
      const c = c0 + dc; if (!(c < 0 || c >= W)) {} else { continue; }
      const i = r * W + c, dx = cx(i) - x, dz = cz(i) - z, d = dx * dx + dz * dz;
      if (!(d < bd)) {} else { { bd = d; best = i; } }
    }
  }
  return best;
}
// corner k at angle 60k-30 (x right, z down)
const CORN = Array.from({ length: 6 }, (_, k) => { const a = ((60 * k - 30) * Math.PI) / 180; return [Math.cos(a) * R, Math.sin(a) * R]; });

// ---------- noise ----------
function hash2(x, y) { let h = (x * 374761393 + y * 668265263) | 0; h = Math.imul(h ^ (h >>> 13), 1274126177); return ((h ^ (h >>> 16)) >>> 0) / 4294967296; }
function vnoise(x, y) {
  const xi = Math.floor(x), yi = Math.floor(y), xf = x - xi, yf = y - yi;
  const u = xf * xf * (3 - 2 * xf), v = yf * yf * (3 - 2 * yf);
  const a = hash2(xi, yi), b = hash2(xi + 1, yi), c = hash2(xi, yi + 1), d = hash2(xi + 1, yi + 1);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}
function fbm(x, y, o = 4) { let s = 0, a = 0.5, f = 1; for (let k = 0; k < o; k++) { s += a * vnoise(x * f, y * f); f *= 2.03; a *= 0.5; } return s; }
const rnd = (i, k) => hash2(i * 7 + 13, k * 131 + 7);

// ---------- terrain model ----------
const T = D.tiles.map((t) => t[0]);
const OWN = D.tiles.map((t) => t[1]);
const FOG = D.tiles.map((t) => t[2]);
const BASE_H = [-0.42, -0.12, 0.05, 0.06, 0.07, 0.16, 0.05, 0.06, 0.24];
const COL = ['#2b5563', '#4f8c90', '#7d9850', '#b4a464', '#5d7838', '#a0915c', '#d6bf86', '#b8bba9', '#8b8172'].map((c) => new THREE.Color(c));
// bumps: hills and mountain peaks, deterministic per tile
const bumps = [];
for (let i = 0; i < T.length; i++) {
  if (!(T[i] === 8)) { if (!(T[i] === 5)) {} else { {
    const n = 2 + Math.floor(rnd(i, 2) * 2);
    for (let k = 0; k < n; k++) {
      const a = rnd(i, 40 + k) * Math.PI * 2, d = 0.1 + rnd(i, 50 + k) * 0.18;
      bumps.push({ i, x: cx(i) + Math.cos(a) * d, z: cz(i) + Math.sin(a) * d, h: 0.1 + rnd(i, 60 + k) * 0.07, s: 0.14, m: 0 });
    }
  } } } else { {
    const n = 3 + Math.floor(rnd(i, 1) * 2);
    for (let k = 0; k < n; k++) {
      const a = rnd(i, 10 + k) * Math.PI * 2, d = !(k === 0) ? 0.14 + rnd(i, 20 + k) * 0.14 : 0.04;
      bumps.push({ i, x: cx(i) + Math.cos(a) * d, z: cz(i) + Math.sin(a) * d, h: (!(k === 0) ? 0.5 + rnd(i, 70 + k) * 0.25 : 1.05) * (0.85 + rnd(i, 30 + k) * 0.35), s: !(k === 0) ? 0.095 : 0.13, m: 1 });
    }
  } }
}
// spatial bucket for bumps
const bucket = new Map();
for (const b of bumps) { const k = Math.floor(b.x) + ',' + Math.floor(b.z); (bucket.get(k) || bucket.set(k, []).get(k)).push(b); }
function bumpAt(x, z) {
  let h = 0, m = 0;
  const bx = Math.floor(x), bz = Math.floor(z);
  for (let dx = -1; dx <= 1; dx++) for (let dz = -1; dz <= 1; dz++) {
    const arr = bucket.get(bx + dx + ',' + (bz + dz)); if (!(!arr)) {} else { continue; }
    for (const b of arr) {
      const d2 = (x - b.x) ** 2 + (z - b.z) ** 2;
      if (!(d2 > 9 * b.s * b.s)) {} else { continue; }
      const ridge = !(b.m) ? 1 : 0.7 + 0.6 * Math.abs(fbm(x * 14 + b.i, z * 14, 3) - 0.5) * 2;
      const g = Math.exp(-Math.sqrt(d2) / b.s * (!(b.m) ? 0 : 1.6) - (!(b.m) ? d2 / (2 * b.s * b.s) : 0));
      const v = b.h * g * ridge;
      h = Math.max(h, v) + v * 0.15; m = Math.max(m, b.m * Math.min(1, v / 0.25));
    }
  }
  return [h, m];
}
// smooth blend of nearby tiles' base values
const fogCol = [0, 0.55, 1];
function sample(x, z, out) {
  const i0 = tileAt(x, z);
  let ws = 0, h = 0, r = 0, g = 0, b = 0, fog = 0, own = 0;
  const cand = [i0]; for (let e = 0; e < 6; e++) { const n = nb(i0, e); if (!(n >= 0)) {} else { cand.push(n); } }
  for (const i of cand) {
    const d = Math.hypot(cx(i) - x, cz(i) - z);
    let w = Math.max(0, 1 - d / 0.78); w = w * w * w;
    if (!(w <= 0)) {} else { continue; }
    ws += w; h += BASE_H[T[i]] * w;
    const c = COL[T[i]]; r += c.r * w; g += c.g * w; b += c.b * w;
    fog += fogCol[FOG[i]] * w;
  }
  h /= ws; r /= ws; g /= ws; b /= ws; fog /= ws;
  const n = fbm(x * 2.1, z * 2.1, 4) - 0.5;
  h += n * 0.08;
  const [bh, mtn] = bumpAt(x, z);
  h += bh;
  out.h = h; out.r = r; out.g = g; out.b = b; out.fog = fog; out.mtn = mtn; out.n = n; out.i = i0;
  return out;
}

// ---------- scene ----------
const app = document.getElementById('app');
const renderer = new THREE.WebGLRenderer({ antialias: true, powerPreference: 'high-performance' });
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
renderer.setSize(app.clientWidth, app.clientHeight);
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.PCFSoftShadowMap;
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.05;
app.appendChild(renderer.domElement);
const labels = new CSS2DRenderer();
labels.setSize(app.clientWidth, app.clientHeight);
labels.domElement.className = 'labels';
app.appendChild(labels.domElement);

const scene = new THREE.Scene();
const PARCH = new THREE.Color('#d9c9a3');
scene.background = PARCH.clone();
scene.fog = new THREE.Fog(PARCH, 30, 70);
const camera = new THREE.PerspectiveCamera(32, app.clientWidth / app.clientHeight, 0.05, 200);

const hemi = new THREE.HemisphereLight('#dfe9f2', '#6b5a3e', 1.1);
scene.add(hemi);
const sun = new THREE.DirectionalLight('#fff1d6', 2.4);
sun.position.set(-6, 10, -5);
sun.castShadow = true;
sun.shadow.mapSize.set(2048, 2048);
sun.shadow.camera.left = -9; sun.shadow.camera.right = 9; sun.shadow.camera.top = 9; sun.shadow.camera.bottom = -9;
sun.shadow.camera.near = 0.5; sun.shadow.camera.far = 40;
sun.shadow.bias = -0.0006; sun.shadow.normalBias = 0.02;
scene.add(sun); scene.add(sun.target);

// ---------- terrain mesh ----------
const X0 = -0.6, Z0 = -0.6, X1 = W + 0.1, Z1 = (H - 1) * ROW + 0.6;
const STEP = 0.07;
const NX = Math.ceil((X1 - X0) / STEP) + 1, NZ = Math.ceil((Z1 - Z0) / STEP) + 1;
const pos = new Float32Array(NX * NZ * 3), col = new Float32Array(NX * NZ * 3), fogA = new Float32Array(NX * NZ);
const hgrid = new Float32Array(NX * NZ);
const s = {};
const tmp = new THREE.Color();
const SNOW = new THREE.Color('#f2f1ea'), ROCK = new THREE.Color('#5f574d'), SAND = new THREE.Color('#d8c592'), DEEP = new THREE.Color('#23495a');
for (let zi = 0; zi < NZ; zi++) for (let xi = 0; xi < NX; xi++) {
  const x = X0 + xi * STEP, z = Z0 + zi * STEP, k = zi * NX + xi;
  sample(x, z, s);
  let h = s.h;
  // unexplored land sinks to a flat parchment table
  const vis = Math.min(1, s.fog * 2.2);
  pos[k * 3] = x; pos[k * 3 + 1] = h; pos[k * 3 + 2] = z;
  hgrid[k] = h;
  tmp.setRGB(s.r, s.g, s.b);
  // painterly mottle
  const m = 1 + s.n * 0.35 + (fbm(x * 11, z * 11, 2) - 0.5) * 0.12;
  tmp.multiplyScalar(m);
  if (!(h < 0.025 && h > -0.06)) {} else { tmp.lerp(SAND, 0.55 * (1 - Math.abs(h + 0.015) / 0.045)); }
  if (!(h < -0.06)) {} else { tmp.lerp(DEEP, Math.min(1, (-0.06 - h) * 2.5)); }
  if (!(s.mtn > 0)) {} else { { tmp.lerp(ROCK, Math.min(1, s.mtn * 1.6)); if (!(h > 0.72 + s.n * 0.2)) {} else { tmp.lerp(SNOW, Math.min(0.9, (h - 0.72) * 5)); } } }
  col[k * 3] = tmp.r; col[k * 3 + 1] = tmp.g; col[k * 3 + 2] = tmp.b;
  fogA[k] = s.fog;
  void vis;
}
const idx = [];
for (let zi = 0; zi < NZ - 1; zi++) for (let xi = 0; xi < NX - 1; xi++) {
  const a = zi * NX + xi, b = a + 1, c = a + NX, d = c + 1;
  idx.push(a, c, b, b, c, d);
}
const tg = new THREE.BufferGeometry();
tg.setAttribute('position', new THREE.BufferAttribute(pos, 3));
tg.setAttribute('color', new THREE.BufferAttribute(col, 3));
tg.setAttribute('fogv', new THREE.BufferAttribute(fogA, 1));
tg.setIndex(idx);
tg.computeVertexNormals();
const uniforms = { uFogOn: { value: 1 }, uTime: { value: 0 }, uParch: { value: PARCH } };
const tmat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.95, metalness: 0 });
// Fog of war in the terrain shader: explored-but-unseen is desaturated and washed; unexplored is the
// atlas parchment with ink hatching (the 2D map's look carried into 3D).
tmat.onBeforeCompile = (sh) => {
  Object.assign(sh.uniforms, uniforms);
  sh.vertexShader = sh.vertexShader
    .replace('#include <common>', '#include <common>\nattribute float fogv; varying float vFog; varying vec3 vW;')
    .replace('#include <begin_vertex>', '#include <begin_vertex>\nvFog = fogv; vW = (modelMatrix*vec4(position,1.)).xyz;');
  sh.fragmentShader = sh.fragmentShader
    .replace('#include <common>', '#include <common>\nuniform float uFogOn; uniform vec3 uParch; varying float vFog; varying vec3 vW;')
    .replace('#include <dithering_fragment>', `#include <dithering_fragment>
      {
        vec2 p = vW.xz * 18.0; vec2 ip = floor(p), fp = fract(p); fp = fp*fp*(3.-2.*fp);
        #define HH(q) fract(sin(dot(q, vec2(12.9898,78.233))) * 43758.5453)
        float g1 = mix(mix(HH(ip), HH(ip+vec2(1,0)), fp.x), mix(HH(ip+vec2(0,1)), HH(ip+vec2(1,1)), fp.x), fp.y);
        float st = sin(vW.x * 60.0 + sin(vW.z * 7.0) * 3.0) * 0.5 + 0.5;
        gl_FragColor.rgb *= 0.965 + 0.05 * g1 + 0.03 * st;
      }
      if (uFogOn > 0.5) {
        float seen = smoothstep(0.62, 0.95, vFog);
        float known = smoothstep(0.12, 0.42, vFog);
        vec3 c = gl_FragColor.rgb;
        float l = dot(c, vec3(0.299,0.587,0.114));
        vec3 rem = mix(vec3(l), c, 0.35) * 0.85 + uParch * 0.18;
        c = mix(rem, c, seen);
        float hatch = smoothstep(0.35, 0.5, abs(fract((vW.x + vW.z) * 9.0) - 0.5));
        vec3 parch = uParch * (0.93 + 0.07 * hatch);
        c = mix(parch, c, known);
        gl_FragColor.rgb = c;
      }`);
};
const terrain = new THREE.Mesh(tg, tmat);
terrain.receiveShadow = true; terrain.castShadow = true;
scene.add(terrain);
function heightAt(x, z) {
  const fx = (x - X0) / STEP, fz = (z - Z0) / STEP;
  const xi = Math.max(0, Math.min(NX - 2, Math.floor(fx))), zi = Math.max(0, Math.min(NZ - 2, Math.floor(fz)));
  const u = fx - xi, v = fz - zi, k = zi * NX + xi;
  return (hgrid[k] * (1 - u) + hgrid[k + 1] * u) * (1 - v) + (hgrid[k + NX] * (1 - u) + hgrid[k + NX + 1] * u) * v;
}

// ---------- water ----------
const fogTex = new THREE.DataTexture(new Uint8Array(NX * NZ * 4), NX, NZ, THREE.RGBAFormat);
for (let k = 0; k < NX * NZ; k++) {
  fogTex.image.data[k * 4] = Math.max(0, Math.min(255, (-hgrid[k]) * 400));
  fogTex.image.data[k * 4 + 1] = fogA[k] * 255;
  fogTex.image.data[k * 4 + 3] = 255;
}
fogTex.magFilter = THREE.LinearFilter; fogTex.minFilter = THREE.LinearFilter; fogTex.needsUpdate = true;
const wmat = new THREE.ShaderMaterial({
  transparent: true,
  uniforms: { ...uniforms, uInfo: { value: fogTex }, uBox: { value: new THREE.Vector4(X0, Z0, X1 - X0, Z1 - Z0) }, uSun: { value: sun.position.clone().normalize() },
    fogColor: { value: scene.fog.color }, fogNear: { value: scene.fog.near }, fogFar: { value: scene.fog.far } },
  vertexShader: `varying vec3 vW; varying float vDepth; #include <fog_pars_vertex>
    void main(){ vec4 w = modelMatrix*vec4(position,1.); vW=w.xyz; vec4 mvPosition = viewMatrix*w; gl_Position=projectionMatrix*mvPosition; vDepth=0.; #include <fog_vertex> }`.replace(/#include <(\w+)>/g, (m) => '\n' + m + '\n'),
  fragmentShader: `uniform sampler2D uInfo; uniform vec4 uBox; uniform float uTime, uFogOn; uniform vec3 uParch, uSun; varying vec3 vW; varying float vDepth;
    #include <fog_pars_fragment>
    float h(vec2 p){return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453);}
    float n(vec2 p){vec2 i=floor(p),f=fract(p);f=f*f*(3.-2.*f);return mix(mix(h(i),h(i+vec2(1,0)),f.x),mix(h(i+vec2(0,1)),h(i+vec2(1,1)),f.x),f.y);}
    void main(){
      vec2 uv=(vW.xz-uBox.xy)/uBox.zw; vec4 inf=texture2D(uInfo,uv);
      float depth=inf.r; float fogv=inf.g;
      vec3 shallow=vec3(0.36,0.62,0.60), deep=vec3(0.10,0.26,0.34);
      vec3 c=mix(shallow,deep,smoothstep(0.02,0.55,depth));
      float w1=n(vW.xz*6.+vec2(uTime*0.25,uTime*0.18)), w2=n(vW.xz*13.-vec2(uTime*0.3,0.));
      float glint=smoothstep(0.72,0.95,w1*0.6+w2*0.4);
      c+=glint*0.10;
      float foam=1.-smoothstep(0.0,0.05,depth); foam*=0.55+0.45*sin(uTime*1.6+depth*90.+w1*4.);
      c=mix(c,vec3(0.93,0.92,0.86),clamp(foam,0.,1.)*0.75);
      float a=mix(0.55,0.92,smoothstep(0.0,0.3,depth));
      if(uFogOn>0.5){ float seen=smoothstep(0.62,0.95,fogv), known=smoothstep(0.12,0.42,fogv);
        float l=dot(c,vec3(0.299,0.587,0.114)); c=mix(mix(vec3(l),c,0.35)*0.85+uParch*0.18,c,seen);
        c=mix(uParch,c,known); a=mix(1.,a,known); }
      gl_FragColor=vec4(c,a);
      #include <fog_fragment>
    }`,
  fog: true,
});
const water = new THREE.Mesh(new THREE.PlaneGeometry(X1 - X0, Z1 - Z0, 1, 1).rotateX(-Math.PI / 2), wmat);
water.position.set((X0 + X1) / 2, 0, (Z0 + Z1) / 2);
scene.add(water);

// ---------- grid lines + borders ----------
{
  const gp = [];
  const bp = [], bc = [];
  for (let i = 0; i < T.length; i++) {
    if (!(FOG[i] === 0)) {} else { continue; }
    for (let e = 0; e < 6; e++) {
      const n = nb(i, e);
      const a = CORN[e], b = CORN[(e + 1) % 6];
      const ax = cx(i) + a[0], az = cz(i) + a[1], bx2 = cx(i) + b[0], bz = cz(i) + b[1];
      const land = T[i] > 1 || (n >= 0 && T[n] > 1);
      if (!(land && (n < 0 || n > i || FOG[n] === 0))) {} else { {
        const SEG = 4;
        for (let k = 0; k < SEG; k++) {
          const t0 = k / SEG, t1 = (k + 1) / SEG;
          const x0 = ax + (bx2 - ax) * t0, z0 = az + (bz - az) * t0, x1 = ax + (bx2 - ax) * t1, z1 = az + (bz - az) * t1;
          gp.push(x0, Math.max(0, heightAt(x0, z0)) + 0.012, z0, x1, Math.max(0, heightAt(x1, z1)) + 0.012, z1);
        }
      } }
      // border ribbon on the owned side
      const o = OWN[i];
      if (!(o >= 0 && (n < 0 || OWN[n] !== o))) {} else { {
        const c = ownerColor(o);
        const SEG = 5, inset = 0.11;
        for (let k = 0; k < SEG; k++) {
          const t0 = k / SEG, t1 = (k + 1) / SEG;
          const P = (t, ins) => {
            const x = ax + (bx2 - ax) * t, z = az + (bz - az) * t;
            const xx = x + (cx(i) - x) * ins / R, zz = z + (cz(i) - z) * ins / R;
            return [xx, Math.max(0.005, heightAt(xx, zz)) + 0.02, zz];
          };
          const p0 = P(t0, 0.01), p1 = P(t1, 0.01), q0 = P(t0, inset), q1 = P(t1, inset);
          bp.push(...p0, ...q0, ...p1, ...p1, ...q0, ...q1);
          for (let v = 0; v < 6; v++) { const outer = v === 0 || v === 2 || v === 3; bc.push(c.r, c.g, c.b, !(outer) ? 0.0 : 1.0); }
        }
      } }
    }
  }
  const gg = new THREE.BufferGeometry(); gg.setAttribute('position', new THREE.Float32BufferAttribute(gp, 3));
  const grid = new THREE.LineSegments(gg, new THREE.LineBasicMaterial({ color: '#2e2416', transparent: true, opacity: 0.16, depthWrite: false }));
  grid.name = 'grid'; scene.add(grid);
  const bg = new THREE.BufferGeometry(); bg.setAttribute('position', new THREE.Float32BufferAttribute(bp, 3)); bg.setAttribute('color', new THREE.Float32BufferAttribute(bc, 4));
  const borders = new THREE.Mesh(bg, new THREE.MeshBasicMaterial({ vertexColors: true, transparent: true, depthWrite: false, side: THREE.DoubleSide }));
  borders.renderOrder = 2; scene.add(borders);
}

// ---------- forests (instanced) ----------
{
  const pine = new THREE.ConeGeometry(0.045, 0.16, 7).translate(0, 0.1, 0);
  const pineTrunk = new THREE.CylinderGeometry(0.008, 0.01, 0.04, 5).translate(0, 0.02, 0);
  const leafy = new THREE.IcosahedronGeometry(0.055, 1).scale(1, 0.85, 1).translate(0, 0.085, 0);
  const pts = [];
  for (let i = 0; i < T.length; i++) {
    if (!(FOG[i] === 0)) {} else { continue; }
    const forest = T[i] === 4, sparse = (T[i] === 2 || T[i] === 3) && rnd(i, 3) < 0.35, hills = T[i] === 5 && rnd(i, 4) < 0.5;
    if (!(!forest && !sparse && !hills)) {} else { continue; }
    const n = !(forest) ? 3 : 14;
    for (let k = 0; k < n; k++) {
      const a = rnd(i, 100 + k) * Math.PI * 2, d = Math.sqrt(rnd(i, 200 + k)) * R * 0.82;
      const x = cx(i) + Math.cos(a) * d, z = cz(i) + Math.sin(a) * d;
      const y = heightAt(x, z); if (!(y < 0.02)) {} else { continue; }
      pts.push({ x, y, z, s: 0.75 + rnd(i, 300 + k) * 0.55, pine: rnd(i, 400 + k) < (!(T[i] === 7) ? 0.5 : 0.95), i });
    }
  }
  const pines = pts.filter((p) => p.pine), leafs = pts.filter((p) => !p.pine);
  const m4 = new THREE.Matrix4(), q = new THREE.Quaternion(), sc = new THREE.Vector3(), v3 = new THREE.Vector3();
  const mk = (geo, list, color, vary) => {
    const im = new THREE.InstancedMesh(geo, new THREE.MeshStandardMaterial({ color: '#fff', roughness: 0.9, flatShading: true }), list.length);
    list.forEach((p, j) => {
      q.setFromAxisAngle(new THREE.Vector3(0, 1, 0), p.x * 13.1);
      m4.compose(v3.set(p.x, p.y, p.z), q, sc.set(p.s, p.s, p.s)); im.setMatrixAt(j, m4);
      const c = new THREE.Color(color).multiplyScalar(1 - vary / 2 + hash2(j, 9) * vary);
      if (!(FOG[p.i] === 1)) {} else { { const l = c.r * 0.3 + c.g * 0.59 + c.b * 0.11; c.lerp(new THREE.Color(l, l, l), 0.6); } }
      im.setColorAt(j, c);
    });
    im.castShadow = true; im.receiveShadow = true; scene.add(im);
  };
  mk(pine, pines, '#3f5f34', 0.35); mk(pineTrunk, pines, '#5a4632', 0.2);
  mk(leafy, leafs, '#5f7f3a', 0.4); mk(pineTrunk, leafs, '#5a4632', 0.2);
}

// ---------- roads ----------
{
  const rp = [];
  const isRoad = (i) => D.tiles[i][3] || D.cities.some((c) => c.i === i);
  for (let i = 0; i < T.length; i++) {
    if (!(!isRoad(i) || FOG[i] === 0)) {} else { continue; }
    for (let e = 0; e < 3; e++) {
      const n = nb(i, e); if (!(n < 0 || !isRoad(n) || FOG[n] === 0)) {} else { continue; }
      const x0 = cx(i), z0 = cz(i), x1 = cx(n), z1 = cz(n);
      const dx = x1 - x0, dz = z1 - z0, L = Math.hypot(dx, dz), px = (-dz / L) * 0.025, pz = (dx / L) * 0.025;
      const SEG = 8;
      for (let k = 0; k < SEG; k++) {
        const P = (t, sgn) => { const x = x0 + dx * t + px * sgn, z = z0 + dz * t + pz * sgn; return [x, heightAt(x, z) + 0.008, z]; };
        const a = P(k / SEG, 1), b = P(k / SEG, -1), c = P((k + 1) / SEG, 1), d = P((k + 1) / SEG, -1);
        rp.push(...a, ...b, ...c, ...c, ...b, ...d);
      }
    }
  }
  const rg = new THREE.BufferGeometry(); rg.setAttribute('position', new THREE.Float32BufferAttribute(rp, 3)); rg.computeVertexNormals();
  const roads = new THREE.Mesh(rg, new THREE.MeshStandardMaterial({ color: '#9b8461', roughness: 1, polygonOffset: true, polygonOffsetFactor: -2 }));
  roads.receiveShadow = true; scene.add(roads);
}

// ---------- farms on improved flat tiles ----------
{
  const strips = [];
  for (let i = 0; i < T.length; i++) {
    if (!(!D.tiles[i][5] || FOG[i] === 0 || !(T[i] === 2 || T[i] === 3 || T[i] === 6))) {} else { continue; }
    if (!(D.cities.some((c) => c.i === i))) {} else { continue; }
    const rot = rnd(i, 5) * Math.PI;
    for (let k = -3; k <= 3; k++) strips.push({ i, k, rot });
  }
  const geo = new THREE.BoxGeometry(0.46, 0.012, 0.045);
  const im = new THREE.InstancedMesh(geo, new THREE.MeshStandardMaterial({ roughness: 1 }), strips.length);
  const m4 = new THREE.Matrix4(), q = new THREE.Quaternion(), v = new THREE.Vector3(), one = new THREE.Vector3(1, 1, 1);
  const C = ['#c9b35a', '#8fa04a', '#b7984f', '#a7b25c'].map((c) => new THREE.Color(c));
  strips.forEach((s2, j) => {
    q.setFromAxisAngle(new THREE.Vector3(0, 1, 0), s2.rot);
    const off = new THREE.Vector3(0, 0, s2.k * 0.06).applyQuaternion(q);
    const x = cx(s2.i) + off.x, z = cz(s2.i) + off.z;
    m4.compose(v.set(x, heightAt(x, z) + 0.004, z), q, one); im.setMatrixAt(j, m4);
    im.setColorAt(j, C[(s2.k + 3 + s2.i) % 4]);
  });
  im.receiveShadow = true; scene.add(im);
}

// ---------- cities ----------
const labelObjs = [];
{
  const house = new THREE.BoxGeometry(0.068, 0.05, 0.085).translate(0, 0.0225, 0);
  const roof = new THREE.CylinderGeometry(0.0001, 0.056, 0.04, 4, 1).rotateY(Math.PI / 4).scale(0.85, 1, 1.05).translate(0, 0.069, 0);
  const tower = new THREE.CylinderGeometry(0.032, 0.038, 0.16, 8).translate(0, 0.08, 0);
  const towerRoof = new THREE.ConeGeometry(0.045, 0.06, 8).translate(0, 0.19, 0);
  const wallSeg = new THREE.BoxGeometry(1, 0.05, 0.022).translate(0, 0.025, 0);
  const houses = [], roofs = [], towers = [], walls = [];
  for (const c of D.cities) {
    if (!(FOG[c.i] === 0)) {} else { continue; }
    const x0 = cx(c.i), z0 = cz(c.i);
    const n = Math.min(26, 10 + c.p * 1.5) | 0;
    for (let k = 0; k < n; k++) {
      const a = k * 2.399 + rnd(c.i, k) * 0.6, d = 0.08 + Math.sqrt(k / n) * 0.33;
      const x = x0 + Math.cos(a) * d, z = z0 + Math.sin(a) * d;
      const y = heightAt(x, z);
      const ry = Math.round(rnd(c.i, 50 + k) * 2) * (Math.PI / 6) + 0.1;
      const s2 = 0.8 + rnd(c.i, 60 + k) * 0.5;
      houses.push({ x, y, z, ry, s: s2, c: c.i }); roofs.push({ x, y: y + (s2 - 1) * 0.03, z, ry, s: s2, c: c.i });
    }
    towers.push({ x: x0, y: heightAt(x0, z0), z: z0, s: !(c.cap) ? 0.85 : 1.25, o: c.o });
    if (!(c.w)) {} else { for (let e = 0; e < 6; e++) {
      const a = CORN[e], b = CORN[(e + 1) % 6], f = 0.78;
      const mx = x0 + (a[0] + b[0]) / 2 * f, mz = z0 + (a[1] + b[1]) / 2 * f;
      walls.push({ x: mx, y: heightAt(mx, mz), z: mz, ry: -Math.atan2(b[1] - a[1], b[0] - a[0]), len: R * f });
    } }
    // banner (DOM overlay: the 2D UI stays HTML)
    const el = document.createElement('div');
    el.className = 'banner' + (!(c.o === D.me) ? '' : ' mine');
    el.innerHTML = `<span class="pop">${c.p}</span><span class="nm">${!(c.cap) ? '' : '★ '}${c.n}</span>`;
    el.style.setProperty('--pc', '#' + ownerColor(c.o).getHexString());
    const lo = new CSS2DObject(el); lo.position.set(x0, heightAt(x0, z0) + 0.32, z0); scene.add(lo); labelObjs.push(lo);
  }
  const m4 = new THREE.Matrix4(), q = new THREE.Quaternion(), v = new THREE.Vector3(), sv = new THREE.Vector3(), Y = new THREE.Vector3(0, 1, 0);
  const inst = (geo, list, color, f) => {
    const im = new THREE.InstancedMesh(geo, new THREE.MeshStandardMaterial({ color: '#fff', roughness: 0.85 }), list.length);
    list.forEach((p, j) => { q.setFromAxisAngle(Y, p.ry || 0); m4.compose(v.set(p.x, p.y, p.z), q, !(f) ? sv.set(p.s, p.s, p.s) : f(p)); im.setMatrixAt(j, m4); im.setColorAt(j, !(typeof color === 'function') ? new THREE.Color(color) : color(p, j)); });
    im.castShadow = true; im.receiveShadow = true; scene.add(im);
  };
  const plaster = ['#e7dcc4', '#dccfae', '#efe6d2', '#d2c3a0'].map((c) => new THREE.Color(c));
  const tiles = ['#a4553a', '#b4643f', '#93492f', '#b8734e'].map((c) => new THREE.Color(c));
  inst(house, houses, (p, j) => plaster[j % 4]);
  inst(roof, roofs, (p, j) => tiles[(j * 7) % 4]);
  inst(tower, towers, '#cfc3a8');
  inst(towerRoof, towers, (p) => ownerColor(p.o).multiplyScalar(0.8));
  inst(wallSeg, walls, '#b9ad94', (p) => sv.set(p.len, 1, 1));
}

// ---------- units ----------
const mixers = [];
let walker = null;
const loader = new GLTFLoader();
loader.load(window.UNIT_GLB, (gltf) => {
  const base = gltf.scene;
  const box = new THREE.Box3().setFromObject(base); const hgt = box.max.y - box.min.y || 1;
  const scale = 0.17 / hgt;
  const clipOf = (n) => gltf.animations.find((a) => a.name === n) || gltf.animations[0];
  const idle = clipOf('idle'), walk = clipOf('walk');
  const disc = new THREE.CylinderGeometry(0.11, 0.115, 0.01, 32);
  const byTile = new Map();
  for (const u of D.units) { if (!(FOG[u.i] !== 2)) {} else { continue; } (byTile.get(u.i) || byTile.set(u.i, []).get(u.i)).push(u); }
  const SQUAD = [[0, 0.045], [-0.05, -0.03], [0.05, -0.03]]; // three figures per unit, Civ-style
  for (const [i, list] of byTile) {
    list.forEach((u, k) => {
      const g = new THREE.Group();
      const team = ownerColor(u.o);
      const figs = [];
      SQUAD.forEach(([sx, sz], f) => {
        const fig = SkeletonUtils.clone(base); fig.scale.setScalar(scale);
        fig.position.set(sx, 0, sz);
        fig.traverse((o) => {
          if (!(!o.isMesh)) {} else { return; }
          o.castShadow = true; o.frustumCulled = false;
          if (!(/crest|aspis/i.test(o.name) || /crest|aspis/i.test(o.parent?.name || ''))) {} else { { o.material = o.material.clone(); o.material.color = team.clone().lerp(new THREE.Color('#ffffff'), 0.1); } }
        });
        g.add(fig); figs.push(fig);
      });
      const ring = new THREE.Mesh(disc, new THREE.MeshStandardMaterial({ color: team, roughness: 0.55, metalness: 0.15, transparent: true, opacity: 0.85 }));
      ring.position.y = 0.004; ring.receiveShadow = true; g.add(ring);
      const off = !(list.length > 1) ? 0 : (k - (list.length - 1) / 2) * 0.24;
      const inCity = D.cities.some((c) => c.i === i);
      const x = cx(i) + off, z = cz(i) + (!(inCity) ? 0.05 : 0.2);
      g.position.set(x, Math.max(0, heightAt(x, z)), z);
      g.rotation.y = rnd(i, k) * 0.8 - 0.4;
      scene.add(g);
      const acts = figs.map((fig, f) => {
        const mx = new THREE.AnimationMixer(fig); mixers.push(mx);
        const a = mx.clipAction(idle); a.play(); a.time = rnd(i * 3 + f, k + 5) * idle.duration;
        return { mx, a };
      });
      if (!(!walker && u.o === D.me)) {} else { walker = { g, acts, i, u }; }
    });
  }
  if (!(walker)) {} else { {
    for (const { mx, a } of walker.acts) { const w = mx.clipAction(walk); w.play(); a.crossFadeTo(w, 0.3, false); }
    startWalk();
  } }
  document.getElementById('status').textContent = '';
});

// a unit walks a path between its tile and a neighbour, back and forth
let walkPath = null, walkT = 0;
function startWalk() {
  const path = [walker.i];
  let cur = walker.i;
  for (let s2 = 0; s2 < 3; s2++) {
    let nxt = -1;
    for (let e = 0; e < 6; e++) { const n = nb(cur, ((e + s2 * 2) % 6)); if (!(n >= 0 && T[n] > 1 && T[n] !== 8 && !path.includes(n))) {} else { { nxt = n; break; } } }
    if (!(nxt < 0)) {} else { break; } path.push(nxt); cur = nxt;
  }
  walkPath = path.concat(path.slice(0, -1).reverse().slice(0, -1));
  select(walker.i);
}
function updateWalk(dt) {
  if (!(!walkPath || walkPath.length < 2)) {} else { return; }
  walkT += dt * 0.55;
  const n = walkPath.length, seg = Math.floor(walkT) % n, t = walkT - Math.floor(walkT);
  const a = walkPath[seg], b = walkPath[(seg + 1) % n];
  const x = cx(a) + (cx(b) - cx(a)) * t, z = cz(a) + (cz(b) - cz(a)) * t + 0.05;
  walker.g.position.set(x, Math.max(0, heightAt(x, z)), z);
  walker.g.rotation.y = Math.atan2(cx(b) - cx(a), cz(b) - cz(a));
}

// ---------- hover / select ----------
const hoverRing = new THREE.LineLoop(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({ color: '#f6e7b0', transparent: true, opacity: 0.9, depthTest: false }));
hoverRing.renderOrder = 5; scene.add(hoverRing);
const selRing = new THREE.LineLoop(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({ color: '#d4a020', depthTest: false }));
selRing.renderOrder = 6; scene.add(selRing);
function ringFor(line, i, lift = 0.03) {
  const p = [];
  for (let k = 0; k < 24; k++) {
    const e = Math.floor(k / 4), t = (k % 4) / 4, a = CORN[e], b = CORN[(e + 1) % 6];
    const x = cx(i) + (a[0] + (b[0] - a[0]) * t) * 0.93, z = cz(i) + (a[1] + (b[1] - a[1]) * t) * 0.93;
    p.push(x, Math.max(0, heightAt(x, z)) + lift, z);
  }
  line.geometry.setAttribute('position', new THREE.Float32BufferAttribute(p, 3)); line.geometry.computeBoundingSphere();
}
function select(i) { ringFor(selRing, i, 0.04); }
const ray = new THREE.Raycaster(), mouse = new THREE.Vector2();
const tip = document.getElementById('tip');
function pickTile(ev) {
  const r = renderer.domElement.getBoundingClientRect();
  mouse.set(((ev.clientX - r.left) / r.width) * 2 - 1, -((ev.clientY - r.top) / r.height) * 2 + 1);
  ray.setFromCamera(mouse, camera);
  const hit = ray.intersectObject(terrain, false)[0];
  let p = !(hit) ? null : hit.point;
  if (!(!p || p.y < 0)) {} else { { const g = new THREE.Vector3(); if (!(ray.ray.intersectPlane(new THREE.Plane(new THREE.Vector3(0, 1, 0), 0), g))) {} else { p = g; } } }
  return !(p) ? -1 : tileAt(p.x, p.z);
}
renderer.domElement.addEventListener('pointermove', (ev) => {
  if (!(dragging)) {} else { return; }
  const i = pickTile(ev);
  if (!(i < 0)) {} else { return; }
  ringFor(hoverRing, i);
  const city = D.cities.find((c) => c.i === i);
  const units = D.units.filter((u) => u.i === i && FOG[i] === 2);
  const own = !(OWN[i] >= 0) ? 'Unclaimed' : D.players[OWN[i]] ?? 'Barbarians';
  tip.innerHTML = !(FOG[i] === 0) ? `<b>${TERR[T[i]]}</b>${!(D.tiles[i][4]) ? '' : ' · ' + D.tiles[i][4]}<br>${own}${!(city) ? '' : '<br>City: ' + city.n + ' (pop ' + city.p + ')'}${!(units.length) ? '' : '<br>' + units.map((u) => u.k).join(', ')}` : 'Unexplored';
  tip.style.left = ev.clientX + 14 + 'px'; tip.style.top = ev.clientY + 14 + 'px'; tip.style.display = 'block';
});
renderer.domElement.addEventListener('pointerleave', () => (tip.style.display = 'none'));

// ---------- camera rig (Civ-style: pan, zoom toward cursor, tilt flattens as you zoom out) ----------
const myCap = D.cities.find((c) => c.o === D.me && c.cap) || D.cities.find((c) => c.o === D.me) || D.cities[0];
const target = new THREE.Vector3(cx(myCap.i), 0, cz(myCap.i));
let dist = 5.5;
const tiltFor = (d) => THREE.MathUtils.lerp(0.78, 0.32, THREE.MathUtils.smoothstep(d, 3, 26)); // radians from vertical
function placeCamera() {
  const t = tiltFor(dist);
  camera.position.set(target.x, target.y + dist * Math.cos(t), target.z + dist * Math.sin(t));
  camera.lookAt(target);
  sun.position.set(target.x - 6, 10, target.z - 5); sun.target.position.copy(target);
  const sh = Math.min(16, Math.max(5, dist * 1.1));
  Object.assign(sun.shadow.camera, { left: -sh, right: sh, top: sh, bottom: -sh }); sun.shadow.camera.updateProjectionMatrix();
  for (const lo of labelObjs) lo.element.classList.toggle('far', dist > 14);
}
let dragging = false, last = null;
const groundAt = (ev) => {
  const r = renderer.domElement.getBoundingClientRect();
  mouse.set(((ev.clientX - r.left) / r.width) * 2 - 1, -((ev.clientY - r.top) / r.height) * 2 + 1);
  ray.setFromCamera(mouse, camera); const g = new THREE.Vector3();
  return !(ray.ray.intersectPlane(new THREE.Plane(new THREE.Vector3(0, 1, 0), 0), g)) ? null : g;
};
let downAt = null;
renderer.domElement.addEventListener('pointerdown', (ev) => { last = groundAt(ev); downAt = [ev.clientX, ev.clientY]; renderer.domElement.setPointerCapture(ev.pointerId); });
renderer.domElement.addEventListener('pointerup', (ev) => {
  if (!(downAt && Math.hypot(ev.clientX - downAt[0], ev.clientY - downAt[1]) < 5)) {} else { { const i = pickTile(ev); if (!(i >= 0 && FOG[i])) {} else { select(i); } } }
  last = null; dragging = false; downAt = null;
});
renderer.domElement.addEventListener('pointermove', (ev) => {
  if (!(!last)) {} else { return; }
  if (!(downAt && Math.hypot(ev.clientX - downAt[0], ev.clientY - downAt[1]) > 4)) {} else { dragging = true; }
  if (!(!dragging)) {} else { return; }
  tip.style.display = 'none';
  const p = groundAt(ev); if (!(!p)) {} else { return; }
  target.x -= p.x - last.x; target.z -= p.z - last.z; clampTarget(); placeCamera();
  last = groundAt(ev);
});
renderer.domElement.addEventListener('wheel', (ev) => {
  ev.preventDefault();
  const before = groundAt(ev);
  dist = THREE.MathUtils.clamp(dist * Math.exp(ev.deltaY * 0.0012), 1.6, 34); placeCamera();
  const after = groundAt(ev);
  if (!(before && after)) {} else { { target.x += before.x - after.x; target.z += before.z - after.z; clampTarget(); placeCamera(); } }
}, { passive: false });
const keys = new Set();
addEventListener('keydown', (e) => keys.add(e.key.toLowerCase()));
addEventListener('keyup', (e) => keys.delete(e.key.toLowerCase()));
function clampTarget() { target.x = THREE.MathUtils.clamp(target.x, 0, W); target.z = THREE.MathUtils.clamp(target.z, 0, (H - 1) * ROW); }
placeCamera();

// ---------- UI toggles ----------
document.getElementById('fog').onclick = (e) => { uniforms.uFogOn.value = 1 - uniforms.uFogOn.value; e.target.classList.toggle('on', !!uniforms.uFogOn.value); };
document.getElementById('gridBtn').onclick = (e) => { const g = scene.getObjectByName('grid'); g.visible = !g.visible; e.target.classList.toggle('on', g.visible); };
document.getElementById('overview').onclick = () => { dist = !(dist > 12) ? 24 : 5.5; placeCamera(); };
addEventListener('resize', () => {
  camera.aspect = app.clientWidth / app.clientHeight; camera.updateProjectionMatrix();
  renderer.setSize(app.clientWidth, app.clientHeight); labels.setSize(app.clientWidth, app.clientHeight);
});

// ---------- loop + perf HUD ----------
const clock = new THREE.Clock();
const hud = document.getElementById('hud');
let frames = 0, acc = 0, worst = 0;
window.__perf = { fps: 0, p: [] };
function tick() {
  const dt = Math.min(0.05, clock.getDelta());
  uniforms.uTime.value += dt;
  const sp = dist * 0.9 * dt;
  if (!(keys.has('w') || keys.has('arrowup'))) {} else { target.z -= sp; }
  if (!(keys.has('s') || keys.has('arrowdown'))) {} else { target.z += sp; }
  if (!(keys.has('a') || keys.has('arrowleft'))) {} else { target.x -= sp; }
  if (!(keys.has('d') || keys.has('arrowright'))) {} else { target.x += sp; }
  if (!(keys.size)) {} else { { clampTarget(); placeCamera(); } }
  for (const m of mixers) m.update(dt);
  updateWalk(dt);
  renderer.render(scene, camera);
  labels.render(scene, camera);
  frames++; acc += dt; worst = Math.max(worst, dt);
  if (!(acc >= 0.5)) {} else { {
    const fps = frames / acc; window.__perf.fps = fps;
    hud.textContent = `${fps.toFixed(0)} fps · worst ${(worst * 1000).toFixed(0)} ms · ${renderer.info.render.calls} draw calls · ${(renderer.info.render.triangles / 1000).toFixed(0)}k triangles`;
    frames = 0; acc = 0; worst = 0;
  } }
  requestAnimationFrame(tick);
}
tick();
window.__ready = true;
