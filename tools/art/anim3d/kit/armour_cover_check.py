"""Skin showing through armour, every frame at 30 fps (LOD0).

For each leg, skin and greave vertices are put in the shin's moving frame (the
leg bone's skinning matrix): a skin vertex inside the greave's angular and
height span that lies farther from the shin axis than the greave surface there
is showing through. Same for the helmet about the head bone. Pure numpy.

    python kit/armour_cover_check.py <unit.glb> [clips=walk,attack]
"""
import sys
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_glb import GLB

g = GLB(sys.argv[1]); clips = (sys.argv[2] if len(sys.argv) > 2 else 'walk,attack').split(',')
d = g.doc; m = d['meshes'][0]; pr = m['primitives'][0]; a = pr['attributes']
P = np.array(g.accessor(a['POSITION'])); J = np.array(g.accessor(a['JOINTS_0'])).astype(int); W = np.array(g.accessor(a['WEIGHTS_0']))
skin = d['skins'][0]; bones = [d['nodes'][j]['name'] for j in skin['joints']]
inv = np.array(g.accessor(skin['inverseBindMatrices'])).reshape(-1, 4, 4).transpose(0, 2, 1)
part = np.empty(len(P), object)
for r in m['extras']['vertex_regions']: part[r['start']:r['start'] + r['count']] = r['part']
body = part == 'body'; greave = part == 'greave'
res = {}
for clip in [c for c in d['animations'] if c['name'] in clips]:
    dur = max(g.accessor(s['input'])[-1][0] for s in clip['samplers']); worst = 0; frames = []
    for f in range(int(round(dur * 30)) + 1):
        w = g.worlds(clip, f / 30); M = np.array([w[j] for j in skin['joints']]).reshape(-1, 4, 4).transpose(0, 2, 1) @ inv
        X = np.einsum('nk,nkij,nj->ni', W, M[J], np.c_[P, np.ones(len(P))])[:, :3]
        bad = 0
        for sd in 'LR':
            k = bones.index('leg_' + sd); L = np.linalg.inv(M[k])         # world -> shin bind space
            Y = X @ L[:3, :3].T + L[:3, 3]
            gm = greave & ((P[:, 0] > 0) == (sd == 'L'))
            if not gm.any(): continue
            # Shin axis in bind space, from the knee (leg_X) and ankle (foot_X) joints.
            kn = np.linalg.inv(inv[bones.index('leg_' + sd)])[:3, 3]; an = np.linalg.inv(inv[bones.index('foot_' + sd)])[:3, 3]
            ax = (kn - an) / np.linalg.norm(kn - an); sx = np.cross(ax, [0, 0, 1.]); sx /= np.linalg.norm(sx); fz = np.cross(sx, ax)
            def polar(Z):
                q = Z - an; h = q @ ax; return np.arctan2(q @ sx, q @ fz), h, np.linalg.norm(q - np.outer(h, ax), axis=1)
            gt, gh, gr = polar(Y[gm]); st, sh, sr = polar(Y[body])
            tb = np.linspace(-np.pi, np.pi, 25); hb = np.linspace(gh.min() + .03, gh.max() - .03, 9)
            rmin = np.full((24, 8), np.nan)
            for i in range(24):
                for j in range(8):
                    s_ = (gt >= tb[i]) & (gt < tb[i + 1]) & (gh >= hb[j]) & (gh < hb[j + 1])
                    if s_.sum() >= 2: rmin[i, j] = gr[s_].min()
            ti = np.clip(np.digitize(st, tb) - 1, 0, 23); hj = np.digitize(sh, hb) - 1
            inside = (hj >= 0) & (hj < 8)
            lim = np.full(len(st), np.nan); lim[inside] = rmin[ti[inside], hj[inside]]
            bad += int((np.isfinite(lim) & (sr > lim + .002) & (sr < .12)).sum())   # this shin only
        if bad: frames.append(f); worst = max(worst, bad)
    res[clip['name']] = {'frames_with_skin_through_greaves': len(frames), 'max_vertices': worst, 'frames': frames[:15]}
print(res)
