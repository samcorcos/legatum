"""Scripted stand-in for the hand sculpt of the fitted Hoplite Warrior parts.

Edits a copy of a *-sculpt.blend in place, moving vertices only (no added or
removed vertices, UVs untouched), so `--sculpt-override` accepts the result.
Coordinates are the fit's glTF bind space: Y up, the figure faces +Z.

    blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/scripted_sculpt.py -- \
        <copy.blend> helmet|greaves [--strand-depth .005] [--fan .25] [--blend-reach .02] [--rim .003]

helmet: on LOD0-helmet, (1) horsehair grooves along the crest's front-to-back
flow and a fuller fan (upper edge out and back, easing to the base); (2) the
face plate's rear boundary pulled onto the dome with proportional falloff, eye
slots, nasal and cheek gap untouched. greaves: on LOD0-layer, each greave top
re-profiled into a convex knee-guard arc with a raised rim.
"""
import argparse, json, math, sys
import bpy
import numpy as np
from mathutils import Vector
from mathutils.bvhtree import BVHTree

argv = sys.argv[sys.argv.index('--') + 1:]
ap = argparse.ArgumentParser()
ap.add_argument('blend'); ap.add_argument('target', choices=['helmet', 'greaves'])
ap.add_argument('--strand-depth', type=float, default=.005); ap.add_argument('--strand-period', type=float, default=.012)
ap.add_argument('--fan', type=float, default=.25); ap.add_argument('--blend-reach', type=float, default=.04)
ap.add_argument('--rim', type=float, default=.003); ap.add_argument('--relax', type=int, default=12); ap.add_argument('--knee-lift', type=float, default=.03)
a = ap.parse_args(argv)
bpy.ops.wm.open_mainfile(filepath=a.blend)
report = {}


def arrays(o):
    me = o.data; n = len(me.vertices)
    P = np.empty(n * 3); me.vertices.foreach_get('co', P); P = P.reshape(-1, 3)
    N = np.empty(n * 3); me.vertices.foreach_get('normal', N); N = N.reshape(-1, 3)
    fm = np.zeros(len(me.polygons), int)
    if 'src_mat' in me.attributes: me.attributes['src_mat'].data.foreach_get('value', fm)
    return me, P, N, fm


def verts_of(me, mask_faces):
    v = np.zeros(len(me.vertices), bool)
    for p in me.polygons:
        if mask_faces[p.index]: v[list(p.vertices)] = True
    return v


def boundary(me, vmask):
    from collections import Counter
    c = Counter()
    for p in me.polygons:
        vs = list(p.vertices)
        if not all(vmask[v] for v in vs): continue
        for i in range(len(vs)): c[tuple(sorted((vs[i], vs[(i + 1) % len(vs)])))] += 1
    b = np.zeros(len(me.vertices), bool)
    for (i, j), k in c.items():
        if k == 1: b[i] = b[j] = True
    return b


def smooth(x): x = np.clip(x, 0, 1); return x * x * (3 - 2 * x)


if a.target == 'helmet':
    o = bpy.data.objects['LOD0-helmet']; names = list(o['mat_names'])
    me, P, N, fm = arrays(o); before = P.copy()
    idx = lambda s: [i for i, n in enumerate(names) if s in n]
    crest = verts_of(me, np.isin(fm, idx('Crest horsehair')))
    plate = verts_of(me, np.isin(fm, idx('Corinthian bronze')))
    inner = verts_of(me, np.isin(fm, idx('Helmet interior')))
    dome_f = np.isin(fm, idx('Outfit'))
    # (1) crest: arc centre in the YZ plane, radial height h from base (0) to top (1).
    C = P[crest]; cz = (C[:, 2].max() + C[:, 2].min()) / 2; cy = C[:, 1].min()
    r = np.hypot(C[:, 1] - cy, C[:, 2] - cz); h = (r - r.min()) / max(np.ptp(r), 1e-6)
    radial = np.stack([np.zeros(len(C)), C[:, 1] - cy, C[:, 2] - cz], 1) / np.maximum(r, 1e-6)[:, None]
    f = smooth(h) ** 1.5
    C[:, 0] *= 1 + a.fan * f                                    # fuller side to side toward the edge
    C += radial * (a.fan * .06 * f)[:, None] + np.array([0, 0, -.012]) * f[:, None]   # out and back
    # Strand grooves follow the flow (concentric with the arc), plus a slow wave along it.
    ang = np.arctan2(C[:, 1] - cy, C[:, 2] - cz)
    groove = .5 + .5 * np.cos(2 * math.pi * (r / a.strand_period + .15 * np.sin(ang * 9)))
    side = np.abs(N[crest, 0]) > .3                             # grooves on the side faces only
    C -= (N[crest] * (a.strand_depth * groove * side * (.3 + .7 * f))[:, None])
    P[crest] = C
    # (2) face plate rear boundary onto the dome, proportional falloff.
    # The solidified plate is closed (no boundary edges): its rear edge is the band
    # of plate vertices at the largest azimuth about the dome's vertical axis.
    dome_v = verts_of(me, dome_f); pc = P[dome_v].mean(0)
    az = np.degrees(np.arctan2(P[:, 0] - pc[0], P[:, 2] - pc[2] + 1e-9))
    amax = np.abs(az[plate]).max()
    rear = plate & (np.abs(az) > amax - 10)
    dome_polys = [list(p.vertices) for p in me.polygons if dome_f[p.index]]
    bvh = BVHTree.FromPolygons([tuple(x) for x in before], dome_polys)
    delta = np.zeros_like(P)
    for i in np.nonzero(rear)[0]:
        hit, nrm, _, _ = bvh.find_nearest(Vector(before[i]))
        if hit is not None: delta[i] = np.array(hit) + np.array(nrm) * .0008 - before[i]
    rv = np.nonzero(rear)[0]; moving = (plate | inner) & ~rear
    for i in np.nonzero(moving)[0]:
        d = np.linalg.norm(before[rv] - before[i], axis=1); j = d.argmin()
        if d[j] < a.blend_reach: delta[i] = delta[rv[j]] * smooth(1 - d[j] / a.blend_reach)
    keep = (np.abs(az) < amax - 45) & (plate | inner)                  # eye slots, nasal, cheek gap untouched
    delta[keep] = 0
    P += delta
    report = {'crest_vertices': int(crest.sum()), 'plate_rear_boundary': int(rear.sum())}
