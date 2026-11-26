"""Fit a third-party character model onto the approved 0 A.D. hoplite rig.

Headless Blender. Imports a glTF/GLB or FBX source in any pose, unposes it
onto the 0 A.D. bind pose and bone lengths, seats the spear, shield and helmet
as rigid props, transfers the 0 A.D. skin weights, builds two LODs, bakes the
source's materials into one atlas (base colour, normal, packed ORM) in Cycles on
the CPU, adds the COLOR_0 team mask, and writes a contract GLB whose skeleton
and five clips are copied byte for byte from the approved GLB.

    source tools/art/anim3d/env.sh
    ~/LegatumRuns/anim-pilot/bin/blender-slot --factory-startup --python-exit-code 1 \\
      --python tools/art/anim3d/kit/fit_source_to_0ad.py -- \\
      --source model.glb --out out/spearman.glb --credit '"Model" by Author, CC BY 4.0'

See kit/FIT.md for every option, how the pose fit works and its limits.
"""
import argparse, hashlib, json, math, os, re, shutil, sys, time
from pathlib import Path
import bpy, bmesh
import numpy as np
from mathutils import Matrix, Vector

KIT = Path(__file__).resolve().parent
sys.path.insert(0, str(Path(__file__).resolve().parent)); sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from fit_rig import Rig, CHAINS, OAD_CREDIT, RIGID
from fit_landmarks import sample_surface, Skeleton, calibrate, joints_from_mesh, map_armature, project
from fit_pose import fix_hip, fit_heels, heal, keep_clearance, unpose, apply_map, apply_blend, _swing
from fit_props import components, classify, find_helmet, seat_spear, seat_shield
from fit_ground import clip_planes, fit as ground_fit
from fit_pack import pack

ROOT = Path(os.environ.get('ANIM_ROOT', Path.home() / 'LegatumRuns/anim-pilot'))
APPROVED = KIT.parent / 'units/spearman-0ad/spearman.glb'
PART_ORDER = ['body', 'layer', 'cloak', 'helmet', 'shield', 'spear']  # 'layer': clothing over the skin, baked on its own
GROUPS = {'torso': ['hip', 'spine', 'spine1', 'chest', 'neck', 'head', 'shoulder_L', 'shoulder_R'],
          'arm_L': ['shoulder_L', 'arm_L', 'forearm_L', 'hand_L', 'finger_L', 'fingertip_L', 'shield_arm'],
          'arm_R': ['shoulder_R', 'arm_R', 'forearm_R', 'hand_R', 'finger_R', 'fingertip_R', 'weapon_R'],
          'leg_L': ['hip', 'thigh_L', 'leg_L', 'foot_L'], 'leg_R': ['hip', 'thigh_R', 'leg_R', 'foot_R']}

argv = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
ap = argparse.ArgumentParser(prog='fit_source_to_0ad.py')
ap.add_argument('--source', type=Path, required=True, help='glTF/GLB or FBX model')
ap.add_argument('--out', type=Path, required=True, help='output .glb (PROVENANCE.md and LICENSE-ART are written beside it if missing)')
ap.add_argument('--credit', required=True, help='credit for the source model, e.g. \'"Hoplite Warrior" by caseydedore, CC BY 4.0\'')
ap.add_argument('--rig', type=Path, default=APPROVED, help='approved GLB with the 0 A.D. skeleton and the five clips')
ap.add_argument('--work', type=Path, help='heavy outputs (.blend, 2048 masters); default ANIM_ROOT/fit/<out stem>')
ap.add_argument('--part', action='append', default=[], metavar='PART=REGEX',
                help='force objects/materials matching REGEX to PART (spear, shield, helmet, cloak, body, drop); repeatable')
ap.add_argument('--team', action='append', default=[], metavar='REGION=REGEX',
                help='faces whose material or object matches REGEX are team colour (REGION: crest, shield rim, cloak trim)')
ap.add_argument('--no-auto-team', action='store_true', help='use only --team rules')
ap.add_argument('--landmarks', type=Path, help='JSON {joint: [x,y,z]} in the source file\'s own glTF coordinates; overrides detected joints')
ap.add_argument('--yaw', type=float, help='degrees about +Y to make the model face +Z (default: from the toes)')
ap.add_argument('--stop-after-landmarks', action='store_true', help='write work/landmarks.blend and stop (debugging)')
ap.add_argument('--flip-spear', action='store_true', help='blade/butt detection got it backwards')
ap.add_argument('--max-height', type=float, default=1.93, help='helmet/crest ceiling in metres (contract: 1.65-1.95 m)')
ap.add_argument('--lod0', type=int, default=9600); ap.add_argument('--lod1', type=int, default=2900)
ap.add_argument('--bake-size', type=int, default=2048); ap.add_argument('--ship-size', type=int, default=1024)
ap.add_argument('--ao-samples', type=int, default=16)
ap.add_argument('--jpeg-quality', type=int, default=80, help='quality of the shipped maps (JPEG or WebP)')
ap.add_argument('--texture-format', choices=['jpeg', 'webp'], default='webp', help='shipped map encoding (WebP keeps a 1024 set under 2 MB)')
ap.add_argument('--skin', help="regex for the object(s) that are the body's skin; other body objects are layers that follow it (default 'body')")
ap.add_argument('--smart-uv', action='store_true', help='smart-project a new atlas instead of repacking the source UV charts')
ap.add_argument('--crest-glb', type=Path, help="take the crest (team region 'crest' plus 'crest_base') from another unit GLB on this rig, seated on this helmet's crown")
ap.add_argument('--keep', action='append', default=[], metavar='PART', help='keep PART at full source topology in LOD0 (thin garments shred when decimated); repeatable')
ap.add_argument('--grade-bronze', action='store_true', help='regrade metallic texels toward warm gold-brown bronze: darker recesses, bright worn edges, rougher')
ap.add_argument('--grade-skin', help="regrade skin texels toward this sRGB hex mean (e.g. b07850 for a tanned tone)")
ap.add_argument('--sleeves', type=float, metavar='FRACTION', help='replace the upper-arm garment with fitted linen sleeve shells down to FRACTION of the upper arm')
ap.add_argument('--strip', action='append', default=[], metavar='BONE', help='delete clothing/armour around a 0 A.D. limb bone (e.g. forearm_L, hidden behind the shield) where it cannot follow the clips cleanly')
ap.add_argument('--corinthian', action='store_true', help='close an open helmet with a Corinthian face plate (cheek and nasal guards, dark T opening)')
ap.add_argument('--press-skin', action='store_true', help='sink skin a few mm under any clothing or armour close above it (no poke-through)')
ap.add_argument('--add-crest', action='store_true', help='model a horsehair crest arc on a crestless helmet (team colour)')
ap.add_argument('--shield-glb', type=Path, help='take the aspis (vertices rigid to shield_arm) from another unit GLB on this rig, replacing the source shield')
ap.add_argument('--license', default='CC-BY-SA-3.0', help='licence of the combined asset (the 0 A.D. clips are CC BY-SA 3.0)')
args = ap.parse_args(argv)
t0 = time.time()
out = args.out.resolve(); out.parent.mkdir(parents=True, exist_ok=True)
work = (args.work or ROOT / 'fit' / out.stem).resolve(); work.mkdir(parents=True, exist_ok=True)
overrides = dict(p.split('=', 1) for p in args.part)
team_rules = [tuple(r.split('=', 1)) for r in args.team]
report = {'source': str(args.source), 'source_sha256': hashlib.sha256(args.source.read_bytes()).hexdigest(), 'args': argv}


def log(*a): print('FIT', f'{time.time() - t0:6.1f}s', *a, flush=True)


# ---------------------------------------------------------------------------------- import
rig = Rig(args.rig)
bpy.ops.wm.read_factory_settings(use_empty=True)
src = str(args.source)
if src.lower().endswith(('.glb', '.gltf')): bpy.ops.import_scene.gltf(filepath=src, merge_vertices=True)
elif src.lower().endswith('.fbx'): bpy.ops.import_scene.fbx(filepath=src, automatic_bone_orientation=True)
else: raise SystemExit('Source must be .glb, .gltf or .fbx')
CONV = Matrix(((1, 0, 0, 0), (0, 0, 1, 0), (0, -1, 0, 0), (0, 0, 0, 1)))  # Blender Z-up -> glTF Y-up
dep = bpy.context.evaluated_depsgraph_get()
armature = next((o for o in bpy.context.scene.objects if o.type == 'ARMATURE'), None)
arm_joints = {}
if armature:
    for pb in armature.pose.bones:
        arm_joints[pb.name] = np.array(CONV @ armature.matrix_world @ pb.head)
objects = []
for ob in [o for o in bpy.context.scene.objects if o.type == 'MESH' and o.visible_get()]:
    me = bpy.data.meshes.new_from_object(ob.evaluated_get(dep), preserve_all_data_layers=True, depsgraph=dep)
    me.transform(CONV @ ob.matrix_world)
    if ob.matrix_world.determinant() < 0: me.flip_normals()
    # Weld split vertices (glTF/FBX split at UV and normal seams) so loose parts
    # and decimation see real topology; UVs and normals stay per corner.
    bm = bmesh.new(); bm.from_mesh(me)
    bmesh.ops.remove_doubles(bm, verts=list(bm.verts), dist=1e-5 * max(ob.dimensions))
    bm.to_mesh(me); bm.free()
    new = bpy.data.objects.new(ob.name, me); objects.append(new)
for ob in list(bpy.context.scene.objects): bpy.data.objects.remove(ob, do_unlink=True)
for ob in objects:
    bpy.context.scene.collection.objects.link(ob)
    if ob.data.shape_keys: ob.shape_key_clear()  # morph targets block modifier_apply
log('imported', len(objects), 'meshes', sum(len(o.data.polygons) for o in objects), 'faces', 'armature' if armature else 'no armature')


def co(ob):
    a = np.empty(len(ob.data.vertices) * 3); ob.data.vertices.foreach_get('co', a); return a.reshape(-1, 3)


