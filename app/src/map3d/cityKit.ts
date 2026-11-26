// Procedural settlement and improvement kit. All placement comes from the fogged PlayerView.
// The meshes have painted vertex colours and are instanced across the entire map.
import * as THREE from 'three';
import type { Faction, PlayerView } from '../types';
import { toOffset } from '../hex';
import { neighborIndex } from '../map/grid';
import { cityMaterial } from './cityMaterial';
import { remembered } from './memoryWash';
import { CORNERS, rnd, tileX, tileZ, type World } from './world';

type Style = 0 | 1 | 2 | 3 | 4;
type P = { x: number; y: number; z: number; yaw: number; sx: number; sy: number; sz: number; fog: number; tint: number; tile: number };
const STONE = '#b8aa8d';
const PALETTES = [
  ['#d4bd94', '#a65d3d', '#604b38', '#e5d6b5'], // Mediterranean limestone and fired tile
  ['#b9ae96', '#514a42', '#694b37', '#ddd0ae'], // northern timber and slate
  ['#d7bc8b', '#caa771', '#715944', '#ebd4a7'], // Nile and Middle Eastern adobe
  ['#d0c2a4', '#46534c', '#75493d', '#ead9ae'], // East / Central Asian tiled eaves
  ['#ad9978', '#876947', '#514839', '#d3bd93'], // Mesoamerican stone terraces
] as const;
const ROOFS = [
  ['#a65d3d', '#b57951', '#94765b', '#bd8159', '#80604e', '#9d6745', '#bf8c5d'],
  ['#514a42', '#6a6557', '#775c46', '#827769', '#62584e', '#545952', '#82766a'],
  ['#caa771', '#b39164', '#aa8961', '#dfbd84', '#957e65', '#b99c70', '#d2b58a'],
  ['#46534c', '#59675b', '#756d59', '#627368', '#8b7d69', '#536357', '#77755f'],
  ['#876947', '#766349', '#9b835b', '#aa8d63', '#6a5848', '#947c5b', '#b49a70'],
] as const;

function styleOf(faction: Faction | undefined): Style {
  if (faction === 'Aztecs') return 4;
  // Karakorum had Chinese-style halls; India's flat roofs and courtyards sit with the Nile/Near East kit; Byzantium is
  // Mediterranean stone and tile.
  if (faction === 'Mongols') return 3;
  if (faction === 'Egypt' || faction === 'India') return 2;
  if (faction === 'Germany' || faction === 'Celts' || faction === 'Switzerland') return 1;
  return 0;
}

class Painter {
  p: number[] = []; c: number[] = [];
  private colors = new Map<string, THREE.Color>();
  tri(a: number[], b: number[], c: number[], color: string) {
    let col = this.colors.get(color);
    if (!col) { col = new THREE.Color(color); this.colors.set(color, col); }
    for (const v of [a, b, c]) { this.p.push(...v); this.c.push(col.r, col.g, col.b); }
  }
  quad(a: number[], b: number[], c: number[], d: number[], color: string) { this.tri(a, b, c, color); this.tri(a, c, d, color); }
  box(x: number, y: number, z: number, w: number, h: number, d: number, color: string) {
    const x0=x-w/2, x1=x+w/2, z0=z-d/2, z1=z+d/2, y1=y+h;
    this.quad([x0,y,z1],[x1,y,z1],[x1,y1,z1],[x0,y1,z1],color);
    this.quad([x1,y,z0],[x0,y,z0],[x0,y1,z0],[x1,y1,z0],color);
    this.quad([x0,y,z0],[x0,y,z1],[x0,y1,z1],[x0,y1,z0],color);
    this.quad([x1,y,z1],[x1,y,z0],[x1,y1,z0],[x1,y1,z1],color);
    this.quad([x0,y1,z1],[x1,y1,z1],[x1,y1,z0],[x0,y1,z0],color);
  }
  roof(w: number, d: number, y: number, rise: number, color: string, hip = false) {
    const x=w/2+0.007, z=d/2+0.009, ridge=hip ? w*0.23 : x;
    this.quad([-x,y,z],[x,y,z],[ridge,y+rise,0],[-ridge,y+rise,0],color);
    this.quad([x,y,-z],[-x,y,-z],[-ridge,y+rise,0],[ridge,y+rise,0],color);
    this.tri([-x,y,-z],[-x,y,z],[-ridge,y+rise,0],color);
    this.tri([x,y,z],[x,y,-z],[ridge,y+rise,0],color);
  }
  tier(w: number, d: number, y: number, h: number, color: string) {
    this.box(0,y,0,w,0.006,d,color);
    this.roof(w,d,y+0.006,h,color,true);
  }
  geometry() {
    const g = new THREE.BufferGeometry();
    g.setAttribute('position', new THREE.Float32BufferAttribute(this.p, 3));
    g.setAttribute('color', new THREE.Float32BufferAttribute(this.c, 3));
    g.computeVertexNormals();
    return g;
  }
}

