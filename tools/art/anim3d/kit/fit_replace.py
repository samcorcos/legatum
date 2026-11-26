"""Swap a fitted part for an external mesh: a helmet shell on the head, greaves on the shins.

bpy + numpy, glTF bind space (Y up, the figure faces +Z). An external part is
imported (glTF, GLB or FBX), converted to glTF axes, optionally re-oriented, and
fitted by a bounding-shell fit: scaled and centred to enclose the target skin with
a small clearance, then grown until no skin vertex pokes through it (BVH ray test).
Source orientation convention: the part's up is +Y and its front +Z in glTF terms,
which is what a Y-up glTF/GLB of a worn piece facing forward gives; `rot` (degrees
about X, Y, Z) fixes anything else.
"""
import math
from pathlib import Path
import bpy, bmesh
import numpy as np
from mathutils import Matrix, Vector, Euler
from mathutils.bvhtree import BVHTree

CONV = Matrix(((1, 0, 0, 0), (0, 0, 1, 0), (0, -1, 0, 0), (0, 0, 0, 1)))  # Blender Z-up -> glTF Y-up


def co(o):
    a = np.empty(len(o.data.vertices) * 3); o.data.vertices.foreach_get('co', a); return a.reshape(-1, 3)


def set_co(o, a):
    o.data.vertices.foreach_set('co', np.ascontiguousarray(a, dtype=np.float64).ravel()); o.data.update()


def import_part(path, obj=None, rot=(0, 0, 0)):
    """Mesh objects of an external file in glTF axes, joined into one object; materials kept."""
    path = Path(path); before = set(bpy.data.objects)
    if path.suffix.lower() in ('.glb', '.gltf'): bpy.ops.import_scene.gltf(filepath=str(path), merge_vertices=True)
    elif path.suffix.lower() == '.fbx': bpy.ops.import_scene.fbx(filepath=str(path))
    else: raise SystemExit(f'--replace-part: {path} must be .glb, .gltf or .fbx')
    new = [o for o in bpy.data.objects if o not in before]
    meshes = [o for o in new if o.type == 'MESH' and (obj is None or o.name == obj or o.name.startswith(obj))]
    if not meshes: raise SystemExit(f'--replace-part: no mesh {obj or ""} in {path}; meshes: {[o.name for o in new if o.type == "MESH"]}')
    dep = bpy.context.evaluated_depsgraph_get(); parts = []
    R = Euler([math.radians(r) for r in rot], 'XYZ').to_matrix().to_4x4()
    for o in meshes:
        me = bpy.data.meshes.new_from_object(o.evaluated_get(dep), preserve_all_data_layers=True, depsgraph=dep)
        me.transform(R @ CONV @ o.matrix_world)
        if o.matrix_world.determinant() < 0: me.flip_normals()
        x = bpy.data.objects.new(o.name, me); parts.append(x)
    for o in new: bpy.data.objects.remove(o, do_unlink=True)
    for x in parts: bpy.context.scene.collection.objects.link(x)
    for x in parts:
        for p in x.data.polygons: p.use_smooth = True
    return parts


def _join(parts, name):
    bpy.ops.object.select_all(action='DESELECT')
    for x in parts: x.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    if len(parts) > 1: bpy.ops.object.join()
    o = bpy.context.view_layer.objects.active; o.name = name; return o


def _pokes(shell, pts, centre):
    """Fraction of pts not covered by the shell: a ray from the point, away from centre, hits nothing."""
    bm = bmesh.new(); bm.from_mesh(shell.data); bvh = BVHTree.FromBMesh(bm); bm.free()
    out = 0; c = Vector(centre)
    for p in pts:
        d = Vector(p) - c; L = d.length
        if L < 1e-6: continue
        if bvh.ray_cast(Vector(p), d / L, 1.0)[0] is None: out += 1
    return out / max(1, len(pts))


def fit_helmet(parts, head_pts, clearance=.004, wall=.95, size=1.0):
    """Seat the helmet shell on the head at its own proportions (about head size).

    The shell is scaled so its inner width/depth (wall: the inner share of the
    outer extent) matches the head plus a small clearance, times `size`; it is
    NOT grown to swallow skin that pokes out: `tuck_skin` moves that skin in.
    """
    o = _join(parts, 'HIGH-helmet-replaced'); P = co(o)
    lo, hi = head_pts.min(0), head_pts.max(0); hc = (lo + hi) / 2; he = hi - lo
    plo, phi = P.min(0), P.max(0); pe = phi - plo
    s = size * max((he[0] + 2 * clearance) / (pe[0] * wall), (he[2] + 2 * clearance) / (pe[2] * wall))
    P = (P - np.array([(plo[0] + phi[0]) / 2, phi[1], (plo[2] + phi[2]) / 2])) * s
    P += np.array([hc[0], hi[1] + clearance + pe[1] * s * .025, hc[2]])
    set_co(o, P)
    centre = np.array([hc[0], hc[1] + he[1] * .15, hc[2]])
    return o, {'scale': round(float(s), 4), 'centre': [round(float(x), 3) for x in centre],
               'triangles': sum(len(p.vertices) - 2 for p in o.data.polygons)}, centre


