"""Find a humanoid's joints in any pose, from its surface alone (or from its armature).

Pure numpy, no bpy. Coordinates are glTF-style: Y up, figure facing +Z.

Mesh route (no armature): sample the surface, voxelise it, and run Dijkstra
from the body centre over the 26-connected voxel graph. Branch tips with a long
geodesic branch are the head, hands and feet; each tip's shortest path back to
the centre is a limb path, and the mean of the voxels in the same geodesic band
around each path voxel gives the limb centre line. Joints sit at fixed arc
lengths from each tip. Those arc lengths are calibrated by running the same
procedure on the 0 A.D. body in its bind pose, where the joints are known, and
scaling by the ratio of the two bodies' crown-to-toe geodesic lengths. Elbows
and knees then snap to the centre line's point of greatest bend when the limb
is visibly bent, so stylised arm and leg proportions still bend in the right
place.
"""
import heapq, math
import numpy as np

from fit_rig import CHAINS

OFFSETS = np.array([(i, j, k) for i in (-1, 0, 1) for j in (-1, 0, 1) for k in (-1, 0, 1) if (i, j, k) != (0, 0, 0)])


def sample_surface(V, F, count=60000, seed=3):
    """Area-weighted surface samples plus all vertices, so thin parts stay connected."""
    tri = V[F]; area = np.linalg.norm(np.cross(tri[:, 1] - tri[:, 0], tri[:, 2] - tri[:, 0]), axis=1) / 2
    if area.sum() <= 0: return V.copy()
    rng = np.random.default_rng(seed)
    pick = rng.choice(len(F), size=count, p=area / area.sum())
    a, b = rng.random(count), rng.random(count); flip = a + b > 1; a[flip], b[flip] = 1 - a[flip], 1 - b[flip]
    t = tri[pick]
    return np.concatenate([V, t[:, 0] + a[:, None] * (t[:, 1] - t[:, 0]) + b[:, None] * (t[:, 2] - t[:, 0])])


class Graph:
    """Occupied voxels of a point cloud, 26-connected, edge weight = centre distance.

    With fill=True the closed interior is added (flood fill from outside), so
    shortest paths run through the body's core instead of along its skin.
    """

    def __init__(self, pts, voxel, fill=True):
        self.voxel = voxel
        ijk = np.floor(pts / voxel).astype(np.int64); lo = ijk.min(0) - 2; ijk -= lo
        dims = ijk.max(0) + 3
        occ = np.zeros(dims, bool); occ[ijk[:, 0], ijk[:, 1], ijk[:, 2]] = True
        if fill:
            outside = np.zeros(dims, bool); outside[0, :, :] = outside[-1, :, :] = True
            outside[:, 0, :] = outside[:, -1, :] = outside[:, :, 0] = outside[:, :, -1] = True
            outside &= ~occ
            while True:
                grow = outside.copy()
                grow[1:] |= outside[:-1]; grow[:-1] |= outside[1:]
                grow[:, 1:] |= outside[:, :-1]; grow[:, :-1] |= outside[:, 1:]
                grow[:, :, 1:] |= outside[:, :, :-1]; grow[:, :, :-1] |= outside[:, :, 1:]
                grow &= ~occ
                if grow.sum() == outside.sum(): break
                outside = grow
            occ |= ~outside
        vijk = np.argwhere(occ)
        key = lambda q: (q[:, 0] * dims[1] + q[:, 1]) * dims[2] + q[:, 2]
        self.keys = key(vijk)  # argwhere is in C order, so keys are sorted
        n = len(self.keys)
        self.point_voxel = np.searchsorted(self.keys, key(ijk))
        counts = np.bincount(self.point_voxel, minlength=n)
        sums = np.stack([np.bincount(self.point_voxel, pts[:, i], n) for i in range(3)], 1)
        self.centre = np.where(counts[:, None] > 0, sums / np.maximum(counts, 1)[:, None], (vijk + lo + .5) * voxel)
        src, dst = [], []
        for o in OFFSETS:
            nk = key(vijk + o)
            pos = np.minimum(np.searchsorted(self.keys, nk), n - 1)
            ok = self.keys[pos] == nk
            src.append(np.nonzero(ok)[0]); dst.append(pos[ok])
        src = np.concatenate(src); dst = np.concatenate(dst)
        order = np.argsort(src, kind='stable'); src, dst = src[order], dst[order]
        self.indptr = np.searchsorted(src, np.arange(n + 1)); self.indices = dst
        self.weight = np.linalg.norm(self.centre[src] - self.centre[dst], axis=1)
        self.n = n

    def dijkstra(self, sources, labels=None, cutoff=np.inf):
        dist = np.full(self.n, np.inf); parent = np.full(self.n, -1); label = np.full(self.n, -1)
        heap = []
        for i, s in enumerate(sources):
            dist[s] = 0; label[s] = labels[i] if labels is not None else s; heap.append((0., int(s)))
        heapq.heapify(heap)
        ip, ind, w = self.indptr, self.indices.tolist(), self.weight.tolist()
        ip = ip.tolist(); dl = dist.tolist(); pl = parent.tolist(); ll = label.tolist()
        while heap:
            d, u = heapq.heappop(heap)
            if d > dl[u]: continue
            if d > cutoff: break
            for e in range(ip[u], ip[u + 1]):
                v = ind[e]; nd = d + w[e]
                if nd < dl[v]:
                    dl[v] = nd; pl[v] = u; ll[v] = ll[u]; heapq.heappush(heap, (nd, v))
        return np.array(dl), np.array(pl), np.array(ll)


