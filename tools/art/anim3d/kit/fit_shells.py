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


def press_under(skin_objs, layer_pts, depth=.008, reach=.02, below=.004):
    """Skin within reach of a clothing/armour vertex sinks to depth below it (hidden anyway)."""
    moved = 0
    for o in skin_objs:
        me = o.data; n = len(me.vertices)
        P = np.empty(n * 3); me.vertices.foreach_get('co', P); P = P.reshape(-1, 3)
        Nn = np.empty(n * 3); me.vertices.foreach_get('normal', Nn); Nn = Nn.reshape(-1, 3)
        for k in range(0, n, 512):
            d2 = ((P[k:k + 512, None] - layer_pts[None]) ** 2).sum(2); j = d2.argmin(1); d = np.sqrt(d2[np.arange(len(j)), j])
            off = ((layer_pts[j] - P[k:k + 512]) * Nn[k:k + 512]).sum(1)          # layer height above the skin
            bad = (d < reach) & (off > -below) & (off < depth)  # below: skin already poking out through the layer
            P[k:k + 512][bad] -= Nn[k:k + 512][bad] * (depth - off[bad])[:, None]; moved += int(bad.sum())
        me.vertices.foreach_set('co', P.ravel()); me.update()
    return moved


def crest_mass(helmet_pts, ceiling, name='Crest horsehair (procedural)', seg=44, ring=22, ridges=11,
               height=.17, width=.095, groove=.009, seed=5, tail=1.14, shift=0.):
    """A horsehair crest as its own part: a lofted mass along a front-to-back arc
    over the helmet crown (a mohawk sweep that trails down at the back), with a
    teardrop section that is fuller toward the top, real strand grooves running
    along the flow, and a streaked texture with darker roots.

    Returns the object and a short report. Coordinates: glTF bind space (Y up, +Z forward).
    """
    H = helmet_pts; top = H[:, 1].max()
    crown = H[H[:, 1] > top - .11]; zf, zb = crown[:, 2].max(), crown[:, 2].min()
    zc = (zf + zb) / 2 + shift; R = (zf - zb) / 2 * 1.15
    def base_y(z):
        near = H[(np.abs(H[:, 2] - z) < .015) & (np.abs(H[:, 0]) < .03)]
        return (near[:, 1].max() if len(near) else top - .04) - .012
    hgt = min(height, ceiling - top - .01)
    rng = np.random.default_rng(seed); phase = rng.random(ridges) * 2 * np.pi
    verts, uvs, faces = [], [], []
    ts = np.linspace(-.02, tail, seg + 1)               # 0 front edge, 1 back edge; past 1 trails down the back
    for i, t in enumerate(ts):
        a = math.pi * np.clip(t, 0, 1)
        z = zc + R * math.cos(a) - .025 * t              # swept back a little
        yb = base_y(np.clip(z, zb, zf))
        if t > 1: yb -= (t - 1) * .35; z -= (t - 1) * .08   # tail falls behind the neck guard
        if t < 0: yb -= -t * .15
        env = math.sin(math.pi * np.clip((t + .02) / (tail + .02), 0, 1)) ** .45
        # Fluffy top edge: tufts every few segments.
        h = (.02 + hgt * env * (1 + .12 * t)) * (1 + .06 * math.cos(t * 2 * math.pi * 7 + 1.3))
        w = width * (.55 + .45 * env)
        lean = -.35 * h * (t - .45)                      # the mass leans back over the crown
        for j in range(ring):
            phi = 2 * math.pi * j / ring
            fy = .5 * (1 - math.cos(phi))                # 0 at the root, 1 at the top edge
            fx = math.sin(phi) * (.45 + .55 * fy ** .7)  # fuller (fan) toward the top
            k = (j * ridges) // ring
            grv = groove * (.5 + .5 * math.cos(2 * math.pi * j * ridges / ring + phase[k % ridges] + 2.5 * t))
            x = fx * w - math.copysign(grv, math.sin(phi)) * (abs(math.sin(phi)) > .2)
            y = yb + fy * h + (grv * .6 if fy > .9 else 0)
            verts.append((x, y, z + lean * fy)); uvs.append((i / seg, j / ring))
    for i in range(seg):
        for j in range(ring):
            a0, a1 = i * ring + j, i * ring + (j + 1) % ring
            faces.append((a0, a1, a1 + ring, a0 + ring))
    faces += [tuple(range(ring - 1, -1, -1)), tuple(seg * ring + j for j in range(ring))]
    def painter(u, v):
        streak = np.random.default_rng(seed + 1).random(u.shape[0])[None, :].repeat(u.shape[0], 0)
        val = (.6 + .4 * streak) * (.55 + .45 * np.clip(np.sin(math.pi * v) * 1.2, 0, 1))
        return np.stack([.85 * val, .78 * val, .74 * val], -1)   # near-neutral: the team tint gives the red
    m = material(name + ' material', None, .82, 0., painter)
    o = _mesh(name, verts, faces, uvs, m)
    return o, {'triangles': 2 * seg * ring + 2 * (ring - 2), 'height_m': round(hgt + .02, 3), 'length_m': round(2 * R, 3)}


