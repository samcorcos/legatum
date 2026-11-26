"""Unpose a source body onto the 0 A.D. bind pose and bone lengths.

Pure numpy. Each fitting bone gets an affine map taking its source segment
(joint -> next joint) onto the 0 A.D. segment: a rotation (full frame for the
torso, swing only for limbs), a stretch along the bone to the 0 A.D. length,
and the same girth. Vertices blend the maps of their chain position with
smooth windows at the joints (linear blend skinning, run backwards). That
removes an action pose and corrects limb proportions in one step, leaving the
source's own silhouette, clothing and surface detail.
"""
import numpy as np

from fit_rig import CHAINS
from fit_landmarks import project

TORSO_LATERAL = {'hip': 'legs', 'spine': 'legs', 'spine1': 'both', 'chest': 'arms', 'neck': 'arms', 'head': 'arms'}


def _frame(u, lateral):
    u = u / np.linalg.norm(u); l = lateral - u * (lateral @ u); l /= np.linalg.norm(l)
    return np.stack([u, l, np.cross(u, l)], 1)


def _swing(a, b):
    a = a / np.linalg.norm(a); b = b / np.linalg.norm(b); v = np.cross(a, b); c = float(a @ b)
    if c < -.9999:
        axis = np.cross(a, [1, 0, 0]) if abs(a[0]) < .9 else np.cross(a, [0, 1, 0]); axis /= np.linalg.norm(axis)
        return 2 * np.outer(axis, axis) - np.eye(3)
    vx = np.array([[0, -v[2], v[1]], [v[2], 0, -v[0]], [-v[1], v[0], 0]])
    return np.eye(3) + vx + vx @ vx / (1 + c)


def fix_hip(src, tgt):
    """Place the source hip joint like 0 A.D.'s: offset from the thigh midpoint in the pelvis frame."""
    def pelvis(j):
        mid = (j['thigh_L'] + j['thigh_R']) / 2
        return mid, _frame(j['spine1'] - mid, j['thigh_L'] - j['thigh_R'])
    ms, fs = pelvis(src); mt, ft = pelvis(tgt)
    src['hip'] = ms + fs @ ft.T @ (tgt['hip'] - mt)
    return src


def bone_maps(src, tgt):
    """{bone: (a_src, a_tgt, R, axis_src, stretch)} for every chain bone."""
    lat = {}
    for name, j in (('src', src), ('tgt', tgt)):
        arms = j['arm_L'] - j['arm_R']; legs = j['thigh_L'] - j['thigh_R']
        lat[name] = {'arms': arms, 'legs': legs,
                     'both': arms / np.linalg.norm(arms) + legs / np.linalg.norm(legs)}
    maps = {}
    for chain, joints in CHAINS.items():
        for a, b in zip(joints, joints[1:]):
            us, ut = src[b] - src[a], tgt[b] - tgt[a]
            if chain == 'torso':
                # Orient torso bones over two segments: the 0 A.D. hip and spine bones
                # are 7 cm long, so their own direction in a detected skeleton is noise.
                i = joints.index(a); c = joints[min(i + 2, len(joints) - 1)]
                key = TORSO_LATERAL[a]
                R = _frame(tgt[c] - tgt[a], lat['tgt'][key]) @ _frame(src[c] - src[a], lat['src'][key]).T
            else:
                R = _swing(us, ut)
            stretch = float(np.clip(np.linalg.norm(ut) / max(np.linalg.norm(us), 1e-6), .5, 2.))
            maps[a] = (src[a], tgt[a], R, us / np.linalg.norm(us), stretch)
    return maps


def apply_map(m, pts):
    a, c, R, u, k = m
    q = pts - a; q = q + np.outer(q @ u, u) * (k - 1)
    return c + q @ R.T


def _smooth(x):
    x = np.clip(x, 0, 1); return x * x * (3 - 2 * x)


