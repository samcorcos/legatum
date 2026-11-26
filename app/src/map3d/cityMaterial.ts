// World-space surface grain for the instanced city kit. It adds no downloaded
// texture and keeps each reused building from looking like flat painted plastic.
import * as THREE from 'three';
import { addMemoryWash } from './memoryWash';

export function cityMaterial(): THREE.MeshStandardMaterial {
  const mat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.92, side: THREE.DoubleSide });
  mat.onBeforeCompile = (shader) => {
    addMemoryWash(shader);
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', '#include <common>\nvarying vec3 vCityWorld; varying vec3 vCityNormal;')
      .replace('#include <begin_vertex>', `#include <begin_vertex>
        vCityWorld = (modelMatrix * instanceMatrix * vec4(position, 1.0)).xyz;
        vCityNormal = normal;`);
    shader.fragmentShader = shader.fragmentShader
      .replace('#include <common>', `#include <common>
        varying vec3 vCityWorld; varying vec3 vCityNormal;
        float cityHash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
        float cityNoise(vec2 p) {
          vec2 i = floor(p), f = fract(p); f *= f * (3.0 - 2.0 * f);
          return mix(mix(cityHash(i), cityHash(i + vec2(1.0, 0.0)), f.x),
            mix(cityHash(i + vec2(0.0, 1.0)), cityHash(i + vec2(1.0, 1.0)), f.x), f.y);
        }`)
      .replace('#include <color_fragment>', `#include <color_fragment>
        float wall = 1.0 - smoothstep(0.08, 0.38, abs(vCityNormal.y));
        float roofGrain = cityNoise(vCityWorld.xz * 85.0 + vCityWorld.y * 13.0);
        float plaster = cityNoise(vCityWorld.xy * 38.0 + vCityWorld.z * 13.0);
        float speckle = cityNoise(vCityWorld.xz * 166.0 + vCityWorld.y * 49.0);
        diffuseColor.rgb *= 0.88 + 0.19 * mix(roofGrain, plaster, wall) + 0.045 * speckle;
        float course = abs(fract(vCityWorld.y * 43.0 + plaster * 0.06) - 0.5);
        float mortar = 1.0 - smoothstep(0.035, 0.085, course);
        diffuseColor.rgb *= 1.0 - 0.095 * mortar * wall;
        float plasterWear = smoothstep(0.61, 0.78, cityNoise(vCityWorld.xz * 24.0 + vCityWorld.y * 31.0));
        diffuseColor.rgb *= 1.0 - wall * plasterWear * 0.07;
        float roofFace = smoothstep(0.40, 0.70, abs(vCityNormal.y));
        float firedTile = roofFace * smoothstep(1.10, 1.39,
          diffuseColor.r / max(diffuseColor.g, 0.05));
        float tileJoint = 1.0 - smoothstep(0.025, 0.075,
          abs(fract(vCityWorld.z * 82.0 + vCityWorld.x * 11.0) - 0.5));
        diffuseColor.rgb *= mix(vec3(1.0), vec3(1.13, 0.98, 0.86), firedTile * 0.65);
        diffuseColor.rgb *= 1.0 - firedTile * tileJoint * 0.12;
        diffuseColor.rgb = mix(diffuseColor.rgb, vec3(0.78, 0.73, 0.63), wall * 0.065);`);
  };
  mat.customProgramCacheKey = () => 'legatum-city-grain-memory-4';
  return mat;
}