def greave_from_skin(skin_pts, knee, ankle, bronze, name='HIGH-greave-built', offset=.006, thick=0.,
                     nt=32, nh=18, seam=.10, ridges=(-.55, .55), knee_cap=False, rise=.105, dome=.024, repousse=True):
    """A greave grown from the fitted shin itself: an offset shell of the shin/calf
    skin from just above the ankle, closed except a narrow seam at the back. Its top
    edge rises at the front into a pointed knee guard over the kneecap (one piece,
    domed over the kneecap), with rolled top and bottom rims and raised embossed ridges."""
    u = (knee - ankle) / np.linalg.norm(knee - ankle); L = float(np.linalg.norm(knee - ankle))
    side = np.cross(u, [0, 0, 1.]); side /= np.linalg.norm(side); front = np.cross(side, u)
    q = skin_pts - ankle; h = q @ u; r = np.linalg.norm(q - np.outer(h, u), axis=1)
    th = np.arctan2(q @ side, q @ front)
    h0, h1 = .06, L - .03
    T = np.linspace(-np.pi + seam, np.pi - seam, nt)
    top = h1 + rise * np.maximum(0, np.cos(T)) ** 2.5            # rises to a point at the front
    S = np.linspace(0, 1, nh)                                    # rows as a fraction of each column
    Hm = h0 + np.outer(S, top - h0)                              # (nh, nt) heights
    sel = (h > h0 - .03) & (h < top.max() + .03) & (r < .11)
    R = np.full((nh, nt), np.nan)
    for i in range(nh):
        for j in range(nt):
            dt = np.abs((th[sel] - T[j] + np.pi) % (2 * np.pi) - np.pi)
            m = (np.abs(h[sel] - Hm[i, j]) < (top[j] - h0) / nh * 1.2) & (dt < np.pi / nt * 1.6)
            if m.sum() >= 2: R[i, j] = np.percentile(r[sel][m], 92)
    for _ in range(nh * nt):
        miss = np.isnan(R)
        if not miss.any(): break
        Rp = np.pad(R, 1, mode='edge'); nb = np.stack([Rp[:-2, 1:-1], Rp[2:, 1:-1], Rp[1:-1, :-2], Rp[1:-1, 2:]])
        fill = np.nanmean(np.where(np.isnan(nb), np.nan, nb), 0); R[miss] = fill[miss]
    R = np.nan_to_num(R, nan=float(np.nanmedian(R)))
    for _ in range(4):
        Rp = np.pad(R, 1, mode='edge'); R = (4 * R + Rp[:-2, 1:-1] + Rp[2:, 1:-1] + Rp[1:-1, :-2] + Rp[1:-1, 2:]) / 8
    R = R + offset
    # Knee guard: a dome over the kneecap, part of the same sheet.
    fr = np.maximum(0, np.cos(T))[None, :] ** 3
    R += dome * np.exp(-((Hm - (L + .005)) / .035) ** 2) * fr
    for rc in ridges:
        R += .0016 * np.exp(-((T[None, :] - rc) / .06) ** 2) * (Hm < L - .02)
    R += .0018 * np.exp(-(T[None, :] / .05) ** 2) * (Hm > L - .03)   # median ridge up the guard
    H = None
    def pt(i, j, extra=0.):
        return ankle + u * Hm[i, j] + (side * math.sin(T[j]) + front * math.cos(T[j])) * (R[i, j] + extra)
    verts, faces, uvs = [], [], []
    for i in range(nh):
        for j in range(nt):
            verts.append(tuple(pt(i, j))); uvs.append((j / (nt - 1), i / (nh - 1)))
    for i in range(nh - 1):
        for j in range(nt - 1):
            faces.append((i * nt + j, i * nt + j + 1, (i + 1) * nt + j + 1, (i + 1) * nt + j))
    shell = _mesh(name, verts, faces, uvs, bronze, thickness=thick)   # single-sided sheet: decimates cleanly
    # Solidify offset=1 pushes outward; pull the shell back so its inner face sits at offset.
    objs = [shell]
    def tube(pts, rad, nm, closed=False):
        vs, fs, us = [], [], []; seg = 6; n = len(pts)
        for a_, p in enumerate(pts):
            t = pts[(a_ + 1) % n if closed else min(a_ + 1, n - 1)] - pts[(a_ - 1) % n if closed else max(a_ - 1, 0)]
            t = t / max(np.linalg.norm(t), 1e-9); b1 = np.cross(t, u); b1 /= max(np.linalg.norm(b1), 1e-9); b2 = np.cross(t, b1)
            for k in range(seg):
                ang = 2 * math.pi * k / seg; vs.append(tuple(p + rad * (b1 * math.cos(ang) + b2 * math.sin(ang)))); us.append((a_ / n, k / seg))
        for a_ in range(n - (0 if closed else 1)):
            b_ = (a_ + 1) % n
            for k in range(seg): fs.append((a_ * seg + k, a_ * seg + (k + 1) % seg, b_ * seg + (k + 1) % seg, b_ * seg + k))
        return _mesh(nm, vs, fs, us, bronze)
    for i, nm, out_ in ((0, '-rim-bottom', .0015), (nh - 1, '-rim-top', .005)):
        # The top rim rolls outward so a high camera does not see inside the greave.
        objs.append(tube(np.array([pt(i, j, thick + out_) for j in range(nt)]), .004 if out_ > .002 else .0035, name + nm))
    if repousse:
        # Repousse as baked detail, not geometry: a scroll pair on the knee guard and a
        # ridge line down the shin, in the shell's own (angle, height) UVs, driving a
        # bump (normal bake) and darker grooves (colour bake).
        S_ = 512; v_, u_ = np.mgrid[0:S_, 0:S_] / (S_ - 1); hgt = np.zeros((S_, S_))
        hgt += np.exp(-((u_ - .5) / .018) ** 2) * (v_ > .08) * (v_ < .72)             # shin ridge
        for sgn in (-1, 1):                                                         # scroll pair
            cu, cv = .5 + sgn * .055, .84
            du, dv = (u_ - cu) * 2.4, v_ - cv; rr = np.hypot(du, dv); aa = np.arctan2(dv, du * sgn)
            spiral = np.cos(rr * 2 * np.pi / .022 - aa) ; hgt += np.clip(spiral - .55, 0, 1) * 2.2 * (rr < .055)
        hgt = np.clip(hgt, 0, 1)
        im = bpy.data.images.new(name + ' repousse', S_, S_, alpha=False)
        im.colorspace_settings.name = 'Non-Color'   # before writing: changing it reloads the buffer
        im.pixels.foreach_set(np.concatenate([np.repeat(hgt[..., None], 3, 2), np.ones((S_, S_, 1))], 2).astype(np.float32).ravel())
        im.pack()
        m = bronze.copy(); m.name = name + ' bronze'; nt_ = m.node_tree; b_ = nt_.nodes['Principled BSDF']
        tx = nt_.nodes.new('ShaderNodeTexImage'); tx.image = im
        bump = nt_.nodes.new('ShaderNodeBump'); bump.inputs['Strength'].default_value = 1.; bump.inputs['Distance'].default_value = .007
        nt_.links.new(tx.outputs['Color'], bump.inputs['Height']); nt_.links.new(bump.outputs['Normal'], b_.inputs['Normal'])
        mix = nt_.nodes.new('ShaderNodeMixRGB'); mix.blend_type = 'MULTIPLY'; mix.inputs[0].default_value = .7
        mix.inputs[1].default_value = tuple(b_.inputs['Base Color'].default_value)
        inv_ = nt_.nodes.new('ShaderNodeInvert'); nt_.links.new(tx.outputs['Color'], inv_.inputs['Color'])
        edge_ = nt_.nodes.new('ShaderNodeBump')  # unused placeholder avoided below
        nt_.nodes.remove(edge_)
        nt_.links.new(inv_.outputs['Color'], mix.inputs[2]); nt_.links.new(mix.outputs[0], b_.inputs['Base Color'])
        shell.data.materials[0] = m
    rep = {'height_m': round(float(top.max() - h0), 3), 'guard_rise_m': rise, 'radius_mm': [round(float(R.min()) * 1000), round(float(R.max()) * 1000)]}
    if knee_cap:
        # Knee guard: a domed shield over the kneecap, wider than tall, with a vertical
        # median ridge and a raised border, sitting on the greave top and rising over the knee.
        j0 = int(np.argmin(np.abs(T))); r_top = float(R[-1, j0])
        kc = knee + front * (r_top + .004) + u * .028
        W_, Hh, D = .06, .095, .013; nu, nv = 20, 14
        vs, fs, us = [], [], []
        for a_ in range(nv + 1):
            y = -1 + 2 * a_ / nv
            for b_ in range(nu + 1):
                x = -1 + 2 * b_ / nu
                rr = min(1., x * x + y * y)
                bulge = D * (1 - rr) ** .6 + .003 * math.exp(-(x / .1) ** 2) * (1 - rr) \
                    + .0025 * math.exp(-((math.sqrt(x * x + y * y) - .82) / .07) ** 2)
                # ellipse outline: squeeze rows to the cap's oval
                s_ = math.sqrt(max(0., 1 - y * y))
                vs.append(tuple(kc + side * (x * s_ * W_) + u * (y * Hh / 2) + front * bulge)); us.append((b_ / nu, a_ / nv))
        w = nu + 1
        for a_ in range(nv):
            for b_ in range(nu): fs.append((a_ * w + b_, a_ * w + b_ + 1, (a_ + 1) * w + b_ + 1, (a_ + 1) * w + b_))
        cap = _mesh(name + '-kneecap', vs, fs, us, bronze)
        objs.append(cap); rep['knee_guard'] = {'width_m': 2 * W_, 'height_m': Hh, 'dome_mm': D * 1000}
    return objs, rep
