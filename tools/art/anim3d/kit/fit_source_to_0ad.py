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
PART_ORDER = ['body', 'layer', 'greave', 'cloak', 'helmet', 'crest', 'shield', 'spear']  # 'layer': clothing over the skin, baked on its own
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
ap.add_argument('--crest-width', type=float, default=1.0, help='side-to-side scale of a --crest-glb crest (fuller horsehair)')
ap.add_argument('--replace-part', action='append', default=[], metavar='PART=PATH[:OBJECT]',
                help='swap the fitted helmet (and face plate) or greaves for an external glTF/GLB/FBX mesh; PART is helmet or greaves')
ap.add_argument('--replace-rot', action='append', default=[], metavar='PART=X,Y,Z',
                help='rotate a replacement part (degrees about glTF X, Y, Z) before fitting; the source should be Y up, facing +Z')
ap.add_argument('--replace-credit', action='append', default=[], metavar='PART=TEXT', help='credit for a replacement part (goes into the credit and PROVENANCE)')
ap.add_argument('--replace-license', action='append', default=[], metavar='PART=ID', help='licence of a replacement part, e.g. CC-BY-4.0 or CC0-1.0')
ap.add_argument('--crest-mass', action='store_true', help='build the crest as its own dense part: lofted horsehair mass with strand grooves (about 2k triangles)')
ap.add_argument('--bronze-palette', default='47290f,a67734,f6dc96', help='dark,mid,light sRGB hex for --grade-bronze (r8b: sampled from spearman-panel.png: 4a2c0c,915d20,e5b475)')
ap.add_argument('--bronze-rough', type=float, default=.50, help='base roughness of graded bronze (raised edges are 0.2 lower)')
ap.add_argument('--bronze-detail', type=float, default=1.3, help='gain on the high-passed source luminance (fine detail) in graded bronze')
ap.add_argument('--bronze-ao', type=float, default=.45, help='AO floor on metal (lower = darker cavities); other surfaces keep 0.45')
ap.add_argument('--no-greave-dome', action='store_true', help='no kneecap dome on replaced greaves')
ap.add_argument('--greave-skin-any', action='store_true', help='delete skin faces with ANY vertex under a greave (not only fully covered ones)')
ap.add_argument('--skin-max-value', type=float, default=1.4, help='cap on skin brightness relative to its mean (pale patches from texture hidden under source armour)')
ap.add_argument('--smooth-skin-normals', action='store_true', help='transfer smooth normals from the undecimated fitted skin onto the LOD skin')
ap.add_argument('--lod1-helmet-face', type=float, nargs=2, metavar=('HEIGHT', 'DEPTH'),
                help='keep the helmet face and cheek pieces at LOD0 topology in LOD1: faces below HEIGHT (fraction of the helmet height) and in front of DEPTH (fraction of its depth, back to front)')
ap.add_argument('--lod1-helmet-face-dissolve', type=float, default=8., help='planar-dissolve angle (degrees) for the kept LOD1 helmet face')
ap.add_argument('--lod1-greave-offset', type=float, default=.003, help='extra radial room for the LOD1 built greaves (m)')
ap.add_argument('--helmet-size', type=float, default=1.0, help='replacement helmet size relative to an exact head fit')
ap.add_argument('--helmet-uv-share', type=float, default=1.0, help='scale the helmet UV islands before packing (2 = four times the atlas area)')
ap.add_argument('--crest-shift', type=float, default=0., help='move the crest arc forward (+) or back (-) along the helmet, metres')
ap.add_argument('--crest-tail', type=float, default=1.14, help='how far the crest mass trails down the back (1 = ends at the crown arc)')
ap.add_argument('--hide-skin-lod0', type=float, default=0, metavar='M', help='drop LOD0 skin faces lying within M metres under clothing or armour (never seen)')
ap.add_argument('--crest-strands', type=float, default=0, help='horsehair strand relief in the crest normal bake (wave scale; about 12 gives 6-8 mm strands)')
ap.add_argument('--knee-smooth', action='store_true', help='smooth skin weights around the knees (removes weight-transfer creases)')
ap.add_argument('--sculpt-override', action='append', default=[], metavar='PART=FILE.blend:OBJECT',
                help='replace the LOD0 geometry of PART with a hand-sculpted copy of that LOD0 object (same vertices and UVs, else the run fails); LOD1 and the bakes are re-derived from it')
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
    biggest = {}
    for p in pieces:
        if p['part'] == 'body': biggest[p['object']] = max(biggest.get(p['object'], 0), len(p['vid']))
    for p in pieces:
        if p['part'] != 'body': continue
        # Small loose pieces of the skin object (bracers, straps) are clothing too.
        if not re.search(skin_rx, p['object'], re.I) or len(p['vid']) < .03 * biggest[p['object']]: p['part'] = 'layer'
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
            # Torn scraps of the source sleeve left loose beside the new shell (r12's
            # red shard at the shield arm): small islands wholly around the upper arm and
            # standing off (> 2.5 cm) from every other garment surface and the new sleeve.
            bm = bmesh.new(); bm.from_mesh(lo_.data); bm.faces.ensure_lookup_table(); bm.verts.ensure_lookup_table(); seen = set(); dead = []
            from mathutils.kdtree import KDTree
            for f0 in bm.faces:
                if f0.index in seen: continue
                comp, st = [], [f0]
                while st:
                    f = st.pop()
                    if f.index in seen: continue
                    seen.add(f.index); comp.append(f); st.extend(y for e in f.edges for y in e.link_faces if y.index not in seen)
                if len(comp) > 300: continue
                c = np.array([f.calc_center_median() for f in comp]) - a; tt = c @ u; rr = np.linalg.norm(c - np.outer(tt, u), axis=1)
                if not ((tt > -.03) & (tt < tmax + .06) & (rr < rmax + .08) & (c[:, 0] * np.sign(a[0]) > -.005)).all(): continue
                mine = {v.index for f in comp for v in f.verts}
                rest = [v.co for v in bm.verts if v.index not in mine] + [v.co for v in o.data.vertices]
                kd = KDTree(len(rest))
                for i_, q_ in enumerate(rest): kd.insert(q_, i_)
                kd.balance()
                if max(kd.find(bm.verts[i_].co)[2] for i_ in mine) > .025: dead += comp
            bmesh.ops.delete(bm, geom=dead, context='FACES'); bm.to_mesh(lo_.data); bm.free(); removed += len(dead)
            report.setdefault('sleeve_scraps_removed', {})[side] = report.get('sleeve_scraps_removed', {}).get(side, 0) + len(dead)
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

