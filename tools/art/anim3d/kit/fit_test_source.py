"""Build a CC0 stand-in for a third-party hoplite: MPFB body in an action pose.

Mimics Sketchfab model #1 ("Hoplite Warrior", a static sculpt in an action pose
with spear and shield): one glTF with separate textured meshes and NO armature.
Everything is CC0 (MPFB/MakeHuman body) or generated here (props, textures).

    source tools/art/anim3d/env.sh
    blender-slot --factory-startup --python-exit-code 1 \
      --python tools/art/anim3d/kit/fit_test_source.py -- --out <dir> [--keep-rig]

Writes <dir>/test-hoplite-action.glb (mesh only) and, with --keep-rig,
<dir>/test-hoplite-action-rigged.glb (skinned, posed MPFB game_engine rig).
"""
import argparse, math, sys
from pathlib import Path
import bpy, bmesh
import numpy as np
from mathutils import Vector, Matrix

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from kit.human import create_body, freeze_macro_mix, add_rig

argv = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
ap = argparse.ArgumentParser()
ap.add_argument('--out', type=Path, required=True)
ap.add_argument('--keep-rig', action='store_true')
args = ap.parse_args(argv)
args.out.mkdir(parents=True, exist_ok=True)
rng = np.random.default_rng(7)

bpy.ops.object.select_all(action='SELECT'); bpy.ops.object.delete(use_global=False)


def image(name, size, painter):
    """Generated texture: painter(u, v) -> rgb arrays in [0, 1]."""
    v, u = np.mgrid[0:size, 0:size] / (size - 1)
    rgb = np.clip(painter(u, v), 0, 1)
    px = np.concatenate([rgb, np.ones((size, size, 1))], axis=2).astype(np.float32)
    im = bpy.data.images.new(name, size, size); im.pixels.foreach_set(px.ravel()); im.pack()
    return im


def noise(size, scale):
    base = rng.random((scale, scale))
    reps = int(math.ceil(size / scale))
    return np.kron(base, np.ones((reps, reps)))[:size, :size]


def material(name, im, rough, metal):
    m = bpy.data.materials.new(name); m.use_nodes = True
    bsdf = m.node_tree.nodes['Principled BSDF']
    tex = m.node_tree.nodes.new('ShaderNodeTexImage'); tex.image = im
    m.node_tree.links.new(tex.outputs['Color'], bsdf.inputs['Base Color'])
    bsdf.inputs['Roughness'].default_value = rough; bsdf.inputs['Metallic'].default_value = metal
    return m


S = 256
grain = noise(S, 32)[..., None]
M = {
    'skin': material('Skin', image('skin', S, lambda u, v: np.stack([.62 + 0*u, .38 + 0*u, .26 + 0*u], -1) * (.9 + .1 * grain)), .55, 0),
    'linen': material('Linen chiton', image('linen', S, lambda u, v: np.stack([.82 + 0*u, .76 + 0*u, .62 + 0*u], -1) * (.92 + .08 * (np.sin(u * 160) > 0)[..., None])), .9, 0),
    'bronze': material('Bronze', image('bronze', S, lambda u, v: np.stack([.72 + 0*u, .46 + 0*u, .18 + 0*u], -1) * (.8 + .2 * grain)), .35, 1),
    'crest': material('Red horsehair crest', image('crest', S, lambda u, v: np.stack([.62 + 0*u, .05 + 0*u, .04 + 0*u], -1) * (.75 + .25 * (np.sin(u * 300) > 0)[..., None])), .8, 0),
    'wood': material('Ash shaft', image('wood', S, lambda u, v: np.stack([.45 + 0*u, .30 + 0*u, .16 + 0*u], -1) * (.85 + .15 * np.sin(v * 90)[..., None])), .7, 0),
    'cloak': material('Crimson cloak', image('cloak', S, lambda u, v: np.where((v < .08)[..., None], np.array([.85, .7, .3]), np.array([.55, .06, .05])) * (.9 + .1 * grain)), .9, 0),
}


