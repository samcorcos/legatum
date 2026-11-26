"""Simple fitted shells that replace garment and helmet geometry that fits badly.

bpy + numpy, glTF bind space (Y up, +Z forward). Each builder returns a Blender
mesh object with UVs and its own material, ready to be baked like any source.
"""
import math
import bpy, bmesh
import numpy as np


def _image(name, size, painter):
    v, u = np.mgrid[0:size, 0:size] / (size - 1)
    rgb = np.clip(painter(u, v), 0, 1)
    px = np.concatenate([rgb, np.ones((size, size, 1))], 2).astype(np.float32)
    im = bpy.data.images.new(name, size, size); im.pixels.foreach_set(px.ravel()); im.pack()
    return im


def material(name, rgb, rough, metal, painter=None):
    m = bpy.data.materials.new(name); m.use_nodes = True
    b = m.node_tree.nodes['Principled BSDF']
    b.inputs['Roughness'].default_value = rough; b.inputs['Metallic'].default_value = metal
    if painter is None:
        b.inputs['Base Color'].default_value = (*rgb, 1)
    else:
        t = m.node_tree.nodes.new('ShaderNodeTexImage'); t.image = _image(name + ' tex', 256, painter)
        m.node_tree.links.new(t.outputs['Color'], b.inputs['Base Color'])
    return m


def _mesh(name, verts, faces, uvs, mat, thickness=0.):
    me = bpy.data.meshes.new(name); me.from_pydata(verts, [], faces); me.update()
    uv = me.uv_layers.new(name='UVMap')
    for p in me.polygons:
        for li in p.loop_indices: uv.data[li].uv = uvs[me.loops[li].vertex_index]
    me.materials.append(mat)
    for p in me.polygons: p.use_smooth = True
    o = bpy.data.objects.new(name, me); bpy.context.scene.collection.objects.link(o)
    if thickness:
        bpy.ops.object.select_all(action='DESELECT'); o.select_set(True); bpy.context.view_layer.objects.active = o
        m = o.modifiers.new('solid', 'SOLIDIFY'); m.thickness = thickness; m.offset = 1
        bpy.ops.object.modifier_apply(modifier=m.name)
    return o


