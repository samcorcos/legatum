"""Workbench contact sheets for checking a fit: source pose, rest fit, and every clip.

    blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/fit_sheet.py -- \
        <model.glb|.blend> <out.png> [--clips idle,walk,attack,hit,death] [--frames 6] [--views front,side]
        [--lod spearman_LOD0] [--size 360] [--close joint[,joint]]

Rows are clips (or one 'rest' row when the file has no animations), columns are
evenly spaced frames x views. --close adds tight crops around named joints to
inspect candy-wrapping (shoulders, elbows, knees).
"""
import argparse, math, sys
from pathlib import Path
import bpy
import numpy as np
from mathutils import Vector, Matrix

argv = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
ap = argparse.ArgumentParser()
ap.add_argument('model', type=Path); ap.add_argument('out', type=Path)
ap.add_argument('--clips', default='idle,walk,attack,hit,death')
ap.add_argument('--frames', type=int, default=6)
ap.add_argument('--views', default='front,side')
ap.add_argument('--lod', default='spearman_LOD0')
ap.add_argument('--size', type=int, default=360)
ap.add_argument('--close', default='', help='comma-separated joint names for close-up crops')
ap.add_argument('--matcap', action='store_true', help='grey clay instead of texture colour (shows folds/pinches)')
ap.add_argument('--colour', choices=['texture', 'material', 'single', 'vertex'], help='workbench colour source (default texture, or single with --matcap)')
ap.add_argument('--xray', action='store_true', help='see joints and centre lines through the body')
ap.add_argument('--no-props', action='store_true', help='delete vertices rigidly bound to weapon_R, shield_arm or helmet')
ap.add_argument('--close-view', default='three', help='view for --close crops (front, side, back, three)')
ap.add_argument('--yup', action='store_true', help='the .blend is authored in glTF space (Y up), e.g. the fit work scenes')
args = ap.parse_args(argv)

if args.model.suffix == '.blend':
    bpy.ops.wm.open_mainfile(filepath=str(args.model))
else:
    bpy.ops.object.select_all(action='SELECT'); bpy.ops.object.delete(use_global=False)
    bpy.ops.import_scene.gltf(filepath=str(args.model))
scene = bpy.context.scene
for o in scene.objects:
    if o.type == 'MESH' and o.name.startswith('spearman_LOD') and args.lod not in o.name:
        o.hide_render = True
if args.no_props:
    import bmesh
    for o in [o for o in scene.objects if o.type == 'MESH']:
        ids = {o.vertex_groups[n].index for n in ('weapon_R', 'shield_arm', 'helmet') if n in o.vertex_groups}
        bm = bmesh.new(); bm.from_mesh(o.data); dl = bm.verts.layers.deform.verify()
        bmesh.ops.delete(bm, geom=[v for v in bm.verts if any(g in ids and w > .999 for g, w in v[dl].items())], context='VERTS')
        bm.to_mesh(o.data); bm.free()
rigs = [o for o in scene.objects if o.type == 'ARMATURE']
rig = rigs[0] if rigs else None
meshes = [o for o in scene.objects if o.type == 'MESH' and not o.hide_render]
scene.render.engine = 'BLENDER_WORKBENCH'
shading = scene.display.shading
shading.light = 'STUDIO'; shading.color_type = (args.colour or ('single' if args.matcap else 'texture')).upper()
if args.xray:
    shading.show_xray = True; shading.xray_alpha = .35
    for o in scene.objects:
        if o.name.startswith(('joint-', 'line-')): o.show_in_front = True
if args.yup:
    pivot = bpy.data.objects.new('yup', None); scene.collection.objects.link(pivot); pivot.rotation_euler.x = math.pi / 2
    for o in list(scene.objects):
        if o.parent is None and o is not pivot: o.parent = pivot
    bpy.context.view_layer.update()