replaced = []; helmet_centre = None
built_greaves = any(x.startswith('greaves=built') for x in args.replace_part)  # knee guard rides the shin bone
if args.replace_part:
    from fit_replace import import_part, fit_helmet, fit_greave, delete_faces, tuck_skin, finish_greave, delete_covered_skin
    kv = lambda items: dict(x.split('=', 1) for x in items)
    rots = {k: tuple(float(c) for c in v.split(',')) for k, v in kv(args.replace_rot).items()}
    credits, licences = kv(args.replace_credit), kv(args.replace_license)
    for spec in args.replace_part:
        part, rest = spec.split('=', 1)
        path, obj = (rest.rsplit(':', 1) if ':' in Path(rest).name else (rest, None))
        built = part == 'greaves' and path == 'built'   # greaves grown from the fitted shin, no source file
        new = [] if built else import_part(path, obj, rots.get(part, (0, 0, 0)))
        skin = all_co(fitted_objects['body'])
        if part == 'helmet':
            # The source helmet and any generated face plate go; the crest (built
            # later from the helmet points) seats on the new shell.
            for o in fitted_objects.pop('helmet', []): bpy.data.objects.remove(o, do_unlink=True)
            neck = rig.joint['neck']
            head = skin[(skin[:, 1] > neck[1] + .01) & (np.linalg.norm(skin - rig.joint['head'], axis=1) < .2)]
            o, rep_, hcentre = fit_helmet(new, head, size=args.helmet_size); o['part'] = 'helmet'; fitted_objects['helmet'] = [o]
            # Skin (skull, ears) poking through the shell moves in, the helmet does not grow.
            rep_['skin_tucked'] = tuck_skin(fitted_objects['body'] + fitted_objects.get('layer', []), o, hcentre)
            helmet_centre = hcentre
        elif part == 'greaves':
            # Old greaves: source clothing on the shin below the knee (not sleeves).
            def on_shin(P):
                m = np.zeros(len(P), bool)
                for sd in 'LR':
                    kj, aj = rig.joint['leg_' + sd], rig.joint['foot_' + sd]
                    rel = P - kj; m |= (P[:, 1] > aj[1] + .03) & (P[:, 1] < kj[1] + (.12 if built_greaves else .07)) & (np.hypot(rel[:, 0], rel[:, 2]) < .12)
                return m
            layers = [o for o in fitted_objects.get('layer', []) if not o.name.startswith('Sleeve')]
            rep_ = {'old_greave_faces_removed': delete_faces(layers, on_shin, any_vertex=True)}  # no leftover rim strips
            if built:
                from fit_shells import greave_from_skin
                bz = shell_material('Greave bronze (generated)', (.62, .43, .2), .35, 1.)
                for sd in 'LR':
                    kj, aj = rig.joint['leg_' + sd], rig.joint['foot_' + sd]
                    leg_ = skin[(np.sign(skin[:, 0]) == (1 if sd == 'L' else -1))]
                    objs_, r2 = greave_from_skin(leg_, kj, aj, bz, name=f'HIGH-greave-built-{sd}')
                    for x in objs_: x['part'] = 'greave'; fitted_objects.setdefault('greave', []).append(x)
                    r2['covered_skin_faces_deleted'] = delete_covered_skin(fitted_objects['body'], objs_[:1], (aj + (kj - aj) * .05, kj))
                    rep_[sd] = r2
            elif len(new) == 1 and (co(new[0])[:, 0] < -.02).any() and (co(new[0])[:, 0] > .02).any():
                # One object holding both greaves (as in a full character): split at x = 0.
                src_ = new[0]; halves = []
                for keep_pos in (False, True):
                    x = src_.copy(); x.data = src_.data.copy(); bpy.context.scene.collection.objects.link(x)
                    bm = bmesh.new(); bm.from_mesh(x.data)
                    bmesh.ops.delete(bm, geom=[f for f in bm.faces if (f.calc_center_median().x > 0) != keep_pos], context='FACES')
                    bm.to_mesh(x.data); bm.free(); halves.append(x)
                bpy.data.objects.remove(src_, do_unlink=True); new = halves
            sides = sorted(new, key=lambda o: co(o).mean(0)[0])
            pairs = [] if built else [('R', [sides[0]], False), ('L', [sides[-1]], False)] if len(new) > 1 and co(sides[0]).mean(0)[0] < 0 < co(sides[-1]).mean(0)[0] \
                else [('L', new, False), ('R', new, True)]
            for sd, objs, mirror in pairs:
                kj, aj = rig.joint['leg_' + sd], rig.joint['foot_' + sd]
                shin = skin[(skin[:, 1] > aj[1]) & (skin[:, 1] < kj[1] + .08) & (np.linalg.norm((skin - kj)[:, [0, 2]], axis=1) < .11)]
                g, r2 = fit_greave(objs, shin, kj, aj, mirror, name=f'HIGH-layer-greave-{sd}')
                g['part'] = 'greave'; fitted_objects.setdefault('greave', []).append(g); rep_[sd] = r2
                extra, r3 = finish_greave(g, kj, aj, shell_material('Greave bronze (generated)', (.62, .43, .2), .35, 1.), with_dome=not args.no_greave_dome)
                for x in extra: x['part'] = 'greave'; fitted_objects['greave'].append(x)
                r2.update(r3)
                r2['covered_skin_faces_deleted'] = delete_covered_skin(fitted_objects['body'], [g] + extra, (aj + (kj - aj) * .05, kj + (kj - aj) / np.linalg.norm(kj - aj) * .06))
            for o in new: bpy.data.objects.remove(o, do_unlink=True)   # fitted copies were made
        else:
            raise SystemExit(f'--replace-part: PART must be helmet or greaves, not {part!r}')
        info_ = {'part': part, 'file': 'built from the fitted shin' if built else Path(path).name, 'object': obj,
                 'sha256': None if built else hashlib.sha256(Path(path).read_bytes()).hexdigest(),
                 'credit': credits.get(part, ''), 'license': licences.get(part, ''), 'fit': rep_}
        replaced.append(info_); log('replaced', part, info_)
    report['replaced_parts'] = replaced
    if args.press_skin and (fitted_objects.get('layer') or fitted_objects.get('greave')):
        press_under(fitted_objects['body'], all_co(fitted_objects.get('layer', []) + fitted_objects.get('greave', [])))
        if fitted_objects.get('greave'):  # shin skin bulging through a fitted greave goes back under it
            report['skin_pressed_under_greaves'] = press_under(fitted_objects['body'], all_co(fitted_objects['greave']), depth=.006, reach=.035, below=.03)

