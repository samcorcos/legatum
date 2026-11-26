"""Find the spear, shield, helmet and cloak in a source model and seat the rigid props.

Pure numpy. Detection order: CLI overrides (regex on object/material names),
then name hints, then geometry of loose parts (a long thin part is a spear, a
large flat round part a shield; small parts inside those envelopes join them).
Helmets are found after the body's head is located: loose parts around the
head, or metallic faces fused to it.
"""
import re
import numpy as np

HINTS = {
    'spear': r'spear|lance|dory|doru|javelin|pike|weapon',
    'shield': r'shield|aspis|hoplon|buckler|scutum',
    'helmet': r'helm|crest|plume|horsehair|corinth',
    'cloak': r'cloak|cape|chlamys|mantle|himation',
}


def components(F, n):
    """Connected-component label per vertex (iterative min-label propagation)."""
    lab = np.arange(n)
    if not len(F): return lab
    while True:
        m = lab[F].min(1); new = lab.copy()
        for k in range(F.shape[1]): np.minimum.at(new, F[:, k], m)
        new = new[new]
        if np.array_equal(new, lab): return lab
        lab = new


def pca(p):
    c = p.mean(0); _, s, vt = np.linalg.svd(p - c, full_matrices=False)
    ext = np.array([np.ptp((p - c) @ v) for v in vt])
    return c, vt, ext


def classify(pieces, height, overrides):
    """pieces: dicts with 'points', 'object', 'materials'. Sets piece['part'] to spear/shield/cloak/body."""
    for p in pieces:
        p['part'] = None
        for part, rx in overrides.items():
            if re.search(rx, p['object'], re.I) or any(re.search(rx, m, re.I) for m in p['materials']):
                p['part'] = part
        if p['part'] is None:
            for part, rx in HINTS.items():
                if re.search(rx, p['object'], re.I) or (p['materials'] and all(re.search(rx, m, re.I) for m in p['materials'])):
                    p['part'] = part; break
    for p in pieces:
        if p['part'] is not None: continue
        c, vt, e = pca(p['points'])
        if e[0] > .45 * height and e[1] < .05 * height: p['part'] = 'spear'
        elif e[0] > .22 * height and e[2] < .2 * e[0] and e[1] > .6 * e[0]: p['part'] = 'shield'
    # Accessories inside a prop's envelope belong to it (blade, butt, boss, rim, straps).
    for part in ('spear', 'shield'):
        core = [p for p in pieces if p['part'] == part]
        if not core: continue
        main = max(core, key=lambda p: np.ptp(p['points'], 0).max())
        pts = main['points']; c, vt, e = pca(pts)
        if part == 'spear':
            # Slings and loose straps carried with a spear swing under the floor
            # once the spear is seated: keep only what lies along the shaft.
            for p in core:
                q = p['points'] - c; along = q @ vt[0]
                if (np.linalg.norm(q - np.outer(along, vt[0]), axis=1) > .04 * height).mean() > .5: p['part'] = 'drop'
        # Accessories inside the prop's envelope join it only from the same source
        # object: a shield held across the chest would otherwise swallow the armour.
        objs = {p['object'] for p in core}
        for p in pieces:
            if p['part'] is not None or p['object'] not in objs or np.ptp(p['points'], 0).max() > .4 * height: continue
            q = p['points'] - c
            if part == 'spear':
                along = q @ vt[0]; radial = np.linalg.norm(q - np.outer(along, vt[0]), axis=1)
                inside = (np.abs(along) < e[0] / 2 + .15 * height) & (radial < .05 * height)
            else:
                n = vt[2]; h = q @ n; radial = np.linalg.norm(q - np.outer(h, n), axis=1)
                inside = (np.abs(h) < .08 * height) & (radial < .6 * e[0])
            if inside.mean() > .9: p['part'] = part
    for p in pieces:
        if p['part'] is None: p['part'] = 'body'
    return pieces


