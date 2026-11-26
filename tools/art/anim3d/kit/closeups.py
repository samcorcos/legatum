"""Close-up review renders of a unit GLB: head and helmet, shield rim, left forearm, knees.

    BLENDER_GPU=1 blender-slot --factory-startup --python-exit-code 1 \
      --python tools/art/anim3d/kit/closeups.py -- <unit.glb> <out dir> [--size 2048]
      [--hdr tools/art/anim3d/viewer/studio_small_09_1k.hdr]

Eevee with a neutral studio HDRI and a soft warm key, team colour applied the way
the runtime does (base colour x mix(1, team, COLOR_0.r)). Pose: idle frame 0;
the knees also at walk mid-stride and attack lunge. Writes one PNG per crop.
"""
import argparse, math, sys
from pathlib import Path
import bpy
from mathutils import Vector

argv = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
ap = argparse.ArgumentParser()
ap.add_argument('glb', type=Path); ap.add_argument('out', type=Path)
ap.add_argument('--size', type=int, default=2048)
ap.add_argument('--hdr', type=Path, default=Path(__file__).resolve().parents[1] / 'viewer/studio_small_09_1k.hdr')
ap.add_argument('--team', default='a52627')
args = ap.parse_args(argv); args.out.mkdir(parents=True, exist_ok=True)

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(args.glb))
scene = bpy.context.scene
rig = next(o for o in scene.objects if o.type == 'ARMATURE')
meshes = {o.name: o for o in scene.objects if o.type == 'MESH'}
team = tuple(int(args.team[i:i + 2], 16) / 255 for i in (0, 2, 4))
team = tuple((c / 12.92) if c <= .04045 else ((c + .055) / 1.055) ** 2.4 for c in team)

# Runtime team mask: COLOR_0.r selects the team tint; it is not a colour multiplier.
for m in bpy.data.materials:
    if not m.use_nodes: continue
    nt = m.node_tree; bsdf = next((n for n in nt.nodes if n.type == 'BSDF_PRINCIPLED'), None)
    tex = next((n for n in nt.nodes if n.type == 'TEX_IMAGE' and n.image and n.image.colorspace_settings.name.lower().startswith('srgb')), None)
    if not bsdf or not tex: continue
    vc = nt.nodes.new('ShaderNodeVertexColor'); vc.layer_name = 'Color'
    sep = nt.nodes.new('ShaderNodeSeparateColor'); nt.links.new(vc.outputs['Color'], sep.inputs[0])
    mul = nt.nodes.new('ShaderNodeMixRGB'); mul.blend_type = 'MULTIPLY'; mul.inputs[0].default_value = 1
    nt.links.new(tex.outputs['Color'], mul.inputs[1]); mul.inputs[2].default_value = (*team, 1)
    mix = nt.nodes.new('ShaderNodeMixRGB'); nt.links.new(sep.outputs[0], mix.inputs[0])
    nt.links.new(tex.outputs['Color'], mix.inputs[1]); nt.links.new(mul.outputs[0], mix.inputs[2])
    nt.links.new(mix.outputs[0], bsdf.inputs['Base Color'])

world = bpy.data.worlds.new('studio'); scene.world = world; world.use_nodes = True
env = world.node_tree.nodes.new('ShaderNodeTexEnvironment'); env.image = bpy.data.images.load(str(args.hdr))
bg = world.node_tree.nodes['Background']; bg.inputs['Strength'].default_value = .8
world.node_tree.links.new(env.outputs['Color'], bg.inputs['Color'])
sun = bpy.data.objects.new('key', bpy.data.lights.new('key', 'SUN')); scene.collection.objects.link(sun)
sun.data.energy = 3.0; sun.data.color = (1, .93, .84); sun.data.angle = math.radians(12)
sun.rotation_euler = (math.radians(50), 0, math.radians(-35))
rim = bpy.data.objects.new('rim', bpy.data.lights.new('rim', 'SUN')); scene.collection.objects.link(rim)
rim.data.energy = 1.2; rim.data.color = (.75, .85, 1); rim.rotation_euler = (math.radians(60), 0, math.radians(160))
scene.render.engine = 'BLENDER_EEVEE'
scene.render.resolution_x = scene.render.resolution_y = args.size
scene.render.film_transparent = False
scene.view_settings.view_transform = 'AgX'
scene.render.image_settings.file_format = 'PNG'
cam = bpy.data.objects.new('cam', bpy.data.cameras.new('cam')); scene.collection.objects.link(cam); scene.camera = cam
cam.data.clip_start = .01


