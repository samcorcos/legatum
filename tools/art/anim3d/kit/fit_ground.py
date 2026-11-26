"""Keep a re-skinned mesh above the floor in every approved clip, without new keys.

Ported from the painted lane's painted_contact.py. Each posed floor plane is a
linear half-space in a vertex's bind space, so a small bind-space push fixes
every sampled frame; joints, weights and the clips stay untouched.
"""
import numpy as np


def clip_planes(g, fps=120):
    skin = g.doc['skins'][0]
    inv = np.array(g.accessor(skin['inverseBindMatrices'])).reshape(-1, 4, 4).transpose(0, 2, 1)
    rows = []
    for clip in g.doc['animations']:
        duration = max(g.accessor(s['input'])[-1][0] for s in clip['samplers'])
        for t in np.linspace(0, duration, round(duration * fps) + 1):
            w = g.worlds(clip, float(t))
            m = np.array([w[j] for j in skin['joints']]).reshape(-1, 4, 4).transpose(0, 2, 1) @ inv
            rows.append(m[:, 1, :])  # world-height row of each joint's skinning matrix
    return rows


def fit(xyz, joints, weights, planes, clearance=.0005, rounds=4):
    """xyz (n,3) bind positions, joints/weights (n,4). Returns adjusted copy and a report."""
    xyz = xyz.copy(); before = xyz.copy(); worst = 0.
    for _ in range(rounds):
        worst = 0.
        for m in planes:
            plane = np.einsum('nij,ni->nj', m[joints], weights)
            height = (plane[:, :3] * xyz).sum(1) + plane[:, 3]
            need = np.maximum(0, -clearance - height); worst = max(worst, float(need.max()))
            ids = need > 0
            xyz[ids] += plane[ids, :3] * (need[ids] / (plane[ids, :3] ** 2).sum(1))[:, None]
        if worst < .00005: break
    delta = np.linalg.norm(xyz - before, axis=1)
    return xyz, {'changed_vertices': int((delta > 1e-7).sum()), 'max_bind_adjustment_m': float(delta.max()),
                 'last_projection_m': worst}