if args.shield_glb:
    # A finished aspis built on this same rig: its vertices rigid to shield_arm are
    # already seated in bind space. Its own materials are baked like the source's.
    for o in fitted_objects.pop('shield', []): bpy.data.objects.remove(o, do_unlink=True)
    # Read the aspis straight from the GLB's audited vertex regions (bind space,
    # same rig): every shield_arm part except the leather handgrip, which was sized
    # for the 0 A.D. forearm and stands off a thicker one as flat flaps.
    from validate_glb import GLB as _GLB
    sg = _GLB(args.shield_glb); smh = sg.doc['meshes'][0]; spr = smh['primitives'][0]
    SP = np.array(sg.accessor(spr['attributes']['POSITION'])); SUV = np.array(sg.accessor(spr['attributes']['TEXCOORD_0']))
    SI = np.array([i[0] for i in sg.accessor(spr['indices'])]).reshape(-1, 3)
    keep_v = np.zeros(len(SP), bool)
    for r in smh['extras']['vertex_regions']:
        if r['bone'] == 'shield_arm' and r['part'] != 'Aspis handgrip': keep_v[r['start']:r['start'] + r['count']] = True
    stri = SI[keep_v[SI].all(1)]; su = np.unique(stri); sr = -np.ones(len(SP), np.int64); sr[su] = np.arange(len(su))
    me = bpy.data.meshes.new('aspis'); me.from_pydata(SP[su].tolist(), [], sr[stri].tolist()); me.update()
    suvl = me.uv_layers.new(name='UVMap')
    for poly in me.polygons:
        for li in poly.loop_indices:
            uvv = SUV[su[me.loops[li].vertex_index]]; suvl.data[li].uv = (uvv[0], 1 - uvv[1])
    stex = sg.doc['textures'][sg.doc['materials'][0]['pbrMetallicRoughness']['baseColorTexture']['index']]
    simg = sg.doc['images'][stex.get('source', stex.get('extensions', {}).get('EXT_texture_webp', {}).get('source'))]
    sbv = sg.doc['bufferViews'][simg['bufferView']]
    sext = {'image/png': 'png', 'image/jpeg': 'jpg', 'image/webp': 'webp'}[simg['mimeType']]
    spath = work / f'shield-source-basecolor.{sext}'; spath.write_bytes(sg.bin[sbv.get('byteOffset', 0):sbv.get('byteOffset', 0) + sbv['byteLength']])
    sm = bpy.data.materials.new('Aspis bronze (source)'); sm.use_nodes = True
    sb = sm.node_tree.nodes['Principled BSDF']; st = sm.node_tree.nodes.new('ShaderNodeTexImage'); st.image = bpy.data.images.load(str(spath))
    sm.node_tree.links.new(st.outputs['Color'], sb.inputs['Base Color']); sb.inputs['Metallic'].default_value = 1; sb.inputs['Roughness'].default_value = .4
    me.materials.append(sm)
    for poly in me.polygons: poly.use_smooth = True
    sh = bpy.data.objects.new('HIGH-shield-imported', me); bpy.context.scene.collection.objects.link(sh)
    sh['part'] = 'shield'; fitted_objects['shield'] = [sh]
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
    # A fuller fan: the painting's crest is a thick horsehair mass, not a fin.
    pos[team_v[used], 0] *= args.crest_width
    # The crest rises from the crown: lift it clear of the brow and back a little
    # (its front end hid the eye slots in a front view).
    pos[team_v[used]] += np.array([0, .035, -.02])
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
        if team and args.crest_strands:
            # Horsehair strands in the normal bake: rings about the crest's arc
            # centre (they follow its front-to-back flow), distorted like hair.
            nt = m.node_tree; geo = nt.nodes.new('ShaderNodeNewGeometry')
            sub = nt.nodes.new('ShaderNodeVectorMath'); sub.operation = 'MULTIPLY'; sub.inputs[1].default_value = (0, 1, 1)
            off = nt.nodes.new('ShaderNodeVectorMath'); off.operation = 'SUBTRACT'
            off.inputs[1].default_value = (0, float(pos[:, 1].min()), float((pos[:, 2].max() + pos[:, 2].min()) / 2))
            nt.links.new(geo.outputs['Position'], off.inputs[0]); nt.links.new(off.outputs[0], sub.inputs[0])
            wave = nt.nodes.new('ShaderNodeTexWave'); wave.wave_type = 'RINGS'; wave.inputs['Scale'].default_value = args.crest_strands
            wave.inputs['Distortion'].default_value = 4; wave.inputs['Detail'].default_value = 3
            nt.links.new(sub.outputs[0], wave.inputs['Vector'])
            bump = nt.nodes.new('ShaderNodeBump'); bump.inputs['Strength'].default_value = .9; bump.inputs['Distance'].default_value = .005
            nt.links.new(wave.outputs['Fac'], bump.inputs['Height']); nt.links.new(bump.outputs['Normal'], bsdf.inputs['Normal'])
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

if args.crest_mass:
    from fit_shells import crest_mass
    cm, crep = crest_mass(all_co(fitted_objects['helmet']), args.max_height, tail=args.crest_tail, shift=args.crest_shift)
    cm['part'] = 'crest'; fitted_objects['crest'] = [cm]
    team_rules.append(('crest', r'^Crest horsehair'))
    report['crest_mass'] = crep; log('crest mass', crep)

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


def covered_along_normal(P, Nn, shells, reach):
    """Skin vertex is hidden only if a ray out along its normal hits a covering shell
    within `reach`: nearness alone deleted thigh skin just below the skirt hem and
    left holes (the pale 'shards' seen through to the leg's back faces)."""
    from mathutils.bvhtree import BVHTree
    bm_ = bmesh.new()
    for x in shells: bm_.from_mesh(x.data)
    bvh = BVHTree.FromBMesh(bm_); bm_.free()
    out = np.zeros(len(P), bool)
    for i in range(len(P)):
        n_ = Vector(Nn[i])
        if n_.length < 1e-6: continue
        out[i] = bvh.ray_cast(Vector(P[i]) + n_ * 1e-4, n_.normalized(), reach)[0] is not None
    return out