function roofCourses(p: Painter, w: number, d: number, y: number, rise: number, hip: boolean, color: string) {
  const halfW=w/2+0.007, halfD=d/2+0.009, ridge=hip?w*0.23:halfW;
  const course=new THREE.Color(color).multiplyScalar(0.78).getStyle();
  for (const side of [-1,1]) for (const t of [0.24,0.42,0.60,0.78]) {
    const t2=t+0.025, z1=side*halfD*t, z2=side*halfD*t2;
    const x1=ridge+(halfW-ridge)*t, x2=ridge+(halfW-ridge)*t2;
    const y1=y+rise*(1-t)+0.0014,y2=y+rise*(1-t2)+0.0014;
    p.quad([-x1,y1,z1],[x1,y1,z1],[x2,y2,z2],[-x2,y2,z2],course);
  }
}

function house(style: Style, variant: number): THREE.BufferGeometry {
  const p=new Painter(), [wall, , trim, light]=PALETTES[style];
  const roof = ROOFS[style][variant];
  const w=[0.078,0.092,0.105,0.087,0.092,0.125,0.096][variant];
  const d=[0.081,0.082,0.075,0.101,0.092,0.096,0.119][variant];
  const h=(style===1 ? 0.066 : 0.059) * (variant===5 ? 1.52 : variant===6 ? 1.26 : variant===2 ? 1.28 : 1);
  p.box(0,0,0,w,h,d,wall);
  // Dark recessed door and paired shutters make a house legible at close zoom.
  p.box(-0.016,0,d/2+0.001,0.014,0.029,0.002,trim);
  p.box(0.017,0.025,d/2+0.001,0.013,0.014,0.002,trim);
  p.box(0,0.004,-d/2-0.001,0.038,0.006,0.003,light);
  if (style===0) {
    p.roof(w+0.01,d+0.01,h,0.022,roof,true);
    roofCourses(p,w+0.01,d+0.01,h,0.022,true,roof);
    p.box(0.022,0.006,d/2+0.003,0.006,0.029,0.004,light);
    p.box(-0.032,0.038,0,0.007,0.006,d,trim);
    p.box(0.025,0,-0.038,0.033,0.034,0.023,wall);
    p.box(0.025,0.034,-0.038,0.039,0.006,0.029,light);
    p.box(-0.016,0.035,0.04,0.01,0.01,0.003,'#665542');
    if (variant && variant !== 5) {
      p.box(-0.021,h+0.01,-0.012,0.012,0.038,0.012,light);
      p.box(-0.021,h+0.048,-0.012,0.018,0.005,0.018,trim);
    }
  } else if (style===1) {
    p.roof(w,d,h,0.055,roof);
    p.box(0,h*0.5,d/2+0.003,w,0.006,0.004,trim);
    p.box(-w/2+0.006,0.006,d/2+0.003,0.005,h-0.008,0.004,trim);
    p.box(w/2-0.006,0.006,d/2+0.003,0.005,h-0.008,0.004,trim);
    p.box(0.012,h+0.013,0,0.014,0.031,0.014,wall);
  } else if (style===2) {
    p.box(0,h,0,w+0.011,0.006,d+0.011,light);
    for (const x of [-0.025,0,0.025]) p.box(x,h+0.006,d/2+0.002,0.012,0.009,0.008,wall);
    p.box(0.02,h+0.006,-0.014,0.022,0.024,0.025,roof);
  } else if (style===3) {
    p.tier(w+0.028,d+0.027,h-0.003,0.021,roof);
    p.box(0,h+0.018,0,0.037,0.016,0.037,wall);
    p.tier(0.054,0.054,h+0.032,0.015,roof);
    p.box(-0.026,0,d/2+0.005,0.007,h,0.007,trim);
    p.box(0.026,0,d/2+0.005,0.007,h,0.007,trim);
  } else {
    p.box(0,h,0,w+0.025,0.009,d+0.025,roof);
    p.box(0,h+0.009,0,w*0.65,0.016,d*0.65,wall);
    p.box(0,h+0.025,0,w*0.34,0.008,d*0.34,light);
    p.box(0,0,d/2+0.003,0.027,0.008,0.006,roof);
  }
  // Two larger plots have distinct silhouettes: a hall with a tall front
  // and a long workshop with an attached court. They share the same instances.
  if (variant===5) {
    p.box(0,h*0.46,d/2+0.003,w*0.78,0.008,0.005,light);
    for (const x of [-0.032,0,0.032]) p.box(x,h*0.67,d/2+0.004,0.013,0.021,0.005,trim);
    p.box(-w*0.32,h*0.78,-d*0.32,0.018,0.058,0.018,wall);
    p.box(-w*0.32,h*0.84,-d*0.32,0.025,0.006,0.025,light);
  } else if (variant===6) {
    p.box(w*0.4,0,-d*0.22,w*0.5,h*0.61,d*0.46,wall);
    p.box(w*0.4,h*0.61,-d*0.22,w*0.55,0.006,d*0.51,light);
    p.box(-w*0.24,h*0.44,d/2+0.004,w*0.38,0.007,0.005,trim);
  }
  return p.geometry();
}