def tuck_skin(skin_objs, shell, centre, gap=.004, reach=.05, front_cone=50, steps=40):
    """Move skin that pokes out through (or touches) a shell inward, toward centre,
    until a ray outward meets the shell at least `gap` away. The face behind the
    eye/mouth openings (a cone about +Z) is left alone."""
    bm = bmesh.new(); bm.from_mesh(shell.data); bvh = BVHTree.FromBMesh(bm); bm.free()
    moved = 0; c = Vector(centre); cos_f = math.cos(math.radians(front_cone)); rim_y = co(shell)[:, 1].min() + .03
    for o in skin_objs:
        P = co(o); changed = False
        for i, p in enumerate(P):
            q = Vector(p)
            if p[1] < rim_y: continue                     # neck and jaw below the helmet rim stay put
            if bvh.find_nearest(q, reach)[0] is None: continue
            d = q - c
            if d.length < 1e-6: continue
            if d.normalized().z > cos_f and p[1] < centre[1] + .05: continue
            for _ in range(steps):
                hit = bvh.ray_cast(q, (q - c).normalized(), .15)
                if hit[0] is not None and hit[3] >= gap: break
                q = c + (q - c) * .985
            if (q - Vector(p)).length > 1e-6: P[i] = q; changed = True; moved += 1
        if changed: set_co(o, P)
    return moved


def inner_faces(o, centre, thresh=.3):
    """Polygon indices facing the centre (the inside of a shell)."""
    out = []
    for p in o.data.polygons:
        v = Vector(centre) - p.center
        if v.length > 1e-6 and p.normal.dot(v.normalized()) > thresh: out.append(p.index)
    return out


def finish_greave(o, knee, ankle, bronze, rim_r=.0045, dome=(.034, .04, .016)):
    """Clean the greave top: smooth its top profile into a level-ish arc, add a rolled
    rim tube along it and a rounded kneecap dome. Returns the new rim and dome objects."""
    P = co(o); u = (knee - ankle) / np.linalg.norm(knee - ankle)
    side = np.cross(u, [0, 0, 1.]); side /= np.linalg.norm(side); front = np.cross(side, u)
    q = P - ankle; h = q @ u; th = np.arctan2(q @ side, q @ front); r = np.linalg.norm(q - np.outer(h, u), axis=1)
    nb = 36; bins = np.linspace(-np.pi, np.pi, nb + 1); bi = np.clip(np.digitize(th, bins) - 1, 0, nb - 1)
    top = np.full(nb, np.nan); rad = np.full(nb, np.nan)
    for b in range(nb):
        m = bi == b
        if m.sum() >= 3:
            top[b] = np.percentile(h[m], 97); rad[b] = r[m][h[m] > top[b] - .02].max() if (h[m] > top[b] - .02).any() else r[m].max()
    ok = np.isfinite(top)
    # Circular low-pass over the present bins, then a gentle arc: highest at the front.
    sm = top.copy()
    for _ in range(6):
        nxt = sm.copy()
        for b in np.nonzero(ok)[0]:
            nb_ = [x for x in ((b - 1) % nb, b, (b + 1) % nb) if ok[x]]
            nxt[b] = np.mean(sm[nb_])
        sm = nxt
    centres = (bins[:-1] + bins[1:]) / 2
    arc = np.where(ok, np.minimum(sm, np.nanmax(sm) - .03 * (1 - np.cos(centres)) / 2), np.nan)
    tgt = np.interp(th, centres[ok], arc[ok], period=2 * np.pi)
    clamp = h > tgt
    P[clamp] -= np.outer(h[clamp] - tgt[clamp], u)
    set_co(o, P)
    # Rolled rim: a tube along the clamped edge, just outside the greave surface.
    idx = np.nonzero(ok)[0]
    # order the present bins into one run (the greave is open at the back)
    gap_at = np.argmax(np.diff(np.r_[idx, idx[0] + nb]))
    run = np.r_[idx[gap_at + 1:], idx[:gap_at + 1]]
    pts = [ankle + u * arc[b] + (side * math.sin(centres[b]) + front * math.cos(centres[b])) * (rad[b] + rim_r * .6) for b in run]
    pts = np.array(pts)
    for _ in range(2): pts[1:-1] = (pts[:-2] + 2 * pts[1:-1] + pts[2:]) / 4
    verts, faces, uvs = [], [], []; seg = 8
    for i, p in enumerate(pts):
        t = pts[min(i + 1, len(pts) - 1)] - pts[max(i - 1, 0)]; t /= np.linalg.norm(t)
        a = np.cross(t, u); a /= max(np.linalg.norm(a), 1e-9); b = np.cross(t, a)
        for j in range(seg):
            ang = 2 * math.pi * j / seg; verts.append(tuple(p + rim_r * (a * math.cos(ang) + b * math.sin(ang)))); uvs.append((i / len(pts), j / seg))
    for i in range(len(pts) - 1):
        for j in range(seg): faces.append((i * seg + j, i * seg + (j + 1) % seg, (i + 1) * seg + (j + 1) % seg, (i + 1) * seg + j))
    from fit_shells import _mesh
    rim = _mesh(o.name + '-rim', verts, faces, uvs, bronze)
    # Kneecap dome on the front of the greave top.
    fb = np.argmin(np.abs(centres))
    kc = ankle + u * ((arc[fb] if ok[fb] else np.nanmax(arc)) - dome[1] * .55) + front * ((rad[fb] if ok[fb] else np.nanmax(rad)) + .002)
    verts, faces, uvs = [], [], []; nu, nv = 16, 8
    for i in range(nv + 1):
        el = math.pi / 2 * i / nv           # 0 at the rim of the cap, pi/2 at its apex
        for j in range(nu + 1):
            az = 2 * math.pi * j / nu
            x, y = math.cos(el) * math.cos(az), math.cos(el) * math.sin(az)
            verts.append(tuple(kc + side * dome[0] * x + u * dome[1] * y + front * dome[2] * math.sin(el))); uvs.append((j / nu, i / nv))
    w = nu + 1
    for i in range(nv):
        for j in range(nu): faces.append((i * w + j, i * w + j + 1, (i + 1) * w + j + 1, (i + 1) * w + j))
    cap = _mesh(o.name + '-kneecap', verts, faces, uvs, bronze)
    return [rim, cap], {'top_clamped_vertices': int(clamp.sum()), 'rim_points': int(len(pts))}