def budgets(total, parts, keep=()):
    raw = {p: sum(ntris(o) for o in fitted_objects[p]) for p in parts}
    kept = {p: raw[p] for p in parts if p in keep}
    total -= sum(kept.values()); parts = [p for p in parts if p not in keep]
    floor = {'spear': 90, 'shield': 300, 'helmet': 250, 'crest': 300, 'greave': 1500, 'cloak': 200, 'body': 1000, 'layer': 800}
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
    if p == 'body' and args.hide_skin_lod0 and fitted_objects.get('layer'):
        # Skin that sits just under clothing or armour is never seen: spend those
        # triangles on the visible parts (and the crest) instead.
        Lp = all_co(fitted_objects['layer'] + fitted_objects.get('greave', [])); me = o.data; n = len(me.vertices)
        P = co(o); Nn = np.empty(n * 3); me.vertices.foreach_get('normal', Nn); Nn = Nn.reshape(-1, 3)
        hidden = np.zeros(n, bool)
        for k in range(0, n, 512):
            d2 = ((P[k:k + 512, None] - Lp[None]) ** 2).sum(2); jx = d2.argmin(1)
            d = np.sqrt(d2[np.arange(len(jx)), jx]); up = ((Lp[jx] - P[k:k + 512]) * Nn[k:k + 512]).sum(1)
            hidden[k:k + 512] = (d < args.hide_skin_lod0) & (up > -.002)
        hidden &= covered_along_normal(P, Nn, fitted_objects['layer'] + fitted_objects.get('greave', []), .025)
        bm = bmesh.new(); bm.from_mesh(me); bm.verts.ensure_lookup_table()
        dead = [f for f in bm.faces if all(hidden[v.index] for v in f.verts)]
        bmesh.ops.delete(bm, geom=dead, context='FACES'); bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context='VERTS')
        bm.to_mesh(me); bm.free(); report['lod0_hidden_skin_faces_removed'] = len(dead)
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
if args.helmet_uv_share != 1 and 'helmet' in lod0:
    bpy.ops.object.mode_set(mode='OBJECT')
    hu = lod0['helmet'].data.uv_layers['atlas'].data; a_ = np.empty(len(hu) * 2); hu.foreach_get('uv', a_); a_ = a_.reshape(-1, 2)
    hu.foreach_set('uv', ((a_ - a_.mean(0)) * args.helmet_uv_share + a_.mean(0)).ravel())
    bpy.ops.object.mode_set(mode='EDIT'); bpy.ops.mesh.select_all(action='SELECT')
bpy.ops.uv.pack_islands(margin=.003, rotate=True, shape_method='AABB', merge_overlap=False)
bpy.ops.object.mode_set(mode='OBJECT')
report['uv'] = 'source charts repacked' if keep_src_uv else 'smart projection'


def vertex_uvs(me, layer):
    """Per vertex, the sorted set of its corner UVs (rounded): topology-independent."""
    out = [set() for _ in me.vertices]; uvl = me.uv_layers[layer].data
    for l in me.loops: out[l.vertex_index].add(tuple(round(x, 4) for x in uvl[l.index].uv))
    return [sorted(x) for x in out]


for spec in args.sculpt_override:
    # A sculpted LOD0 part (saved from <work>/fitted-lods.blend by hand): same
    # vertices in the same order and the same UVs, new positions. The LOD0 here is
    # rebuilt deterministically, so the sculpt's positions drop straight in; the
    # full-resolution part follows by the sculpt delta of its nearest LOD0 vertices,
    # so the bakes (normals, AO, colour) see the sculpted shape too.
    part, rest = spec.split('=', 1); path, obj = rest.rsplit(':', 1)
    if part not in lod0: raise SystemExit(f'--sculpt-override: no LOD0 part {part!r} (have {sorted(lod0)})')
    before_objs = set(bpy.data.objects)
    with bpy.data.libraries.load(path, link=False) as (src_, dst_):
        if obj not in src_.objects: raise SystemExit(f'--sculpt-override: {obj!r} not in {path}')
        dst_.objects = [obj]
    sc = dst_.objects[0]; target = lod0[part]
    sme, tme = sc.data, target.data
    if len(sme.vertices) != len(tme.vertices):
        raise SystemExit(f'--sculpt-override {part}: sculpt has {len(sme.vertices)} vertices, LOD0 has {len(tme.vertices)}; '
                         'sculpt without remeshing or dyntopo')
    if 'atlas' not in sme.uv_layers or vertex_uvs(sme, 'atlas') != vertex_uvs(tme, 'atlas'):
        raise SystemExit(f'--sculpt-override {part}: UVs differ from the LOD0 part; do not edit UVs or reorder vertices')
    old = co(target); new = np.array([v.co[:] for v in sme.vertices]) @ np.array(sc.matrix_world)[:3, :3].T + np.array(sc.matrix_world)[:3, 3]
    delta = new - old; moved = np.linalg.norm(delta, axis=1)
    set_co(target, new)
    for h in fitted_objects[part]:
        H_ = co(h); acc = np.zeros_like(H_)
        for k in range(0, len(H_), 1024):
            d2 = ((H_[k:k + 1024, None] - old[None]) ** 2).sum(2); idx = np.argsort(d2, 1)[:, :4]
            w = 1 / (np.sqrt(np.take_along_axis(d2, idx, 1)) + 1e-4); w /= w.sum(1, keepdims=True)
            acc[k:k + 1024] = (delta[idx] * w[..., None]).sum(1)
        set_co(h, H_ + acc)
    bpy.data.objects.remove(sc, do_unlink=True)
    report.setdefault('sculpt_override', {})[part] = {'file': path, 'object': obj, 'sha256': hashlib.sha256(Path(path).read_bytes()).hexdigest(),
                                                      'vertices_moved': int((moved > 1e-6).sum()), 'max_move_m': round(float(moved.max()), 5)}
    log('sculpt override', part, report['sculpt_override'][part])
# LOD1 is what players see at normal zoom: garments must survive. Skin hidden
# under clothing or armour goes first (it only pokes through a coarse garment),
# then the budget favours garments over skin.
lod1_weight = {'layer': 1.6, 'greave': .75 if built_greaves else .35, 'cloak': 1.0, 'body': .8, 'helmet': .45, 'crest': .3, 'shield': .7, 'spear': .1}  # the aspis rim shreds below ~500
wsum_ = sum(lod1_weight.get(p, .5) for p in parts)


def helmet_face_mask(o):
    P_ = co(o); lo_, hi_ = P_.min(0), P_.max(0); Cf = np.array([pl.center for pl in o.data.polygons])
    return (Cf[:, 1] < lo_[1] + args.lod1_helmet_face[0] * (hi_[1] - lo_[1])) & (Cf[:, 2] > lo_[2] + args.lod1_helmet_face[1] * (hi_[2] - lo_[2]))