def chain_blend(chain, s, arcs, blend):
    """Per vertex: (distal bone, proximal bone, the joint between them, t).

    t is the proximal bone's weight: 0 well below the joint, 1 well above it,
    a smoothstep across a window of 2 * blend around it. Windows never overlap,
    so each vertex blends at most two neighbouring bones.
    """
    joints = CHAINS[chain]
    bones = list(reversed(joints[:-1]))                 # distal -> proximal
    pivots = list(reversed(joints[:-1]))                # joint at the top of each bone
    if chain == 'torso': pivots = pivots[:-1]
    else: bones.append('chest' if chain.startswith('arm') else 'hip')
    bounds = np.array([arcs[j] for j in pivots])
    k = np.abs(s[:, None] - bounds[None]).argmin(1)
    t = _smooth((s - bounds[k]) / (2 * blend) + .5)
    lo = np.array(bones)[k]; hi = np.array(bones)[k + 1]; piv = np.array(pivots)[k]
    return lo, hi, piv, t


def _quat(R):
    w = np.sqrt(max(0., 1 + R[0, 0] + R[1, 1] + R[2, 2])) / 2
    if w > 1e-4:
        return np.array([w, (R[2, 1] - R[1, 2]) / (4 * w), (R[0, 2] - R[2, 0]) / (4 * w), (R[1, 0] - R[0, 1]) / (4 * w)])
    i = int(np.argmax(np.diag(R))); j, k = (i + 1) % 3, (i + 2) % 3
    r = np.sqrt(max(0., 1 + R[i, i] - R[j, j] - R[k, k])); q = np.zeros(4)
    q[1 + i] = r / 2; q[0] = (R[k, j] - R[j, k]) / (2 * r); q[1 + j] = (R[j, i] + R[i, j]) / (2 * r); q[1 + k] = (R[k, i] + R[i, k]) / (2 * r)
    return q


def _rotate(q, v):
    """Rotate vectors v (n,3) by unit quaternions q (n,4) (w, x, y, z)."""
    w = q[:, :1]; u = q[:, 1:]
    return v + 2 * np.cross(u, np.cross(u, v) + w * v)


def apply_blend(maps, src, pts, lo, hi, piv, t):
    """Rotation-blended unpose: slerp the two bones' rotations about their shared joint.

    Linear blending of two maps that differ by a large rotation (a raised arm
    brought down 150 degrees) collapses the joint; blending the rotation
    about the joint keeps its volume, like dual-quaternion skinning.
    """
    out = np.empty_like(pts)
    for key in set(zip(lo.tolist(), hi.tolist(), piv.tolist())):
        sel = (lo == key[0]) & (hi == key[1]) & (piv == key[2])
        a, b = maps[key[0]], maps[key[1]]; J = src[key[2]]; tt = t[sel][:, None]
        qa, qb = _quat(a[2]), _quat(b[2])
        if qa @ qb < 0: qb = -qb
        dot = float(np.clip(qa @ qb, -1, 1)); ang = np.arccos(dot)
        if ang < 1e-4: q = qa[None] * (1 - tt) + qb[None] * tt
        else: q = (np.sin((1 - tt) * ang) * qa[None] + np.sin(tt * ang) * qb[None]) / np.sin(ang)
        q /= np.linalg.norm(q, axis=1, keepdims=True)
        d = pts[sel] - J
        local = (1 - tt) * (d + np.outer(d @ a[3], a[3]) * (a[4] - 1)) + tt * (d + np.outer(d @ b[3], b[3]) * (b[4] - 1))
        pivot = (1 - tt) * apply_map(a, J[None]) + tt * apply_map(b, J[None])
        out[sel] = pivot + _rotate(q, local)
    return out


