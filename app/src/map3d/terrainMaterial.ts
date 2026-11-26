// One draw call for the whole ground. The four vertex weights are a compact splat map made from
// the fogged PlayerView; all fine surface detail is deterministic world-space shader work.
import * as THREE from 'three';
import { memoryWashGLSL } from './memoryWash';
import { rockSurfaceGLSL, rockTexture } from './rockSurface';

const surface = `
  float terrainHash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
  float terrainNoise(vec2 p) {
    vec2 i = floor(p), f = fract(p); f = f * f * (3.0 - 2.0 * f);
    return mix(mix(terrainHash(i), terrainHash(i + vec2(1., 0.)), f.x),
               mix(terrainHash(i + vec2(0., 1.)), terrainHash(i + vec2(1., 1.)), f.x), f.y);
  }
  // Short, rotated blade clusters at close zoom. Their world-space cells do not
  // repeat per hex, and each stroke is only a few screen pixels at distance 3.
  float terrainTufts(vec2 p) {
    vec2 cell = floor(p), f = fract(p) - 0.5;
    float seed = terrainHash(cell + vec2(9.0, 17.0));
    f -= vec2(terrainHash(cell + 21.0), terrainHash(cell + 47.0)) * 0.42 - 0.21;
    float angle = (seed - 0.5) * 1.5;
    vec2 q = vec2(f.x * cos(angle) - f.y * sin(angle), f.x * sin(angle) + f.y * cos(angle));
    float blade = (1.0 - smoothstep(0.055, 0.14, abs(q.x))) *
      (1.0 - smoothstep(0.18, 0.36, abs(q.y)));
    return blade * smoothstep(0.3, 0.85, seed);
  }
  float terrainPebbles(vec2 p) {
    float aggregate = terrainNoise(p * 2.3 + vec2(2.1, 9.3));
    float packed = terrainNoise(p * 0.28 + vec2(4.7, 11.0));
    return smoothstep(0.54, 0.76, aggregate) * (0.5 + 0.5 * packed);
  }
  float terrainGrain(vec2 p, vec4 weights, float slope) {
    float coarse = terrainNoise(p * 5.5);
    float fine = terrainNoise(p * 24.0);
    float grass = terrainNoise(p * 37.0 + vec2(11., 4.));
    float dune = sin(p.x * 43.0 + p.y * 7.0 + coarse * 5.0) * 0.5 + 0.5;
    float rock = terrainNoise(p * 16.0 + vec2(coarse * 2.0, 0.0));
    float land = max(0.001, dot(weights, vec4(1.0)));
    float textureMix = dot(weights, vec4(grass * 0.6 + fine * 0.4,
      coarse * 0.45 + fine * 0.55, dune * 0.75 + fine * 0.25,
      fine * 0.65 + coarse * 0.35)) / land;
    float fineFade = 1.0 - smoothstep(0.5, 1.3, max(fwidth(p.x), fwidth(p.y)) * 24.0);
    return mix(coarse, mix(textureMix, rock, smoothstep(0.18, 0.65, slope)), fineFade);
  }
  vec3 terrainBump(vec3 surfPos, vec3 surfNormal, vec2 dh) {
    vec3 sx = normalize(dFdx(surfPos)), sy = normalize(dFdy(surfPos));
    vec3 r1 = cross(sy, surfNormal), r2 = cross(surfNormal, sx);
    float det = dot(sx, r1);
    return normalize(abs(det) * surfNormal - sign(det) * (dh.x * r1 + dh.y * r2));
  }
`;

