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


def fit_helmet(parts, head_pts, clearance=.008):
    """Enclose the head (skin above the neck) with the helmet shell."""
    o = _join(parts, 'HIGH-helmet-replaced'); P = co(o)
    lo, hi = head_pts.min(0), head_pts.max(0); hc = (lo + hi) / 2; he = hi - lo
    plo, phi = P.min(0), P.max(0); pe = phi - plo
    # Shell walls take ~12% of each extent; enclose head + clearance horizontally,
    # crown just above the head top, the lower rim wherever the shell's shape puts it.
    s = max((he[0] + 2 * clearance) / (pe[0] * .88), (he[2] + 2 * clearance) / (pe[2] * .88))
    P = (P - np.array([(plo[0] + phi[0]) / 2, phi[1], (plo[2] + phi[2]) / 2])) * s
    P += np.array([hc[0], hi[1] + clearance + pe[1] * s * .04, hc[2]])  # crown: head top + clearance + shell wall
    set_co(o, P)
    centre = np.array([hc[0], hc[1] + he[1] * .15, hc[2]])
    probe = head_pts[head_pts[:, 1] > hc[1] + he[1] * .25]   # crown and brow: above the eye openings
    grow = 1.0; poke = _pokes(o, probe[::max(1, len(probe) // 600)], centre)
    for _ in range(12):
        if poke < .01: break
        k = 1.03; grow *= k; P = centre + (P - centre) * k; set_co(o, P)
        poke = _pokes(o, probe[::max(1, len(probe) // 600)], centre)
    return o, {'scale': round(float(s * grow), 4), 'grown': round(grow, 3), 'skin_poking_out': round(float(poke), 4),
               'triangles': sum(len(p.vertices) - 2 for p in o.data.polygons)}


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
