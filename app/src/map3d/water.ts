// One sea surface for the whole fogged PlayerView. The RG channels carry sampled depth and
// visibility, so coastline detail costs one draw call and never reveals an unexplored tile.
import * as THREE from 'three';
import type { Field } from './world';
import { memoryWashGLSL } from './memoryWash';

// A single pair of CC0 normal maps is shared by every rebuild. Both are linear data.
let waterNormals: THREE.Texture[] | null = null;
function normalMaps(): THREE.Texture[] {
  if (!waterNormals) waterNormals = ['ocean-normal', 'wave-normal'].map((name) => {
    const texture = new THREE.TextureLoader().load(`${import.meta.env.BASE_URL}textures/map3d/${name}.webp`);
    texture.wrapS = texture.wrapT = THREE.RepeatWrapping;
    texture.minFilter = THREE.LinearMipmapLinearFilter;
    return texture;
  });
  return waterNormals;
}

export class MapWater {
  mesh: THREE.Mesh | null = null;
  private info: THREE.DataTexture | null = null;

  constructor(private readonly scene: THREE.Scene, private readonly uniforms: {
    uTime: { value: number }; uParch: { value: THREE.Color };
  }) {}

  update(field: Field, dirty?: readonly number[]): void {
    const tex = this.info;
    if (!tex) return;
    const data = tex.image.data as Uint8Array;
    for (const k of dirty ?? field.height.keys()) {
      data[k * 4] = Math.round(THREE.MathUtils.clamp(-field.height[k] * 400, 0, 255));
      data[k * 4 + 1] = Math.round(field.fog[k] * 255);
      data[k * 4 + 3] = 255;
    }
    tex.needsUpdate = true;
  }