def set_co(ob, a):
    ob.data.vertices.foreach_set('co', np.ascontiguousarray(a, dtype=np.float64).ravel()); ob.data.update()


def tris(ob):
    me = ob.data; me.calc_loop_triangles()
    t = np.empty(len(me.loop_triangles) * 3, dtype=np.int64); me.loop_triangles.foreach_get('vertices', t)
    p = np.empty(len(me.loop_triangles), dtype=np.int64); me.loop_triangles.foreach_get('polygon_index', p)
    return t.reshape(-1, 3), p


# ------------------------------------------------------- loose parts and prop classification
pieces = []; info = {}
for ob in objects:
    V = co(ob); F, poly = tris(ob)
    mats = [m.name if m else '' for m in ob.data.materials]
    pmat = np.empty(len(ob.data.polygons), dtype=np.int64); ob.data.polygons.foreach_get('material_index', pmat)
    lab = components(F, len(V)); info[ob.name] = dict(V=V, F=F, poly=poly, lab=lab, pmat=pmat)
    for c in np.unique(lab[F[:, 0]]) if len(F) else []:
        vid = np.nonzero(lab == c)[0]; fsel = lab[F[:, 0]] == c
        used = sorted(set(pmat[poly[fsel]].tolist()))
        pieces.append(dict(object=ob.name, comp=int(c), vid=vid, points=V[vid],
                           materials=[mats[i] for i in used if i < len(mats)]))
allp = np.concatenate([p['points'] for p in pieces])
height0 = np.percentile(allp[:, 1], 99.5) - allp[:, 1].min()
classify(pieces, height0, overrides)
# Clothing and armour layers move with the skin under them: each layer vertex
# takes the unpose blend of its nearest skin vertex. Skin alone also gives a
# cleaner skeleton: loose shells (skirts, cuirass flaps) sprout false limb tips.
skin_rx = args.skin or r'body'
if any(p['part'] == 'body' and re.search(skin_rx, p['object'], re.I) for p in pieces):
    for p in pieces:
        if p['part'] == 'body' and not re.search(skin_rx, p['object'], re.I): p['part'] = 'layer'
log('parts', {k: sum(1 for p in pieces if p['part'] == k) for k in set(p['part'] for p in pieces)})


def gather(parts):
    """Concatenated vertices and triangles of every piece in parts, with a back-map."""
    Vs, Fs, back, off = [], [], [], 0
    for ob in objects:
        d = info[ob.name]; want = np.zeros(len(d['V']), bool)
        for p in pieces:
            if p['object'] == ob.name and p['part'] in parts: want[p['vid']] = True
        if not want.any(): continue
        idx = np.nonzero(want)[0]; remap = -np.ones(len(d['V']), np.int64); remap[idx] = np.arange(len(idx)) + off
        F = d['F'][want[d['F']].all(1)]
        Vs.append(d['V'][idx]); Fs.append(remap[F]); back.append((ob.name, idx)); off += len(idx)
    return np.concatenate(Vs), np.concatenate(Fs), back


def transform_all(M):
    for ob in objects:
        d = info[ob.name]; d['V'] = d['V'] @ M[:3, :3].T + M[:3, 3]
    for p in pieces:
        p['points'] = info[p['object']]['V'][p['vid']]


# -------------------------------------------------------------- orientation and scale
cal, cal_sk, _ = calibrate(rig)
report['calibration'] = {'span_m': cal['span'], 'arc_m': cal['arc']}
V, F, _ = gather({'body'})
sk = Skeleton(sample_surface(V, F))
if args.yaw is not None:
    a = math.radians(args.yaw); R = np.array([[math.cos(a), 0, math.sin(a)], [0, 1, 0], [-math.sin(a), 0, math.cos(a)]])
else:
    R = _swing(sk.forward(), np.array([0, 0, 1.]))
s = cal['span'] / sk.span
M = np.eye(4); M[:3, :3] = R * s; transform_all(M)
report['normalise'] = {'scale': s, 'yaw_deg': math.degrees(math.atan2(R[0, 2], R[0, 0]))}
log('normalised: scale', round(s, 4), 'yaw', round(report['normalise']['yaw_deg'], 1))
landmarks = None
if args.landmarks:
    landmarks = {k: (M[:3, :3] @ np.array(v)) + M[:3, 3] for k, v in json.loads(args.landmarks.read_text()).items()}


def locate():
    V, F, back = gather({'body'})
    pts = sample_surface(V, F)
    joints, sk, chains, names = joints_from_mesh(pts, cal, landmarks)
    if armature:
        mapped = map_armature(list(arm_joints))
        for j, b in mapped.items():
            if j in rig.joint: joints[j] = M[:3, :3] @ arm_joints[b] + M[:3, 3]
        for chain, (line, s_, arcs) in chains.items():
            for j in CHAINS[chain]:
                if j in mapped: arcs[j] = float(project(line, s_, joints[j][None])[0][0])
        report['armature_map'] = mapped
    return V, F, back, joints, sk, chains


V, F, back, joints, sk, chains = locate()
if not any(p['part'] == 'helmet' for p in pieces):
    found = find_helmet(pieces, joints['head'], joints['head_tip'], joints['neck'])
    if found:
        log('helmet: loose parts around the head', len(found), [(p['object'], len(p['points']), np.round(p['points'].mean(0), 3).tolist()) for p in found][:6], 'head', np.round(joints['head'], 3).tolist(), 'tip', np.round(joints['head_tip'], 3).tolist())
        V, F, back, joints, sk, chains = locate()
joints = fix_hip(joints, rig.joint)
report['tips'] = {k: dict(pos=np.round(t['pos'], 3).tolist(), branch=round(t['branch'], 3), geodesic=round(t['geodesic'], 3))
                  for k, t in sk.tips.items()}
report['extra_tips'] = [np.round(t['pos'], 3).tolist() for t in sk.extra_tips]
log('tips', report['tips'], 'extra', report['extra_tips'])
# Landmark check scene: the normalised source pose with a marker at every joint.
for ob in objects: set_co(ob, info[ob.name]['V'])
markers = []
for name, pos in joints.items():
    bpy.ops.mesh.primitive_ico_sphere_add(radius=.025, location=tuple(pos)); m = bpy.context.object
    m.name = 'joint-' + name; markers.append(m)
    mat = bpy.data.materials.get('marker') or bpy.data.materials.new('marker'); mat.diffuse_color = (0, .9, .2, 1)
    m.data.materials.append(mat)
for chain, (line, s_, arcs) in chains.items():
    cu = bpy.data.curves.new('line-' + chain, 'CURVE'); cu.dimensions = '3D'; cu.bevel_depth = .006
    sp = cu.splines.new('POLY'); sp.points.add(len(line) - 1)
    for i, q in enumerate(line): sp.points[i].co = (*q, 1)
    c = bpy.data.objects.new('line-' + chain, cu); bpy.context.scene.collection.objects.link(c); markers.append(c)
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=str(work / 'landmarks.blend'))
for m in markers: bpy.data.objects.remove(m, do_unlink=True)
if args.stop_after_landmarks:
    (work / 'fit-report.json').write_text(json.dumps(report, indent=2, default=float) + '\n'); raise SystemExit(0)
report['joints_source'] = {k: [round(float(x), 4) for x in v] for k, v in joints.items()}
log('landmarks', {k: np.round(v, 3).tolist() for k, v in joints.items() if k in ('head', 'arm_L', 'hand_R', 'leg_L', 'foot_R')})

# ------------------------------------------------------------------------ unpose
blend = .035 * sk.height
fitted, weights, maps, spec = unpose(V, sk.graph.point_voxel[:len(V)], sk, chains, joints, rig.joint, blend)
order = list(chains)
wsum = np.zeros((len(V), len(order)))
# Group membership for the weight-transfer masks: the chain each vertex was unposed with.
for ci, chain in enumerate(order):
    bones = set(CHAINS[chain][:-1])
    wsum[:, ci] = sum(w for b, w in weights.items() if b in bones)
group_mask = {chain: wsum[:, ci] for ci, chain in enumerate(order)}
# Chest/hip weight carried by a limb label belongs to the torso group.
group_mask['torso'] = 1 - sum(group_mask[c] for c in order if c != 'torso')


def scatter(back, values, key):
    off = 0
    for name, idx in back:
        info[name].setdefault(key, np.zeros((len(info[name]['V']),) + values.shape[1:]))[idx] = values[off:off + len(idx)]
        off += len(idx)


# Shoulders carry the largest unposing rotations (a raised arm comes down to
# the 0 A.D. A-pose); relax the displacement around them as well as at tears.
shoulders = np.zeros(len(V), bool)
for j in ('arm_L', 'arm_R'):
    shoulders |= np.linalg.norm(V - joints[j], axis=1) < .07 * sk.height
fitted, report['torn_edges_healed'] = heal(V, fitted, F, extra=shoulders)
report['heel_scale'] = fit_heels(fitted, weights, rig)
scatter(back, fitted, 'fitted')
if os.environ.get('FIT_DEBUG'):
    pal = {b: np.random.default_rng(abs(hash(b)) % 2**32).random(3) for b in rig.bones}
    lo_, hi_, _, t_ = spec
    scatter(back, np.array([pal[a] * (1 - t) + pal[b] * t for a, b, t in zip(lo_, hi_, t_)]), 'debug_col')