def shield_paint(u, v):
    x, y = u - .5, v - .5; r = np.hypot(x, y) * 2; a = np.arctan2(y, x)
    field = np.where((np.cos(a * 12) > .55) & (r > .2) & (r < .8), 1, 0)[..., None]
    col = np.where(field, np.array([.95, .75, .3]), np.array([.55, .32, .1]))
    return np.where((r > .88)[..., None], np.array([.55, .05, .04]), col)


M['shield'] = material('Painted aspis', image('aspis', 512, shield_paint), .4, .8)

# --- Body (CC0 MPFB), heroic build, game_engine rig --------------------------------
body = create_body(dict(gender=1., age=.42, muscle=1., weight=.72, height=.5, proportions=.65), 'Body')
freeze_macro_mix(body)
rig = add_rig(body, 'Rig')
bpy.context.view_layer.update()
pb = rig.pose.bones


def aim(name, direction, roll=0):
    """Rotate a pose bone (about its head) so its world head->tail axis points along direction."""
    bpy.context.view_layer.update()
    b = pb[name]; w = rig.matrix_world @ b.matrix
    head = w.translation.copy(); axis = (w.to_3x3() @ Vector((0, 1, 0))).normalized()
    q = axis.rotation_difference(Vector(direction).normalized())
    if roll: q = Matrix.Rotation(roll, 3, Vector(direction).normalized()).to_quaternion() @ q
    world = Matrix.Translation(head) @ q.to_matrix().to_4x4() @ Matrix.Translation(-head) @ w
    b.matrix = rig.matrix_world.inverted() @ world


def turn(name, angle, axis):
    bpy.context.view_layer.update()
    b = pb[name]; w = rig.matrix_world @ b.matrix; head = w.translation.copy()
    world = Matrix.Translation(head) @ Matrix.Rotation(math.radians(angle), 4, axis) @ Matrix.Translation(-head) @ w
    b.matrix = rig.matrix_world.inverted() @ world


# Blender: Z up, the figure faces -Y, its left is +X.  An overhand-thrust lunge.
turn('pelvis', 18, 'Z'); pb['pelvis'].location.z -= .0
turn('spine_01', 8, 'X'); turn('spine_02', 6, 'X'); turn('spine_03', -10, 'Z')
aim('thigh_l', (.12, -.55, -.83)); aim('calf_l', (.05, .12, -1)); aim('foot_l', (.05, -1, -.35))
aim('thigh_r', (-.14, .42, -.9)); aim('calf_r', (-.06, .62, -.78)); aim('foot_r', (-.15, -.75, -.65))
aim('upperarm_r', (-.62, .25, .74)); aim('lowerarm_r', (-.15, -.55, .42)); aim('hand_r', (0, -.8, .3))
aim('upperarm_l', (.45, -.62, -.42)); aim('lowerarm_l', (-.55, -.75, .2)); aim('hand_l', (-.6, -.7, .1))
turn('head', -8, 'Z')
bpy.context.view_layer.update()


def world_head(name): return rig.matrix_world @ pb[name].head
def world_tail(name): return rig.matrix_world @ pb[name].tail


# --- Props, modelled around the posed hands --------------------------------------------
def add(ob, mat, name):
    ob.name = name; ob.data.materials.clear(); ob.data.materials.append(M[mat]); return ob


