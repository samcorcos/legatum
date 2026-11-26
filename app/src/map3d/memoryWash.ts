// Shared post-lighting treatment for known tiles outside the current view.
// The input is interpolated per terrain vertex or supplied per instance.
export const memoryWashGLSL = `
  // Remembered tiles read like the 2D map's faded ink: the colour's luminance mapped onto a ramp from dark
  // umber to the 2D map's paper (#f5eedb), blended most of the way, a little darker, so relief and light stay.
  vec3 memoryRamp(float l) {
    // Linear-space stops: umber, warm tan, the 2D map's paper. The tan midtone carries the sepia hue, which a
    // straight umber-to-paper line loses once it is converted to sRGB.
    vec3 umber = vec3(0.090, 0.052, 0.022);
    vec3 tan = vec3(0.560, 0.410, 0.230);
    vec3 paper = vec3(0.913, 0.846, 0.701);
    float t = clamp(l, 0.0, 1.0);
    return t < 0.5 ? mix(umber, tan, t * 2.0) : mix(tan, paper, t * 2.0 - 1.0);
  }
  vec3 memoryWash(vec3 color) {
    float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
    // Lifted toward the 2D map's pale remembered land, relief kept by the luminance term.
    return mix(color, memoryRamp(0.22 + luma * 0.85), 0.72) * 0.95;
  }
  // Water: lift the dark sea further so remembered sea reads as pale muted water, not a dark slab.
  vec3 memoryWashWater(vec3 color) {
    float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
    return mix(color, memoryRamp(0.40 + luma * 0.7), 0.62) * 0.97;
  }
`;
/** Inject into a standard/basic material's compiled shader. The geometry owns the attribute. */
export function addMemoryWash(shader: { vertexShader: string; fragmentShader: string }, attribute = 'instanceMemory'): void {
  shader.vertexShader = shader.vertexShader
    .replace('#include <common>', `#include <common>\nattribute float ${attribute}; varying float vRemembered;`)
    .replace('#include <begin_vertex>', `#include <begin_vertex>\nvRemembered = ${attribute};`);
  shader.fragmentShader = shader.fragmentShader
    .replace('#include <common>', `#include <common>\nvarying float vRemembered;\n${memoryWashGLSL}`)
    .replace('#include <dithering_fragment>', '#include <dithering_fragment>\ngl_FragColor.rgb = mix(gl_FragColor.rgb, memoryWash(gl_FragColor.rgb), vRemembered);');
}

export const remembered = (fog: number): number => fog === 1 ? 1 : 0;