function landmark(style: Style): THREE.BufferGeometry {
  const p=new Painter(), [wall,roof,trim,light]=PALETTES[style];
  // A raised civic precinct with a style-specific skyline, visible above the houses.
  p.box(0,0,0,0.19,0.013,0.16,trim);
  p.box(0,0.013,0,0.175,0.013,0.145,light);
  if (style===0) {
    p.box(0,0.026,-0.008,0.115,0.092,0.085,wall);
    p.box(0,0.026,0.045,0.136,0.012,0.032,light);
    for (const x of [-0.052,-0.031,-0.01,0.01,0.031,0.052]) p.box(x,0.038,0.056,0.008,0.081,0.008,light);
    p.box(0,0.119,0.044,0.15,0.012,0.046,trim);
    p.roof(0.126,0.099,0.118,0.048,roof);
    roofCourses(p,0.126,0.099,0.118,0.048,false,roof);
    p.box(0,0.026,-0.064,0.062,0.17,0.052,wall);
    for (const y of [0.08,0.13,0.18]) p.box(0,y,-0.091,0.044,0.009,0.004,trim);
    p.roof(0.076,0.065,0.196,0.055,roof,true);
    for (const x of [-0.071,0.071]) {
      p.box(x,0.026,-0.022,0.025,0.052,0.085,wall);
      p.roof(0.035,0.098,0.078,0.02,roof,true);
    }
  } else if (style===1) {
    p.box(0,0.026,0,0.099,0.085,0.079,wall);
    p.roof(0.11,0.092,0.111,0.067,roof);
    p.box(-0.045,0.026,-0.019,0.04,0.125,0.043,wall);
    p.roof(0.046,0.051,0.151,0.054,roof,true);
    for (const x of [-0.034,0,0.034]) p.box(x,0.067,0.042,0.01,0.019,0.004,trim);
  } else if (style===2) {
    p.box(0,0.026,0,0.115,0.078,0.085,wall);
    p.box(0,0.104,0,0.127,0.009,0.098,light);
    for (const x of [-0.049,-0.016,0.016,0.049]) p.box(x,0.029,0.051,0.01,0.072,0.01,light);
    p.box(0,0.113,-0.012,0.048,0.043,0.055,roof);
    p.box(0,0.156,-0.012,0.055,0.006,0.062,light);
  } else if (style===3) {
    p.box(0,0.026,0,0.098,0.075,0.084,wall);
    p.tier(0.145,0.13,0.101,0.03,roof);
    p.box(0,0.126,0,0.072,0.055,0.062,wall);
    p.tier(0.105,0.094,0.181,0.033,roof);
    p.box(0,0.211,0,0.04,0.026,0.035,wall);
    p.tier(0.07,0.064,0.237,0.025,roof);
  } else {
    for (let t=0;t<4;t++) p.box(0,0.026+t*0.035,0,0.122-t*0.023,0.035,0.105-t*0.019,t%2?wall:roof);
    p.box(0,0.166,0,0.047,0.037,0.039,wall);
    p.box(0,0.203,0,0.054,0.009,0.047,light);
    for (let t=0;t<4;t++) p.box(0,0.027+t*0.034,0.054-t*0.008,0.018,0.004,0.013,light);
  }
  return p.geometry();
}