scatter(back, np.stack([group_mask[c] for c in order], 1), 'groups')
# Cloaks follow the nearest body vertex's map blend.
cV, cF, cback = gather({'cloak'}) if any(p['part'] == 'cloak' for p in pieces) else (np.zeros((0, 3)), None, [])
if len(cV):
    # Cloaks hang from the torso: follow the nearest torso vertex, never an arm or
    # leg (a raised arm would otherwise drag the cloak's corner with it).
    torso = np.nonzero(np.isin(spec[0], CHAINS['torso']))[0]
    near = np.empty(len(cV), np.int64)
    for k in range(0, len(cV), 2048):
        near[k:k + 2048] = torso[np.argmin(((cV[k:k + 2048, None] - V[torso][None]) ** 2).sum(2), 1)]
    # Same per-vertex blend as the nearest body vertex, applied at the cloak vertex.
    cfit = apply_blend(maps, joints, cV, *(x[near] for x in spec))
    # No longer than the 0 A.D. cape: its bones were animated for that hem, and a
    # longer cloak sinks into the floor in walk and death.
    hem = rig.P[np.unique(rig.cloak_faces), 1].min() if len(rig.cloak_faces) else cfit[:, 1].min()
    top = cfit[:, 1].max()
    if cfit[:, 1].min() < hem:
        k = (top - hem) / (top - cfit[:, 1].min()); cfit[:, 1] = top - (top - cfit[:, 1]) * k
        report['cloak_length_scale'] = round(float(k), 4)
    scatter(cback, cfit, 'fitted')
    scatter(cback, np.stack([group_mask[c][near] for c in order], 1), 'groups')
