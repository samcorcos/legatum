import * as THREE from 'three';
import { remembered } from './memoryWash';
import { tileAt, type World } from './world';

/** Refresh the merged layers' remembered-land attributes without reallocating their meshes. */
export function refreshMemory(group: THREE.Group, world: World): void {
  group.traverse((object) => {
    const mesh = object as THREE.Mesh;
    const geometry = mesh.geometry;
    if (!geometry) return;
    const instances = geometry.getAttribute('instanceMemory') as THREE.InstancedBufferAttribute | undefined;
    if (instances && mesh instanceof THREE.InstancedMesh) {
      const tiles = mesh.userData.memoryTiles as ArrayLike<number> | undefined;
      const matrix = mesh.instanceMatrix.array;
      for (let j = 0; j < mesh.count; j++) {
        const i = tiles?.[j] ?? tileAt(matrix[j * 16 + 12], matrix[j * 16 + 14], world.w, world.h);
        instances.setX(j, remembered(world.fog[i]));
      }
      instances.needsUpdate = true;
    }
    const vertices = geometry.getAttribute('vertexMemory') as THREE.BufferAttribute | undefined;
    if (vertices) {
      const tiles = mesh.userData.memoryTiles as ArrayLike<number> | undefined;
      const road = mesh.userData.memoryRoad as { from: ArrayLike<number>; to: ArrayLike<number>; blend: ArrayLike<number> } | undefined;
      const positions = geometry.getAttribute('position') as THREE.BufferAttribute;
      for (let j = 0; j < vertices.count; j++) {
        if (road) {
          const t = road.blend[j];
          vertices.setX(j, remembered(world.fog[road.from[j]]) * (1 - t) + remembered(world.fog[road.to[j]]) * t);
          continue;
        }
        const i = tiles?.[j] ?? tileAt(positions.getX(j), positions.getZ(j), world.w, world.h);
        vertices.setX(j, remembered(world.fog[i]));
      }
      vertices.needsUpdate = true;
    }
  });
}
