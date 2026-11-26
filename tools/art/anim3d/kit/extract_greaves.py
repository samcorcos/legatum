"""Cut the greaves out of a fused character scan (e.g. a Tripo-generated hoplite).

Note: the r9 trial used the Aegis Sentinel scan, an AI-generated (Tripo) CC BY 4.0 source that
was evaluated and not shipped; it is not a shipped source.

    blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/extract_greaves.py -- \
        <scan.glb> <out.glb> [--lo .06 --hi .34 --faces 600]

Faces in each leg's shin-and-knee band (fractions of figure height) whose baked
base colour is not skin (skin: clearly warmer red than blue) are kept; per leg the
largest connected piece survives, grown by one ring, small holes closed, islands
dropped, then decimated to --faces per greave. The scan's own material and UVs are
kept for the bake. Writes one GLB with both greaves (Y up, facing +Z).
"""
import argparse, sys
import bpy, bmesh
import numpy as np

a = argparse.ArgumentParser()
a.add_argument('src'); a.add_argument('out')
a.add_argument('--lo', type=float, default=.06); a.add_argument('--hi', type=float, default=.34)
a.add_argument('--faces', type=int, default=600)
a = a.parse_args(sys.argv[sys.argv.index('--') + 1:])
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=a.src, merge_vertices=True)
meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']
dep = bpy.context.evaluated_depsgraph_get()
for o in meshes:
    me = bpy.data.meshes.new_from_object(o.evaluated_get(dep), preserve_all_data_layers=True, depsgraph=dep)
    me.transform(o.matrix_world); o.data = me; o.parent = None; o.matrix_world.identity()
bpy.ops.object.select_all(action='DESELECT')
for o in meshes: o.select_set(True)
bpy.context.view_layer.objects.active = meshes[0]; bpy.ops.object.join(); o = meshes[0]
for x in list(bpy.context.scene.objects):
    if x is not o: bpy.data.objects.remove(x, do_unlink=True)
me = o.data
# Base colour image of the material.
mat = me.materials[0]; bsdf = next(n for n in mat.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
img = bsdf.inputs['Base Color'].links[0].from_node.image
W, H = img.size; px = np.array(img.pixels[:], np.float32).reshape(H, W, img.channels)[..., :3]
P = np.array([v.co[:] for v in me.vertices]); z0, z1 = P[:, 2].min(), P[:, 2].max(); h = z1 - z0
uvl = me.uv_layers.active.data
# Leg axis per side and height: trimmed-mean centre of the band's faces, shrinking
# the radius so a cape or skirt hanging behind the legs drops out.
FC = np.array([p.center[:] for p in me.polygons]); fz = (FC[:, 2] - z0) / h
slices = np.linspace(a.lo, a.hi, 15); centre = {}
for sd in (-1, 1):
    for k in range(len(slices) - 1):
        m = (fz > slices[k]) & (fz <= slices[k + 1]) & (np.sign(FC[:, 0]) == sd) & (np.abs(FC[:, 0]) > .015 * h)
        if m.sum() < 5: continue
        c = np.median(FC[m, :2], 0)
        for r in (.12, .09, .07):
            near = m & (np.linalg.norm(FC[:, :2] - c, axis=1) < r * h)
            if near.sum() >= 5: c = FC[near, :2].mean(0)
        centre[(sd, k)] = c
keep = np.zeros(len(me.polygons), bool)
for p in me.polygons:
    c = p.center; f = (c.z - z0) / h
    if not (a.lo < f < a.hi) or abs(c.x) < .015 * h: continue
    k = min(len(slices) - 2, max(0, int(np.searchsorted(slices, f) - 1)))
    cc = centre.get((np.sign(c.x), k))
    if cc is None or np.hypot(c.x - cc[0], c.y - cc[1]) > .075 * h: continue   # off the leg (cape, skirt)
    u, v = np.mean([tuple(uvl[li].uv) for li in p.loop_indices], 0)
    r, g, b = px[min(H - 1, max(0, int(v * H))), min(W - 1, max(0, int(u * W)))]
    skin = r > b + .12 and r > .18          # warm flesh tones
    keep[p.index] = not skin
bm = bmesh.new(); bm.from_mesh(me); bm.faces.ensure_lookup_table()
sel = set(f for f in bm.faces if keep[f.index])
# grow one ring inside the band, so seams between metal and strap shading stay closed
ring = set()
for f in sel:
    for e in f.edges:
        for g_ in e.link_faces:
            fz = (g_.calc_center_median().z - z0) / h
            if g_ not in sel and a.lo < fz < a.hi: ring.add(g_)
sel |= ring
bmesh.ops.delete(bm, geom=[f for f in bm.faces if f not in sel], context='FACES')
bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context='VERTS')
# components; keep the largest per side
comp = {}; seen = set(); cid = 0
for f in bm.faces:
    if f in seen: continue
    stack = [f]; seen.add(f); grp = []
    while stack:
        x = stack.pop(); grp.append(x)
        for e in x.edges:
            for y in e.link_faces:
                if y not in seen: seen.add(y); stack.append(y)
    comp[cid] = grp; cid += 1
best = {}; score = {}
for k, grp in comp.items():
    C = np.array([x.calc_center_median()[:] for x in grp])
    if len(C) < 30: continue
    ax = np.linalg.svd(C - C.mean(0), full_matrices=False)[2][0]
    if abs(ax[2]) < .85: continue                     # not upright: a sword, a strap, a cape edge
    side = np.sign(C[:, 0].mean()); sc = np.ptp(C[:, 2]) * len(C) ** .5
    if side not in best or sc > score[side]: best[side] = k; score[side] = sc
keepf = set(f for k in best.values() for f in comp[k])
bmesh.ops.delete(bm, geom=[f for f in bm.faces if f not in keepf], context='FACES')
bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context='VERTS')
bmesh.ops.holes_fill(bm, edges=bm.edges, sides=12)
bm.to_mesh(me); bm.free()
stats = {'sides': len(best), 'faces_before_decimate': len(me.polygons)}
bpy.context.view_layer.objects.active = o
m = o.modifiers.new('d', 'DECIMATE'); m.ratio = min(1, 2 * a.faces / max(1, len(me.polygons))); bpy.ops.object.modifier_apply(modifier=m.name)
m = o.modifiers.new('s', 'SMOOTH'); m.factor = .4; m.iterations = 2; bpy.ops.object.modifier_apply(modifier=m.name)
for p in me.polygons: p.use_smooth = True
stats['faces'] = len(me.polygons)
o.name = 'Greaves'
bpy.ops.object.select_all(action='DESELECT'); o.select_set(True)
bpy.ops.export_scene.gltf(filepath=a.out, export_format='GLB', use_selection=True)
print('GREAVES', stats, flush=True)