grip = (world_head('hand_r') + world_tail('hand_r')) / 2
spear_dir = Vector((.08, -1, .12)).normalized()
bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=.014, depth=2.3, calc_uvs=True)
shaft = add(bpy.context.object, 'wood', 'Spear shaft')
shaft.rotation_mode = 'QUATERNION'; shaft.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(spear_dir)
shaft.location = grip + spear_dir * .35
bpy.ops.mesh.primitive_cone_add(vertices=12, radius1=.04, radius2=0, depth=.28, calc_uvs=True)
blade = add(bpy.context.object, 'bronze', 'Spear blade')
blade.rotation_mode = 'QUATERNION'; blade.rotation_quaternion = shaft.rotation_quaternion
blade.location = grip + spear_dir * (.35 + 1.15 + .13)
blade.scale = (1, .35, 1)
bpy.ops.mesh.primitive_cone_add(vertices=12, radius1=.02, radius2=0, depth=.14, calc_uvs=True)
butt = add(bpy.context.object, 'bronze', 'Spear butt spike')
butt.rotation_mode = 'QUATERNION'; butt.rotation_quaternion = (-spear_dir).to_track_quat('Z', 'Y')
butt.location = grip + spear_dir * (.35 - 1.15 - .06)

# Aspis: a convex dish with a raised rim, strapped on the left forearm, face forward.
fore = (world_head('lowerarm_l') + world_tail('lowerarm_l')) / 2
normal = Vector((-.15, -1, .05)).normalized()
bm = bmesh.new()
rings, seg, R = 6, 48, .45
verts = [bm.verts.new((0, 0, .09))]
for j in range(1, rings + 1):
    r = R * j / rings
    for i in range(seg):
        a = 2 * math.pi * i / seg
        verts.append(bm.verts.new((r * math.cos(a), r * math.sin(a), .09 * (1 - (r / R) ** 2))))
for i in range(seg): bm.faces.new((verts[0], verts[1 + i], verts[1 + (i + 1) % seg]))
for j in range(rings - 1):
    for i in range(seg):
        a = 1 + j * seg + i; b = 1 + j * seg + (i + 1) % seg
        bm.faces.new((verts[a], verts[b], verts[b + seg], verts[a + seg]))
uv = bm.loops.layers.uv.new()
for f in bm.faces:
    for l in f.loops: l[uv].uv = (.5 + l.vert.co.x / (2 * R), .5 + l.vert.co.y / (2 * R))
me = bpy.data.meshes.new('aspis'); bm.to_mesh(me); bm.free()
shield = bpy.data.objects.new('Shield', me); bpy.context.collection.objects.link(shield)
add(shield, 'shield', 'Shield')
mod = shield.modifiers.new('thick', 'SOLIDIFY'); mod.thickness = .012
bpy.context.view_layer.objects.active = shield; bpy.ops.object.modifier_apply(modifier=mod.name)
shield.rotation_mode = 'QUATERNION'; shield.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(normal)
shield.location = fore + normal * .07

# Helmet dome + crest, on the head.
head_c = world_head('head') + (world_tail('head') - world_head('head')) * .55
bpy.ops.mesh.primitive_uv_sphere_add(segments=32, ring_count=16, radius=.135, calc_uvs=True)
helm = add(bpy.context.object, 'bronze', 'Helmet')
bm = bmesh.new(); bm.from_mesh(helm.data)
bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < -.06 and v.co.y < -.02], context='VERTS')
bm.to_mesh(helm.data); bm.free()
helm.scale = (.95, 1.08, 1.05); helm.location = head_c + Vector((0, .01, .015))
bm = bmesh.new()
for k in range(25):
    t = math.pi * k / 24
    for side in (-1, 1):
        for h in (0, 1):
            bm.verts.new((side * .022, -.15 * math.cos(t) * (1 + .5 * h), .12 + (.04 + .13 * h) * math.sin(t)))
bm.verts.ensure_lookup_table(); vs = bm.verts
for k in range(24):
    i, j = k * 4, (k + 1) * 4
    # Per k: 0 = (-x, low), 1 = (-x, high), 2 = (+x, low), 3 = (+x, high).
    bm.faces.new((vs[i], vs[i + 2], vs[j + 2], vs[j]))          # bottom
    bm.faces.new((vs[i + 1], vs[j + 1], vs[j + 3], vs[i + 3]))  # top
    bm.faces.new((vs[i], vs[j], vs[j + 1], vs[i + 1]))          # side -x
    bm.faces.new((vs[i + 2], vs[i + 3], vs[j + 3], vs[j + 2]))  # side +x