def sleeve(skin, a, b, length_frac, mat, name, gap=.005, seg=28, ring=.01):
    """A tube following the skin around the bone a->b, from the shoulder to length_frac of it."""
    u = (b - a) / np.linalg.norm(b - a); L = np.linalg.norm(b - a)
    ref = np.cross(u, [0, 0, 1.]); ref = ref if np.linalg.norm(ref) > .1 else np.cross(u, [1., 0, 0]); ref /= np.linalg.norm(ref)
    ref2 = np.cross(u, ref)
    q = skin - a; t = q @ u; rv = q - np.outer(t, u); r = np.linalg.norm(rv, axis=1)
    ang = np.arctan2(rv @ ref2, rv @ ref)
    ts = np.arange(-.05, length_frac * L + 1e-6, ring); bins = np.linspace(-np.pi, np.pi, seg + 1)  # from over the shoulder cap
    R = np.zeros((len(ts), seg))
    for i, tt in enumerate(ts):
        sel = (np.abs(t - tt) < ring) & (r < .14)
        for j in range(seg):
            s2 = sel & (ang >= bins[j]) & (ang < bins[j + 1])
            R[i, j] = r[s2].max() if s2.any() else np.nan
    med = np.nanmedian(R[len(ts) // 2:]) if np.isfinite(R).any() else .05
    R = np.where(np.isfinite(R), R, med); R = np.minimum(R, 1.15 * med)
    for _ in range(2): R = (np.roll(R, 1, 1) + 2 * R + np.roll(R, -1, 1)) / 4
    R += gap + np.linspace(0, .003, len(ts))[:, None]   # close linen, barely flared hem
    verts, uvs, faces = [], [], []
    for i, tt in enumerate(ts):
        for j in range(seg + 1):
            th = bins[j] if j < seg else bins[0] + 2 * np.pi; rr = R[i, j % seg]
            verts.append(tuple(a + u * tt + rr * (np.cos(th) * ref + np.sin(th) * ref2))); uvs.append((j / seg, i / (len(ts) - 1)))
    w = seg + 1
    for i in range(len(ts) - 1):
        for j in range(seg): faces.append((i * w + j, i * w + j + 1, (i + 1) * w + j + 1, (i + 1) * w + j))
    return _mesh(name, verts, faces, uvs, mat, thickness=.003), (u, ts.max(), float(R.max()))


def corinthian(helmet_pts, face_pts, bronze, dark, name='Corinthian face plate', seg=64, rings=34):
    """Close an open helmet: cheek guards to the jaw, a raised nasal, a dark T opening.

    An ellipsoidal plate wrapping the face from over the brow (where it tucks
    under the dome) to below the jaw and round to the helmet's sides, with two
    eye slots either side of the nasal and the vertical gap between the cheek
    guards cut out; a dark inner surface behind it; and a separate raised,
    bevelled nasal strip so the T's upright catches light in geometry, normals
    and ORM rather than only in the albedo.
    """
    nose = face_pts[np.argmax(face_pts[:, 2])]
    cy = nose[1] + .025; cz = float(np.median(helmet_pts[:, 2]))
    band = helmet_pts[np.abs(helmet_pts[:, 1] - cy) < .03]
    rx = (float(np.abs(band[:, 0]).max()) if len(band) else .11) + .004
    rz = float(nose[2] - cz) + .016; ry = .15
    def point(az, el, k=1.):
        return (k * rx * math.sin(az) * math.cos(el), cy + k * ry * math.sin(el), cz + k * rz * math.cos(az) * math.cos(el))
    AZ = np.radians(np.linspace(-120, 120, seg + 1)); EL = np.radians(np.linspace(-62, 55, rings + 1))
    def cut(az, el):
        a, e = abs(math.degrees(az)), math.degrees(el)
        eye = 0 <= e <= 13 and 7.5 <= a <= 50     # two eye slots, split by the nasal
        mouth = e < -15 and a < 10                 # the gap between the cheek guards
        return eye or mouth
    out = []
    for k, mat, holes, nm in ((1., bronze, True, name), (.975, dark, False, name + ' interior')):
        verts, uvs, faces = [], [], []
        for i, el in enumerate(EL):
            for j, az in enumerate(AZ):
                verts.append(point(az, el, k)); uvs.append((j / seg, i / rings))
        w = seg + 1
        for i in range(rings):
            for j in range(seg):
                if holes and cut((AZ[j] + AZ[j + 1]) / 2, (EL[i] + EL[i + 1]) / 2): continue
                faces.append((i * w + j, (i + 1) * w + j, (i + 1) * w + j + 1, i * w + j + 1))
        out.append(_mesh(nm, verts, faces, uvs, mat, thickness=.003 if holes else 0.))
    # Raised nasal: a strip 4 mm proud of the plate, bevelled along its edges.
    verts, uvs, faces = [], [], []
    els = np.radians(np.linspace(-15, 30, 10)); azs = np.radians([-6.5, -3, 0, 3, 6.5])
    for i, el in enumerate(els):
        for j, az in enumerate(azs):
            lift = 1.035 - .012 * abs(math.degrees(az)) / 6.5      # rounded cross-section
            verts.append(point(az, el, lift)); uvs.append((j / 4, i / 9))
    for i in range(9):
        for j in range(4): faces.append((i * 5 + j, (i + 1) * 5 + j, (i + 1) * 5 + j + 1, i * 5 + j + 1))
    nas = _mesh(name + ' nasal', verts, faces, uvs, bronze, thickness=.004)
    bpy.ops.object.select_all(action='DESELECT'); nas.select_set(True); bpy.context.view_layer.objects.active = nas
    m = nas.modifiers.new('bevel', 'BEVEL'); m.width = .0015; m.segments = 2; m.limit_method = 'ANGLE'
    bpy.ops.object.modifier_apply(modifier=m.name)
    out.append(nas)
    return out, {'centre': [0, round(cy, 3), round(cz, 3)], 'radii_m': [round(rx, 3), ry, round(rz, 3)]}


def press_under(skin_objs, layer_pts, depth=.008, reach=.02):
    """Skin within reach of a clothing/armour vertex sinks to depth below it (hidden anyway)."""
    moved = 0
    for o in skin_objs:
        me = o.data; n = len(me.vertices)
        P = np.empty(n * 3); me.vertices.foreach_get('co', P); P = P.reshape(-1, 3)
        Nn = np.empty(n * 3); me.vertices.foreach_get('normal', Nn); Nn = Nn.reshape(-1, 3)
        for k in range(0, n, 512):
            d2 = ((P[k:k + 512, None] - layer_pts[None]) ** 2).sum(2); j = d2.argmin(1); d = np.sqrt(d2[np.arange(len(j)), j])
            off = ((layer_pts[j] - P[k:k + 512]) * Nn[k:k + 512]).sum(1)          # layer height above the skin
            bad = (d < reach) & (off > -.004) & (off < depth)
            P[k:k + 512][bad] -= Nn[k:k + 512][bad] * (depth - off[bad])[:, None]; moved += int(bad.sum())
        me.vertices.foreach_set('co', P.ravel()); me.update()
    return moved