shading.single_color = (.7, .68, .64); shading.show_cavity = True; shading.cavity_type = 'BOTH'
shading.show_shadows = False; shading.show_backface_culling = False
scene.display.render_aa = '8'
scene.render.resolution_x = scene.render.resolution_y = args.size
scene.render.film_transparent = False
scene.world = scene.world or bpy.data.worlds.new('w')
scene.render.image_settings.file_format = 'PNG'
cam = bpy.data.objects.new('cam', bpy.data.cameras.new('cam')); scene.collection.objects.link(cam)
cam.data.type = 'ORTHO'; scene.camera = cam
tmp = args.out.parent / f'.{args.out.stem}-tiles'; tmp.mkdir(parents=True, exist_ok=True)


def bounds():
    dep = bpy.context.evaluated_depsgraph_get(); pts = []
    for o in meshes:
        ev = o.evaluated_get(dep); me = ev.to_mesh()
        a = np.empty(len(me.vertices) * 3); me.vertices.foreach_get('co', a)
        a = a.reshape(-1, 3); m = np.array(ev.matrix_world)
        pts.append(a @ m[:3, :3].T + m[:3, 3]); ev.to_mesh_clear()
    p = np.concatenate(pts); return p.min(0), p.max(0)


def shoot(path, view, centre, extent):
    d = {'front': Vector((0, -1, 0)), 'side': Vector((1, 0, 0)), 'back': Vector((0, 1, 0)),
         'three': Vector((.6, -1, .25)).normalized(), 'top': Vector((0, -.2, 1)).normalized()}[view]
    cam.location = Vector(centre) + d * 10
    cam.rotation_euler = (-d).to_track_quat('-Z', 'Y').to_euler()
    cam.data.ortho_scale = extent; cam.data.clip_end = 30
    scene.render.filepath = str(path); bpy.ops.render.render(write_still=True)


rows = []
clips = [c for c in args.clips.split(',') if c in bpy.data.actions] if rig and bpy.data.actions else []
views = args.views.split(',')
close = [j for j in args.close.split(',') if j]
if not clips:
    lo, hi = bounds(); c = (lo + hi) / 2; e = float((hi - lo).max()) * 1.08
    rows.append([tmp / f'rest-{v}.png' for v in views])
    for v, p in zip(views, rows[-1]): shoot(p, v, c, e)
else:
    for clip in clips:
        action = bpy.data.actions[clip]
        rig.animation_data.action = action
        if action.slots: rig.animation_data.action_slot = action.slots[0]
        a, b = action.frame_range
        frames = [round(a + (b - a) * i / max(1, args.frames - 1)) for i in range(args.frames)]
        scene.frame_set(int(a)); lo, hi = bounds()
        for f in frames:
            scene.frame_set(f); l2, h2 = bounds(); lo = np.minimum(lo, l2); hi = np.maximum(hi, h2)
        c = (lo + hi) / 2; e = float((hi - lo).max()) * 1.05
        row = []
        for f in frames:
            scene.frame_set(f)
            for v in views:
                p = tmp / f'{clip}-{f:03d}-{v}.png'; shoot(p, v, c, e); row.append(p)
            for j in close:
                pb = rig.pose.bones[j]; centre = rig.matrix_world @ pb.head
                p = tmp / f'{clip}-{f:03d}-{j}.png'; shoot(p, args.close_view, centre, .42); row.append(p)
        rows.append(row)

width = max(len(r) for r in rows); S = args.size
sheet = np.ones((S * len(rows), S * width, 4), dtype=np.float32)
for ri, row in enumerate(rows):
    for ci, p in enumerate(row):
        im = bpy.data.images.load(str(p)); px = np.array(im.pixels[:], dtype=np.float32).reshape(S, S, 4)
        y0 = (len(rows) - 1 - ri) * S; sheet[y0:y0 + S, ci * S:(ci + 1) * S] = px; bpy.data.images.remove(im)
out = bpy.data.images.new('sheet', S * width, S * len(rows)); out.pixels.foreach_set(sheet.ravel())
out.filepath_raw = str(args.out); out.file_format = 'PNG'; out.save()
print('SHEET', args.out, [r[0].name.split('-')[0] for r in rows], flush=True)