def delete_covered_skin(skin_objs, shells, axis_pts, nb=36, nh=16):
    """Delete skin faces lying inside a shell's angular and height span about the
    limb axis (bind pose): that skin is behind the armour, or poking through it."""
    a0, a1 = axis_pts; u = (a1 - a0) / np.linalg.norm(a1 - a0)
    side = np.cross(u, [0, 0, 1.]); side /= np.linalg.norm(side); front = np.cross(side, u)
    S = np.concatenate([co(x) for x in shells]); q = S - a0
    st = np.arctan2(q @ side, q @ front); sh = q @ u
    tb = np.linspace(-np.pi, np.pi, nb + 1); hb = np.linspace(sh.min(), sh.max(), nh + 1)
    cov = np.zeros((nb, nh), bool)
    ti = np.clip(np.digitize(st, tb) - 1, 0, nb - 1); hi_ = np.clip(np.digitize(sh, hb) - 1, 0, nh - 1)
    cov[ti, hi_] = True
    sr = np.linalg.norm(q - np.outer(sh, u), axis=1); rmin = np.full((nb, nh), np.inf)
    np.minimum.at(rmin, (ti, hi_), sr)
    for _ in range(2):  # fill neighbours so edge bins have a radius too
        rmin = np.minimum(rmin, np.minimum(np.roll(rmin, 1, 0), np.roll(rmin, -1, 0)))
        rmin = np.minimum(rmin, np.minimum(np.roll(rmin, 1, 1), np.roll(rmin, -1, 1)))
    cov &= np.roll(cov, 1, 0) | np.roll(cov, -1, 0)      # ignore isolated bins at the open edges
    cov |= np.roll(cov, 1, 1) & np.roll(cov, -1, 1)       # close single-row gaps up the shin
    gone = 0
    for o in skin_objs:
        P = co(o); q = P - a0; t = np.arctan2(q @ side, q @ front); h = q @ u
        r = np.linalg.norm(q - np.outer(h, u), axis=1)
        i = np.clip(np.digitize(t, tb) - 1, 0, nb - 1); j = np.digitize(h, hb) - 1
        inside = (j >= 0) & (j < nh) & (r < .12)
        c_ = np.zeros(len(P), bool); c_[inside] = cov[i[inside], j[inside]]
        # Skin left at the shell's edges (faces only partly covered) sinks 5 mm
        # inside the shell's inner surface at that angle and height.
        lim = np.full(len(P), np.inf); lim[inside] = rmin[i[inside], j[inside]] - .005
        pull = c_ & (r > lim) & (j > 0) & (j < nh - 1)   # not at the shell's top and bottom rows (spikes)
        if pull.any():
            radial = (q - np.outer(h, u))[pull]; P[pull] = a0 + np.outer(h[pull], u) + radial / r[pull][:, None] * lim[pull][:, None]
            set_co(o, P)
        bm2 = bmesh.new(); bm2.from_mesh(o.data); bm2.verts.ensure_lookup_table()
        dead = [f for f in bm2.faces if all(c_[v.index] for v in f.verts)]
        bmesh.ops.delete(bm2, geom=dead, context='FACES'); bm2.to_mesh(o.data); bm2.free(); gone += len(dead)
    return gone