def _tips(graph, dist, parent, min_branch, limit=8):
    """Farthest-point skeleton growth: each new tip is the voxel geodesically
    farthest from every path found so far; stop when that distance is short.
    (Ordering by distance from the root alone fails in thick limbs, where
    parallel shortest paths only merge far from the tip.)"""
    reach = np.isfinite(dist)
    first = int(np.argmax(np.where(reach, dist, -1)))
    tips = [dict(voxel=first, branch=float(dist[first]), geodesic=float(dist[first]))]
    on_path = _path(parent, first)
    while len(tips) < limit:
        d, _, _ = graph.dijkstra(on_path)
        d = np.where(reach & np.isfinite(d), d, -1); v = int(np.argmax(d))
        if d[v] < min_branch: break
        tips.append(dict(voxel=v, branch=float(d[v]), geodesic=float(dist[v])))
        on_path = on_path + _path(parent, v)
    return tips


def _path(parent, v):
    out = []
    while v != -1: out.append(v); v = parent[v]
    return out


def _centreline(graph, dist, path, radius):
    """Mean of voxels near each path voxel and in the same geodesic band: the limb axis."""
    c = graph.centre; out = []
    band = graph.voxel * 1.5
    for v in path:
        near = np.nonzero(np.linalg.norm(c - c[v], axis=1) < radius)[0]
        near = near[np.abs(dist[near] - dist[v]) < band]
        out.append(c[near].mean(0) if len(near) else c[v])
    out = np.array(out)
    # Light smoothing along the line; keep the ends.
    for _ in range(3): out[1:-1] = (out[:-2] + 2 * out[1:-1] + out[2:]) / 4
    s = np.concatenate([[0], np.cumsum(np.linalg.norm(np.diff(out, axis=0), axis=1))])
    return out, s


def _at(line, s, value):
    value = min(max(value, 0), s[-1]); i = min(np.searchsorted(s, value), len(s) - 1)
    if i == 0: return line[0]
    t = (value - s[i - 1]) / max(s[i] - s[i - 1], 1e-9)
    return line[i - 1] + (line[i] - line[i - 1]) * t


def project(line, s, pts):
    """Arc length of each point's nearest position on a polyline, and the distance."""
    a, b = line[:-1], line[1:]; ab = b - a; L = np.maximum((ab * ab).sum(1), 1e-12)
    best_s = np.zeros(len(pts)); best_d = np.full(len(pts), np.inf)
    for k in range(0, len(pts), 8192):
        p = pts[k:k + 8192]
        t = np.clip(((p[:, None] - a[None]) * ab[None]).sum(2) / L[None], 0, 1)
        q = a[None] + t[..., None] * ab[None]; d = np.linalg.norm(p[:, None] - q, axis=2)
        j = d.argmin(1); best_d[k:k + 8192] = d[np.arange(len(p)), j]
        best_s[k:k + 8192] = s[j] + t[np.arange(len(p)), j] * np.sqrt(L[j])
    return best_s, best_d