lV, lF, lback = gather({'layer'}) if any(p['part'] == 'layer' for p in pieces) else (np.zeros((0, 3)), None, [])
if len(lV):
    near = np.empty(len(lV), np.int64)
    for k in range(0, len(lV), 256):
        near[k:k + 256] = np.argmin(((lV[k:k + 256, None] - V[None]) ** 2).sum(2), 1)
    # A garment piece mostly over the torso (cuirass, shoulder plates, pteruges)
    # moves with the torso as a whole: letting its corners follow arm-labelled
    # skin drags them out with the arms when a raised arm is brought down.
    torso_skin = np.isin(spec[0], CHAINS['torso']) & np.isin(spec[1], CHAINS['torso'])
    comp = np.concatenate([info[name]['lab'][idx] + i * 10**8 for i, (name, idx) in enumerate(lback)])
    cname = {i: name for i, (name, _) in enumerate(lback)}
    tidx = np.nonzero(torso_skin)[0]; rigid_pieces = 0
    stats = []
    for c in np.unique(comp):
        sel = np.nonzero(comp == c)[0]
        if len(sel) > 300: stats.append((len(sel), cname[int(c // 10**8)][:14], round(float(torso_skin[near[sel]].mean()), 2)))
        if torso_skin[near[sel]].mean() > .45 and not torso_skin[near[sel]].all():
            for k in range(0, len(sel), 256):
                s_ = sel[k:k + 256]
                near[s_] = tidx[np.argmin(((lV[s_, None] - V[tidx][None]) ** 2).sum(2), 1)]
            rigid_pieces += 1
    agg = {}
    for n_, nm, sh in stats: agg.setdefault(nm, []).append((n_, sh))
    log('garment pieces (object: largest pieces with torso share)', {k: sorted(v, reverse=True)[:3] for k, v in agg.items()})
    lfit = apply_blend(maps, joints, lV, *(x[near] for x in spec))
    lfit, healed = heal(lV, lfit, lF)
    lfit, pushed = keep_clearance(V, F, fitted, lV, lfit, near)
    fit_heels(lfit, {b: w[near] for b, w in weights.items()}, rig)  # shoes and greaves too
    scatter(lback, lfit, 'fitted')
    scatter(lback, np.stack([group_mask[c][near] for c in order], 1), 'groups')
    report['layers_follow_skin'] = {'skin_vertices': int(len(V)), 'layer_vertices': int(len(lV)), 'torn_edges_healed': healed, 'pushed_outside_skin': pushed, 'torso_garment_pieces': rigid_pieces}
    log('layers follow skin', report['layers_follow_skin'])

# ---------------------------------------------------------------------- rigid props
seats = {}
hV, _, hback = gather({'helmet'}) if any(p['part'] == 'helmet' for p in pieces) else (np.zeros((0, 3)), None, [])
if len(hV):
    hfit = apply_map(maps['head'], hV)
    # Contract ceiling: the figure (crest included) stays under 1.95 m. Shrink the
    # helmet about the head joint only when the source crest would exceed it.
    ceiling = args.max_height
    if hfit[:, 1].max() > ceiling:
        piv = rig.joint['head']; k = (ceiling - piv[1]) / (hfit[:, 1].max() - piv[1])
        hfit = piv + (hfit - piv) * k; report['helmet_scaled'] = round(float(k), 4)
    scatter(hback, hfit, 'fitted')
for part, fn in (('spear', lambda p: seat_spear(p, rig.spear_frame(), args.flip_spear)),
                 ('shield', lambda p: seat_shield(p, rig.shield_frame(), np.array([0, 1., 0])))):
    pV, _, pback = gather({part}) if any(p['part'] == part for p in pieces) else (np.zeros((0, 3)), None, [])
    if not len(pV): raise SystemExit(f'No {part} found: pass --part {part}=REGEX (object or material name)')
    Mp, meta = fn(pV); seats[part] = meta
    scatter(pback, pV @ Mp[:3, :3].T + Mp[:3, 3], 'fitted')
report['props'] = seats
missing = [p for p in ('helmet',) if not any(q['part'] == p for q in pieces)]
if missing: raise SystemExit('No helmet found: pass --part helmet=REGEX (object or material name)')

# --------------------------------------------- write fitted geometry, split by part
fitted_objects = {}
for ob in objects:
    d = info[ob.name]
    if 'fitted' not in d:
        bpy.data.objects.remove(ob, do_unlink=True); continue
    set_co(ob, d['fitted'])
    vpart = np.empty(len(d['V']), dtype=object)
    for p in pieces:
        if p['object'] == ob.name: vpart[p['vid']] = p['part']
    if 'groups' in d:  # (layers and cloaks carry their nearest skin vertex's groups)
        for ci, chain in enumerate(order):
            vg = ob.vertex_groups.new(name='grp_' + chain)
            for i in np.nonzero(d['groups'][:, ci] > 1e-4)[0]: vg.add([int(i)], float(d['groups'][i, ci]), 'REPLACE')
    if 'debug_col' in d:
        ca = ob.data.color_attributes.new('debug', 'FLOAT_COLOR', 'POINT')
        ca.data.foreach_set('color', np.hstack([d['debug_col'], np.ones((len(d['V']), 1))]).ravel())
    for part in PART_ORDER:
        if not (vpart == part).any(): continue
        x = ob.copy(); x.data = ob.data.copy(); bpy.context.scene.collection.objects.link(x)
        bm = bmesh.new(); bm.from_mesh(x.data); bm.verts.ensure_lookup_table()
        bmesh.ops.delete(bm, geom=[v for v in bm.verts if vpart[v.index] != part], context='VERTS')
        bm.to_mesh(x.data); bm.free(); x.name = f'HIGH-{part}-{ob.name}'; x['part'] = part
        fitted_objects.setdefault(part, []).append(x)
    bpy.data.objects.remove(ob, do_unlink=True)
for part in [p for p in PART_ORDER if p in fitted_objects]:
    for x in fitted_objects[part]:
        for poly in x.data.polygons: poly.use_smooth = True


def all_co(objs): return np.concatenate([co(o) for o in objs])


from fit_shells import sleeve, corinthian, press_under, material as shell_material
if args.sleeves and 'body' in fitted_objects:
    # Linen sleeves rebuilt as simple shells around the 0 A.D. upper arms: the
    # source's sleeve pieces tore and let the skin through once the arms moved.
    skin_pts = all_co(fitted_objects['body'])
    linen = shell_material('Linen sleeve (generated)', None, .9, 0,
                           lambda u, v: np.stack([.90 + 0 * u, .85 + 0 * u, .72 + 0 * u], -1) * (.93 + .07 * (np.sin(u * 900) > 0))[..., None] * np.where((v > .9)[..., None], .82, 1))
    removed = 0
    for side in 'LR':
        a, b = rig.joint['arm_' + side], rig.joint['forearm_' + side]
        o, (u, tmax, rmax) = sleeve(skin_pts, a, b, args.sleeves, linen, f'Sleeve {side}')
        o['part'] = 'layer'; fitted_objects.setdefault('layer', []).append(o)
        for lo_ in fitted_objects.get('layer', []):
            if lo_ is o or lo_.name.startswith('Sleeve'): continue
            bm = bmesh.new(); bm.from_mesh(lo_.data)
            dead = []
            for f in bm.faces:
                c = np.array(f.calc_center_median()) - a; tt = c @ u; rr = np.linalg.norm(c - u * tt)
                # Only garment lateral of the shoulder joint: the cuirass beside the
                # arm (same distance from the bone axis) must stay.
                if .035 < tt < tmax + .03 and rr < rmax + .015 and c[0] * np.sign(a[0]) > -.005: dead.append(f)
            bmesh.ops.delete(bm, geom=dead, context='FACES'); bm.to_mesh(lo_.data); bm.free(); removed += len(dead)
    report['sleeves'] = {'fraction': args.sleeves, 'source_faces_removed': removed}
    log('sleeves rebuilt', report['sleeves'])
NEXT = {'arm_L': 'forearm_L', 'forearm_L': 'hand_L', 'arm_R': 'forearm_R', 'forearm_R': 'hand_R',
        'thigh_L': 'leg_L', 'leg_L': 'foot_L', 'thigh_R': 'leg_R', 'leg_R': 'foot_R'}
for bone_ in args.strip:
    a, b = rig.joint[bone_], rig.joint[NEXT[bone_]]; u = (b - a) / np.linalg.norm(b - a); L_ = np.linalg.norm(b - a)
    gone = 0
    for lo_ in fitted_objects.get('layer', []):
        if lo_.name.startswith('Sleeve'): continue
        bm = bmesh.new(); bm.from_mesh(lo_.data); dead = []
        for f in bm.faces:
            c = np.array(f.calc_center_median()) - a; tt = c @ u
            if .02 < tt < L_ + .02 and np.linalg.norm(c - u * tt) < .075: dead.append(f)
        bmesh.ops.delete(bm, geom=dead, context='FACES'); bm.to_mesh(lo_.data); bm.free(); gone += len(dead)
    report.setdefault('stripped', {})[bone_] = gone; log('stripped clothing around', bone_, gone)
if args.corinthian and 'helmet' in fitted_objects:
    hel = [o for o in fitted_objects['helmet'] if not o.name.startswith('Crest')]
    face = all_co(fitted_objects['body']); hp = all_co(hel)
    face = face[(face[:, 1] > hp[:, 1].min() - .05) & (np.abs(face[:, 0]) < .07)]
    rng_ = np.random.default_rng(11); ham = np.kron(rng_.random((32, 32)), np.ones((8, 8)))
    plates, info_ = corinthian(hp, face, shell_material('Corinthian bronze (generated)', None, .4, 1.,
                               lambda u, v: np.stack([.36 + 0 * u, .24 + 0 * u, .10 + 0 * u], -1) * (.8 + .3 * ham[..., None])),
                               shell_material('Helmet interior (generated)', (.015, .012, .01), .9, 0.))
    for o in plates: o['part'] = 'helmet'; fitted_objects['helmet'].append(o)
    report['corinthian'] = info_; log('corinthian face plate', info_)
if args.press_skin and 'body' in fitted_objects and fitted_objects.get('layer'):
    report['skin_pressed_under_layers'] = press_under(fitted_objects['body'], all_co(fitted_objects['layer']))
    log('skin pressed under layers', report['skin_pressed_under_layers'])

if args.shield_glb:
    # A finished aspis built on this same rig: its vertices rigid to shield_arm are
    # already seated in bind space. Its own materials are baked like the source's.
    for o in fitted_objects.pop('shield', []): bpy.data.objects.remove(o, do_unlink=True)
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(args.shield_glb))
    new_objs = [o for o in bpy.data.objects if o not in before]
    lod = next(o for o in new_objs if o.type == 'MESH' and o.name.startswith('spearman_LOD0'))
    me = lod.data.copy(); me.transform(CONV @ lod.matrix_world)
    gi = lod.vertex_groups['shield_arm'].index
    bm = bmesh.new(); bm.from_mesh(me); dl = bm.verts.layers.deform.verify()
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if v[dl].get(gi, 0) < .999], context='VERTS')
    bm.to_mesh(me); bm.free()
    for o in new_objs: bpy.data.objects.remove(o, do_unlink=True)
    # That unit's COLOR_0 is a team mask, not a tint: the glTF importer multiplies
    # it into the base colour (black where red is 0). Feed the texture directly.
    for m in me.materials:
        nt = m.node_tree; bsdf = next(n for n in nt.nodes if n.type == 'BSDF_PRINCIPLED')
        tex = next((n for n in nt.nodes if n.type == 'TEX_IMAGE' and n.image and n.image.colorspace_settings.name.lower().startswith('srgb')), None)
        if tex: nt.links.new(tex.outputs['Color'], bsdf.inputs['Base Color'])
    sh = bpy.data.objects.new('HIGH-shield-imported', me); bpy.context.scene.collection.objects.link(sh)
    sh.vertex_groups.clear(); sh['part'] = 'shield'; fitted_objects['shield'] = [sh]
    c = co(sh).mean(0); report['shield_glb'] = {'file': args.shield_glb.name, 'centre_offset_m': round(float(np.linalg.norm(c - rig.shield_frame()['centre'])), 4),
                                               'sha256': hashlib.sha256(args.shield_glb.read_bytes()).hexdigest()}
    log('shield from', args.shield_glb.name, report['shield_glb'])

if args.crest_glb:
    # A finished strand crest built on this rig (painted lane): same head joint,
    # so only the height of this helmet's crown differs. Its own atlas is baked
    # like any source material; the crest is team colour.
    from validate_glb import GLB as _GLB
    cg = _GLB(args.crest_glb); cm = cg.doc['meshes'][0]; cpr = cm['primitives'][0]
    CP = np.array(cg.accessor(cpr['attributes']['POSITION'])); CUV = np.array(cg.accessor(cpr['attributes']['TEXCOORD_0']))
    CI = np.array([i[0] for i in cg.accessor(cpr['indices'])]).reshape(-1, 3)
    keep_r = [r for r in cm['extras']['vertex_regions'] if r['part'] in ('crest', 'crest_base')]
    vmask = np.zeros(len(CP), bool)
    for r in keep_r: vmask[r['start']:r['start'] + r['count']] = True
    tri = CI[vmask[CI].all(1)]; used = np.unique(tri); remap = -np.ones(len(CP), np.int64); remap[used] = np.arange(len(used))
    team_v = np.zeros(len(CP), bool)
    for r in keep_r:
        if r['team']: team_v[r['start']:r['start'] + r['count']] = True
    H = all_co(fitted_objects['helmet']); top = H[np.abs(H[:, 0]) < .03, 1].max() if (np.abs(H[:, 0]) < .03).any() else H[:, 1].max()
    src_helm = cg.accessor(cpr['attributes']['POSITION'])
    helm_r = [r for r in cm['extras']['vertex_regions'] if r['part'] == 'helmet']
    ptop = max(src_helm[i][1] for r in helm_r for i in range(r['start'], r['start'] + r['count']) if abs(src_helm[i][0]) < .03)
    pos = CP[used] + np.array([0, top - ptop - .004, 0])
    if pos[:, 1].max() > args.max_height:
        k = (args.max_height - top) / (pos[:, 1].max() - top)
        pos[:, 1] = top + (pos[:, 1] - top) * k
    img_bytes = None
    tex_i = cg.doc['materials'][0]['pbrMetallicRoughness']['baseColorTexture']['index']
    t = cg.doc['textures'][tex_i]; src_i = t.get('source', t.get('extensions', {}).get('EXT_texture_webp', {}).get('source'))
    im_d = cg.doc['images'][src_i]; bv = cg.doc['bufferViews'][im_d['bufferView']]
    ext = {'image/png': 'png', 'image/jpeg': 'jpg', 'image/webp': 'webp'}[im_d['mimeType']]
    ipath = work / f'crest-source-basecolor.{ext}'; ipath.write_bytes(cg.bin[bv.get('byteOffset', 0):bv.get('byteOffset', 0) + bv['byteLength']])
    def crest_obj(faces, name, team):
        u2 = np.unique(faces); r2 = -np.ones(len(CP), np.int64); r2[u2] = np.arange(len(u2))
        me = bpy.data.meshes.new(name); me.from_pydata((CP[u2] + (pos[remap[u2]] - CP[u2])).tolist(), [], r2[faces].tolist()); me.update()
        uvl = me.uv_layers.new(name='UVMap')
        for poly in me.polygons:
            for li in poly.loop_indices: uvl.data[li].uv = (CUV[u2[me.loops[li].vertex_index]][0], 1 - CUV[u2[me.loops[li].vertex_index]][1])
        m = bpy.data.materials.new(name + ' material'); m.use_nodes = True
        bsdf = m.node_tree.nodes['Principled BSDF']; tx = m.node_tree.nodes.new('ShaderNodeTexImage'); tx.image = bpy.data.images.load(str(ipath))
        m.node_tree.links.new(tx.outputs['Color'], bsdf.inputs['Base Color']); bsdf.inputs['Roughness'].default_value = .85 if team else .5
        bsdf.inputs['Metallic'].default_value = 0 if team else .8
        me.materials.append(m)
        for poly in me.polygons: poly.use_smooth = True
        o = bpy.data.objects.new(name, me); bpy.context.scene.collection.objects.link(o); o['part'] = 'helmet'
        fitted_objects['helmet'].append(o)
    tteam = team_v[tri].all(1)
    crest_obj(tri[tteam], 'Crest horsehair (strand crest)', True)
    if (~tteam).any(): crest_obj(tri[~tteam], 'Crest holder', False)
    team_rules.append(('crest', r'^Crest horsehair'))
    report['crest_from'] = {'file': args.crest_glb.name, 'sha256': hashlib.sha256(args.crest_glb.read_bytes()).hexdigest(),
                            'vertices': int(len(used)), 'top_m': round(float(pos[:, 1].max()), 3)}
    log('crest from', report['crest_from'])

if args.add_crest:
    # A horsehair crest arc, front to back over the helmet's crown, as in the
    # Spearman painting. Generated strands texture; team colour via its material.
    H = all_co(fitted_objects['helmet']); top = H[:, 1].max()
    crown = H[H[:, 1] > top - .11]; zf, zb = crown[:, 2].max(), crown[:, 2].min()
    zc = (zf + zb) / 2; R = (zf - zb) / 2 * 1.3
    def base_y(z):
        near = H[np.abs(H[:, 2] - z) < .02]
        return (near[:, 1].max() if len(near) else top) - .01
    room = args.max_height - top
    hgt = min(.17, room - .035)
    vs, fs, uvs = [], [], []
    n = 40
    for i in range(n + 1):
        t = math.pi * i / n; z = zc + R * math.cos(t)
        yb = base_y(max(min(z, zf), zb)); h = .03 + hgt * math.sin(t) ** .5
        w0, w1 = .022, .042 * (.55 + .45 * math.sin(t))
        # Section: base pair, flared upper pair (fuller at the top, like the painting).
        for x, y in ((-w0, yb), (w0, yb), (w1, yb + h * .7), (0, yb + h), (-w1, yb + h * .7)):
            vs.append((x, y, z + (y - yb) * -.25 * math.cos(t)))
            uvs.append((i / n, (y - yb) / max(h, 1e-3)))
    k = 5
    for i in range(n):
        a, b = i * k, (i + 1) * k
        for j in range(k):
            fs.append((a + j, a + (j + 1) % k, b + (j + 1) % k, b + j))
    fs += [tuple(range(k - 1, -1, -1)), tuple(n * k + j for j in range(k))]
    me = bpy.data.meshes.new('crest'); me.from_pydata(vs, [], fs); me.update()
    uv = me.uv_layers.new(name='UVMap')
    for poly in me.polygons:
        for li in poly.loop_indices: uv.data[li].uv = uvs[me.loops[li].vertex_index]
    S = 512; vv, uu = np.mgrid[0:S, 0:S] / (S - 1)
    rng = np.random.default_rng(5); streak = rng.random(S)[None, :].repeat(S, 0)
    val = (.55 + .45 * streak) * (.45 + .55 * vv)   # darker roots, brighter fluffy tips
    px = np.stack([.55 * val, .045 * val, .035 * val, np.ones_like(val)], -1).astype(np.float32)
    im = bpy.data.images.new('crest strands', S, S); im.pixels.foreach_set(px.ravel()); im.pack()
    m = bpy.data.materials.new('Crest horsehair (generated)'); m.use_nodes = True
    bsdf = m.node_tree.nodes['Principled BSDF']; tex = m.node_tree.nodes.new('ShaderNodeTexImage'); tex.image = im
    m.node_tree.links.new(tex.outputs['Color'], bsdf.inputs['Base Color']); bsdf.inputs['Roughness'].default_value = .85
    me.materials.append(m)
    for poly in me.polygons: poly.use_smooth = True
    cr = bpy.data.objects.new('HIGH-helmet-crest', me); bpy.context.scene.collection.objects.link(cr); cr['part'] = 'helmet'
    fitted_objects['helmet'].append(cr); team_rules.append(('crest', r'^Crest horsehair'))
    report['crest_added'] = {'height_m': round(hgt + .03, 3), 'length_m': round(2 * R, 3)}
    log('crest added', report['crest_added'])
# A bulkier source fist or forearm can poke through the seated shield's dish.
# The hand always sits behind the shield in the clips (it holds the grip), so
# press any body vertex that crosses the shield's back face back behind it.
if 'shield' in fitted_objects:
    fr = rig.shield_frame(); n_ = fr['normal']; c_ = fr['centre']
    Sh = all_co(fitted_objects['shield']) - c_
    sh_h = Sh @ n_; sh_r = np.linalg.norm(Sh - np.outer(sh_h, n_), axis=1); R_ = sh_r.max()
    bins = np.linspace(0, R_, 17); back_h = np.full(16, np.inf)
    for i in range(16):
        sel = (sh_r >= bins[i]) & (sh_r < bins[i + 1])
        if sel.any(): back_h[i] = sh_h[sel].min()
    # First slide the shield out (up to 4 cm, rigid on shield_arm, so it holds in
    # every clip); pressing many vertices flat leaves a jagged forearm.
    Bd = all_co([o for p_ in ('body', 'layer') for o in fitted_objects.get(p_, [])]) - c_
    bh = Bd @ n_; br = np.linalg.norm(Bd - np.outer(bh, n_), axis=1)
    lim0 = back_h[np.minimum((br / R_ * 16).astype(int), 15)] - .005
    inb = (br < .97 * R_) & (bh > lim0) & (bh < lim0 + .12)
    slide = float(min(.04, (bh[inb] - lim0[inb]).max())) if inb.any() else 0.
    if slide > 0:
        for o in fitted_objects['shield']: set_co(o, co(o) + n_ * slide)
        back_h += slide
    report['shield_slide_m'] = round(slide, 4)
    moved = 0
    for o in [o for p_ in ('body', 'layer') for o in fitted_objects.get(p_, [])]:
        P_ = co(o); q = P_ - c_; h = q @ n_; r = np.linalg.norm(q - np.outer(h, n_), axis=1)
        lim = back_h[np.minimum((r / R_ * 16).astype(int), 15)] - .005
        bad = (r < .97 * R_) & (h > lim) & (h < lim + .12)
        if bad.any():
            P_[bad] -= np.outer(h[bad] - lim[bad], n_); set_co(o, P_); moved += int(bad.sum())
    report['pressed_behind_shield'] = moved
log('fitted', {k: sum(len(o.data.polygons) for o in v) for k, v in fitted_objects.items()})
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=str(work / 'fitted-high.blend'))
# Check scene: the fitted figure with the 0 A.D. joints it was fitted to.
markers = []
for name, pos in rig.joint.items():
    if name.startswith('cape'): continue
    bpy.ops.mesh.primitive_ico_sphere_add(radius=.02, location=tuple(pos)); m = bpy.context.object
    m.name = 'joint-' + name; markers.append(m)
    mat = bpy.data.materials.get('marker') or bpy.data.materials.new('marker'); mat.diffuse_color = (0, .9, .2, 1)
    m.data.materials.append(mat)