function wallGeometry(): THREE.BufferGeometry {
  const p=new Painter();
  p.box(0,0,0,1,0.071,0.026,STONE);
  p.box(0,0.071,0,1,0.009,0.037,'#d2c4a7');
  for (let x=-0.43;x<0.5;x+=0.17) p.box(x,0.08,0,0.073,0.013,0.038,STONE);
  return p.geometry();
}
function towerGeometry(): THREE.BufferGeometry {
  const p=new Painter();
  p.box(0,0,0,0.058,0.109,0.058,STONE);
  p.box(0,0.109,0,0.073,0.01,0.073,'#d2c4a7');
  for (const x of [-0.025,0.025]) for (const z of [-0.025,0.025]) p.box(x,0.119,z,0.018,0.024,0.018,STONE);
  p.box(0,0.06,0.031,0.012,0.025,0.003,'#5a4c3e');
  return p.geometry();
}
function gateGeometry(): THREE.BufferGeometry {
  const p=new Painter();
  for (const x of [-0.041,0.041]) p.box(x,0,0,0.024,0.106,0.038,STONE);
  p.box(0,0.083,0,0.11,0.026,0.042,STONE);
  p.box(0,0.109,0,0.123,0.011,0.048,'#d2c4a7');
  p.box(0,0,0,0.046,0.082,0.006,'#594632');
  p.box(0,0.058,0.005,0.006,0.008,0.007,'#bf9e55');
  return p.geometry();
}
function courtGeometry(): THREE.BufferGeometry {
  const p=new Painter();
  p.box(0,0,0,0.083,0.003,0.083,'#baa881');
  for (const x of [-0.03,0.03]) for (const z of [-0.03,0.03]) {
    p.box(x,0.003,z,0.018,0.006,0.018,'#657848');
    p.box(x,0.009,z,0.008,0.012,0.008,'#829457');
  }
  p.box(0,0.003,0,0.028,0.012,0.028,'#d5c8a7');
  p.box(0,0.015,0,0.019,0.003,0.019,'#719597');
  return p.geometry();
}
function marketGeometry(): THREE.BufferGeometry {
  const p=new Painter();
  for (const x of [-0.028,0.028]) for (const z of [-0.023,0.023]) p.box(x,0,z,0.005,0.055,0.005,'#725438');
  p.box(0,0.032,0,0.058,0.007,0.047,'#b18c5d');
  p.roof(0.072,0.061,0.055,0.019,'#c4a66d',true);
  p.box(-0.012,0.039,0.014,0.015,0.01,0.012,'#736b43');
  p.box(0.014,0.039,-0.009,0.016,0.012,0.014,'#9e6943');
  return p.geometry();
}
function townGround(): THREE.BufferGeometry {
  const p=new Painter();
  const colours=['#aa9a79','#b7a683','#c0ad86','#a59676'];
  for (let ring=0;ring<6;ring++) {
    const count=7+ring*5, radius=0.035+ring*0.081;
    for (let k=0;k<count;k++) {
      if (ring>2 && rnd(ring*53+k,91)>0.78) continue;
      const a=(k+rnd(ring*37+k,92)*0.28)*Math.PI*2/count;
      const r=radius+(rnd(ring*61+k,93)-0.5)*0.018;
      const x=Math.cos(a)*r,z=Math.sin(a)*r;
      const w=0.037+0.019*rnd(ring*19+k,94),d=0.032+0.019*rnd(ring*13+k,95);
      p.quad([x-w/2,0,z+d/2],[x+w/2,0,z+d/2],[x+w/2,0,z-d/2],[x-w/2,0,z-d/2],colours[(k+ring)%colours.length]);
    }
  }
  return p.geometry();
}
function patchGeometry(): THREE.BufferGeometry {
  const p=new Painter();
  const colors=['#ac9a51','#c6b366','#85924a','#b7a152','#8f9a58','#d0bc79'];
  for (let row=-2;row<=2;row++) for (let col=-2;col<=2;col++) {
    if (row===2 && col===2) continue;
    const x=col*0.105+(row%2)*0.014, z=row*0.097;
    p.box(x,0,z,0.096,0.004,0.087,colors[(row*7+col*3+30)%colors.length]);
    for (let f=-2;f<=2;f++) p.box(x+f*0.016,0.004,z,0.003,0.003,0.077,row%2?'#64773c':'#ddc782');
    if ((row+col)%3===0) p.box(x+0.05,0.004,z,0.006,0.022,0.09,'#536642');
  }
  // Broken hedgerows and a pale farmstead keep the field readable as worked land.
  for (const z of [-0.245,0.245]) for (const x of [-0.17,0,0.17]) p.box(x,0,z,0.12,0.022,0.009,'#53643b');
  for (const x of [-0.265,0.265]) for (const z of [-0.14,0.02,0.14]) p.box(x,0,z,0.009,0.023,0.11,'#53643b');
  p.box(0.17,0.004,0.18,0.083,0.057,0.066,'#c8b58d');
  p.roof(0.094,0.074,0.061,0.029,'#866046',true);
  p.box(0.17,0.004,0.214,0.022,0.032,0.004,'#66523d');
  return p.geometry();
}
function outskirtsGeometry(): THREE.BufferGeometry {
  const p=new Painter();
  const soil=['#8d784d','#a08b58','#ab975f','#7d8751'];
  const crop=['#677947','#9c9d55','#b9a96a','#657b4a'];
  for (let row=0;row<2;row++) for (let col=0;col<2;col++) {
    const x=(col-0.5)*0.225,z=(row-0.5)*0.225,k=row*2+col;
    p.box(x,0,z,0.207,0.003,0.205,soil[k]);
    for (let f=-3;f<=3;f++) p.box(x+f*0.026,0.003,z,0.009,0.002,0.184,crop[k]);
  }
  for (const z of [-0.225,0,0.225]) p.box(0,0.003,z,0.46,0.008,0.008,'#78834d');
  for (const x of [-0.225,0,0.225]) p.box(x,0.003,0,0.008,0.008,0.46,'#78834d');
  return p.geometry();
}
function pastureGeometry(): THREE.BufferGeometry {
  const p=new Painter();
  p.box(0,0,0,0.38,0.003,0.35,'#82934c');
  for (const s of [-1,1]) {
    p.box(s*0.19,0,s*0.0,0.006,0.035,0.36,'#826d4a');
    p.box(0,0,s*0.175,0.39,0.035,0.006,'#826d4a');
    for (const k of [-2,-1,0,1,2]) {
      p.box(s*0.19,0,k*0.07,0.011,0.055,0.011,'#6e5b3e');
      p.box(k*0.075,0,s*0.175,0.011,0.055,0.011,'#6e5b3e');
    }
  }
  p.box(0.06,0.003,-0.04,0.072,0.045,0.06,'#c8b599');
  p.roof(0.08,0.068,0.048,0.024,'#785e45',true);
  return p.geometry();
}
function pitGeometry(): THREE.BufferGeometry {
  const p=new Painter();
  // Open cut: dark recessed floor, stepped quarry rim, spoil and a timber hoist.
  p.box(0,0,0,0.35,0.006,0.33,'#4c483e');
  for (const s of [-1,1]) {
    p.box(s*0.14,0,s*0.0,0.09,0.027,0.31,'#98866a');
    p.box(0,0,s*0.13,0.3,0.027,0.07,'#b2a07d');
  }
  p.box(0.13,0.027,0.11,0.07,0.028,0.075,'#b6a17c');
  p.box(-0.12,0.027,-0.105,0.065,0.024,0.065,'#c7b28d');
  for (const x of [-0.045,0.045]) p.box(x,0.006,-0.045,0.009,0.087,0.009,'#6e5339');
  p.box(0,0.088,-0.045,0.11,0.009,0.012,'#6e5339');
  return p.geometry();
}