def find_helmet(pieces, head, head_tip, neck):
    """Loose parts around the head and above the neck become the helmet (crest included)."""
    r = max(np.linalg.norm(head_tip - head) * 1.35, .12)
    centre = head + (head_tip - head) * .6
    up = (head_tip - neck) / np.linalg.norm(head_tip - neck)
    found = []
    for p in pieces:
        if p['part'] not in ('body', 'layer'): continue
        d = np.linalg.norm(p['points'] - centre, axis=1)
        above = ((p['points'] - neck) @ up > 0).mean()
        if (d < r * 1.6).mean() > .9 and above > .9 and len(p['points']) < .5 * sum(len(q['points']) for q in pieces if q['part'] in ('body', 'layer')):
            p['part'] = 'helmet'; found.append(p)
    return found


# -- seating ------------------------------------------------------------------------
def rigid_fit(src_frame, dst_frame):
    """4x4 taking an orthonormal source frame (origin, 3x3 columns) to a destination frame."""
    (o1, F1), (o2, F2) = src_frame, dst_frame
    R = F2 @ F1.T; M = np.eye(4); M[:3, :3] = R; M[:3, 3] = o2 - R @ o1
    return M


def _basis(axis, hint):
    u = axis / np.linalg.norm(axis); l = hint - u * (hint @ u)
    if np.linalg.norm(l) < 1e-6: l = np.array([1., 0, 0]) - u * u[0]
    l /= np.linalg.norm(l); return np.stack([u, l, np.cross(u, l)], 1)


def seat_spear(pts, frame, flip=False):
    """Align the source spear's axis and grip with the 0 A.D. spear (blade forward as in 0 A.D.)."""
    c, vt, e = pca(pts); ax = vt[0]; t = (pts - c) @ ax
    def radius(end):
        sel = np.abs(t - end) < .1 * np.ptp(t); q = pts[sel] - c
        return np.linalg.norm(q - np.outer(q @ ax, ax), axis=1).max()
    if radius(t.min()) > radius(t.max()): ax = -ax; t = -t
    if flip: ax = -ax; t = -t
    # Same grip fraction from the butt as the 0 A.D. spear.
    frac = (frame['grip'] - frame['butt']) / (frame['tip'] - frame['butt'])
    grip_src = c + ax * (t.min() + frac * np.ptp(t))
    grip_dst = frame['centre'] + frame['axis'] * frame['grip']
    hint = np.array([0, 1., 0])
    # Same length as the 0 A.D. spear about the grip: the clips' ground clearance,
    # thrust reach and death drop were tuned for that length.
    M0 = np.eye(4); k = (frame['tip'] - frame['butt']) / np.ptp(t)
    M0[:3, :3] += (k - 1) * np.outer(ax, ax); M0[:3, 3] = grip_src - M0[:3, :3] @ grip_src
    return rigid_fit((grip_src, _basis(ax, hint)), (grip_dst, _basis(frame['axis'], hint))) @ M0, \
        dict(length=float(np.ptp(t)), oad_length=float(frame['tip'] - frame['butt']))


def seat_shield(pts, frame, up_hint):
    """Rim centre to rim centre, outer face outward, inner face at the 0 A.D. strap depth."""
    c, vt, e = pca(pts); n = vt[2]
    # The dish bulges outward: the centre region sits further along n than the rim.
    radial = np.linalg.norm((pts - c) - np.outer((pts - c) @ n, n), axis=1)
    inner, outer = radial < .3 * radial.max(), radial > .8 * radial.max()
    if inner.any() and outer.any() and ((pts[inner] - c) @ n).mean() < ((pts[outer] - c) @ n).mean(): n = -n
    rim_c = pts[outer].mean(0) if outer.any() else c
    src = _basis(n, up_hint); dst = _basis(frame['normal'], frame['up'])
    M = rigid_fit((rim_c, src), (frame['centre'], dst))
    # At most 10% wider than the 0 A.D. aspis: the clips' floor clearance (the
    # death clip lays it flat) was tuned for that size.
    k = min(1., 1.1 * frame['radius'] / radial.max())
    if k < 1:
        S = np.eye(4); S[:3, :3] *= k; S[:3, 3] = frame['centre'] * (1 - k); M = S @ M
    moved = pts @ M[:3, :3].T + M[:3, 3]
    shift = frame['inner'] - ((moved - frame['centre']) @ frame['normal']).min()
    M[:3, 3] += frame['normal'] * shift
    return M, dict(radius=float(radial.max()), oad_radius=float(frame['radius']), scaled=round(float(k), 4))