class Skeleton:
    """Tips, limb centre lines and the geodesic graph of one body point cloud."""

    def __init__(self, pts, voxel=None):
        self.pts = pts
        height = pts[:, 1].max() - pts[:, 1].min(); self.height = height
        self.graph = g = Graph(pts, voxel or height / 90)
        # The largest connected component holds the body; start at its centre.
        d0, _, lab = g.dijkstra([int(np.argmin(np.linalg.norm(g.centre - pts.mean(0), axis=1)))])
        if np.isinf(d0).sum() > .3 * g.n:
            raise RuntimeError('Body surface is split into large disconnected pieces; pass --landmarks')
        self.root = int(np.argmin(np.linalg.norm(g.centre - g.centre[np.isfinite(d0)].mean(0), axis=1)))
        self.dist, self.parent, _ = g.dijkstra([self.root])
        tips = _tips(g, self.dist, self.parent, .15 * height)
        if len(tips) < 5: raise RuntimeError(f'Found {len(tips)} limb tips, need head, hands and feet; pass --landmarks')
        c = g.centre; y0 = c[:, 1].min()
        for t in tips: t['pos'] = c[t['voxel']]
        low = sorted(tips, key=lambda t: t['pos'][1])
        feet = [t for t in low[:2] if t['pos'][1] - y0 < .3 * height]
        if len(feet) < 2: raise RuntimeError('Could not find two feet near the ground; pass --landmarks')
        rest = [t for t in tips if t not in feet]
        # The head is the bulkiest tip: most voxels within a short geodesic radius.
        # (Geodesic distance from the root cannot separate head from hands: hands
        # resting on a flared skirt or raised overhead short-circuit it.)
        for t in rest:
            d, _, _ = g.dijkstra([t['voxel']], cutoff=.1 * height); t['bulk'] = int((d < .1 * height).sum())
        high = [t for t in rest if t['pos'][1] - y0 > .45 * height] or rest
        head = max(high, key=lambda t: t['bulk'])
        hands = sorted([t for t in rest if t is not head], key=lambda t: -t['branch'])[:2]
        if len(hands) < 2: raise RuntimeError('Could not find two hands; pass --landmarks')
        self.tips = {'head': head, 'foot_a': feet[0], 'foot_b': feet[1], 'hand_a': hands[0], 'hand_b': hands[1]}
        self.extra_tips = [t for t in tips if t not in self.tips.values()]
        self.lines = {}
        for name, t in self.tips.items():
            path = _path(self.parent, t['voxel'])
            line, s = _centreline(g, self.dist, path, .07 * height if name != 'head' else .09 * height)
            self.lines[name] = (line, s, path)
        self.span = (self.tips['head']['geodesic'] + (self.tips['foot_a']['geodesic'] + self.tips['foot_b']['geodesic']) / 2)

    def forward(self):
        """Horizontal toe direction, averaged over both feet (unit, y = 0)."""
        out = np.zeros(3)
        for f in ('foot_a', 'foot_b'):
            line, s, _ = self.lines[f]
            v = line[0] - _at(line, s, .08 * self.height); v[1] = 0
            out += v / max(np.linalg.norm(v), 1e-9)
        return out / max(np.linalg.norm(out), 1e-9)


def calibrate(rig):
    """Arc length of every chain joint from its tip, on the 0 A.D. body (bind pose)."""
    from fit_rig import Rig  # noqa: F401  (rig is a fit_rig.Rig)
    used = np.unique(rig.body_faces); remap = np.zeros(len(rig.P), np.int64); remap[used] = np.arange(len(used))
    pts = sample_surface(rig.P[used], remap[rig.body_faces], 40000)
    sk = Skeleton(pts)
    side = lambda t: 'L' if sk.lines[t][0][min(len(sk.lines[t][0]) - 1, 20)][0] > 0 else 'R'
    names = {'head': 'torso'}
    for t in ('hand_a', 'hand_b'): names[t] = 'arm_' + side(t)
    for t in ('foot_a', 'foot_b'): names[t] = 'leg_' + side(t)
    # Joint arc lengths from each tip are the 0 A.D. bone lengths summed from the
    # tip. (Projecting joints onto the donor's own paths fails: its hanging
    # hands touch the flared tunic, so the hand paths shortcut through the hips.)
    cal = {'span': sk.span, 'height': sk.height, 'arc': {}}
    for chain, joints in CHAINS.items():
        acc = 0.; cal['arc'][joints[-1]] = 0.
        for a, b in zip(reversed(joints[:-1]), reversed(joints[1:])):
            acc += float(np.linalg.norm(rig.joint[a] - rig.joint[b])); cal['arc'][a] = acc
    return cal, sk, names


def _bend(line, s, a, b, min_dev):
    """Point of greatest deviation from the chord between arc lengths a < b (or None)."""
    sel = (s > a) & (s < b)
    if sel.sum() < 3: return None
    p0, p1 = _at(line, s, a), _at(line, s, b); u = (p1 - p0) / max(np.linalg.norm(p1 - p0), 1e-9)
    q = line[sel] - p0; dev = np.linalg.norm(q - np.outer(q @ u, u), axis=1)
    i = dev.argmax()
    return float(s[sel][i]) if dev[i] > min_dev else None