b1 = {p: int(args.lod1 * lod1_weight.get(p, .5) / wsum_) for p in parts}
cover = [lod0[p] for p in ('layer', 'greave', 'cloak') + (('helmet',) if args.lod1_helmet_face else ()) if p in lod0]
lod1 = {}; helmet_face_tris = 0
order_ = parts
if args.lod1_helmet_face and 'helmet' in parts:
    # The kept helmet face and the kept aspis rim are fixed costs: build those parts
    # first (dome at 100 triangles), then share what is left of the budget.
    # Each later part's share is recomputed from what is actually left (garments
    # resist collapse and overshoot their share).
    # Garments keep their usual share (below ~990 the chiton and skirt collapse);
    # greaves, skin, crest and spear share what is left, greaves first.
    order_ = [p for p in ('shield', 'helmet', 'layer') if p in parts] + [p for p in parts if p not in ('shield', 'helmet', 'layer')]; b1['helmet'] = 100
    rest_weight = {'greave': 1.2, 'body': .6, 'crest': .3, 'spear': .1}
for p in order_:
    if args.lod1_helmet_face and p not in ('shield', 'helmet', 'layer'):
        left_ = args.lod1 - sum(ntris(o_) for o_ in lod1.values()); w_ = sum(rest_weight.get(q, .5) for q in order_ if q not in lod1)
        b1[p] = int(left_ * rest_weight.get(p, .5) / w_)
    o = joined_copy([lod0[p]], f'LOD1-{p}'); o['part'] = p
    if p == 'body' and cover:
        Lp = np.concatenate([co(c) for c in cover]); me = o.data; n = len(me.vertices)
        P = co(o); Nn = np.empty(n * 3); me.vertices.foreach_get('normal', Nn); Nn = Nn.reshape(-1, 3)
        hidden = np.zeros(n, bool)
        for k in range(0, n, 512):
            d2 = ((P[k:k + 512, None] - Lp[None]) ** 2).sum(2); j = d2.argmin(1)
            d = np.sqrt(d2[np.arange(len(j)), j]); up = ((Lp[j] - P[k:k + 512]) * Nn[k:k + 512]).sum(1)
            hidden[k:k + 512] = (d < .03) & (up > -.004)
        hidden &= covered_along_normal(P, Nn, cover, .035)
        bm = bmesh.new(); bm.from_mesh(me); bm.verts.ensure_lookup_table()
        dead = [f for f in bm.faces if all(hidden[v.index] for v in f.verts)]
        bmesh.ops.delete(bm, geom=dead, context='FACES'); bm.to_mesh(me); bm.free()
        report['lod1_hidden_skin_faces_removed'] = len(dead)
    if p == 'shield':
        # Keep the rolled team rim at LOD0 topology and decimate only the field:
        # decimated rim faces straddle rim and field texels (pale slivers, red wedges).
        fr_ = rig.shield_frame(); P_ = co(o); me = o.data
        q_ = P_ - fr_['centre']; h_ = q_ @ fr_['normal']; r_ = np.linalg.norm(q_ - np.outer(h_, fr_['normal']), axis=1)
        rim_v = r_ > .86 * r_.max()
        rim = joined_copy([o], 'LOD1-shield-rim')
        for obj, drop in ((o, True), (rim, False)):
            bm = bmesh.new(); bm.from_mesh(obj.data); bm.verts.ensure_lookup_table()
            dead = [f for f in bm.faces if all(rim_v[v.index] for v in f.verts) == drop]
            bmesh.ops.delete(bm, geom=dead, context='FACES'); bm.to_mesh(obj.data); bm.free()
        room = max(60, b1[p] - ntris(rim))
        if ntris(o) > room: apply_mod(o, 'DECIMATE', ratio=room / ntris(o), use_collapse_triangulate=True)
        bpy.ops.object.select_all(action='DESELECT'); o.select_set(True); rim.select_set(True)
        bpy.context.view_layer.objects.active = o; bpy.ops.object.join()
    elif p == 'helmet' and args.lod1_helmet_face:
        # Collapse decimation folds the thin Corinthian face and cheek plates into
        # pale shards with skin behind them: keep that region at LOD0 topology (as
        # the aspis rim) and decimate only the dome.
        face_f = helmet_face_mask(o)
        face = joined_copy([o], 'LOD1-helmet-face')
        for obj, drop in ((o, True), (face, False)):
            bm = bmesh.new(); bm.from_mesh(obj.data); bm.faces.ensure_lookup_table()
            dead = [f for f in bm.faces if bool(face_f[f.index]) == drop]
            bmesh.ops.delete(bm, geom=dead, context='FACES'); bm.to_mesh(obj.data); bm.free()
        # Planar dissolve only merges near-coplanar faces inside UV islands: no vertex
        # moves, so nothing folds into a shard.
        bm = bmesh.new(); bm.from_mesh(face.data)
        bmesh.ops.dissolve_limit(bm, angle_limit=math.radians(args.lod1_helmet_face_dissolve), use_dissolve_boundaries=False, verts=bm.verts[:], edges=bm.edges[:], delimit={'UV', 'NORMAL'})
        bmesh.ops.triangulate(bm, faces=bm.faces[:]); bm.to_mesh(face.data); bm.free()
        helmet_face_tris = ntris(face)
        # The dome's inner shell is only seen through the face opening, which the kept
        # face covers: drop it so the dome budget goes to the outside (no dark slabs).
        cen_ = co(o).mean(0); bm = bmesh.new(); bm.from_mesh(o.data)
        inner = [f for f in bm.faces if np.dot(np.array(f.normal), np.array(f.calc_center_median()) - cen_) < 0]
        bmesh.ops.delete(bm, geom=inner, context='FACES'); bm.to_mesh(o.data); bm.free()
        report['lod1_helmet_inner_faces_dropped'] = len(inner)
        if ntris(o) > b1[p]: apply_mod(o, 'DECIMATE', ratio=b1[p] / ntris(o), use_collapse_triangulate=True)
        bpy.ops.object.select_all(action='DESELECT'); o.select_set(True); face.select_set(True)
        bpy.context.view_layer.objects.active = o; bpy.ops.object.join()
        report['lod1_helmet_face_tris_kept'] = helmet_face_tris
    elif ntris(o) > b1[p]: apply_mod(o, 'DECIMATE', ratio=b1[p] / ntris(o), use_collapse_triangulate=True)
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