def fit_greave(parts, shin_pts, knee, ankle, mirror=False, clearance=.01, name='HIGH-layer-greave'):
    """Wrap a greave around one shin from just above the ankle to over the knee."""
    src = []
    for p in parts:
        x = p.copy(); x.data = p.data.copy(); bpy.context.scene.collection.objects.link(x); src.append(x)
    o = _join(src, name)
    if mirror:
        set_co(o, co(o) * np.array([-1, 1, 1])); o.data.flip_normals()
    P = co(o); u = (knee - ankle) / np.linalg.norm(knee - ankle)
    top = knee + u * .05; bottom = ankle + u * .07; span = np.linalg.norm(top - bottom)
    plo, phi = P.min(0), P.max(0); pe = phi - plo
    # Cross-section of the shin about its axis (x lateral, z front-back).
    q = shin_pts - bottom; t = q @ u
    radial = np.linalg.norm(q - np.outer(t, u), axis=1)
    sel = (t > 0) & (t < span) & (radial < .075)        # this shin only, not the other leg
    q = q[sel]
    side = np.cross(u, [0, 0, 1.]); side /= np.linalg.norm(side); front = np.cross(side, u)
    ext = lambda v: float(np.percentile(v, 98) - np.percentile(v, 2))
    wx = ext(q @ side) + 2 * clearance; wz = ext(q @ front) + 2 * clearance
    sx, sz = wx / (pe[0] * .82), wz / (pe[2] * .82); sy = span / pe[1]   # walls and the open back take ~18%
    mid = np.array([(plo[0] + phi[0]) / 2, plo[1], (plo[2] + phi[2]) / 2])
    L = (P - mid) * np.array([sx, sy, sz])
    R = np.stack([side, u, front], 1)      # local x, y, z -> lateral, shin axis, front
    centre_off = side * float(np.median(q @ side)) + front * float(np.median(q @ front))
    P = bottom + centre_off + L @ R.T
    set_co(o, P)
    axis_c = bottom + centre_off
    probe = shin_pts[sel][::max(1, sel.sum() // 400)]
    grow = 1.0
    for _ in range(4):   # at most +12%: the skin press handles the rest; more blew greaves up into flaps
        # Points are tested from the shin axis at their own height.
        bm = bmesh.new(); bm.from_mesh(o.data); bvh = BVHTree.FromBMesh(bm); bm.free()
        out = 0
        for p in probe:
            c = axis_c + u * float((p - axis_c) @ u); d = Vector(p - c); Ld = d.length
            if Ld > 1e-6 and (p - c) @ front > 0 and bvh.ray_cast(Vector(p), d / Ld, 1.0)[0] is None: out += 1
        poke = out / max(1, len(probe))
        if poke < .03: break
        grow *= 1.04
        rel = P - axis_c; along = np.outer(rel @ u, u); P = axis_c + along + (rel - along) * 1.04; set_co(o, P)
    return o, {'span_m': round(float(span), 3), 'scale_xyz': [round(float(sx * grow), 3), round(float(sy), 3), round(float(sz * grow), 3)],
               'front_skin_poking_out': round(float(poke), 4), 'mirrored': mirror,
               'triangles': sum(len(p.vertices) - 2 for p in o.data.polygons)}


def delete_faces(objs, test, any_vertex=False):
    """Delete faces whose every (or, with any_vertex, any) vertex satisfies test(positions) -> bool array."""
    gone = 0
    for o in objs:
        P = co(o); keep = test(P)
        bm = bmesh.new(); bm.from_mesh(o.data); bm.verts.ensure_lookup_table()
        dead = [f for f in bm.faces if (any if any_vertex else all)(keep[v.index] for v in f.verts)]
        bmesh.ops.delete(bm, geom=dead, context='FACES'); bm.to_mesh(o.data); bm.free(); gone += len(dead)
    return gone
