// Seven CC0 Kenney tree forms, flattened once into vertex-coloured geometry for
// the instanced forest. See app/public/models/PROVENANCE-trees.md.
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

export const TREE_MODELS = [
  'tree-oak', 'tree-detailed', 'tree-default', 'tree-plateau',
  'tree-pinedefaulta', 'tree-pinedefaultb', 'tree-pinerounda',
] as const;
export type TreeModel = typeof TREE_MODELS[number];

const loader = new GLTFLoader();
const cache = new Map<TreeModel, Promise<THREE.BufferGeometry>>();

function flattenTree(root: THREE.Object3D, conifer: boolean): THREE.BufferGeometry {
  root.updateMatrixWorld(true);
  const positions: number[] = [], colors: number[] = [];
  const point = new THREE.Vector3();
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    const geometry = object.geometry as THREE.BufferGeometry;
    const source = geometry.getAttribute('position');
    const index = geometry.getIndex();
    const materials = Array.isArray(object.material) ? object.material : [object.material];
    const groups = geometry.groups.length ? geometry.groups : [{ start: 0, count: index?.count ?? source.count, materialIndex: 0 }];
    for (const group of groups) {
      const mat = materials[group.materialIndex ?? 0] as THREE.Material;
      const foliage = /leaf|defaultMat/i.test(mat.name);
      const color = foliage
        ? new THREE.Color(conifer ? '#596e50' : '#788b55')
        : new THREE.Color('#76644a');
      for (let k = group.start; k < group.start + group.count; k++) {
        const j = index ? index.getX(k) : k;
        point.fromBufferAttribute(source, j).applyMatrix4(object.matrixWorld);
        positions.push(point.x, point.y, point.z);
        colors.push(color.r, color.g, color.b);
      }
    }
  });
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
  geometry.computeBoundingBox();
  const box = geometry.boundingBox!;
  geometry.translate(-(box.min.x + box.max.x) * 0.5, -box.min.y, -(box.min.z + box.max.z) * 0.5);
  geometry.scale(0.23, 0.23, 0.23);
  geometry.computeVertexNormals();
  return geometry;
}

export function loadTreeModel(model: TreeModel): Promise<THREE.BufferGeometry> {
  let pending = cache.get(model);
  if (!pending) {
    pending = loader.loadAsync(`${import.meta.env.BASE_URL}models/trees/${model}.glb`)
      .then((gltf) => flattenTree(gltf.scene, model.includes('pine')));
    cache.set(model, pending);
  }
  return pending;
}