function instance(geo: THREE.BufferGeometry, list: P[], out: THREE.Object3D[], shadows=true) {
  if (!list.length) { geo.dispose(); return; }
  const im=new THREE.InstancedMesh(geo,cityMaterial(),list.length);
  im.userData.memoryTiles = Int16Array.from(list, (p) => p.tile);
  geo.setAttribute('instanceMemory', new THREE.InstancedBufferAttribute(new Float32Array(list.map((p) => remembered(p.fog))), 1));
  const matrices=im.instanceMatrix.array as Float32Array, colors=new Float32Array(list.length*3);
  list.forEach((p,j)=> {
    const y=Math.sin(p.yaw/2), w=Math.cos(p.yaw/2), yy=y*(y+y), wy=w*(y+y), off=j*16;
    matrices[off]=(1-yy)*p.sx; matrices[off+1]=0; matrices[off+2]=-wy*p.sx; matrices[off+3]=0;
    matrices[off+4]=0; matrices[off+5]=p.sy; matrices[off+6]=0; matrices[off+7]=0;
    matrices[off+8]=wy*p.sz; matrices[off+9]=0; matrices[off+10]=(1-yy)*p.sz; matrices[off+11]=0;
    matrices[off+12]=p.x; matrices[off+13]=p.y; matrices[off+14]=p.z; matrices[off+15]=1;
    colors[j*3]=p.tint; colors[j*3+1]=p.tint; colors[j*3+2]=p.tint;
  });
  im.instanceColor=new THREE.InstancedBufferAttribute(colors,3);
  im.castShadow=shadows; im.receiveShadow=true; im.computeBoundingSphere(); out.push(im);
}