bpy.ops.wm.save_as_mainfile(filepath=str(work / 'fitted-check.blend'))
for m in markers: bpy.data.objects.remove(m, do_unlink=True)


# ------------------------------------------------------------------------- LODs
def active(o):
    bpy.ops.object.select_all(action='DESELECT'); o.select_set(True); bpy.context.view_layer.objects.active = o


def apply_mod(o, kind, **kw):
    active(o); m = o.modifiers.new(kind, kind)
    for k, v in kw.items(): setattr(m, k, v)
    bpy.ops.object.modifier_apply(modifier=m.name)


def ntris(o): return sum(len(p.vertices) - 2 for p in o.data.polygons)


def joined_copy(items, name):
    copies = []
    for o in items:
        x = o.copy(); x.data = o.data.copy(); bpy.context.scene.collection.objects.link(x); copies.append(x)
    bpy.ops.object.select_all(action='DESELECT')
    for x in copies: x.select_set(True)
    bpy.context.view_layer.objects.active = copies[0]
    if len(copies) > 1: bpy.ops.object.join()
    x = bpy.context.view_layer.objects.active; x.name = name; return x


def budgets(total, parts, keep=()):
    raw = {p: sum(ntris(o) for o in fitted_objects[p]) for p in parts}
    kept = {p: raw[p] for p in parts if p in keep}
    total -= sum(kept.values()); parts = [p for p in parts if p not in keep]
    floor = {'spear': 90, 'shield': 300, 'helmet': 250, 'cloak': 200, 'body': 1000, 'layer': 800}
    weight = {p: raw[p] ** .75 * {'body': 1.6}.get(p, 1) for p in parts}
    share = {p: max(floor.get(p, 100), total * weight[p] / sum(weight.values())) for p in parts}
    scale = total / sum(share.values())
    return dict({p: int(share[p] * scale) for p in parts}, **kept), raw


parts = [p for p in PART_ORDER if p in fitted_objects]
b0, raw = budgets(args.lod0, parts, set(args.keep))
lod0 = {}
for p in parts:
    o = joined_copy(fitted_objects[p], f'LOD0-{p}'); o['part'] = p
    # Remember each face's source material: --team rules select faces by it.
    o['mat_names'] = [m.name if m else '' for m in o.data.materials]
    sm = o.data.attributes.new('src_mat', 'INT', 'FACE')
    mi = np.empty(len(o.data.polygons), np.int64); o.data.polygons.foreach_get('material_index', mi); sm.data.foreach_set('value', mi)
    apply_mod(o, 'TRIANGULATE')
    if ntris(o) > b0[p]: apply_mod(o, 'DECIMATE', ratio=b0[p] / ntris(o), use_collapse_triangulate=True)
    lod0[p] = o
# One shared atlas over every LOD0 part.
bpy.ops.object.select_all(action='DESELECT')
for o in lod0.values(): o.select_set(True)
bpy.context.view_layer.objects.active = lod0['body']
# Reuse the artist's UV charts where every face has them (few, large islands and
# seams where the artist put them), repacked at even texel density; otherwise
# fall back to smart projection. Smart projection of a decimated sculpt makes
# thousands of tiny islands: blurry, seamy and vertex-heavy.
keep_src_uv = all(len(o.data.uv_layers) for o in lod0.values()) and not args.smart_uv
for o in lod0.values():
    src_uv = o.data.uv_layers[0] if o.data.uv_layers else None
    atlas = o.data.uv_layers.new(name='atlas')
    if keep_src_uv:
        a = np.empty(len(o.data.loops) * 2); src_uv.data.foreach_get('uv', a); atlas.data.foreach_set('uv', a)
    while o.data.uv_layers[0].name != 'atlas': o.data.uv_layers.remove(o.data.uv_layers[0])
    o.data.uv_layers.active = o.data.uv_layers['atlas']
bpy.ops.object.mode_set(mode='EDIT'); bpy.ops.mesh.select_all(action='SELECT')
if not keep_src_uv: bpy.ops.uv.smart_project(angle_limit=math.radians(60), island_margin=.003, area_weight=1.0)
bpy.ops.uv.average_islands_scale()
bpy.ops.uv.pack_islands(margin=.003, rotate=True, shape_method='AABB', merge_overlap=False)
bpy.ops.object.mode_set(mode='OBJECT')
report['uv'] = 'source charts repacked' if keep_src_uv else 'smart projection'
# LOD1 is what players see at normal zoom: garments must survive. Skin hidden
# under clothing or armour goes first (it only pokes through a coarse garment),
# then the budget favours garments over skin.
lod1_weight = {'layer': 1.8, 'cloak': 1.0, 'body': 1.0, 'helmet': .6, 'shield': .35, 'spear': .12}
wsum_ = sum(lod1_weight.get(p, .5) for p in parts)
b1 = {p: int(args.lod1 * lod1_weight.get(p, .5) / wsum_) for p in parts}
cover = [lod0[p] for p in ('layer', 'cloak') if p in lod0]
lod1 = {}
for p in parts:
    o = joined_copy([lod0[p]], f'LOD1-{p}'); o['part'] = p
    if p == 'body' and cover:
        Lp = np.concatenate([co(c) for c in cover]); me = o.data; n = len(me.vertices)
        P = co(o); Nn = np.empty(n * 3); me.vertices.foreach_get('normal', Nn); Nn = Nn.reshape(-1, 3)
        hidden = np.zeros(n, bool)
        for k in range(0, n, 512):
            d2 = ((P[k:k + 512, None] - Lp[None]) ** 2).sum(2); j = d2.argmin(1)
            d = np.sqrt(d2[np.arange(len(j)), j]); up = ((Lp[j] - P[k:k + 512]) * Nn[k:k + 512]).sum(1)
            hidden[k:k + 512] = (d < .03) & (up > -.004)
        bm = bmesh.new(); bm.from_mesh(me); bm.verts.ensure_lookup_table()
        dead = [f for f in bm.faces if all(hidden[v.index] for v in f.verts)]
        bmesh.ops.delete(bm, geom=dead, context='FACES'); bm.to_mesh(me); bm.free()
        report['lod1_hidden_skin_faces_removed'] = len(dead)
    if ntris(o) > b1[p]: apply_mod(o, 'DECIMATE', ratio=b1[p] / ntris(o), use_collapse_triangulate=True)
    lod1[p] = o