enforce(lod0, args.lod0, set(args.keep)); enforce(lod1, args.lod1, {'layer', 'greave', 'cloak', 'shield'} | ({'helmet'} if helmet_face_tris else set()))
if helmet_face_tris and 'body' in lod1:
    # LOD1's coarse head skin pokes through the kept cheek plates in walk and death:
    # pull head skin inside the helmet's footprint 8 mm in toward the head axis.
    H_ = co(lod0['helmet']); hc_ = H_.mean(0); P_ = co(lod1['body'])
    inside = (P_[:, 1] > H_[:, 1].min()) & (np.abs(P_[:, 0] - hc_[0]) < (H_[:, 0].max() - H_[:, 0].min()) / 2) & (np.abs(P_[:, 2] - hc_[2]) < (H_[:, 2].max() - H_[:, 2].min()) / 2)
    rad_ = P_[inside] - hc_; rad_[:, 1] = 0; nr_ = np.linalg.norm(rad_, axis=1, keepdims=True)
    P_[inside] -= rad_ / np.maximum(nr_, 1e-6) * np.minimum(.008, nr_ * .5); set_co(lod1['body'], P_)
    report['lod1_head_skin_sunk_vertices'] = int(inside.sum())
if args.smooth_skin_normals and 'body' in lod0:
    # Decimated skin reads as flat planes: carry the undecimated fitted skin's smooth
    # normals onto both LODs as custom normals (exported as the corner normals).
    hb_ = joined_copy(fitted_objects['body'], 'normal source')
    for p_ in hb_.data.polygons: p_.use_smooth = True
    for lod in (lod0, lod1):
        o_ = lod['body']
        for p_ in o_.data.polygons: p_.use_smooth = True
        m_ = o_.modifiers.new('nrm', 'DATA_TRANSFER'); m_.object = hb_
        m_.use_loop_data = True; m_.data_types_loops = {'CUSTOM_NORMAL'}; m_.loop_mapping = 'POLYINTERP_NEAREST'
        active(o_); bpy.ops.object.modifier_apply(modifier=m_.name)
    bpy.data.objects.remove(hb_, do_unlink=True); report['skin_normals'] = 'transferred from the undecimated fit'
if built_greaves and 'greave' in lod1:
    # LOD1's coarser greave gets a little more room over the skin.
    for sd in 'LR':
        kn = rig.joint['leg_' + sd]; an = rig.joint['foot_' + sd]; ax = (kn - an) / np.linalg.norm(kn - an)
        P_ = co(lod1['greave']); m_ = (P_[:, 0] > 0) == (sd == 'L'); q = P_[m_] - an; h = q @ ax
        rad = q - np.outer(h, ax); P_[m_] += rad / np.maximum(np.linalg.norm(rad, axis=1, keepdims=True), 1e-6) * args.lod1_greave_offset
        set_co(lod1['greave'], P_)
if fitted_objects.get('greave') and 'body' in lod0:
    # Skin behind a replaced greave is never seen (and pokes through where the shell is
    # thin): remove it from both LODs, per leg, by the greave's angular/height span.
    from fit_replace import delete_covered_skin
    gone_ = 0
    for sd in 'LR':
        kj, aj = rig.joint['leg_' + sd], rig.joint['foot_' + sd]
        sh_ = [g_ for g_ in fitted_objects['greave'] if (co(g_)[:, 0].mean() > 0) == (sd == 'L')]
        if sh_:
            for lod in (lod0, lod1): gone_ += delete_covered_skin([lod['body']], sh_, (aj, kj + (kj - aj) / np.linalg.norm(kj - aj) * .06), any_vertex=args.greave_skin_any)
    report['skin_behind_greaves_deleted_lod_faces'] = gone_
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
        if p in RIGID or p in ('spear', 'shield', 'helmet', 'crest'):
            bone = {'spear': 'weapon_R', 'shield': 'shield_arm', 'helmet': 'helmet', 'crest': 'helmet'}[p]
            o.vertex_groups[bone].add(list(range(len(o.data.vertices))), 1., 'REPLACE')
        elif p == 'cloak' and donors['cloak']:
            transfer(o, donors['cloak'])
        elif p == 'greave':
            # Greaves are rigid shells on the shin; the top eases onto the thigh across
            # the knee. Direct weights: a nearest-face transfer picked up the other leg.
            P_ = co(o)
            for sd in 'LR':
                side_ = (P_[:, 0] > 0) if sd == 'L' else (P_[:, 0] <= 0)
                ky = rig.joint['leg_' + sd][1] + (.12 if built_greaves else 0); tw = np.clip((P_[:, 1] - (ky - .02)) / .08, 0, 1); tw = tw * tw * (3 - 2 * tw) * .5
                for i in np.nonzero(side_)[0]:
                    o.vertex_groups['leg_' + sd].add([int(i)], float(1 - tw[i]), 'REPLACE')
                    if tw[i] > 0: o.vertex_groups['thigh_' + sd].add([int(i)], float(tw[i]), 'REPLACE')
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
if args.knee_smooth:
    # Weight-transfer creases at the knees: relax thigh/shin weights within 9 cm
    # of each knee joint on skin and clothing, then renormalise to four.
    for lod in (lod0, lod1):
        for p in ('body', 'layer', 'greave'):
            if p not in lod: continue
            o = lod[p]; P = co(o)
            near = np.zeros(len(P), bool)
            for kj in ('leg_L', 'leg_R'): near |= np.linalg.norm(P - rig.joint[kj], axis=1) < .09
            if not near.any(): continue
            o.data.vertices.foreach_set('select', near.tolist()); active(o)
            bpy.ops.object.mode_set(mode='WEIGHT_PAINT'); o.data.use_paint_mask_vertex = True
            bpy.ops.object.vertex_group_smooth(group_select_mode='ALL', factor=.6, repeat=6, expand=0)
            o.data.use_paint_mask_vertex = False; bpy.ops.object.mode_set(mode='OBJECT')
            bpy.ops.object.vertex_group_limit_total(limit=4); bpy.ops.object.vertex_group_normalize_all(lock_active=False)
    report['knee_weights_smoothed'] = True
for o in list(bpy.data.objects):
    if o.name.startswith('donor_'): bpy.data.objects.remove(o, do_unlink=True)
log('weights transferred')