  rebuild(field: Field): void {
    this.dispose();
    const { nx, nz, x0, z0, step } = field.grid;
    this.info = new THREE.DataTexture(new Uint8Array(nx * nz * 4), nx, nz, THREE.RGBAFormat);
    this.info.magFilter = THREE.LinearFilter;
    this.info.minFilter = THREE.LinearFilter;
    this.info.wrapS = this.info.wrapT = THREE.ClampToEdgeWrapping;
    this.info.generateMipmaps = false;
    this.update(field);
    const w = (nx - 1) * step, h = (nz - 1) * step;
    const fog = this.scene.fog as THREE.Fog;
    const normals = normalMaps();
    const material = new THREE.ShaderMaterial({
      transparent: true, depthWrite: false, fog: true,
      uniforms: {
        ...this.uniforms,
        uInfo: { value: this.info }, uBox: { value: new THREE.Vector4(x0, z0, w, h) },
        uNormalA: { value: normals[0] }, uNormalB: { value: normals[1] },
        fogColor: { value: fog.color }, fogNear: { value: fog.near }, fogFar: { value: fog.far },
      },
      vertexShader: `varying vec3 vW; varying vec3 vView;
        #include <fog_pars_vertex>
        void main() {
          vec4 w = modelMatrix * vec4(position, 1.0);
          vW = w.xyz; vView = cameraPosition - w.xyz;
          vec4 mvPosition = viewMatrix * w;
          gl_Position = projectionMatrix * mvPosition;
          #include <fog_vertex>
        }`,
      fragmentShader: `uniform sampler2D uInfo; uniform sampler2D uNormalA; uniform sampler2D uNormalB; uniform vec4 uBox;
        uniform float uTime; uniform vec3 uParch;
        varying vec3 vW; varying vec3 vView;
        #include <fog_pars_fragment>
        ${memoryWashGLSL}
        float hash21(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
        float noise2(vec2 p) {
          vec2 i = floor(p), f = fract(p); f = f*f*(3.0-2.0*f);
          return mix(mix(hash21(i), hash21(i+vec2(1,0)), f.x),
                     mix(hash21(i+vec2(0,1)), hash21(i+vec2(1,1)), f.x), f.y);
        }
        void main() {
          vec2 uv = (vW.xz-uBox.xy)/uBox.zw;
          vec4 inf = texture2D(uInfo, uv);
          float depth = inf.r, fogv = inf.g;
          float seen = smoothstep(0.62, 0.95, fogv);
          float known = smoothstep(0.12, 0.42, fogv);
          vec2 p = vW.xz;
          float waveFade = 1.0-smoothstep(7.0,15.0,length(vView));
          vec3 nA = texture2D(uNormalA, p*0.82 + vec2(uTime*0.018,uTime*0.011)).xyz*2.0-1.0;
          vec3 nB = texture2D(uNormalB, mat2(0.82,-0.57,0.57,0.82)*p*2.1 + vec2(-uTime*0.014,uTime*0.023)).xyz*2.0-1.0;
          float w0 = nA.x*0.7 + nB.y*0.3;
          float broad = noise2(p*0.28+vec2(uTime*0.012,0.0))*0.65
            + noise2(p*0.71+vec2(0.0,uTime*0.022))*0.35;
          vec3 normal = normalize(vec3((nA.x*1.15+nB.x*0.42)*waveFade,
                                       1.0,
                                       (nA.y*1.15+nB.y*0.42)*waveFade));
          vec3 viewDir = normalize(vView);
          float fresnel = pow(1.0-max(dot(normal,viewDir),0.0), 3.0);
          float shelfBed = noise2(p*0.72+vec2(11.0,37.0))*0.7
            + noise2(p*1.8+vec2(43.0,5.0))*0.3;
          float shelfDepth = clamp(depth + (broad-0.5)*0.04
            + (shelfBed-0.5)*0.055 + w0*0.006, 0.0, 1.0);
          float deep = smoothstep(0.12,0.65,shelfDepth);
          float shelf = 1.0-smoothstep(0.045,0.43,shelfDepth);
          float lagoon = 1.0-smoothstep(0.008,0.14,shelfDepth);
          vec3 c = mix(vec3(0.055,0.25,0.33),vec3(0.018,0.10,0.18),deep);
          c = mix(c,vec3(0.12,0.38,0.38),shelf*0.64);
          c = mix(c,vec3(0.43,0.59,0.47),lagoon*0.46);
          float sandbar = smoothstep(0.44,0.72,noise2(p*1.35+vec2(19.0,42.0)));
          c = mix(c,vec3(0.36,0.55,0.43),sandbar*shelf*0.22);
          c += vec3(0.014,0.024,0.028)*w0*waveFade;
          c += vec3(0.025,0.044,0.052)*(broad-0.5);
          float distantSwell = sin(p.x*1.65+p.y*0.57+broad*3.2+uTime*0.08);
          float distantBreak = noise2(p*0.9+vec2(13.0,7.0));
          c += vec3(0.027,0.052,0.062)*distantSwell*distantBreak*(1.0-waveFade);
          c = mix(c,vec3(0.45,0.62,0.65),fresnel*0.35);
          vec3 sunDir = normalize(vec3(-0.55,0.67,-0.48));
          float glint = pow(max(dot(normal,normalize(sunDir+viewDir)),0.0),28.0);
          c += vec3(0.36,0.31,0.22)*glint*0.32*waveFade*seen;
          // The shore has a broad wet shelf and several feathered, broken swash marks.
          float beach = 1.0-smoothstep(0.01,0.12,depth);
          c = mix(c,vec3(0.62,0.70,0.61),beach*0.09);
          float shoreNoise = noise2(p*5.4+vec2(uTime*0.026,0.0));
          float shoreDistance = abs(depth-(0.024+0.010*(shoreNoise-0.5)));
          float swash = 1.0-smoothstep(0.002,0.046,shoreDistance);
          float breakup = 0.25+0.75*noise2(p*9.0+vec2(uTime*0.06,0.0));
          float mist = (1.0-smoothstep(0.005,0.075,depth))*0.10;
          c = mix(c,vec3(0.78,0.86,0.79),(swash*breakup*0.18+mist)*known);
          float aerial = smoothstep(fogNear,fogFar,vFogDepth);
          c = mix(c,fogColor,aerial*0.55);
          c = mix(memoryWashWater(c),c,seen);
          c = mix(uParch,c,known);
          float alpha = mix(1.0,mix(0.91,0.99,smoothstep(0.01,0.40,depth)),known);
          gl_FragColor = vec4(c,alpha);
        }`,
    });
    this.mesh = new THREE.Mesh(new THREE.PlaneGeometry(w, h).rotateX(-Math.PI / 2), material);
    this.mesh.position.set(x0+w/2, 0, z0+h/2);
    this.scene.add(this.mesh);
  }

  dispose(): void {
    if (this.mesh) {
      this.scene.remove(this.mesh);
      this.mesh.geometry.dispose();
      (this.mesh.material as THREE.Material).dispose();
      this.mesh = null;
    }
    this.info?.dispose();
    this.info = null;
  }
}
