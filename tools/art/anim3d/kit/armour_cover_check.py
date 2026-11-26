"""Skin showing through armour, every frame at 30 fps.

For each leg, skin and greave vertices are put in the shin's moving frame (the
leg bone's skinning matrix): a skin vertex inside the greave's angular and height
span that lies farther from the shin axis than the greave surface there is
showing through. Only vertices that some triangle uses count. Pure numpy.

    python kit/armour_cover_check.py <unit.glb> [clips=walk,attack]

`flagged(...)` is also used by the fit to drop such skin before export.
"""
import sys
from pathlib import Path
import numpy as np


def flagged(g, P, J, W, part, used, clips=('walk', 'attack'), per_frame=False, edge=.03, tol=.002):
    """Indices of skin vertices showing through a greave in any frame of `clips`."""
    d = g.doc; skin = d['skins'][0]; bones = [d['nodes'][j]['name'] for j in skin['joints']]
    inv = np.array(g.accessor(skin['inverseBindMatrices'])).reshape(-1, 4, 4).transpose(0, 2, 1)
    body = (part == 'body') & used; greave = (part == 'greave') & used
    out = set(); frames = {}
    if not greave.any(): return (out, frames) if per_frame else out
    axes = {}
    for sd in 'LR':
        kn = np.linalg.inv(inv[bones.index('leg_' + sd)])[:3, 3]; an = np.linalg.inv(inv[bones.index('foot_' + sd)])[:3, 3]
        ax = (kn - an) / np.linalg.norm(kn - an); sx = np.cross(ax, [0, 0, 1.]); sx /= np.linalg.norm(sx)
        axes[sd] = (an, ax, sx, np.cross(sx, ax), bones.index('leg_' + sd))
    bi = np.nonzero(body)[0]
    for clip in [c for c in d['animations'] if c['name'] in clips]:
        dur = max(g.accessor(s['input'])[-1][0] for s in clip['samplers']); frames[clip['name']] = []
        for f in range(int(round(dur * 30)) + 1):
            w = g.worlds(clip, f / 30); M = np.array([w[j] for j in skin['joints']]).reshape(-1, 4, 4).transpose(0, 2, 1) @ inv
            X = np.einsum('nk,nkij,nj->ni', W, M[J], np.c_[P, np.ones(len(P))])[:, :3]
            bad_f = 0
            for sd, (an, ax, sx, fz, k) in axes.items():
                gm = greave & ((P[:, 0] > 0) == (sd == 'L'))
                if not gm.any(): continue
                L = np.linalg.inv(M[k]); Y = X @ L[:3, :3].T + L[:3, 3]
                def polar(Z):
                    q = Z - an; h = q @ ax; return np.arctan2(q @ sx, q @ fz), h, np.linalg.norm(q - np.outer(h, ax), axis=1)
                gt, gh, gr = polar(Y[gm]); st, sh, sr = polar(Y[bi])
                tb = np.linspace(-np.pi, np.pi, 25); hb = np.linspace(gh.min() + edge, gh.max() - edge, 9)
                rmin = np.full((24, 8), np.inf)
                ti_g = np.clip(np.digitize(gt, tb) - 1, 0, 23); hj_g = np.digitize(gh, hb) - 1
                okg = (hj_g >= 0) & (hj_g < 8); np.minimum.at(rmin, (ti_g[okg], hj_g[okg]), gr[okg])
                cnt = np.zeros((24, 8)); np.add.at(cnt, (ti_g[okg], hj_g[okg]), 1); rmin[cnt < 2] = np.inf
                ti = np.clip(np.digitize(st, tb) - 1, 0, 23); hj = np.digitize(sh, hb) - 1
                inside = (hj >= 0) & (hj < 8)
                lim = np.full(len(st), np.inf); lim[inside] = rmin[ti[inside], hj[inside]]
                bad = np.isfinite(lim) & (sr > lim + tol) & (sr < .12)
                out.update(bi[bad].tolist()); bad_f += int(bad.sum())
            if bad_f: frames[clip['name']].append((f, bad_f))
    return (out, frames) if per_frame else out


if __name__ == '__main__':
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    from validate_glb import GLB
    g = GLB(sys.argv[1]); clips = tuple((sys.argv[2] if len(sys.argv) > 2 else 'walk,attack').split(','))
    res = {}
    for m in g.doc['meshes']:
        pr = m['primitives'][0]; a = pr['attributes']
        P = np.array(g.accessor(a['POSITION'])); J = np.array(g.accessor(a['JOINTS_0'])).astype(int); W = np.array(g.accessor(a['WEIGHTS_0']))
        part = np.empty(len(P), object)
        for r in m['extras']['vertex_regions']: part[r['start']:r['start'] + r['count']] = r['part']
        used = np.zeros(len(P), bool); used[[i[0] for i in g.accessor(pr['indices'])]] = True
        _, fr = flagged(g, P, J, W, part, used, clips, per_frame=True)
        res[m['name']] = {c: {'frames_with_skin_through_greaves': len(v), 'max_vertices': max([b for _, b in v], default=0)} for c, v in fr.items()}
    print(res)
