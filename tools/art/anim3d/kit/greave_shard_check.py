"""Skin shards inside the greave silhouette, as the close-up cameras see them.

    blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/greave_shard_check.py -- <unit.glb> [step=3]

For every LOD, every third frame of walk and attack, and views from the front, both
sides, three-quarter and the back of each knee, rays are cast on a 160 x 160 grid
over the shin (bind-to-pose skinning in numpy, BVH in Blender). A ray whose first
hit is skin but which hits a greave less than 2 cm behind that skin is a shard:
skin poking through, or seen through a gap in, the greave. Skin seen through the
open back seam has the greave much farther behind it and is not counted.
"""
import sys
from pathlib import Path
import numpy as np
from mathutils import Vector
from mathutils.bvhtree import BVHTree

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_glb import GLB

argv = sys.argv[sys.argv.index('--') + 1:]
g = GLB(argv[0]); step = int(argv[1]) if len(argv) > 1 else 3
d = g.doc; skin = d['skins'][0]; bones = [d['nodes'][j]['name'] for j in skin['joints']]
inv = np.array(g.accessor(skin['inverseBindMatrices'])).reshape(-1, 4, 4).transpose(0, 2, 1)
res = {}
for m in d['meshes']:
    pr = m['primitives'][0]; a = pr['attributes']
    P = np.array(g.accessor(a['POSITION'])); J = np.array(g.accessor(a['JOINTS_0'])).astype(int); W = np.array(g.accessor(a['WEIGHTS_0']))
    T = np.array([i[0] for i in g.accessor(pr['indices'])]).reshape(-1, 3)
    part = np.empty(len(P), object)
    for r in m['extras']['vertex_regions']: part[r['start']:r['start'] + r['count']] = r['part']
    tri_part = part[T[:, 0]]
    total = 0; worst = (0, None)
    for clip in [c for c in d['animations'] if c['name'] in ('walk', 'attack')]:
        dur = max(g.accessor(s['input'])[-1][0] for s in clip['samplers'])
        for f in range(0, int(round(dur * 30)) + 1, step):
            w = g.worlds(clip, f / 30); M = np.array([w[j] for j in skin['joints']]).reshape(-1, 4, 4).transpose(0, 2, 1) @ inv
            X = np.einsum('nk,nkij,nj->ni', W, M[J], np.c_[P, np.ones(len(P))])[:, :3]
            bvh = BVHTree.FromPolygons([tuple(v) for v in X], [tuple(t) for t in T])
            n_f = 0
            for sd in 'LR':
                k = bones.index('leg_' + sd); kn = M[k][:3, 3] + M[k][:3, :3] @ np.linalg.inv(inv[k])[:3, 3] * 0
                kn = (M[k] @ np.linalg.inv(inv[k]))[:3, 3]; fo = (M[bones.index('foot_' + sd)] @ np.linalg.inv(inv[bones.index('foot_' + sd)]))[:3, 3]
                c = (kn + fo) / 2; ax = (kn - fo) / np.linalg.norm(kn - fo)
                for vd in ([0, 0, 1.], [1., 0, 0], [-1., 0, 0], [.7, 0, .7], [-.7, 0, .7], [0, 0, -1.]):
                    v = np.array(vd); v = v - ax * (v @ ax); v /= np.linalg.norm(v); s_ = np.cross(ax, v)
                    for i in np.linspace(-.25, .25, 160):
                        for j in np.linspace(-.09, .09, 160 // 3):
                            o = c + ax * i + s_ * j + v * .5
                            hit = bvh.ray_cast(Vector(o), Vector(-v), 1.0)
                            if hit[0] is None or tri_part[hit[2]] != 'body': continue
                            h2 = bvh.ray_cast(hit[0] - Vector(v) * 1e-4, Vector(-v), .02)
                            if h2[0] is not None and tri_part[h2[2]] == 'greave': n_f += 1
            total += n_f
            if n_f > worst[0]: worst = (n_f, f'{clip["name"]} {f}')
    res[m['name']] = {'shard_rays': total, 'worst_frame': worst}
print('SHARDS', res, flush=True)