uv = bm.loops.layers.uv.new()
for f in bm.faces:
    for l in f.loops: l[uv].uv = (.5 + l.vert.co.y, l.vert.co.z * 3)
me = bpy.data.meshes.new('crest'); bm.to_mesh(me); bm.free()
crest = bpy.data.objects.new('Crest', me); bpy.context.collection.objects.link(crest); add(crest, 'crest', 'Crest')
crest.location = helm.location

# Cloak: a curved sheet hanging from both shoulders down the back to the knees.
sl, sr = world_head('upperarm_l'), world_head('upperarm_r')
bm = bmesh.new(); grid = []
for j in range(9):
    t = j / 8; row = []
    for i in range(11):
        s = i / 10 - .5
        x = (sl.x + sr.x) / 2 + s * (.48 + .25 * t)
        y = (sl.y + sr.y) / 2 + .15 + .04 * t - .035 * math.cos(s * math.pi) + .025 * math.sin(i * 1.9) * t
        z = (sl.z + sr.z) / 2 + .02 - .95 * t
        row.append(bm.verts.new((x, y, z)))
    grid.append(row)
for j in range(8):
    for i in range(10): bm.faces.new((grid[j][i], grid[j][i + 1], grid[j + 1][i + 1], grid[j + 1][i]))
uv = bm.loops.layers.uv.new()
for f in bm.faces:
    for l in f.loops:
        idx = [(j, i) for j in range(9) for i in range(11) if grid[j][i] == l.vert][0]
        l[uv].uv = (idx[1] / 10, 1 - idx[0] / 8)
me = bpy.data.meshes.new('cloak'); bm.to_mesh(me); bm.free()
cloak = bpy.data.objects.new('Cloak', me); bpy.context.collection.objects.link(cloak); add(cloak, 'cloak', 'Cloak')
mod = cloak.modifiers.new('thick', 'SOLIDIFY'); mod.thickness = .008
bpy.context.view_layer.objects.active = cloak; bpy.ops.object.modifier_apply(modifier=mod.name)

# --- Body materials: skin, with a linen chiton band from the hips to the chest ---------
body.data.materials.clear(); body.data.materials.append(M['skin']); body.data.materials.append(M['linen'])
bpy.context.view_layer.update()
dep = bpy.context.evaluated_depsgraph_get()
posed = bpy.data.meshes.new_from_object(body.evaluated_get(dep), preserve_all_data_layers=True, depsgraph=dep)
if not args.keep_rig:
    baked = bpy.data.objects.new('Hoplite body', posed); bpy.context.collection.objects.link(baked)
    baked.matrix_world = body.matrix_world.copy()
    zmin = min((baked.matrix_world @ v.co).z for v in posed.vertices)
    for p in posed.polygons:
        c = baked.matrix_world @ p.center
        # Linen chiton over the torso and hips; arms, legs and head stay skin.
        p.material_index = int(.74 < (c.z - zmin) < 1.36 and abs(c.x) < .24)
    for p in posed.polygons: p.use_smooth = True
    bpy.data.objects.remove(body, do_unlink=True); bpy.data.objects.remove(rig, do_unlink=True)
    meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']
else:
    meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']
# Ground: lowest vertex of anything at z = 0.
low = min((o.matrix_world @ Vector(c)).z for o in meshes for c in [v.co for v in o.data.vertices])
for o in bpy.context.scene.objects:
    if o.parent is None: o.location.z -= low
name = 'test-hoplite-action-rigged.glb' if args.keep_rig else 'test-hoplite-action.glb'
bpy.ops.object.select_all(action='SELECT')
bpy.ops.export_scene.gltf(filepath=str(args.out / name), export_format='GLB', use_selection=True,
                          export_animations=False, export_apply=True)
print('TEST_SOURCE', args.out / name, sum(len(o.data.polygons) for o in meshes), 'faces', flush=True)