def enforce(lod, total, keep=()):
    # Many small loose pieces resist collapse decimation; take the excess from
    # the parts with the most triangles until the LOD fits its budget.
    for _ in range(6):
        have = sum(ntris(o) for o in lod.values())
        if have <= total: return
        big = sorted([o for p, o in lod.items() if p not in keep], key=ntris, reverse=True)[:3]
        for o in big:
            apply_mod(o, 'DECIMATE', ratio=max(.3, 1 - (have - total) * 1.1 / max(1, sum(map(ntris, big)))), use_collapse_triangulate=True)


enforce(lod0, args.lod0, set(args.keep)); enforce(lod1, args.lod1, {'layer', 'cloak'})
report['triangles'] = {'source': raw, 'lod0': {p: ntris(o) for p, o in lod0.items()}, 'lod1': {p: ntris(o) for p, o in lod1.items()}}
log('lods', report['triangles'])

# ------------------------------------------------------------------- skin weights
def donor(name, faces):
    used = np.unique(faces); remap = -np.ones(len(rig.P), np.int64); remap[used] = np.arange(len(used))
    me = bpy.data.meshes.new(name); me.from_pydata(rig.P[used].tolist(), [], remap[faces].tolist()); me.update()
    o = bpy.data.objects.new(name, me); bpy.context.scene.collection.objects.link(o)
    groups = [o.vertex_groups.new(name=b) for b in rig.bones]
    for i, v in enumerate(used):
        for j, w in zip(rig.J[v], rig.W[v]):
            if w > 0: groups[j].add([i], float(w), 'ADD')
    return o


def group_faces(bones):
    k = [rig.bones.index(b) for b in bones]
    share = (rig.W * np.isin(rig.J, k)).sum(1)
    return rig.body_faces[share[rig.body_faces].mean(1) > .3]


donors = {chain: donor('donor_' + chain, group_faces(GROUPS[chain])) for chain in order}
donors['cloak'] = donor('donor_cloak', rig.cloak_faces) if len(rig.cloak_faces) else None


def transfer(o, src, mask=None):
    m = o.modifiers.new('xfer', 'DATA_TRANSFER'); m.object = src
    m.use_vert_data = True; m.data_types_verts = {'VGROUP_WEIGHTS'}; m.vert_mapping = 'POLYINTERP_NEAREST'
    m.layers_vgroup_select_src = 'ALL'; m.layers_vgroup_select_dst = 'NAME'; m.mix_mode = 'ADD'
    if mask: m.vertex_group = mask
    active(o); bpy.ops.object.modifier_apply(modifier=m.name)


for lod in (lod0, lod1):
    for p, o in lod.items():
        for b in rig.bones:
            if b not in o.vertex_groups: o.vertex_groups.new(name=b)
        if p in RIGID or p in ('spear', 'shield', 'helmet'):
            bone = {'spear': 'weapon_R', 'shield': 'shield_arm', 'helmet': 'helmet'}[p]
            o.vertex_groups[bone].add(list(range(len(o.data.vertices))), 1., 'REPLACE')
        elif p == 'cloak' and donors['cloak']:
            transfer(o, donors['cloak'])
        elif p == 'layer' and 'body' in lod:
            # Clothing takes the weights of the skin under it (already transferred
            # from 0 A.D.), so cloth and skin deform together and do not cross.
            transfer(o, lod['body'])
        else:
            for chain in order: transfer(o, donors[chain], 'grp_' + chain)
        for g in list(o.vertex_groups):
            if g.name.startswith('grp_'): o.vertex_groups.remove(g)
        active(o)
        bpy.ops.object.vertex_group_limit_total(limit=4)
        bpy.ops.object.vertex_group_normalize_all(lock_active=False)
for o in list(bpy.data.objects):
    if o.name.startswith('donor_'): bpy.data.objects.remove(o, do_unlink=True)
log('weights transferred')

# -------------------------------------------------------------------------- bake
scene = bpy.context.scene; scene.render.engine = 'CYCLES'; scene.cycles.device = 'CPU'
scene.cycles.samples = 1; scene.render.threads_mode = 'FIXED'; scene.render.threads = 3
bake = scene.render.bake; bake.use_selected_to_active = True; bake.margin = 16; bake.target = 'IMAGE_TEXTURES'
diag = float(np.ptp(np.concatenate([co(o) for o in lod0.values()]), 0).max())
bake.cage_extrusion = .006 * diag; bake.max_ray_distance = .02 * diag
N = args.bake_size


def img(name, data):
    im = bpy.data.images.new('fit_' + name, N, N, alpha=False, float_buffer=False)  # prefix: sources have 'normal' etc.
    if data: im.colorspace_settings.name = 'Non-Color'
    return im


maps_img = {'basecolor': img('basecolor', False), 'normal': img('normal', True), 'ao': img('ao', True),
            'roughness': img('roughness', True), 'metallic': img('metallic', True)}
receiver = bpy.data.materials.new('bake receiver'); receiver.use_nodes = True
node = receiver.node_tree.nodes.new('ShaderNodeTexImage'); receiver.node_tree.nodes.active = node
for o in lod0.values():
    o['source_materials'] = [m.name for m in o.data.materials if m]
    o.data.materials.clear(); o.data.materials.append(receiver)
    for poly in o.data.polygons: poly.material_index = 0


def run_bake(kind, im, **kw):
    node.image = im; first = True
    for p in parts:
        bpy.ops.object.select_all(action='DESELECT')
        for h in fitted_objects[p]: h.select_set(True)
        lod0[p].select_set(True); bpy.context.view_layer.objects.active = lod0[p]
        bake.use_clear = first; first = False
        bpy.ops.object.bake(type=kind, use_clear=bake.use_clear, **kw)
    im.filepath_raw = str(work / f'{im.name}-master.png'); im.file_format = 'PNG'; im.save()
    log('baked', kind, im.name)


run_bake('NORMAL', maps_img['normal'], normal_space='TANGENT')
# Thin double-sided shells (cloaks, shield dishes) can return the opposite
# surface. Keep each texel in the receiving hemisphere and limit relief to a
# 30-degree cone: form shading belongs to the mesh normals (painted-lane fix).
pix = np.array(maps_img['normal'].pixels[:], dtype=np.float32).reshape(-1, 4)
vec = pix[:, :3] * 2 - 1; vec[vec[:, 2] < 0] *= -1; vec[:, 2] = np.maximum(vec[:, 2], .001)
lim = vec[:, 2] * math.tan(math.radians(30)); ln = np.linalg.norm(vec[:, :2], axis=1)
vec[:, :2] *= np.minimum(1, lim / np.maximum(ln, 1e-8))[:, None]; vec /= np.linalg.norm(vec, axis=1)[:, None]
pix[:, :3] = vec * .5 + .5; maps_img['normal'].pixels.foreach_set(pix.ravel()); maps_img['normal'].save()
# Ambient occlusion of each low part on its own (3 cm reach): selected-to-active AO from the high shells gave black blotches.
scene.world = scene.world or bpy.data.worlds.new('fit'); scene.world.light_settings.distance = .03
scene.cycles.samples = args.ao_samples; bake.use_selected_to_active = False; node.image = maps_img['ao']
hidden = {o: o.hide_render for o in bpy.context.scene.objects}
for i, p in enumerate(parts):
    # Each part on its own: skin under a skirt or greaves otherwise bakes as
    # blotchy dark patches that show wherever the clips move the cloth away.
    for o in bpy.context.scene.objects: o.hide_render = o is not lod0[p]
    active(lod0[p]); bpy.ops.object.bake(type='AO', use_clear=i == 0)
for o, h in hidden.items(): o.hide_render = h
ao_px = np.array(maps_img['ao'].pixels[:], dtype=np.float32).reshape(-1, 4)
ao_px[:, :3] = .45 + .55 * ao_px[:, :3]  # never fully black: the runtime adds its own shadowing
maps_img['ao'].pixels.foreach_set(ao_px.ravel())
maps_img['ao'].filepath_raw = str(work / 'ao-master.png'); maps_img['ao'].file_format = 'PNG'; maps_img['ao'].save()
bake.use_selected_to_active = True; scene.cycles.samples = 1; log('baked AO (low mesh, 3 cm)')


def emission_from(socket_name):
    """Route each source material's <socket> into an emission shader."""
    for m in {m for ps in fitted_objects.values() for o in ps for m in o.data.materials if m}:
        if not m.use_nodes: m.use_nodes = True
        nt = m.node_tree; bsdf = next((n for n in nt.nodes if n.type == 'BSDF_PRINCIPLED'), None)
        outn = next((n for n in nt.nodes if n.type == 'OUTPUT_MATERIAL' and n.is_active_output), None) or \
            next((n for n in nt.nodes if n.type == 'OUTPUT_MATERIAL'), None)
        if outn is None: outn = nt.nodes.new('ShaderNodeOutputMaterial')
        em = nt.nodes.get('FIT emission') or nt.nodes.new('ShaderNodeEmission'); em.name = 'FIT emission'
        for l in list(em.inputs['Color'].links): nt.links.remove(l)
        fallback = {'Base Color': (.5, .5, .5, 1), 'Roughness': .6, 'Metallic': 0.}[socket_name]
        if bsdf is None:
            v = fallback
        else:
            sock = bsdf.inputs[socket_name]
            if sock.links: nt.links.new(sock.links[0].from_socket, em.inputs['Color']); v = None
            else: v = sock.default_value
        if v is not None:
            em.inputs['Color'].default_value = tuple(v)[:4] if hasattr(v, '__len__') else (v, v, v, 1)
        nt.links.new(em.outputs['Emission'], outn.inputs['Surface'])