def unpose(V, voxel_of_vertex, sk, chains, src, tgt, blend):
    """Fitted positions for the body vertices V.

    Returns (fitted, weights {bone: (n,)}, maps, spec) where spec = (lo, hi,
    pivot, t) per vertex, reusable for nearby loose geometry (cloaks).
    """
    g = sk.graph
    names = {}  # chain -> tip key in sk.lines
    for tip, (line, s, path) in sk.lines.items():
        for chain, (cl, cs, arcs) in chains.items():
            if cl is line: names[chain] = tip
    # Geodesic Voronoi over chain paths (limb paths stop just past their root joint).
    sources, labels = [], []
    order = list(chains)
    for ci, chain in enumerate(order):
        line, s, arcs = chains[chain]; path = sk.lines[names[chain]][2]
        stop = arcs[CHAINS[chain][0]] + (.04 * sk.height if chain != 'torso' else np.inf)
        keep = [v for v, sv in zip(path, s) if sv <= stop]
        sources += keep; labels += [ci] * len(keep)
    _, _, lab = g.dijkstra(sources, labels)
    lost = lab < 0
    if lost.any():
        known = np.nonzero(~lost)[0]
        for i in np.nonzero(lost)[0]:
            lab[i] = lab[known[np.argmin(np.linalg.norm(g.centre[known] - g.centre[i], axis=1))]]
    vlab = lab[voxel_of_vertex]
    # The torso region can grow down into a thigh swung back in the source pose.
    # Below the hip joints, vertices belong to the leg on their side.
    up = src['spine1'] - src['hip']; up = up / np.linalg.norm(up)
    mid = (src['thigh_L'] + src['thigh_R']) / 2; side = src['thigh_L'] - src['thigh_R']
    torso = order.index('torso')
    low = (vlab == torso) & (((V - mid) @ up) < -.02 * sk.height)
    vlab[low & ((V - mid) @ side > 0)] = order.index('leg_L')
    vlab[low & ((V - mid) @ side <= 0)] = order.index('leg_R')
    maps = bone_maps(src, tgt)
    n = len(V); lo = np.empty(n, object); hi = np.empty(n, object); piv = np.empty(n, object); t = np.zeros(n)
    for ci, chain in enumerate(order):
        idx = np.nonzero(vlab == ci)[0]
        if not len(idx): continue
        line, s, arcs = chains[chain]
        if chain != 'torso':
            # Limbs: project onto the straight bones through the detected joints,
            # carried on past the root joint. The voxel centre line bends sharply
            # into the body there, and its projection jumps (jagged tears).
            js = CHAINS[chain][::-1]
            pts = [src[j] for j in js]; root_dir = src[js[-1]] - src[js[-2]]
            pts.append(src[js[-1]] + root_dir / np.linalg.norm(root_dir) * .2 * sk.height)
            line = np.array(pts); s = np.concatenate([[0], np.cumsum(np.linalg.norm(np.diff(line, axis=0), axis=1))])
            arcs = {j: float(s[i]) for i, j in enumerate(js)}
        if chain == 'torso':
            # Carry the spine line on through the hip joint, so the waist gets the
            # same hip map as the leg-labelled pelvis beside it (no tear at the seam).
            down = src['hip'] - line[-1]; ext = [line[-1] + down * f for f in (.5, 1., 1.5)]
            ext.append(ext[-1] + (src['hip'] - src['spine1']) * .5)
            line = np.vstack([line, ext]); s = np.concatenate([s, s[-1] + np.cumsum(np.linalg.norm(np.diff(line[-5:], axis=0), axis=1))])
        sv, dv = project(line, s, V[idx])
        if chain == 'torso':
            # Shoulders and the upper back sit far from the spine line at neck height:
            # they belong to the chest, not the neck (else they tear from the arms).
            far = _smooth((dv - .05 * sk.height) / (.03 * sk.height))
            sv = sv + (np.maximum(sv, arcs['neck'] + blend) - sv) * far
        if chain.startswith('leg'):
            # Belly and buttock above the hip joint follow the pelvis, never a thigh
            # raised in the source pose (that would fold the waist when unposed).
            up = src['spine1'] - src['hip']; up = up / np.linalg.norm(up)
            above = _smooth(((V[idx] - src[CHAINS[chain][0]]) @ up) / (.04 * sk.height))
            # At the crotch the two thighs' maps meet head on; ease the innermost
            # vertices halfway to the pelvis map so the legs do not tear apart there.
            side = src['thigh_L'] - src['thigh_R']; half = np.linalg.norm(side) / 2; side /= 2 * half
            mid = (src['thigh_L'] + src['thigh_R']) / 2
            inner = _smooth(1 - np.abs((V[idx] - mid) @ side) / (.6 * half)) * _smooth((sv - arcs[CHAINS[chain][1]]) / (.1 * sk.height))
            above = np.maximum(above, .5 * inner)
            sv = sv + (np.maximum(sv, arcs[CHAINS[chain][0]] + blend) - sv) * above
        lo[idx], hi[idx], piv[idx], t[idx] = chain_blend(chain, sv, arcs, blend)
    out = apply_blend(maps, src, V, lo, hi, piv, t)
    weights = {}
    for bones, w in ((lo, 1 - t), (hi, t)):
        for b in set(bones.tolist()):
            weights.setdefault(b, np.zeros(n))[bones == b] += w[bones == b]
    return out, weights, maps, (lo, hi, piv, t)