else:
    o = bpy.data.objects['LOD0-layer']; me, P, N, fm = arrays(o); before = P.copy()
    rig_knee = {'L': np.array([.114, .486, -.014]), 'R': np.array([-.114, .486, -.014])}
    moved_total = 0
    for side, kj in rig_knee.items():
        rel = P - kj; rad = np.hypot(rel[:, 0], rel[:, 2])
        g = (P[:, 1] > .08) & (P[:, 1] < kj[1] + .12) & (rad < .1)
        g &= verts_of(me, np.array([n == 0 for n in fm]))      # source outfit (bronze greaves), not the sleeves
        if g.sum() < 10: continue
        th = np.arctan2(rel[:, 0], rel[:, 2])                    # 0 = front (+Z, the kneecap)
        bins = np.linspace(-math.pi, math.pi, 25); bi = np.clip(np.digitize(th, bins) - 1, 0, 23)
        top = np.full(24, np.nan)
        for b in range(24):
            s = g & (bi == b)
            if s.any(): top[b] = P[s, 1].max()
        ok = np.isfinite(top); top = np.interp(np.arange(24), np.nonzero(ok)[0], top[ok], period=24)
        front = top[[11, 12]].max(); back = np.percentile(top, 20)
        centres = (bins[:-1] + bins[1:]) / 2
        # A broad rounded guard over the knee: wide plateau at the front, easing to the back.
        front = front - .012
        target = np.minimum(back + (front - back) * (.5 + .5 * np.cos(centres)) ** .45, top + a.knee_lift)
        target = np.maximum(target, top - .01)
        tcur = np.interp(th, centres, top, period=2 * math.pi); ttar = np.interp(th, centres, target, period=2 * math.pi)
        base = tcur - .07; zone = g & (P[:, 1] > base)
        k = (ttar - base) / np.maximum(tcur - base, 1e-6)
        P[zone, 1] = base[zone] + (P[zone, 1] - base[zone]) * k[zone]
        # Relax the re-profiled band (graph Laplacian, band only): the source top edge
        # zigzags in its topology, and a pure height remap keeps the teeth.
        nb = [[] for _ in range(len(P))]
        for e in me.edges:
            i, j = e.vertices
            if zone[i] and zone[j]: nb[i].append(j); nb[j].append(i)
        zi = [i for i in np.nonzero(zone)[0] if nb[i]]
        r0 = np.hypot(P[:, 0] - kj[0], P[:, 2] - kj[2])
        for _ in range(a.relax):
            avg = np.array([P[nb[i]].mean(0) for i in zi])
            P[zi] = .5 * P[zi] + .5 * avg
            rel2 = P[zi] - kj; rr = np.hypot(rel2[:, 0], rel2[:, 2]); k2 = np.maximum(1, r0[zi] / np.maximum(rr, 1e-6))
            P[zi, 0] = kj[0] + rel2[:, 0] * k2; P[zi, 2] = kj[2] + rel2[:, 2] * k2   # never sink toward the skin
        rel = P - kj; rad = np.hypot(rel[:, 0], rel[:, 2])
        # Raised rim: the top 8 mm stands out 3 mm from the leg axis.
        near_top = g & (P[:, 1] > ttar - .008)
        out = np.stack([rel[:, 0], np.zeros(len(P)), rel[:, 2]], 1) / np.maximum(rad, 1e-6)[:, None]
        w = smooth(1 - (ttar - P[:, 1]) / .008)
        P[near_top] += out[near_top] * (a.rim * w[near_top])[:, None]
        moved_total += int(zone.sum())
        report[f'greave_{side}'] = {'vertices': int(g.sum()), 'reprofiled': int(zone.sum()), 'front_top_m': round(float(front), 3), 'back_top_m': round(float(back), 3)}
moved = np.linalg.norm(P - before, axis=1)
me.vertices.foreach_set('co', P.ravel()); me.update()
report.update(vertices_moved=int((moved > 1e-6).sum()), max_move_m=round(float(moved.max()), 4))
print('SCULPTED', json.dumps(report), flush=True)
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_mainfile()