for chan, name in (('Base Color', 'basecolor'), ('Roughness', 'roughness'), ('Metallic', 'metallic')):
    emission_from(chan); run_bake('EMIT', maps_img[name])


def px(im): return np.array(im.pixels[:], dtype=np.float32).reshape(N, N, 4)


def raster_mask(tris):
    """Texels covered by UV triangles (each a list of three (u, v))."""
    mask = np.zeros((N, N), bool)
    for tri in tris:
        t = np.array(tri) * N; lo = np.clip(np.floor(t.min(0)).astype(int), 0, N - 1); hi = np.clip(np.ceil(t.max(0)).astype(int) + 1, 0, N)
        if (hi - lo).min() <= 0: continue
        yy, xx = np.mgrid[lo[1]:hi[1], lo[0]:hi[0]] + .5
        a, b, c = t; d = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1])
        if abs(d) < 1e-12: continue
        w1 = ((b[1] - c[1]) * (xx - c[0]) + (c[0] - b[0]) * (yy - c[1])) / d
        w2 = ((c[1] - a[1]) * (xx - c[0]) + (a[0] - c[0]) * (yy - c[1])) / d
        mask[lo[1]:hi[1], lo[0]:hi[0]] |= (w1 >= -.02) & (w2 >= -.02) & (1 - w1 - w2 >= -.02)
    return mask


def uv_tris(o):
    me = o.data; me.calc_loop_triangles(); uvl = me.uv_layers['atlas'].data
    return [[tuple(uvl[li].uv) for li in t.loops] for t in me.loop_triangles]


def srgb(hexstr): return np.array([int(hexstr[i:i + 2], 16) / 255 for i in (0, 2, 4)], np.float32)


if args.grade_bronze:
    # Painted bronze (spearman-panel.png): warm gold-brown, not glossy copper.
    # Value comes from the source's own luminance, darkened in cavities (AO) and
    # lifted on convex edges (Cycles pointiness of the low mesh: worn, polished).
    pm = bpy.data.materials.new('fit pointiness'); pm.use_nodes = True; nt = pm.node_tree
    for n_ in list(nt.nodes): nt.nodes.remove(n_)
    geo = nt.nodes.new('ShaderNodeNewGeometry'); em = nt.nodes.new('ShaderNodeEmission'); out_ = nt.nodes.new('ShaderNodeOutputMaterial')
    nt.links.new(geo.outputs['Pointiness'], em.inputs['Color']); nt.links.new(em.outputs[0], out_.inputs['Surface'])
    ptex = nt.nodes.new('ShaderNodeTexImage'); pimg = img('pointiness', True); ptex.image = pimg; nt.nodes.active = ptex
    bake.use_selected_to_active = False
    for i, p in enumerate(parts):
        o = lod0[p]; o.data.materials[0] = pm; active(o); bpy.ops.object.bake(type='EMIT', use_clear=i == 0)
        o.data.materials[0] = receiver
    bake.use_selected_to_active = True
    B_ = px(maps_img['basecolor']); M_ = px(maps_img['metallic'])[..., 0]; A_ = px(maps_img['ao'])[..., 0]
    R_ = px(maps_img['roughness']); P_ = px(pimg)[..., 0]
    metal = M_ > .5
    if metal.any():
        lum = B_[..., :3] @ np.array([.2126, .7152, .0722], np.float32); l = lum / max(float(lum[metal].mean()), 1e-3)
        if 'shield' in lod0:
            # Aspis: the device only on the outer face; the inner face plain bronze;
            # the field lifted from dark brown to mid bronze.
            nrm = rig.shield_frame()['normal']; me_ = lod0['shield'].data; me_.calc_loop_triangles(); uvl_ = me_.uv_layers['atlas'].data
            inner, outer = [], []
            for t in me_.loop_triangles:
                (inner if np.dot(np.array(me_.polygons[t.polygon_index].normal), nrm) < -.2 else outer).append([tuple(uvl_[li].uv) for li in t.loops])
            im_, om_ = raster_mask(inner), raster_mask(outer)
            l = np.where(om_, .5 + .6 * l, l); l = np.where(im_ & ~om_, .95, l)
            report['aspis_faces'] = {'inner_tris': len(inner), 'outer_tris': len(outer)}
        # Edge wear from curvature: convex pointiness, widened by a small blur so
        # raised rims and ridges read as polished bands, not single texels.
        pe = np.clip((P_ - .505) * 14, 0, 1)
        for _ in range(2): pe = np.maximum(pe, (np.roll(pe, 1, 0) + np.roll(pe, -1, 0) + np.roll(pe, 1, 1) + np.roll(pe, -1, 1)) / 4 * .9)
        edge = pe ** .8; cav = np.clip((A_ - .45) / .55, 0, 1)
        t_ = np.clip(.08 + .45 * l * (.55 + .45 * cav) + .55 * edge, 0, 1)
        dark, mid, light = srgb('47290f'), srgb('a67734'), srgb('f6dc96')
        col = np.where(t_[..., None] < .5, dark + (mid - dark) * (t_[..., None] * 2), mid + (light - mid) * ((t_[..., None] - .5) * 2))
        B_[metal, :3] = col[metal]
        R_[metal, :3] = np.clip(.50 - .34 * edge[metal] + .16 * (1 - cav[metal]), .14, .8)[:, None]
        maps_img['basecolor'].pixels.foreach_set(B_.ravel()); maps_img['roughness'].pixels.foreach_set(R_.ravel())
        report['bronze_grade'] = {'texels': int(metal.sum()), 'palette': ['47290f', 'a67734', 'f6dc96']}
        log('bronze graded', report['bronze_grade'])

if args.grade_skin and 'body' in lod0:
    # Warmer, tanned skin: match the skin texels' mean to the target colour,
    # keeping 90% of their own variation (pores, shading-free detail).
    B_ = px(maps_img['basecolor']); M_ = px(maps_img['metallic'])[..., 0]
    skin = raster_mask(uv_tris(lod0['body'])) & (M_ < .3)
    if skin.any():
        mean = B_[skin, :3].mean(0); tgt = srgb(args.grade_skin)
        # Hue locked to the target (no grey cast), value from the source's own
        # luminance, plus a little of its original colour variation.
        lum = B_[skin, :3] @ np.array([.2126, .7152, .0722], np.float32); k_ = np.clip(lum / max(float(lum.mean()), 1e-3), .6, 1.4)
        B_[skin, :3] = np.clip(.85 * tgt * k_[:, None] + .15 * (tgt + (B_[skin, :3] - mean)), 0, 1)
        maps_img['basecolor'].pixels.foreach_set(B_.ravel())
        report['skin_grade'] = {'texels': int(skin.sum()), 'from_srgb': (mean * 255).round().tolist(), 'to': args.grade_skin}
        log('skin graded', report['skin_grade'])

orm = bpy.data.images.new('orm', N, N, alpha=False); orm.colorspace_settings.name = 'Non-Color'
o_px = np.ones((N, N, 4), np.float32)
o_px[..., 0] = px(maps_img['ao'])[..., 0]; o_px[..., 1] = px(maps_img['roughness'])[..., 0]; o_px[..., 2] = px(maps_img['metallic'])[..., 0]
orm.pixels.foreach_set(o_px.ravel()); orm.filepath_raw = str(work / 'orm-master.png'); orm.file_format = 'PNG'; orm.save()
base_px = px(maps_img['basecolor'])

# --------------------------------------------------------------------- team mask


def hsv(rgb):
    mx = rgb.max(-1); mn = rgb.min(-1); d = mx - mn + 1e-9
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    h = np.where(mx == r, ((g - b) / d) % 6, np.where(mx == g, (b - r) / d + 2, (r - g) / d + 4)) * 60
    return h, np.where(mx > 0, d / (mx + 1e-9), 0), mx


shield_frame = rig.shield_frame()


def team_faces(o, p):
    """Per-polygon team region name (or '') for one LOD part."""
    me = o.data; n = len(me.polygons); region = np.array([''] * n, dtype=object)
    uv = np.empty(len(me.loops) * 2); me.uv_layers.active.data.foreach_get('uv', uv); uv = uv.reshape(-1, 2)
    li = np.empty(n, np.int64); me.polygons.foreach_get('loop_start', li)
    lt = np.empty(n, np.int64); me.polygons.foreach_get('loop_total', lt)
    cuv = np.array([uv[a:a + b].mean(0) for a, b in zip(li, lt)])
    col = base_px[np.clip((cuv[:, 1] * N).astype(int), 0, N - 1), np.clip((cuv[:, 0] * N).astype(int), 0, N - 1), :3]
    centre = np.empty(n * 3); me.polygons.foreach_get('center', centre); centre = centre.reshape(-1, 3)
    names_ = list(o.get('mat_names', []))
    smi = np.zeros(n, np.int64)
    if 'src_mat' in me.attributes: me.attributes['src_mat'].data.foreach_get('value', smi)
    face_mat = [names_[i] if i < len(names_) else '' for i in smi]
    if not args.no_auto_team:
        if p == 'helmet':
            h, s_, v = hsv(col ** (1 / 2.2))
            red = ((h < 22) | (h > 335)) & (s_ > .45) & (v > .12)
            region[red] = 'crest'
        if p == 'shield':
            q = centre - shield_frame['centre']; hgt = q @ shield_frame['normal']
            r = np.linalg.norm(q - np.outer(hgt, shield_frame['normal']), axis=1)
            region[r > .86 * r.max()] = 'shield rim'
        if p == 'cloak':
            y = centre[:, 1]; x = np.abs(centre[:, 0])
            region[(y < y.min() + .12 * np.ptp(y)) | (x > .88 * x.max())] = 'cloak trim'
    for name, rx in team_rules:
        if re.search(rx, o.name, re.I): region[:] = name; continue
        hit = np.array([bool(re.search(rx, m, re.I)) for m in face_mat], bool)
        region[hit] = name
    return region