export function terrainMaterial(parchment: THREE.Color): THREE.MeshStandardMaterial {
  const mat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.96, metalness: 0 });
  mat.onBeforeCompile = (sh) => {
    sh.uniforms.uParch = { value: parchment };
    sh.uniforms.uRockDiffuse = { value: rockTexture() };
    sh.vertexShader = sh.vertexShader
      .replace('#include <common>', '#include <common>\nattribute float fogv; attribute float shorev; attribute vec4 terrainWeights; varying float vFog; varying float vShore; varying vec4 vTerrainWeights; varying vec3 vW; varying vec3 vTerrainNormal;')
      .replace('#include <begin_vertex>', '#include <begin_vertex>\nvFog = fogv; vShore = shorev; vTerrainWeights = terrainWeights; vW = (modelMatrix * vec4(position, 1.0)).xyz; vTerrainNormal = normal;');
    sh.fragmentShader = sh.fragmentShader
      .replace('#include <common>', `#include <common>
        uniform vec3 uParch; varying float vFog; varying float vShore; varying vec4 vTerrainWeights;
        varying vec3 vW; varying vec3 vTerrainNormal;
        ${memoryWashGLSL}
        ${rockSurfaceGLSL}
        ${surface}`)
      .replace('#include <color_fragment>', `#include <color_fragment>
        vec2 ground = vW.xz;
        vec4 splat = max(vTerrainWeights, vec4(0.0));
        float slope = 1.0 - abs(normalize(vTerrainNormal).y);
        float macro = terrainNoise(ground * 0.09) * 0.28 + terrainNoise(ground * 0.36) * 0.37 + terrainNoise(ground * 1.25) * 0.35;
        float grain = terrainGrain(ground, splat, slope);
        float grass = splat.x, plains = splat.y, desert = splat.z, tundra = splat.w;
        float land = grass + plains + desert + tundra;
        float beach = smoothstep(-0.004, 0.009, vW.y) * (1.0 - smoothstep(0.057, 0.08, vW.y)) * vShore;
        float stone = smoothstep(0.18, 0.55, slope) * smoothstep(0.03, 0.18, vW.y);
        stone = max(stone, smoothstep(0.3, 0.8, vW.y) * smoothstep(0.1, 0.3, slope));
        stone = max(stone, smoothstep(0.24, 0.55, slope) * smoothstep(0.012, 0.065, vW.y));
        stone = max(stone, smoothstep(0.12, 0.38, slope) * smoothstep(0.1, 0.24, vW.y) * 0.82);
        // Snow settles on the upper shoulders, then thins on near-vertical faces.
        // The low frequency noise breaks the edge without making contour stripes.
        float summit = terrainNoise(ground * 0.71 + vec2(49.0, 8.0));
        float snowLine = mix(0.75, 0.93, summit);
        float snowEdge = (terrainNoise(ground * 5.2 + vec2(27.0, 13.0)) - 0.5) * 0.115
          + (terrainNoise(ground * 13.0) - 0.5) * 0.03;
        float snowSlope = smoothstep(0.34, 0.60, abs(normalize(vTerrainNormal).y));
        float snow = smoothstep(snowLine - 0.065, snowLine + 0.055, vW.y + snowEdge) * snowSlope;
        vec3 tint = vec3(1.0);
        tint *= 0.88 + 0.20 * macro;
        tint *= 0.93 + 0.14 * grain;
        tint *= 1.0 + grass * (grain - 0.5) * 0.14;
        tint *= 1.0 + plains * (grain - 0.5) * 0.1;
        tint *= 1.0 + desert * (grain - 0.5) * 0.16;
        tint *= 1.0 + tundra * (grain - 0.5) * 0.09;
        vec3 groundPalette = (grass * vec3(0.245, 0.335, 0.165) +
          plains * vec3(0.38, 0.32, 0.17) + desert * vec3(0.51, 0.37, 0.18) +
          tundra * vec3(0.34, 0.38, 0.33)) / max(land, 0.001);
        diffuseColor.rgb = mix(diffuseColor.rgb, groundPalette, clamp(land * 0.87, 0.0, 0.87));
        diffuseColor.rgb *= tint;
        float detailFade = (1.0 - smoothstep(4.0, 9.0, length(cameraPosition - vW)))
          * (1.0 - smoothstep(0.7, 1.4, max(fwidth(ground.x), fwidth(ground.y)) * 18.0));
        float tufts = terrainTufts(ground * 76.0);
        float pebbles = terrainPebbles(ground * 16.0 + 7.0);
        float soil = terrainNoise(ground * 13.0 + 3.0);
        diffuseColor.rgb *= 1.0 - grass * tufts * detailFade * 0.13;
        diffuseColor.rgb *= 1.0 + plains * pebbles * detailFade * 0.14;
        diffuseColor.rgb *= 0.985 + 0.03 * soil;
        float grassSoil = smoothstep(0.48, 0.73, terrainNoise(ground * 27.0 + vec2(31.0, 5.0)));
        diffuseColor.rgb = mix(diffuseColor.rgb, vec3(0.27, 0.25, 0.15), grass * grassSoil * detailFade * 0.10);
        float grassPatches = terrainNoise(ground * 3.5 + vec2(19.0, 7.0));
        float exposedSoil = smoothstep(0.57, 0.73, grassPatches);
        diffuseColor.rgb = mix(diffuseColor.rgb, vec3(0.32, 0.27, 0.16), grass * exposedSoil * 0.11);
        float lush = smoothstep(0.28, 0.7, terrainNoise(ground * 1.15 + vec2(6.0, 22.0)));
        diffuseColor.rgb *= 1.0 + grass * (lush - 0.5) * 0.10;
        float drySoil = smoothstep(0.49, 0.76, terrainNoise(ground * 5.0 + vec2(29.0, 4.0)));
        diffuseColor.rgb = mix(diffuseColor.rgb, vec3(0.36, 0.27, 0.14), plains * drySoil * 0.16);
        float dryRidges = sin(ground.x * 62.0 + ground.y * 13.0 + terrainNoise(ground * 4.0) * 4.0);
        diffuseColor.rgb *= 1.0 + desert * dryRidges * detailFade * 0.1;
        float crack = 1.0 - smoothstep(0.015, 0.06, abs(terrainNoise(ground * 24.0) - 0.5));
        diffuseColor.rgb *= 1.0 - desert * crack * 0.07;
        float meadowPatch = smoothstep(0.42, 0.72, terrainNoise(ground * 1.6 + vec2(17.0, 9.0)));
        diffuseColor.rgb = mix(diffuseColor.rgb, diffuseColor.rgb * vec3(0.82, 0.94, 0.79), grass * meadowPatch * 0.22);
        float broadDry = smoothstep(0.40, 0.67, terrainNoise(ground * 0.48 + vec2(71.0, 18.0)));
        float broadWet = smoothstep(0.43, 0.70, terrainNoise(ground * 0.72 + vec2(8.0, 83.0)));
        float mottled = terrainNoise(ground * 2.1 + vec2(33.0, 14.0));
        float meadow = grass * broadWet * (1.0 - broadDry * 0.56);
        diffuseColor.rgb = mix(diffuseColor.rgb, vec3(0.25, 0.39, 0.18), meadow * 0.32);
        float worn = (grass * 0.57 + plains * 0.88) * broadDry * (0.58 + 0.42 * mottled);
        diffuseColor.rgb = mix(diffuseColor.rgb, vec3(0.48, 0.36, 0.20), worn * 0.33);
        // One continuous climate field crosses tile edges. The splat weights keep
        // its dry and lush palettes grounded in the known terrain under each pixel.
        float climate = terrainNoise(ground * 0.29 + vec2(81.0, 22.0)) * 0.76
          + terrainNoise(ground * 0.11 + vec2(4.0, 13.0)) * 0.24;
        float moisture = smoothstep(0.41, 0.59, climate);
        vec3 regionPalette = mix(vec3(0.54, 0.40, 0.20), vec3(0.16, 0.40, 0.19), moisture);
        diffuseColor.rgb = mix(diffuseColor.rgb, regionPalette,
          (grass * 0.78 + plains * 0.88) * 0.76);
        vec2 plotUV = ground * 2.25 + vec2(7.3, 19.7);
        vec2 plotCell = floor(plotUV), plotLocal = fract(plotUV);
        float plotSeed = terrainHash(plotCell);
        float cultivated = smoothstep(0.58, 0.76, broadDry * 0.65 +
          terrainNoise(ground * 0.86 + vec2(37.0, 52.0)) * 0.35);
        float plot = cultivated * (grass * 0.62 + plains * 0.8) * step(0.30, plotSeed);
        float plotEdge = 1.0 - smoothstep(0.025, 0.075, min(min(plotLocal.x, plotLocal.y),
          min(1.0 - plotLocal.x, 1.0 - plotLocal.y)));
        float furrow = 0.5 + 0.5 * sin((plotLocal.x + mottled * 0.04) * 75.0);
        diffuseColor.rgb = mix(diffuseColor.rgb, diffuseColor.rgb * vec3(0.80, 0.79, 0.68),
          plot * (0.12 + 0.08 * plotEdge + 0.055 * furrow));
        vec3 sand = vec3(0.72, 0.59, 0.34) * (0.88 + 0.22 * grain);
        diffuseColor.rgb = mix(diffuseColor.rgb, sand, beach * (1.0 - stone * 0.95) * 0.92);
        float wetEdge = smoothstep(-0.004, 0.006, vW.y) * (1.0 - smoothstep(0.018, 0.038, vW.y)) * vShore;
        diffuseColor.rgb = mix(diffuseColor.rgb, vec3(0.31, 0.26, 0.17) * (0.85 + 0.3 * grain), wetEdge * (1.0 - stone * 0.9) * 0.92);
        if (stone > 0.01) {
          vec3 rockSample = rockTriplanar(vW * 3.2, normalize(vTerrainNormal));
          float rockGrain = dot(rockSample, vec3(0.299, 0.587, 0.114));
          float face = smoothstep(0.31, 0.58, slope);
          float rangeDip = terrainNoise(ground * 0.22 + vec2(43.0, 9.0));
          float dip = dot(ground, vec2(0.035 + rangeDip * 0.04, -0.055 + rangeDip * 0.035));
          float layerWarp = (terrainNoise(ground * 1.7 + vec2(5.0, 13.0)) - 0.5) * 0.42
            + (terrainNoise(ground * 4.3 + vec2(19.0, 2.0)) - 0.5) * 0.13;
          float layer = vW.y + dip + layerWarp;
          float bandWidth = mix(29.0, 41.0, terrainNoise(ground * 0.92 + vec2(7.0, 31.0)));
          float broadBed = sin(layer * bandWidth);
          float thinBed = sin(layer * (bandWidth * 2.19) + terrainNoise(ground * 2.8) * 1.7);
          float strata = smoothstep(-0.52, 0.65, broadBed * 0.76 + thinBed * 0.24);
          // Use a stable world-space downslope axis. A per-fragment normal axis
          // bent the previous gullies into zigzag contours.
          float across = dot(ground, vec2(0.82, 0.57));
          float meander = (terrainNoise(ground * 1.8 + vec2(31.0, 3.0)) - 0.5) * 0.19;
          float drainage = terrainNoise(vec2((across + meander) * 10.0, vW.y * 0.38));
          float gully = (1.0 - smoothstep(0.27, 0.43, drainage)) * face;
          float fineDrain = terrainNoise(vec2((across + meander) * 24.0 + 9.0, vW.y * 0.65));
          float rill = (1.0 - smoothstep(0.29, 0.42, fineDrain)) * face;
          float fracture = terrainNoise(ground * 11.0 + vec2(layer * 2.0, 0.0));
          float cavity = clamp(gully * (0.63 + fracture * 0.25) + rill * 0.17
            + (1.0 - strata) * face * 0.26, 0.0, 1.0);
          vec3 rock = mix(vec3(0.31, 0.32, 0.31), vec3(0.48, 0.43, 0.36), smoothstep(0.12, 0.67, rockGrain));
          rock *= 0.93 + face * (strata - 0.5) * 0.26;
          rock *= 1.0 - 0.42 * cavity * smoothstep(0.16, 0.35, vW.y);
          float talus = (1.0 - smoothstep(0.23, 0.37, vW.y)) * stone;
          rock = mix(rock, vec3(0.42, 0.39, 0.32) * (0.78 + 0.38 * terrainNoise(ground * 24.0)), talus * 0.76);
          diffuseColor.rgb = mix(diffuseColor.rgb, rock, stone * 0.92);
        }
        // Replace the whole tinted vertex colour on snow, so underlying rock,
        // ambient tint and the terrain palette cannot turn the cap grey.
        diffuseColor.rgb = mix(diffuseColor.rgb, vec3(0.88, 0.92, 0.98) * (0.96 + 0.06 * grain), snow);`)
      .replace('#include <roughnessmap_fragment>', '#include <roughnessmap_fragment>\nroughnessFactor = mix(roughnessFactor, 0.82, snow);')
      .replace('#include <normal_fragment_maps>', `#include <normal_fragment_maps>
        float surfaceSlope = 1.0 - abs(normalize(vTerrainNormal).y);
        float surfaceHeight = terrainGrain(vW.xz, max(vTerrainWeights, vec4(0.0)), surfaceSlope);
        float rockHeight = terrainNoise(vW.xz * 17.0) * 0.65 + terrainNoise(vW.xz * 31.0) * 0.35;
        float rockMask = smoothstep(0.12, 0.4, surfaceSlope) * smoothstep(0.08, 0.2, vW.y);
        surfaceHeight = mix(surfaceHeight, rockHeight, rockMask);
        float bumpLand = smoothstep(0.55, 0.9, dot(vTerrainWeights, vec4(1.0))) * smoothstep(0.07, 0.14, vW.y);
        float bumpFade = 1.0 - smoothstep(0.04, 0.09, max(fwidth(vW.x), fwidth(vW.z)));
        normal = terrainBump(-vViewPosition, normal, vec2(dFdx(surfaceHeight), dFdy(surfaceHeight)) * 0.018 * max(bumpLand, rockMask) * bumpFade);`)
      .replace('#include <dithering_fragment>', `#include <dithering_fragment>
        // Cool sky fill is applied after lighting, before the existing memory wash.
        // This also avoids the old relief-grade injection that greyed every peak.
        vec3 litSnow = gl_FragColor.rgb;
        vec3 skyFill = vec3(0.48, 0.59, 0.76);
        gl_FragColor.rgb = mix(litSnow, max(litSnow, skyFill), snow * 0.72);
        float seen = clamp((vFog - 0.62) / 0.33, 0.0, 1.0);
        seen = seen * seen * (3.0 - 2.0 * seen);
        float known = smoothstep(0.12, 0.42, vFog);
        vec3 c = gl_FragColor.rgb;
        c = mix(memoryWash(c), c, seen);
        float hatch = smoothstep(0.35, 0.5, abs(fract((vW.x + vW.z) * 9.0) - 0.5));
        c = mix(uParch * (0.93 + 0.07 * hatch), c, known);
        gl_FragColor.rgb = c;`);
  };
  mat.customProgramCacheKey = () => 'terrain-splat-rock-15';
  return mat;
}