const geometryTemplates = new Map<string, THREE.BufferGeometry>();
function kitGeometry(key: string, make: () => THREE.BufferGeometry): THREE.BufferGeometry {
  let template = geometryTemplates.get(key);
  if (!template) {
    template = make();
    template.computeBoundingSphere();
    geometryTemplates.set(key, template);
  }
  // Placements own their instanceMemory attribute; immutable painted vertices are shared.
  const geometry = new THREE.BufferGeometry();
  for (const name of Object.keys(template.attributes)) geometry.setAttribute(name, template.getAttribute(name));
  if (template.index) geometry.setIndex(template.index);
  geometry.boundingSphere = template.boundingSphere?.clone() ?? null;
  return geometry;
}

/** Satellite district tiles are chosen only from the fogged view. */
export function districtTiles(view: PlayerView, wd: World, city: PlayerView['cities'][number]): number[] {
  if (city.population < 4) return [];
  const [col, row] = toOffset(city.pos), i = row * wd.w + col;
  if (i < 0 || i >= wd.w * wd.h || !wd.fog[i]) return [];
  const seed = i * 11 + city.id * 17;
  const ownTerritory = city.detail ? new Set(city.detail.territory.map((h) => {
    const [c, r] = toOffset(h); return r * wd.w + c;
  })) : null;
  const cityCentres = new Set(view.cities.map((c) => {
    const [cc, rr] = toOffset(c.pos); return rr * wd.w + cc;
  }));
  const worked = city.detail ? new Set(city.detail.yields.worked.map((h) => {
    const [c, r] = toOffset(h); return r * wd.w + c;
  })) : null;
  const candidates = Array.from({ length: 6 }, (_, e) => neighborIndex(i, e, wd.w, wd.h))
    .filter((n) => n >= 0 && wd.fog[n] > 0 && wd.owner[n] === city.owner
      && (!ownTerritory || ownTerritory.has(n)) && !cityCentres.has(n)
      && (wd.cls[n] === 2 || wd.cls[n] === 3 || wd.cls[n] === 5 || wd.cls[n] === 6 || wd.cls[n] === 7)
      && !view.tiles[n]?.improvement)
    .sort((a, b) => Number(!!worked?.has(b)) - Number(!!worked?.has(a)) || rnd(seed + a, 904) - rnd(seed + b, 904));
  const count = Math.min(candidates.length, 3, Math.ceil((city.population - 3) / 4));
  return candidates.slice(0, count);
}