# -------------------------------------------------------------------------- bake
scene = bpy.context.scene; scene.render.engine = 'CYCLES'; scene.cycles.device = 'CPU'
scene.cycles.samples = 1; scene.render.threads_mode = 'FIXED'; scene.render.threads = 3
bake = scene.render.bake; bake.use_selected_to_active = True; bake.margin = 32; bake.target = 'IMAGE_TEXTURES'
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
    for p_ in ('greave', 'helmet'):  # replaced armour is bronze whatever its source metallic map says
        if p_ in lod0: metal |= raster_mask(uv_tris(lod0[p_]))
    Mt_ = px(maps_img['metallic']); Mt_[metal, :3] = np.maximum(Mt_[metal, :3], .85 if args.bronze_ao >= .45 else .95); maps_img['metallic'].pixels.foreach_set(Mt_.ravel())
    if metal.any():
        lum = B_[..., :3] @ np.array([.2126, .7152, .0722], np.float32); l = lum / max(float(lum[metal].mean()), 1e-3)
        # Keep fine detail (engraving, embossing, hammer marks), drop the low-frequency
        # dirt blotches: high-pass the luminance against a masked 24 px box blur.
        def box(a_, r_=12):
            c_ = np.cumsum(np.cumsum(np.pad(a_, ((1, 0), (1, 0))), 0), 1); n_ = a_.shape[0]
            i0 = np.clip(np.arange(n_) - r_, 0, n_); i1 = np.clip(np.arange(n_) + r_ + 1, 0, n_)
            return c_[i1][:, i1] - c_[i0][:, i1] - c_[i1][:, i0] + c_[i0][:, i0]
        mf = metal.astype(np.float32); lb = box(l * mf) / np.maximum(box(mf), 1e-6)
        l = np.clip(1 + (l - lb) * args.bronze_detail, .6, 1.5)
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
        dark, mid, light = (srgb(c_) for c_ in args.bronze_palette.split(','))
        col = np.where(t_[..., None] < .5, dark + (mid - dark) * (t_[..., None] * 2), mid + (light - mid) * ((t_[..., None] - .5) * 2))
        B_[metal, :3] = col[metal]
        R_[metal, :3] = np.clip(args.bronze_rough - .2 * edge[metal] + .14 * (1 - cav[metal]), .12, .8)[:, None]
        maps_img['basecolor'].pixels.foreach_set(B_.ravel()); maps_img['roughness'].pixels.foreach_set(R_.ravel())
        report['bronze_grade'] = {'texels': int(metal.sum()), 'palette': args.bronze_palette}
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
        lum = B_[skin, :3] @ np.array([.2126, .7152, .0722], np.float32); k_ = np.clip(lum / max(float(lum.mean()), 1e-3), .6, args.skin_max_value)
        B_[skin, :3] = np.clip(.85 * tgt * k_[:, None] + .15 * (tgt + (B_[skin, :3] - mean)), 0, 1)
        maps_img['basecolor'].pixels.foreach_set(B_.ravel())
        report['skin_grade'] = {'texels': int(skin.sum()), 'from_srgb': (mean * 255).round().tolist(), 'to': args.grade_skin}
        log('skin graded', report['skin_grade'])

if helmet_centre is not None and 'helmet' in lod0:
    # The helmet's inside (seen through the eye and mouth openings) is a dark void.
    from fit_replace import inner_faces
    hm = lod0['helmet'].data; ins = set(inner_faces(lod0['helmet'], helmet_centre)); uvl_ = hm.uv_layers['atlas'].data
    hm.calc_loop_triangles()
    m_in = raster_mask([[tuple(uvl_[li].uv) for li in t.loops] for t in hm.loop_triangles if t.polygon_index in ins])
    B_ = px(maps_img['basecolor']); B_[m_in, :3] = (.035, .027, .02); maps_img['basecolor'].pixels.foreach_set(B_.ravel())
    for nm_, val_ in (('roughness', .95), ('metallic', 0.)):
        X_ = px(maps_img[nm_]); X_[m_in, :3] = val_; maps_img[nm_].pixels.foreach_set(X_.ravel())
    report['helmet_interior_texels'] = int(m_in.sum())

orm = bpy.data.images.new('orm', N, N, alpha=False); orm.colorspace_settings.name = 'Non-Color'
o_px = np.ones((N, N, 4), np.float32)
o_px[..., 0] = px(maps_img['ao'])[..., 0]; o_px[..., 1] = px(maps_img['roughness'])[..., 0]; o_px[..., 2] = px(maps_img['metallic'])[..., 0]
if args.smooth_skin_normals and 'body' in lod0:
    # Low-mesh AO on the decimated skin outlines every facet: keep only a trace of it there.
    skm_ = raster_mask(uv_tris(lod0['body'])) & (o_px[..., 2] < .5)
    o_px[..., 0] = np.where(skm_, .85 + .15 * o_px[..., 0], o_px[..., 0])
if args.bronze_ao < .45:  # darker cavities on metal: re-floor its AO
    mt_ = o_px[..., 2] > .8; raw_ = np.clip((o_px[..., 0] - .45) / .55, 0, 1)
    o_px[..., 0] = np.where(mt_, args.bronze_ao + (1 - args.bronze_ao) * raw_, o_px[..., 0])
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
names = {'body': 'body', 'layer': 'garment', 'greave': 'greave', 'cloak': 'cloak', 'helmet': 'helmet', 'crest': 'crest', 'shield': 'aspis', 'spear': 'spear'}
team_part = {'crest': 'crest', 'shield rim': 'aspis', 'cloak trim': 'cloak'}
planes = clip_planes(rig.g)
log('clip floor planes', len(planes))


team_lod0_masks = {}


