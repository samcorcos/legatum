"""Shards as the close-up cameras see them: skin through greaves and helmet, loose scraps.

    blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/greave_shard_check.py -- <unit.glb> [step=3]

Every LOD, every third frame (bind-to-pose skinning in numpy, BVH in Blender), rays
cast on grids from several directions:
- shard_rays (walk and attack, over each shin from the front, both sides,
  three-quarter and the back of each knee, 160 x 160 grid): first hit skin with a greave
  less than 2 cm behind it, i.e. skin poking through, or seen through a gap in, the
  greave. Skin seen through the open back seam has the greave farther behind it.
- helmet_shard_rays (every clip, around the head from the front, both sides,
  three-quarter and the map's high front camera): first hit skin with the helmet less
  than 2 cm behind it, i.e. face skin poking through the face and cheek plates.
- scrap_rays (every clip, around both shoulders and upper arms): first hit a loose
  scrap, a small island (<= 80 triangles, welded by position) more than 2.5 cm from the
  rest of the mesh in bind pose (garment or skin only; shield, spear and crest pieces
  are loose by design), such as a torn source-sleeve piece left floating
  beside a new sleeve (r12's orange-red shard at the shield arm). Scraps are listed.
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
binv = np.linalg.inv(inv)


def loose_scraps(P, T, part, J, W):
    _, wid = np.unique(np.round(P / 1e-4).astype(np.int64), axis=0, return_inverse=True); wid = wid.ravel()
    par = np.arange(wid.max() + 1)

    def find(x):
        while par[x] != x: par[x] = par[par[x]]; x = par[x]
        return x
    for t in wid[T]:
        ra = find(t[0]); par[find(t[1])] = ra; par[find(t[2])] = ra
    root = np.array([find(x) for x in wid[T[:, 0]]]); mask = np.zeros(len(T), bool); found = []
    used = np.unique(T)
    for r_, c_ in zip(*np.unique(root, return_counts=True)):
        if c_ > 80 or part[T[root == r_][0, 0]] not in ('garment', 'body'): continue
        vi = np.unique(T[root == r_]); O = P[np.setdiff1d(used, vi)]
        gap = np.sqrt(((P[vi][:, None] - O[None]) ** 2).sum(2).min(1)).max()
        if gap > .025:
            mask |= root == r_; v0 = vi[0]
            found.append({'tris': int(c_), 'part': part[v0], 'at': P[vi].mean(0).round(3).tolist(),
                          'bone': bones[J[v0][W[v0].argmax()]], 'gap_m': round(float(gap), 3)})
    return mask, found


def grid_hits(bvh, c, ax, views, half_len, half_w, n, nj, test):
    hits = 0
    for vd in views:
        v = np.array(vd, float); v = v - ax * (v @ ax); v /= np.linalg.norm(v); s_ = np.cross(ax, v)
        for i in np.linspace(-half_len, half_len, n):
            for j in np.linspace(-half_w, half_w, nj):
                o = c + ax * i + s_ * j + v * .5
                hit = bvh.ray_cast(Vector(o), Vector(-v), 1.0)
                if hit[0] is not None and test(hit, v, bvh): hits += 1
    return hits


res = {}
for m in d['meshes']:
    pr = m['primitives'][0]; a = pr['attributes']
    P = np.array(g.accessor(a['POSITION'])); J = np.array(g.accessor(a['JOINTS_0'])).astype(int); W = np.array(g.accessor(a['WEIGHTS_0']))
    T = np.array([i[0] for i in g.accessor(pr['indices'])]).reshape(-1, 3)
    part = np.empty(len(P), object)
    for r in m['extras']['vertex_regions']: part[r['start']:r['start'] + r['count']] = r['part']
    tri_part = part[T[:, 0]]
    scrap, scraps = loose_scraps(P, T, part, J, W)

    def behind(kind):
        def test(hit, v, bvh):
            if tri_part[hit[2]] != 'body': return False
            h2 = bvh.ray_cast(hit[0] - Vector(v) * 1e-4, Vector(-v), .02)
            return h2[0] is not None and tri_part[h2[2]] == kind
        return test
    greave_t, helmet_t = behind('greave'), behind('helmet')
    scrap_t = lambda hit, v, bvh: bool(scrap[hit[2]])
    tot = {'shard_rays': 0, 'helmet_shard_rays': 0, 'scrap_rays': 0}; worst = {k: (0, None) for k in tot}
    for clip in d['animations']:
        dur = max(g.accessor(s['input'])[-1][0] for s in clip['samplers'])
        for f in range(0, int(round(dur * 30)) + 1, step):
            w = g.worlds(clip, f / 30); M = np.array([w[j] for j in skin['joints']]).reshape(-1, 4, 4).transpose(0, 2, 1) @ inv
            X = np.einsum('nk,nkij,nj->ni', W, M[J], np.c_[P, np.ones(len(P))])[:, :3]
            bvh = BVHTree.FromPolygons([tuple(v) for v in X], [tuple(t) for t in T])
            jp = lambda b: (M[bones.index(b)] @ binv[bones.index(b)])[:3, 3]
            n = {k: 0 for k in tot}
            if clip['name'] in ('walk', 'attack'):
                for sd in 'LR':
                    kn, fo = jp('leg_' + sd), jp('foot_' + sd); ax = (kn - fo) / np.linalg.norm(kn - fo)
                    n['shard_rays'] += grid_hits(bvh, (kn + fo) / 2, ax, ([0, 0, 1.], [1., 0, 0], [-1., 0, 0], [.7, 0, .7], [-.7, 0, .7], [0, 0, -1.]),
                                                 .25, .09, 160, 160 // 3, greave_t)
            hd, nk = jp('head'), jp('neck'); ax = (hd - nk) / np.linalg.norm(hd - nk)
            n['helmet_shard_rays'] += grid_hits(bvh, hd + ax * .05, ax, ([0, 0, 1.], [1., 0, 0], [-1., 0, 0], [.7, 0, .7], [-.7, 0, .7], [-.34, .7, .62]),
                                                .16, .14, 80, 70, helmet_t)
            for sd in 'LR':
                sh, el = jp('shoulder_' + sd), jp('forearm_' + sd); ax = (el - sh) / np.linalg.norm(el - sh)
                n['scrap_rays'] += grid_hits(bvh, (sh + el) / 2, ax, ([0, 0, 1.], [1., 0, 0], [-1., 0, 0], [.7, 0, .7], [-.7, 0, .7], [0, 0, -1.], [-.34, .7, .62]),
                                             .2, .14, 60, 42, scrap_t)
            for k in tot:
                tot[k] += n[k]
                if n[k] > worst[k][0]: worst[k] = (n[k], f'{clip["name"]} {f}')
    res[m['name']] = {**{k: tot[k] for k in tot}, 'worst_frames': worst, 'scraps': scraps}
print('SHARDS', res, flush=True)