# ------------------------------------------------------------------------- export
names = {'body': 'body', 'layer': 'garment', 'cloak': 'cloak', 'helmet': 'helmet', 'shield': 'aspis', 'spear': 'spear'}
team_part = {'crest': 'crest', 'shield rim': 'aspis', 'cloak trim': 'cloak'}
planes = clip_planes(rig.g)
log('clip floor planes', len(planes))


def export_lod(lod, mesh_name):
    attrs = {k: [] for k in ['POSITION', 'NORMAL', 'TANGENT', 'TEXCOORD_0', 'JOINTS_0', 'WEIGHTS_0', 'COLOR_0']}
    idx = []; regions = []
    for p in parts:
        o = lod[p]; region = team_faces(o, p)
        tattr = o.data.attributes.new('fit_team', 'INT', 'FACE')
        kinds = sorted(set(region.tolist()))
        for i, r in enumerate(region): tattr.data[i].value = kinds.index(r)
        apply_mod(o, 'TRIANGULATE'); me = o.data
        me.calc_tangents(uvmap=me.uv_layers.active.name)
        team_of = [kinds[a.value] for a in me.attributes['fit_team'].data]
        if mesh_name.endswith('LOD0'):
            uvl = me.uv_layers.active.data
            for poly in me.polygons:
                if team_of[poly.index]: team_uv.setdefault(team_of[poly.index], []).append([tuple(uvl[li].uv) for li in poly.loop_indices])
        gname = {g.index: g.name for g in o.vertex_groups}
        bone = {'spear': 'weapon_R', 'shield': 'shield_arm', 'helmet': 'helmet'}.get(p)
        for kind in kinds:
            start = len(attrs['POSITION']); mapping = {}
            for poly in me.polygons:
                if team_of[poly.index] != kind: continue
                for li in poly.loop_indices:
                    loop = me.loops[li]; v = me.vertices[loop.vertex_index]
                    uv = me.uv_layers.active.data[li].uv; n = Vector(me.corner_normals[li].vector).normalized()
                    if n.length < .5: n = poly.normal.normalized()
                    t = loop.tangent - n * loop.tangent.dot(n)
                    t = t.normalized() if t.length > 1e-6 else n.orthogonal().normalized()
                    ww = sorted([(rig.bones.index(gname[g.group]), g.weight) for g in v.groups
                                 if g.weight > 1e-6 and gname[g.group] in rig.bones], key=lambda x: -x[1])[:4]
                    if not ww: ww = [(rig.bones.index('hip'), 1.)]
                    tot = sum(w for _, w in ww); js = [j for j, _ in ww] + [0] * (4 - len(ww))
                    ws = [w / tot for _, w in ww] + [0] * (4 - len(ww))
                    key = (v.index, tuple(round(x, 5) for x in n), tuple(round(x, 6) for x in uv))
                    if key not in mapping:
                        mapping[key] = len(attrs['POSITION'])
                        attrs['POSITION'].append(tuple(v.co)); attrs['NORMAL'].append(tuple(n))
                        attrs['TANGENT'].append((*t, -loop.bitangent_sign if loop.bitangent_sign else 1.))
                        attrs['TEXCOORD_0'].append((uv.x, 1 - uv.y)); attrs['JOINTS_0'].append(js)
                        attrs['WEIGHTS_0'].append(ws); attrs['COLOR_0'].append((1. if kind else 0., 0., 0., 1.))
                    idx.append((mapping[key],))
            count = len(attrs['POSITION']) - start
            if count:
                part = team_part[kind] if kind else names[p]
                regions.append({'part': part, 'start': start, 'count': count, 'bone': bone, 'team': kind or None})
    before = np.array(attrs['POSITION'])
    xyz, rep = ground_fit(before, np.array(attrs['JOINTS_0']), np.array(attrs['WEIGHTS_0']), planes)
    moved = np.linalg.norm(xyz - before, axis=1)
    rep['max_by_part_m'] = {r['part']: round(float(moved[r['start']:r['start'] + r['count']].max()), 4) for r in regions}
    attrs['POSITION'] = [tuple(map(float, r)) for r in xyz]
    log(mesh_name, len(idx) // 3, 'triangles; ground fit', rep)
    return {'name': mesh_name, 'attrs': attrs, 'indices': idx, 'regions': regions}, rep


meshes = []; report['ground_fit'] = {}; team_uv = {}
for lod, name in ((lod0, 'spearman_LOD0'), (lod1, 'spearman_LOD1')):
    m, rep = export_lod(lod, name); meshes.append(m); report['ground_fit'][name] = rep
report['team_regions'] = {m['name']: sorted({r['team'] for r in m['regions'] if r['team']}) for m in meshes}
# Team areas carry neutral value only (the runtime multiplies in the team colour,
# README round 6): light for crest and trim, mid grey for the rim so its dark
# border survives every colour. Hue painted there would muddy the tint.
bp = np.array(maps_img['basecolor'].pixels[:], dtype=np.float32).reshape(N, N, 4)
target = {'crest': .78, 'cloak trim': .82, 'shield rim': .72}
for kind, tris_ in team_uv.items():
    mask = np.zeros((N, N), bool)
    for tri in tris_:
        t = np.array(tri) * N; lo = np.floor(t.min(0)).astype(int); hi = np.ceil(t.max(0)).astype(int) + 1
        lo = np.clip(lo, 0, N - 1); hi = np.clip(hi, 0, N)
        if (hi - lo).min() <= 0: continue
        yy, xx = np.mgrid[lo[1]:hi[1], lo[0]:hi[0]] + .5
        a, b, c = t; d = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1])
        if abs(d) < 1e-12: continue
        w1 = ((b[1] - c[1]) * (xx - c[0]) + (c[0] - b[0]) * (yy - c[1])) / d
        w2 = ((c[1] - a[1]) * (xx - c[0]) + (a[0] - c[0]) * (yy - c[1])) / d
        inside = (w1 >= -.02) & (w2 >= -.02) & (1 - w1 - w2 >= -.02)
        mask[lo[1]:hi[1], lo[0]:hi[0]] |= inside
    if not mask.any(): continue
    lum = bp[..., :3] @ np.array([.2126, .7152, .0722], np.float32)
    grey = np.clip(lum / max(float(lum[mask].mean()), 1e-3) * target.get(kind, .75), .15, 1)
    bp[mask, :3] = grey[mask, None]
maps_img['basecolor'].pixels.foreach_set(bp.ravel()); maps_img['basecolor'].update()
report['team_neutralised_texels'] = {k: len(v) for k, v in team_uv.items()}
images = []
for name, im in (('basecolor', maps_img['basecolor']), ('normal', maps_img['normal']), ('orm', orm)):
    # From the in-memory pixels: Image.copy() reloads the saved master and would
    # drop the grades and the neutral team texels applied after the bake.
    small = bpy.data.images.new(f'ship {name}', N, N, alpha=False); small.colorspace_settings.name = im.colorspace_settings.name
    small.pixels.foreach_set(np.array(im.pixels[:], dtype=np.float32)); small.scale(args.ship_size, args.ship_size)
    im.filepath_raw = str(work / f'{name}-master-final.png'); im.file_format = 'PNG'; im.save()
    ext, fmt, mime = {'jpeg': ('jpg', 'JPEG', 'image/jpeg'), 'webp': ('webp', 'WEBP', 'image/webp')}[args.texture_format]
    path = work / f'{name}-{args.ship_size}.{ext}'
    small.filepath_raw = str(path); small.file_format = fmt
    small.save(filepath=str(path), quality=args.jpeg_quality)
    images.append((name, mime, path.read_bytes()))
credit = f'{args.credit}; animation and rig: {OAD_CREDIT}'
pack(rig.g, meshes, images, out, credit, args.license,
     extras={'texture_master_size': args.bake_size, 'texture_delivery': {'encoding': args.texture_format.upper(), 'quality': args.jpeg_quality, 'pixels': args.ship_size},
             'source_model': {'file': args.source.name, 'sha256': report['source_sha256'], 'credit': args.credit},
             'fit': 'tools/art/anim3d/kit/fit_source_to_0ad.py'},
     material_name='Fitted ' + args.source.stem)
for name in ('LICENSE-ART',):
    if not (out.parent / name).exists() and (args.rig.parent / name).exists(): shutil.copy(args.rig.parent / name, out.parent / name)
prov = out.parent / 'PROVENANCE.md'
if not prov.exists():
    prov.write_text(f"""# {out.name}: provenance

Generated by `tools/art/anim3d/kit/fit_source_to_0ad.py`; replace this stub with a reviewed record before shipping.

- Model: {args.credit}. Source file `{args.source.name}`, SHA-256 `{report['source_sha256']}`.
- Rig and the five clips (idle, walk, attack, hit, death): {OAD_CREDIT}. Copied byte for byte from
  `{args.rig.name}` (SHA-256 `{hashlib.sha256(args.rig.read_bytes()).hexdigest()}`).
- Changes: unposed to the 0 A.D. bind pose, proportions fitted to the 0 A.D. bone lengths, re-skinned with
  0 A.D. weights, props re-seated, decimated to two LODs, materials baked to one atlas, team mask added.
- Combined asset licence: {args.license}.
""")
report.update(glb=str(out), glb_bytes=out.stat().st_size, seconds=round(time.time() - t0, 1))
bpy.ops.wm.save_as_mainfile(filepath=str(work / 'fitted-lods.blend'))
(work / 'fit-report.json').write_text(json.dumps(report, indent=2, default=float) + '\n')
log('wrote', out, out.stat().st_size, 'bytes')
