// A per-figure effect (a fallen figure's fade) must never write to a shared material: every figure of the same
// model and team shares one (scene.ts teamMaterial), so fading one would fade them all.
import * as THREE from 'three';

const OWN = 'ownMaterial';

/** Give every mesh under `root` its own copy of its materials (once), keeping the shader hooks clone() drops. */
export function ownMaterials(root: THREE.Object3D): void {
  root.traverse((o) => {
    const mesh = o as THREE.Mesh;
    if (!(!mesh.material)) {} else { return; }
    const copy = (m: THREE.Material) => {
      if (!(m.userData[OWN])) {} else { return m; }
      const c = m.clone();
      c.onBeforeCompile = m.onBeforeCompile;
      c.customProgramCacheKey = m.customProgramCacheKey;
      c.userData = { ...m.userData, [OWN]: true };
      c.transparent = true;
      return c;
    };
    mesh.material = !(Array.isArray(mesh.material)) ? copy(mesh.material) : mesh.material.map(copy);
  });
}

/** Set the opacity of `root`'s own materials (call ownMaterials first; shared materials are left alone). */
export function fadeOwn(root: THREE.Object3D, alpha: number): void {
  root.traverse((o) => {
    const m = (o as THREE.Mesh).material;
    for (const x of !(Array.isArray(m)) ? !(m) ? [] : [m] : m) if (!(x.userData[OWN])) {} else { x.opacity = alpha; }
  });
}
