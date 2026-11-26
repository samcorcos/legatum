// Small CC0 rock colour map sampled on all three world axes, so the ridge mesh
// keeps one continuous material without UV seams or per-tile draws.
import * as THREE from 'three';

let texture: THREE.Texture | null = null;
export function rockTexture(): THREE.Texture {
  if (!texture) {
    texture = typeof document === 'undefined' ? new THREE.Texture()
      : new THREE.TextureLoader().load(`${import.meta.env.BASE_URL}textures/map3d/rock-strata.webp`);
    texture.wrapS = texture.wrapT = THREE.RepeatWrapping;
    texture.colorSpace = THREE.SRGBColorSpace;
  }
  return texture;
}

export const rockSurfaceGLSL = `
  uniform sampler2D uRockDiffuse;
  vec3 rockTriplanar(vec3 p, vec3 n) {
    vec3 blend = pow(abs(n), vec3(4.0));
    blend /= max(dot(blend, vec3(1.0)), 0.001);
    return texture2D(uRockDiffuse, p.zy).rgb * blend.x +
           texture2D(uRockDiffuse, p.xz).rgb * blend.y +
           texture2D(uRockDiffuse, p.xy).rgb * blend.z;
  }
`;