def joints_from_mesh(pts, cal, landmarks=None):
    """Source joint positions for every chain entry, plus labelling data for unposing.

    Returns (joints, skeleton, chain_lines) where chain_lines[chain] = (line, s,
    {joint: arc}) is used to weight vertices along each chain.
    """
    sk = Skeleton(pts)
    k = sk.span / cal['span']
    side = lambda t, i=20: 'L' if sk.lines[t][0][min(len(sk.lines[t][0]) - 1, i)][0] > 0 else 'R'
    names = {'head': 'torso'}
    ha, hb = ('hand_a', 'hand_b')
    sa = _at(sk.lines[ha][0], sk.lines[ha][1], k * cal['arc']['arm_L'])[0]
    sb = _at(sk.lines[hb][0], sk.lines[hb][1], k * cal['arc']['arm_L'])[0]
    names[ha], names[hb] = ('arm_L', 'arm_R') if sa > sb else ('arm_R', 'arm_L')
    fa = _at(sk.lines['foot_a'][0], sk.lines['foot_a'][1], k * cal['arc']['thigh_L'])[0]
    fb = _at(sk.lines['foot_b'][0], sk.lines['foot_b'][1], k * cal['arc']['thigh_L'])[0]
    names['foot_a'], names['foot_b'] = ('leg_L', 'leg_R') if fa > fb else ('leg_R', 'leg_L')
    joints = {}; chains = {}
    for tip, chain in names.items():
        line, s, path = sk.lines[tip]; arcs = {}
        for j in CHAINS[chain]:
            ref = j.replace('_R', '_L') if chain != 'torso' else j
            arcs[j] = k * cal['arc'][ref]
        mid = CHAINS[chain][1] if chain != 'torso' else None
        if mid:
            bent = _bend(line, s, arcs[CHAINS[chain][2]], arcs[CHAINS[chain][0]], .025 * sk.height)
            if bent is not None: arcs[mid] = bent
        for j, a in arcs.items():
            joints[j] = _at(line, s, a)
        chains[chain] = (line, s, arcs)
    if landmarks:
        joints.update({k2: np.array(v, float) for k2, v in landmarks.items()})
    return joints, sk, chains, names


# -- armature route -----------------------------------------------------------------
import re

ARMATURE_NAMES = {
    # 0 A.D. chain joint: regexes tried in order against lower-case bone names.
    'hip': [r'(^|[:_ ])(hips?|pelvis)$', r'root$'],
    'spine': [r'spine_?0?1$', r'(^|[:_ ])spine$'],
    'spine1': [r'spine_?0?2$', r'spine1$'],
    'chest': [r'spine_?0?3$', r'spine2$', r'chest$', r'upper_?chest$'],
    'neck': [r'neck(_?0?1)?$'],
    'head': [r'(^|[:_ ])head$'],
    'arm_{s}': [r'upper_?arm[_.]?{s}$', r'{side}_?arm$', r'arm[_.]?{s}$', r'{s}[_.]?upper_?arm$'],
    'forearm_{s}': [r'(lower_?arm|fore_?arm)[_.]?{s}$', r'{side}_?fore_?arm$', r'{s}[_.]?(fore|lower)_?arm$'],
    'hand_{s}': [r'(^|[:_ ])hand[_.]?{s}$', r'{side}_?hand$', r'{s}[_.]?hand$'],
    'thigh_{s}': [r'thigh[_.]?{s}$', r'{side}_?up_?leg$', r'upper_?leg[_.]?{s}$', r'{s}[_.]?thigh$'],
    'leg_{s}': [r'(calf|shin|lower_?leg)[_.]?{s}$', r'{side}_?leg$', r'{s}[_.]?(calf|shin)$'],
    'foot_{s}': [r'(^|[:_ ])foot[_.]?{s}$', r'{side}_?foot$', r'{s}[_.]?foot$'],
}


def map_armature(bone_names):
    """0 A.D. joint -> source bone name for common rigs (MPFB/Unreal, Mixamo, Rigify-like)."""
    out = {}
    low = {b.lower(): b for b in bone_names}
    for key, patterns in ARMATURE_NAMES.items():
        for s, side in ([('l', 'left'), ('r', 'right')] if '{s}' in key else [(None, None)]):
            name = key.format(s='L' if s == 'l' else 'R') if s else key
            for pat in patterns:
                rx = re.compile(pat.format(s=s or '', side=side or ''))
                hit = [b for lb, b in low.items() if rx.search(lb)]
                if hit: out[name] = sorted(hit, key=len)[0]; break
    return out
