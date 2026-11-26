// Lighting and inexpensive colour finish for the 3D map. A tiny procedural sky texture
// affects only the background; the unexplored ground remains the terrain's parchment shader.
import * as THREE from 'three';

export class MapAtmosphere {
  private sky: THREE.DataTexture;
  private fill: THREE.HemisphereLight;
  private hazeNear = new THREE.Color('#aebdc0');
  private hazeFar = new THREE.Color('#ddd2b8');

  constructor(private readonly scene: THREE.Scene, renderer: THREE.WebGLRenderer,
    sun: THREE.DirectionalLight, private readonly parchment: THREE.Color) {
    const pixels = new Uint8Array(4 * 128);
    const horizon = new THREE.Color('#d3c5aa');
    const zenith = new THREE.Color('#829fa9');
    for (let y = 0; y < 128; y++) {
      const t = THREE.MathUtils.smoothstep(y / 127, 0.04, 0.92);
      const c = horizon.clone().lerp(zenith, t * 0.88);
      pixels[y * 4] = Math.round(THREE.ColorManagement.fromWorkingColorSpace(c.clone(), THREE.SRGBColorSpace).r * 255);
      pixels[y * 4 + 1] = Math.round(THREE.ColorManagement.fromWorkingColorSpace(c.clone(), THREE.SRGBColorSpace).g * 255);
      pixels[y * 4 + 2] = Math.round(THREE.ColorManagement.fromWorkingColorSpace(c, THREE.SRGBColorSpace).b * 255);
      pixels[y * 4 + 3] = 255;
    }
    this.sky = new THREE.DataTexture(pixels, 1, 128, THREE.RGBAFormat);
    this.sky.colorSpace = THREE.SRGBColorSpace;
    this.sky.magFilter = THREE.LinearFilter;
    this.sky.needsUpdate = true;
    this.fill = new THREE.HemisphereLight('#e7edf0', '#91866e', 1.25);
    scene.add(this.fill);
    sun.color.set('#fff0d7');
    sun.intensity = 2.65;
    sun.shadow.radius = 1.6;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.19;
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    // The sky is used only if a close, low-tilt view looks beyond the terrain's edge.
    scene.background = this.sky;
    scene.fog = new THREE.Fog('#aebdc0', 18, 34);
  }

  update(dist: number): void {
    // Strategic view must read like a map, without a blue sky band at its rim.
    this.scene.background = dist < 7 ? this.sky : this.parchment;
    // Distance haze: cool air at close zoom, a warm paper-coloured haze from working zoom out, so the strategic
    // view does not wash remembered sepia (and the whole map) toward blue-grey.
    const fog = this.scene.fog as THREE.Fog;
    fog.color.copy(this.hazeNear).lerp(this.hazeFar, THREE.MathUtils.smoothstep(dist, 6, 14));
    fog.near = Math.max(6, dist * 1.1);
    fog.far = Math.max(15, dist * 2.4);
  }

  dispose(): void {
    this.scene.remove(this.fill);
    this.sky.dispose();
  }
}

/** One optional full-screen pass: small contact occlusion, highlight-only glow and a restrained grade. */
export class MapPost {
  private target: THREE.WebGLRenderTarget | null = null;
  private quad: THREE.Mesh;
  private quadScene = new THREE.Scene();
  private quadCamera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  private material: THREE.ShaderMaterial;
  private enabled: boolean;

  constructor(private readonly renderer: THREE.WebGLRenderer) {
    this.enabled = renderer.capabilities.isWebGL2 &&
      window.localStorage.getItem('legatum.map3d.quality') !== 'low';
    // Keep renderer.info inclusive of the scene and the final screen pass.
    renderer.info.autoReset = false;
    this.material = new THREE.ShaderMaterial({
      depthTest: false, depthWrite: false,
      uniforms: {
        uColor: { value: null }, uDepth: { value: null },
        uPixel: { value: new THREE.Vector2(1, 1) }, uNear: { value: 0.05 }, uFar: { value: 200 },
      },
      vertexShader: `varying vec2 vUv;
        void main() { vUv = uv; gl_Position = vec4(position.xy,0.0,1.0); }`,
      fragmentShader: `uniform sampler2D uColor; uniform sampler2D uDepth;
        uniform vec2 uPixel; uniform float uNear; uniform float uFar;
        varying vec2 vUv;
        float eyeDepth(float z) { return (uNear*uFar)/(uFar-z*(uFar-uNear)); }
        void main() {
          vec3 c = texture2D(uColor,vUv).rgb;
          float center = eyeDepth(texture2D(uDepth,vUv).r);
          vec2 d = uPixel*5.0;
          vec2 offsets[4];
          offsets[0]=vec2(d.x,0.0); offsets[1]=vec2(-d.x,0.0);
          offsets[2]=vec2(0.0,d.y); offsets[3]=vec2(0.0,-d.y);
          float occ=0.0; vec3 glow=vec3(0.0);
          for(int i=0;i<4;i++) {
            vec2 uv=clamp(vUv+offsets[i],vec2(0.0),vec2(1.0));
            float neighbour=eyeDepth(texture2D(uDepth,uv).r);
            occ+=smoothstep(0.015,0.28,center-neighbour)*(1.0-smoothstep(0.3,1.3,center-neighbour));
            vec3 n=texture2D(uColor,uv).rgb;
            glow+=max(n-vec3(0.76),vec3(0.0));
          }
          c*=1.0-0.035*occ;
          c+=glow*0.025;
          float l=dot(c,vec3(0.2126,0.7152,0.0722));
          c=mix(vec3(l),c,1.015);
          float green=smoothstep(0.015,0.11,c.g-max(c.r,c.b));
          float blue=smoothstep(0.025,0.14,c.b-c.r);
          c=mix(c,c*vec3(0.99,1.012,0.99),green*0.22);
          c=mix(c,c*vec3(0.98,1.005,1.025),blue*0.22);
          gl_FragColor=vec4(c,1.0);
          #include <colorspace_fragment>
        }`,
    });
    this.quad = new THREE.Mesh(new THREE.PlaneGeometry(2, 2), this.material);
    this.quadScene.add(this.quad);
  }

  resize(width: number, height: number): void {
    if (!this.enabled) return;
    const ratio = this.renderer.getPixelRatio();
    const w = Math.max(1, Math.round(width * ratio));
    const h = Math.max(1, Math.round(height * ratio));
    this.target?.dispose();
    this.target = new THREE.WebGLRenderTarget(w, h, { depthBuffer: true });
    this.target.texture.colorSpace = THREE.SRGBColorSpace;
    this.target.depthTexture = new THREE.DepthTexture(w, h);
    this.target.depthTexture.type = THREE.UnsignedIntType;
    this.material.uniforms.uColor.value = this.target.texture;
    this.material.uniforms.uDepth.value = this.target.depthTexture;
    this.material.uniforms.uPixel.value.set(1 / w, 1 / h);
  }

  render(scene: THREE.Scene, camera: THREE.PerspectiveCamera): void {
    this.renderer.info.reset();
    if (!this.target) { this.renderer.render(scene, camera); return; }
    this.material.uniforms.uNear.value = camera.near;
    this.material.uniforms.uFar.value = camera.far;
    this.renderer.setRenderTarget(this.target);
    this.renderer.render(scene, camera);
    this.renderer.setRenderTarget(null);
    this.renderer.render(this.quadScene, this.quadCamera);
  }

  dispose(): void {
    this.target?.dispose();
    this.quad.geometry.dispose();
    this.material.dispose();
  }
}
