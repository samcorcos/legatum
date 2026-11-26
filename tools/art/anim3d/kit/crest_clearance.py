"""Crest clearance through every clip at 30 fps: does the crest pass through the
spear shaft or the shield disc? Pure numpy on the GLB (LOD0).

    python kit/crest_clearance.py <unit.glb>
"""
import sys
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from validate_glb import GLB

g = GLB(sys.argv[1]); d = g.doc; m = d['meshes'][0]; pr = m['primitives'][0]
P = np.array(g.accessor(pr['attributes']['POSITION'])); J = np.array(g.accessor(pr['attributes']['JOINTS_0'])).astype(int)
skin = d['skins'][0]; inv = np.array(g.accessor(skin['inverseBindMatrices'])).reshape(-1, 4, 4).transpose(0, 2, 1)
part = {}
for r in m['extras']['vertex_regions']: part.setdefault(r['part'], []).extend(range(r['start'], r['start'] + r['count']))
crest, spear, shield = (np.array(part[k]) for k in ('crest', 'spear', 'aspis'))
def placed(idx, M):  # rigid parts: one joint each
    j = J[idx, 0][0]; X = np.c_[P[idx], np.ones(len(idx))] @ M[j].T; return X[:, :3]
worst = {}
for clip in d['animations']:
    dur = max(g.accessor(s['input'])[-1][0] for s in clip['samplers']); bad = []
    for f in range(int(round(dur * 30)) + 1):
        w = g.worlds(clip, f / 30); M = np.array([w[j] for j in skin['joints']]).reshape(-1, 4, 4).transpose(0, 2, 1) @ inv
        C = placed(crest, M); S = placed(spear, M); H = placed(shield, M)
        c0 = S.mean(0); ax = np.linalg.svd(S - c0)[2][0]; t = (S - c0) @ ax
        q = C - c0; tt = np.clip(q @ ax, t.min(), t.max()); dist = np.linalg.norm(q - np.outer(tt, ax), axis=1)
        sp = int((dist < .016).sum())
        hc = H.mean(0); n = np.linalg.svd(H - hc)[2][2]; R = np.linalg.norm((H - hc) - np.outer((H - hc) @ n, n), axis=1).max()
        qh = C - hc; hh = qh @ n; rr = np.linalg.norm(qh - np.outer(hh, n), axis=1)
        sh = int(((np.abs(hh) < .02) & (rr < .95 * R)).sum())
        if sp or sh: bad.append((f, sp, sh))
    worst[clip['name']] = {'frames_with_contact': len(bad), 'max_spear_verts': max([b[1] for b in bad], default=0), 'max_shield_verts': max([b[2] for b in bad], default=0), 'frames': [b[0] for b in bad][:12]}
print(worst)
