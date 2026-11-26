"""CC0 placeholder parts for testing --replace-part: a Corinthian-style helmet and one greave.

    blender-slot --factory-startup --python-exit-code 1 --python tools/art/anim3d/kit/placeholder_parts.py -- <out dir>

Writes helmet.glb and greave.glb (glTF convention: Y up, facing +Z; the greave is a
left greave, the fit mirrors it for the right). Generated here, so CC0.
"""
import math, sys
from pathlib import Path
import bpy, bmesh
import numpy as np

out = Path(sys.argv[sys.argv.index('--') + 1]); out.mkdir(parents=True, exist_ok=True)


def bronze():
    m = bpy.data.materials.new('Placeholder bronze'); m.use_nodes = True
    b = m.node_tree.nodes['Principled BSDF']; S = 256
    rng = np.random.default_rng(3); ham = np.kron(rng.random((32, 32)), np.ones((8, 8)))
    px = np.concatenate([np.stack([.62 + 0 * ham, .43 + 0 * ham, .2 + 0 * ham], -1) * (.85 + .2 * ham[..., None]), np.ones((S, S, 1))], 2)
    im = bpy.data.images.new('bronze', S, S); im.pixels.foreach_set(px.astype(np.float32).ravel()); im.pack()
    t = m.node_tree.nodes.new('ShaderNodeTexImage'); t.image = im
    m.node_tree.links.new(t.outputs['Color'], b.inputs['Base Color'])
    b.inputs['Metallic'].default_value = 1; b.inputs['Roughness'].default_value = .38
    return m


def grid_mesh(name, point, nu, nv, keep=lambda u, v: True, thickness=.004):
    """Surface from point(u, v) on a (nu+1) x (nv+1) grid, faces filtered by keep(u, v)."""
    bm = bmesh.new(); uv = bm.loops.layers.uv.new()
    V = [[bm.verts.new(point(i / nu, j / nv)) for j in range(nv + 1)] for i in range(nu + 1)]
    for i in range(nu):
        for j in range(nv):
            if not keep((i + .5) / nu, (j + .5) / nv): continue
            f = bm.faces.new((V[i][j], V[i + 1][j], V[i + 1][j + 1], V[i][j + 1]))
            for l, (a, b) in zip(f.loops, ((i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1))): l[uv].uv = (a / nu, b / nv)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-6)
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context='VERTS')
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    o = bpy.data.objects.new(name, me); bpy.context.scene.collection.objects.link(o)
    me.materials.append(bronze())
    for p in me.polygons: p.use_smooth = True
    bpy.context.view_layer.objects.active = o
    m = o.modifiers.new('s', 'SOLIDIFY'); m.thickness = thickness; m.offset = -1; bpy.ops.object.modifier_apply(modifier=m.name)
    return o


def export(o, name):
    bpy.ops.object.select_all(action='DESELECT'); o.select_set(True)
    bpy.ops.export_scene.gltf(filepath=str(out / name), export_format='GLB', use_selection=True)


bpy.ops.object.select_all(action='SELECT'); bpy.ops.object.delete()
# Helmet (Blender: Z up, faces -Y). az 0 = front; el from the crown (90) to below the jaw.
def helmet_pt(u, v):
    az = math.radians(-180 + 360 * u); el = math.radians(90 - 160 * v)
    flare = 1 + .18 * max(0, -math.sin(el)) ** 2 * (.5 + .5 * abs(math.sin(az / 2)))   # neck guard and cheek flare
    return (.115 * math.sin(az) * math.cos(el) * flare, -.13 * math.cos(az) * math.cos(el) * flare, .14 * math.sin(el))
def helmet_keep(u, v):
    az = abs(-180 + 360 * u); el = 90 - 160 * v
    eye = -4 <= el <= 10 and 7 <= az <= 58
    mouth = el < -4 and az < 11
    return not (eye or mouth) and not (el < -55 and az > 120)
export(grid_mesh('Corinthian helmet', helmet_pt, 64, 40, helmet_keep), 'helmet.glb')
bpy.ops.object.select_all(action='SELECT'); bpy.ops.object.delete()
# Greave: wraps the front 250 degrees of the shin, knee guard rounded at the top.
def greave_pt(u, v):
    th = math.radians(-125 + 250 * u); top = .36 - .05 * (1 - math.cos(math.radians(-125 + 250 * u))) / 2
    h = top * v; r = .056 + .016 * math.sin(math.pi * min(1, h / .3)) + .01 * max(0, (h - .3) / .06)
    return (r * math.sin(th), -r * math.cos(th), h)
export(grid_mesh('Greave', greave_pt, 40, 24, thickness=.003), 'greave.glb')
print('PLACEHOLDERS', out)
