import { describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { fadeOwn, ownMaterials } from './ownMaterials';

describe('ownMaterials', () => {
  it('fading a fallen figure leaves a live figure sharing its material opaque', () => {
    const shared = new THREE.MeshStandardMaterial();
    shared.onBeforeCompile = () => {};
    shared.customProgramCacheKey = () => 'team-mask';
    const ghost = new THREE.Group().add(new THREE.Mesh(new THREE.BufferGeometry(), shared));
    const live = new THREE.Group().add(new THREE.Mesh(new THREE.BufferGeometry(), shared));
    ownMaterials(ghost);
    fadeOwn(ghost, 0.05);
    const gm = (ghost.children[0] as THREE.Mesh).material as THREE.Material;
    const lm = (live.children[0] as THREE.Mesh).material as THREE.Material;
    expect(gm).not.toBe(shared);
    expect(gm.opacity).toBe(0.05);
    expect(gm.onBeforeCompile).toBe(shared.onBeforeCompile);
    expect(gm.customProgramCacheKey()).toBe('team-mask');
    expect(lm).toBe(shared);
    expect(lm.opacity).toBe(1);
    expect(lm.transparent).toBe(false);
  });
  it('copies once, so a later call keeps the faded copy', () => {
    const ghost = new THREE.Group().add(new THREE.Mesh(new THREE.BufferGeometry(), new THREE.MeshBasicMaterial()));
    ownMaterials(ghost);
    const first = (ghost.children[0] as THREE.Mesh).material;
    ownMaterials(ghost);
    expect((ghost.children[0] as THREE.Mesh).material).toBe(first);
  });
});