def export_lod(lod, mesh_name):
    attrs = {k: [] for k in ['POSITION', 'NORMAL', 'TANGENT', 'TEXCOORD_0', 'JOINTS_0', 'WEIGHTS_0', 'COLOR_0']}
    idx = []; regions = []
    for p in parts:
        o = lod[p]; region = team_faces(o, p)
        if False:
            # LOD1 shares LOD0's atlas: a face is team exactly where LOD0's team
            # texels are, so the mask and the neutral texels always agree.
            me_ = o.data; uvl_ = me_.uv_layers.active.data; region = np.array([''] * len(me_.polygons), dtype=object)
            for poly in me_.polygons:
                u_, v_ = np.mean([tuple(uvl_[li].uv) for li in poly.loop_indices], 0)
                x_, y_ = min(N - 1, max(0, int(u_ * N))), min(N - 1, max(0, int(v_ * N)))
                for kind_, m_ in team_lod0_masks.items():
                    if m_[y_, x_]: region[poly.index] = kind_

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
        bone = {'spear': 'weapon_R', 'shield': 'shield_arm', 'helmet': 'helmet', 'crest': 'helmet'}.get(p)
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
    if any(r['part'] == 'greave' for r in regions):
        # Skin that shows through a greave in any frame of the clips: drop its triangles
        # (it is behind the armour). Same test as kit/armour_cover_check.py.
        from armour_cover_check import flagged
        part_ = np.empty(len(xyz), object)
        for r in regions: part_[r['start']:r['start'] + r['count']] = r['part']
        J_ = np.array(attrs['JOINTS_0']).astype(int); W_ = np.array(attrs['WEIGHTS_0']); T_ = np.array([i[0] for i in idx]).reshape(-1, 3)
        dropped = 0; sunk = 0; pushed_g = 0
        inv_ = np.array(rig.g.accessor(rig.g.doc['skins'][0]['inverseBindMatrices'])).reshape(-1, 4, 4).transpose(0, 2, 1)
        for it in range(30):
            used_ = np.zeros(len(xyz), bool); used_[T_.ravel()] = True
            clips_ = ('walk', 'attack', 'idle', 'hit', 'death')
            bad_ = flagged(rig.g, xyz, J_, W_, part_, used_, clips_, tol=-.002)     # the check's own bins
            if built_greaves: bad_ |= flagged(rig.g, xyz, J_, W_, part_, used_, clips_, edge=.003, tol=-.002)  # and up to the rims
            log(mesh_name, 'skin through greaves pass', it, len(bad_))
            if not bad_: break
            bl = np.array(sorted(bad_))
            if it < 16 and built_greaves and it % 2 == 0:
                # Push the greave out locally where skin reaches it (at most 6 mm in all).
                gidx = np.nonzero(part_ == 'greave')[0]
                if not hasattr(np, '_fit_push'): np._fit_push = {}
                for sd in 'LR':
                    kn = rig.joint['leg_' + sd]; an = rig.joint['foot_' + sd]; ax = (kn - an) / np.linalg.norm(kn - an)
                    gs = gidx[(xyz[gidx, 0] > 0) == (sd == 'L')]; fb = bl[(xyz[bl, 0] > 0) == (sd == 'L')]
                    if not len(gs) or not len(fb): continue
                    d2 = np.min(((xyz[gs][:, None] - xyz[fb][None]) ** 2).sum(2), 1); near = gs[d2 < .025 ** 2]
                    done = np.array([np._fit_push.get((mesh_name, int(i)), 0.) for i in near])
                    near = near[done < .006]
                    q = xyz[near] - an; h = q @ ax; rad = q - np.outer(h, ax)
                    xyz[near] += rad / np.maximum(np.linalg.norm(rad, axis=1, keepdims=True), 1e-6) * .0015
                    for i in near: np._fit_push[(mesh_name, int(i))] = np._fit_push.get((mesh_name, int(i)), 0.) + .0015
                pushed_g += 1; continue
            if it < 26:
                # Sink flagged skin 5 mm toward its shin axis (bind space): no holes.
                for sd in 'LR':
                    kn = rig.joint['leg_' + sd]; an = rig.joint['foot_' + sd]; ax = (kn - an) / np.linalg.norm(kn - an)
                    m_ = bl[(xyz[bl, 0] > 0) == (sd == 'L')]
                    q = xyz[m_] - an; h = q @ ax; rad = q - np.outer(h, ax); rn = np.linalg.norm(rad, axis=1, keepdims=True)
                    xyz[m_] = an + np.outer(h, ax) + rad * np.maximum(rn - .005, .01) / np.maximum(rn, 1e-6)
                sunk += len(bl); continue
            badm = np.zeros(len(xyz), bool); badm[bl] = True
            if os.environ.get('FIT_NO_DROP'): break
            keep_ = ~badm[T_].all(1); dropped += int((~keep_).sum()); T_ = T_[keep_]
        attrs['POSITION'] = [tuple(map(float, r)) for r in xyz]
        idx = [(int(i),) for i in T_.ravel()]
        rep['skin_under_greaves'] = {'vertex_sinks': sunk, 'greave_push_passes': pushed_g, 'triangles_dropped': dropped}
    log(mesh_name, len(idx) // 3, 'triangles; ground fit', rep)
    return {'name': mesh_name, 'attrs': attrs, 'indices': idx, 'regions': regions}, rep


meshes = []; report['ground_fit'] = {}; team_uv = {}
for lod, name in ((lod0, 'spearman_LOD0'), (lod1, 'spearman_LOD1')):
    m, rep = export_lod(lod, name); meshes.append(m); report['ground_fit'][name] = rep
    if name.endswith('LOD0'): team_lod0_masks = {k: raster_mask(v) for k, v in team_uv.items()}
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
    # LOD1's coarser UVs sample a few texels past LOD0's islands: grow the team
    # area into the empty gutter only, never onto texels other faces use.
    if 'occupied' not in globals():
        occupied = raster_mask([t for p_ in parts for t in uv_tris(lod0[p_])])
    for _ in range(6):
        grow = mask | np.roll(mask, 1, 0) | np.roll(mask, -1, 0) | np.roll(mask, 1, 1) | np.roll(mask, -1, 1)
        mask = mask | (grow & ~occupied)
    lum = bp[..., :3] @ np.array([.2126, .7152, .0722], np.float32)
    grey = np.clip(lum / max(float(lum[mask].mean()), 1e-3) * target.get(kind, .75), .15, 1)
    # Shield rim: pale warm bronze rather than grey. Rim texels sampled by the
    # neighbouring (non-team) field faces then read as bronze, not as a pale sliver.
    tint = np.array([1.08, .95, .70], np.float32) if kind == 'shield rim' else np.ones(3, np.float32)
    bp[mask, :3] = np.clip(grey[mask, None] * tint, 0, 1)
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
credit = f'{args.credit}; ' + ''.join(f"{r['part']}: {r['credit']}; " for r in replaced if r['credit']) + f'animation and rig: {OAD_CREDIT}'
pack(rig.g, meshes, images, out, credit, args.license,
     extras={'texture_master_size': args.bake_size, 'texture_delivery': {'encoding': args.texture_format.upper(), 'quality': args.jpeg_quality, 'pixels': args.ship_size},
             'source_model': {'file': args.source.name, 'sha256': report['source_sha256'], 'credit': args.credit},
             'fit': 'tools/art/anim3d/kit/fit_source_to_0ad.py',
             'replaced_parts': [{k: r[k] for k in ('part', 'file', 'sha256', 'credit', 'license')} for r in replaced]},
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
""" + ''.join(f"- Replacement {r['part']}: {r['credit'] or '(credit not given)'}, licence {r['license'] or '(not given)'}. "
              f"File `{r['file']}`, SHA-256 `{r['sha256']}`.\n" for r in replaced))
report.update(glb=str(out), glb_bytes=out.stat().st_size, seconds=round(time.time() - t0, 1))
bpy.ops.wm.save_as_mainfile(filepath=str(work / 'fitted-lods.blend'))
(work / 'fit-report.json').write_text(json.dumps(report, indent=2, default=float) + '\n')
log('wrote', out, out.stat().st_size, 'bytes')