def pose(clip, frac=0.):
    act = bpy.data.actions[clip]; rig.animation_data.action = act
    if act.slots: rig.animation_data.action_slot = act.slots[0]
    a, b = act.frame_range; scene.frame_set(int(round(a + (b - a) * frac))); bpy.context.view_layer.update()


def joint(name): return rig.matrix_world @ rig.pose.bones[name].head


def show(lod):
    for n, o in meshes.items(): o.hide_render = not n.startswith(lod)


def shoot(name, target, direction, size=None, distance=None, lens=50):
    d = Vector(direction).normalized()
    if distance:
        cam.data.type = 'PERSP'; cam.data.lens = lens; cam.location = target + d * distance
    else:
        cam.data.type = 'ORTHO'; cam.data.ortho_scale = size; cam.location = target + d * 4
    cam.rotation_euler = (-d).to_track_quat('-Z', 'Y').to_euler()
    scene.render.filepath = str(args.out / f'{name}.png'); bpy.ops.render.render(write_still=True)
    print('CLOSEUP', name, flush=True)


# Blender axes after import: the unit faces -Y, its left is +X, up is +Z.
FRONT, LEFT, RIGHT, BACK, UP = Vector((0, -1, 0)), Vector((1, 0, 0)), Vector((-1, 0, 0)), Vector((0, 1, 0)), Vector((0, 0, 1))
pose('idle'); show('spearman_LOD0')
head = joint('head') + Vector((0, 0, .09))
shoot('head-front', head, FRONT + UP * .05, .42)
shoot('head-three', head, FRONT + RIGHT * .9 + UP * .1, .42)
shoot('head-side', head, RIGHT, .42)
shoot('head-top', head, UP + FRONT * .15, .42)
shoot('crest-side', head + Vector((0, .02, .12)), RIGHT + FRONT * .15, .62)
shoot('crest-three', head + Vector((0, .02, .12)), FRONT + RIGHT * .9 + UP * .15, .62)
show('spearman_LOD1')
sh = joint('shield_arm')
# The aspis faces outward (+X) from the left forearm in the clips' guard.
shoot('shield-lod1-1.2m', sh, LEFT + FRONT * .6 + UP * .2, distance=1.2)
shoot('shield-lod1-close', sh + Vector((0, -.25, .2)), LEFT + FRONT * .6 + UP * .2, .28)
show('spearman_LOD0')
fa = (joint('forearm_L') + joint('hand_L')) / 2
shoot('forearm-left-front', fa, FRONT + RIGHT * .5 + UP * .2, .32)
shoot('forearm-left-back', fa, BACK + RIGHT * .6 + UP * .2, .32)
knees = (joint('leg_L') + joint('leg_R')) / 2
shoot('knees-front', knees, FRONT + UP * .1, .5)
shoot('knees-side', knees, RIGHT + FRONT * .3, .5)
pose('walk', .25); knees = (joint('leg_L') + joint('leg_R')) / 2
shoot('knees-walk', knees, FRONT + RIGHT * .6 + UP * .1, .55)
pose('attack', .55); knees = (joint('leg_L') + joint('leg_R')) / 2; fa = (joint('forearm_L') + joint('hand_L')) / 2
shoot('knees-attack', knees, FRONT + RIGHT * .6 + UP * .1, .6)
shoot('forearm-left-attack', fa, FRONT + RIGHT * .6 + UP * .2, .36)
print('CLOSEUPS_DONE', args.out, flush=True)