def fit_heels(out, weights, rig):
    """Match the heel's reach behind the ankle to the 0 A.D. foot.

    The foot map stretches ankle-to-toe; a longer source heel would still poke
    out behind and catch the floor in the clips' planted poses.
    """
    report = {}
    for side in 'LR':
        k = rig.bones.index('foot_' + side)
        mine = (rig.J == k) & (rig.W > .5)
        donor = rig.P[mine.any(1) & (rig.P[:, 1] < .1)]
        ankle = rig.joint['foot_' + side]
        if not len(donor) or 'foot_' + side not in weights: continue
        w = weights['foot_' + side]; sel = (w > .5) & (out[:, 1] < .1)
        if not sel.any(): continue
        want = ankle[2] - donor[:, 2].min(); have = ankle[2] - out[sel, 2].min()
        if have <= want: continue
        behind = (out[:, 2] < ankle[2]) & (w > 0)
        f = 1 - (1 - want / have) * w[behind]
        out[behind, 2] = ankle[2] + (out[behind, 2] - ankle[2]) * f
        report[side] = round(float(want / have), 3)
    return report


def heal(V, out, F, rings=3, iters=40, extra=None):
    """Close tears where neighbouring vertices took very different maps.

    Finds edges stretched past 1.6x their source length, grows that region by a
    few rings, and relaxes the *displacement* (fitted - source) inside it with
    the rest of the mesh held fixed. Relaxing displacement rather than position
    keeps the source's surface detail.
    """
    E = np.unique(np.sort(np.concatenate([F[:, [0, 1]], F[:, [1, 2]], F[:, [2, 0]]]), axis=1), axis=0)
    l0 = np.linalg.norm(V[E[:, 0]] - V[E[:, 1]], axis=1); l1 = np.linalg.norm(out[E[:, 0]] - out[E[:, 1]], axis=1)
    bad = np.zeros(len(V), bool); torn = l1 > 1.6 * l0 + 1e-4
    bad[E[torn, 0]] = bad[E[torn, 1]] = True
    region = bad.copy()
    if extra is not None: region |= extra
    for _ in range(rings):
        grow = region.copy(); grow[E[region[E[:, 0]], 1]] = True; grow[E[region[E[:, 1]], 0]] = True; region = grow
    if not region.any(): return out, 0
    D = out - V; deg = np.bincount(E.ravel(), minlength=len(V)).astype(float)
    for _ in range(iters):
        acc = np.zeros_like(D)
        np.add.at(acc, E[:, 0], D[E[:, 1]]); np.add.at(acc, E[:, 1], D[E[:, 0]])
        avg = acc / np.maximum(deg, 1)[:, None]
        D[region] = .5 * D[region] + .5 * avg[region]
    return V + D, int(torn.sum())


def vertex_normals(V, F):
    n = np.cross(V[F[:, 1]] - V[F[:, 0]], V[F[:, 2]] - V[F[:, 0]])
    out = np.zeros_like(V)
    for k in range(3): np.add.at(out, F[:, k], n)
    return out / np.maximum(np.linalg.norm(out, axis=1, keepdims=True), 1e-12)


def keep_clearance(Vs, Fs, fit_s, Vl, fit_l, near, floor=.004):
    """Layer vertices that sat outside the skin stay outside it after fitting:
    push them back along the fitted skin normal to their source clearance."""
    n0 = vertex_normals(Vs, Fs)[near]; n1 = vertex_normals(fit_s, Fs)[near]
    o0 = ((Vl - Vs[near]) * n0).sum(1); o1 = ((fit_l - fit_s[near]) * n1).sum(1)
    # Restore at most 1 cm of clearance: enough to stop skin showing through, but a
    # cape or skirt hanging well off the body is left where the blend put it.
    want = np.clip(o0, floor, .01); push = (o0 > 0) & (o1 < want)
    fit_l = fit_l.copy(); fit_l[push] += n1[push] * (want - o1)[push, None]
    return fit_l, int(push.sum())