/** Returns a globally instanced kit (no per-city draw calls). */
export function buildCityKit(view: PlayerView, wd: World, height: (x:number,z:number)=>number): THREE.Object3D[] {
  const houses: P[][][]=Array.from({length:5},()=>Array.from({length:7},()=>[])), landmarks: P[][]=Array.from({length:5},()=>[]);
  const grounds:P[]=[], streets:P[]=[], plazas:P[]=[], courts:P[]=[], markets:P[]=[], walls:P[]=[], towers:P[]=[], gates:P[]=[];
  const farms:P[]=[], pastures:P[]=[], pits:P[]=[], outskirts:P[]=[];
  const owners=new Map(view.players.map(p=>[p.id,p.faction]));
  let memorySource = -1;
  const pos=(x:number,z:number,yaw=0,sx=1,sy=1,sz=1,fog=2,tint=1):P=>({x,y:Math.max(0,height(x,z))+0.003,z,yaw,sx,sy,sz,fog,tint,tile:memorySource});
  for (let i=0;i<wd.w*wd.h;i++) {
    memorySource = i;
    if (!wd.fog[i]) continue;
    const kind=view.tiles[i]?.improvement?.kind;
    if (!kind) continue;
    const x=tileX(i,wd.w), z=tileZ(i,wd.w), yaw=rnd(i,83)*Math.PI*2;
    const p=pos(x,z,yaw,1,1,1,wd.fog[i],0.94+rnd(i,88)*0.12);
    if (kind==='Farm') farms.push(p);
    else if (kind==='Pasture') pastures.push(p);
    else pits.push(p);
  }
  for (const city of view.cities) {
    const [col,row]=toOffset(city.pos), i=row*wd.w+col;
    memorySource = i;
    if (i<0 || i>=wd.w*wd.h || !wd.fog[i]) continue;
    const x0=tileX(i,wd.w), z0=tileZ(i,wd.w), fog=wd.fog[i], style=styleOf(owners.get(city.owner));
    const seed=i*11+city.id*17;
    // A few visible, owned lowland neighbours carry worked plots even before
    // an explicit Farm improvement. Keep forests, buildings and fog clear.
    const nearby=new Set<number>();
    for (let e=0;e<6;e++) {
      const first=neighborIndex(i,e,wd.w,wd.h);
      if (first<0) continue;
      nearby.add(first);
      for (let e2=0;e2<6;e2++) nearby.add(neighborIndex(first,e2,wd.w,wd.h));
    }
    const plots=[...nearby].filter((n)=>n>=0 && wd.fog[n]>0 && wd.owner[n]===city.owner
      && (wd.cls[n]===2 || wd.cls[n]===3) && !view.tiles[n]?.improvement
      && !view.cities.some((other)=>{ const [c,r]=toOffset(other.pos); return r*wd.w+c===n; }))
      .sort((a,b)=>rnd(seed+a,1012)-rnd(seed+b,1012)).slice(0,4);
    for (const n of plots) outskirts.push(pos(tileX(n,wd.w),tileZ(n,wd.w),rnd(seed+n,1013)*Math.PI,
      0.82,1,0.82,wd.fog[n],0.88+rnd(seed+n,1014)*0.13));
    const count=Math.min(city.walls ? 36 : 45,16+Math.max(0,city.population)*4);
    const axis=rnd(seed,1)*Math.PI/3;
    // A continuous packed-earth footprint and six thin spokes tie the blocks together.
    grounds.push(pos(x0,z0,axis,1,1,1,fog,0.94));
    plazas.push(pos(x0,z0,axis,1,1,1,fog,0.9));
    for (let lane=0;lane<6;lane++) {
      const a=axis+lane*Math.PI/3;
      const x=x0+Math.cos(a)*0.245,z=z0+Math.sin(a)*0.245;
      streets.push(pos(x,z,Math.PI/2-a,1.0,1,1.05,fog,0.93));
      if (lane%2===0) {
        const cx=x0+Math.cos(a+0.22)*0.34,cz=z0+Math.sin(a+0.22)*0.34;
        courts.push(pos(cx,cz,a,0.7,1,0.7,fog,0.94));
        const mx=x0+Math.cos(a+0.22)*0.205,mz=z0+Math.sin(a+0.22)*0.205;
        markets.push(pos(mx,mz,a,1,1,1,fog,0.9+rnd(seed,lane+240)*0.12));
      }
    }
    const ringCounts=[8,12,18,24], radii=[0.15,0.265,0.37,0.46];
    let placed=0;
    for (let ring=1;ring<4 && placed<count;ring++) for (let slot=0;slot<ringCounts[ring] && placed<count;slot++) {
      const a=axis+(slot+0.5*(ring%2))*Math.PI*2/ringCounts[ring]+(rnd(seed,placed+10)-0.5)*0.075;
      // Three lanes point from the civic precinct to the rim; keep their plots open.
      const laneAngle=((a-axis)%(Math.PI*2)+Math.PI*2)%(Math.PI*2);
      if (ring>0 && Math.min(...[0,1,2,3,4,5].map(l=>Math.abs(Math.atan2(Math.sin(laneAngle-l*Math.PI/3),Math.cos(laneAngle-l*Math.PI/3)))))<0.065) continue;
      const sector=Math.floor((((a-axis)%(Math.PI*2)+Math.PI*2)%(Math.PI*2))/(Math.PI/3));
      const shape=0.83+0.24*rnd(seed,420+sector)+0.055*Math.sin(a*3+axis);
      const r=Math.min(0.49,(radii[ring]+(rnd(seed,placed+40)-0.5)*0.052)*shape);
      const x=x0+Math.cos(a)*r,z=z0+Math.sin(a)*r;
      const scale=(ring===3 ? 0.83 : 0.95)+rnd(seed,placed+60)*0.22;
      const variant=Math.min(6,Math.floor(rnd(seed,placed+120)*7));
      houses[style][variant].push(pos(x,z,a+Math.PI/2,scale,0.8+rnd(seed,placed+80)*0.35,scale*(0.84+rnd(seed,placed+90)*0.23),fog,0.87+rnd(seed,placed+100)*0.23));
      placed++;
    }
    // Larger settlements occupy nearby worked land, but keep farms, forests,
    // mountain slopes, and other city centres legible. Only observed tiles may contribute.
    if (city.population>=4) {
      for (const n of districtTiles(view, wd, city)) {
        memorySource = n;
        const x=tileX(n,wd.w),z=tileZ(n,wd.w),f=wd.fog[n];
        const dx=x-x0,dz=z-z0;
        const length=Math.hypot(dx,dz), sideX=-dz/length, sideZ=dx/length;
        grounds.push(pos(x,z,axis,0.85,1,0.85,f,0.93));
        courts.push(pos(x,z,axis,0.8,1,0.8,f,0.94));
        markets.push(pos(x+0.08,z-0.07,axis,1,1,1,f,0.94));
        streets.push(pos((x+x0)/2,(z+z0)/2,Math.atan2(dx,dz),0.8,1,1.8,f,0.93));
        // Shopfronts on both sides of the street join the satellite block to
        // the main town, instead of leaving a row of houses isolated in grass.
        for (let k=0;k<4;k++) for (const side of [-1,1]) {
          const along=0.52+k*0.075, offset=side*(0.072+0.018*rnd(seed+n,k+930));
          const variant=Math.min(6,Math.floor(rnd(seed+n,k*3+side+940)*7));
          houses[style][variant].push(pos(x0+dx*along+sideX*offset,z0+dz*along+sideZ*offset,
            Math.atan2(dz,dx)+Math.PI/2,0.82,0.82+0.18*rnd(seed+n,k+950),0.82,f,
            0.9+0.12*rnd(seed+n,k+960)));
        }
        for (let k=0;k<16;k++) {
          const a=axis+k*Math.PI*2/16+(rnd(seed+n,k+911)-0.5)*0.22;
          const r=0.16+0.16*rnd(seed+n,k+912);
          const variant=Math.min(6,Math.floor(rnd(seed+n,k+913)*7));
          houses[style][variant].push(pos(x+Math.cos(a)*r,z+Math.sin(a)*r,a+Math.PI/2,
            0.8+0.18*rnd(seed+n,k+914),0.8+0.25*rnd(seed+n,k+915),
            0.8+0.18*rnd(seed+n,k+916),f,0.9+0.16*rnd(seed+n,k+917)));
        }
      }
    }
    memorySource = i;
    landmarks[style].push(pos(x0,z0,axis,city.is_capital?1.5:1.05,city.is_capital?1.45:1,city.is_capital?1.5:1.05,fog,1));
    if (city.walls) {
      const f=0.78;
      for (let e=0;e<6;e++) {
        const a=CORNERS[e],b=CORNERS[(e+1)%6];
        const ax=x0+a[0]*f, az=z0+a[1]*f,bx=x0+b[0]*f,bz=z0+b[1]*f;
        const dx=bx-ax,dz=bz-az,len=Math.hypot(dx,dz), yaw=-Math.atan2(dz,dx);
        if (e===1) {
          for (const s of [-1,1]) walls.push(pos((ax+bx)/2+s*dx*0.31,(az+bz)/2+s*dz*0.31,yaw,len*0.38,1,1,fog,0.9));
          gates.push(pos((ax+bx)/2,(az+bz)/2,yaw,1,1,1,fog,0.92));
        } else walls.push(pos((ax+bx)/2,(az+bz)/2,yaw,len,1,1,fog,0.9));
        towers.push(pos(ax,az,0,1,1,1,fog,0.92));
      }
    }
  }
  const out:THREE.Object3D[]=[];
  for (let style=0;style<5;style++) {
    for (let variant=0;variant<7;variant++) if (houses[style][variant].length) instance(kitGeometry(`house-v2-${style}-${variant}`, () => house(style as Style,variant)),houses[style][variant],out);
    if (landmarks[style].length) instance(kitGeometry(`landmark-v2-${style}`, () => landmark(style as Style)),landmarks[style],out);
  }
  if (grounds.length) instance(kitGeometry('town-ground-v2', townGround),grounds,out,false);
  if (plazas.length) instance(kitGeometry('plaza', () => { const p=new Painter(); p.box(0,0.002,0,0.18,0.004,0.17,'#c5b18c'); return p.geometry(); }),plazas,out,false);
  if (streets.length) instance(kitGeometry('street-v2', () => { const p=new Painter(); p.box(0,0.002,0,0.066,0.004,0.35,'#b8a37e'); p.box(0,0.006,0,0.005,0.001,0.34,'#d4c2a0'); return p.geometry(); }),streets,out,false);
  if (courts.length) instance(kitGeometry('court-v2', courtGeometry),courts,out,false);
  if (markets.length) instance(kitGeometry('market-v2', marketGeometry),markets,out,false);
  if (walls.length) instance(kitGeometry('wall-v2', wallGeometry),walls,out);
  if (towers.length) instance(kitGeometry('tower-v2', towerGeometry),towers,out);
  if (gates.length) instance(kitGeometry('gate-v2', gateGeometry),gates,out);
  if (farms.length) instance(kitGeometry('farm-v2', patchGeometry),farms,out,false);
  if (outskirts.length) instance(kitGeometry('outskirts', outskirtsGeometry),outskirts,out,false);
  if (pastures.length) instance(kitGeometry('pasture-v2', pastureGeometry),pastures,out);
  if (pits.length) instance(kitGeometry('pit-v2', pitGeometry),pits,out);
  return out;
}
