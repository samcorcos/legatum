// Retire the small grey shard prop mesh. The continuous ridge surface now
// carries the rock detail; the prop silhouette made it look fragmented.
import * as THREE from 'three';

export function removeRockShards(group: THREE.Group): void {
  for (const child of [...group.children]) {
    if (!(child instanceof THREE.Mesh)) continue;
    const material = child.material;
    if (Array.isArray(material) || material.customProgramCacheKey() !== 'flora-rock-memory-v1') continue;
    group.remove(child);
    child.geometry.dispose();
    material.dispose();
  }
}
