"""Write a contract GLB around the approved rig: new meshes and maps, untouched motion.

Pure Python. The skeleton nodes, inverse binds and every animation accessor are
copied byte for byte from the approved GLB, so the five clips are identical to
the approved round by construction. Adapted from the painted lane's packager.
"""
import copy, hashlib, json, struct


def plain_webp(raw):
    """Keep only the bitstream chunk of a still WebP. Blender writes an extended
    (VP8X) file with a colour profile, which the Khronos validator flags."""
    if raw[:4] != b'RIFF' or raw[12:16] != b'VP8X': return raw
    pos = 12
    while pos < len(raw):
        tag = raw[pos:pos + 4]; size = struct.unpack_from('<I', raw, pos + 4)[0]
        if tag in (b'VP8 ', b'VP8L'):
            chunk = raw[pos:pos + 8 + size + (size & 1)]
            return b'RIFF' + struct.pack('<I', 4 + len(chunk)) + b'WEBP' + chunk
        pos += 8 + size + (size & 1)
    return raw


def pack(g, meshes, images, out, credit, license_id='CC-BY-SA-3.0', extras=None, material_name='Fitted unit'):
    """meshes: [{'name', 'attrs': {POSITION, NORMAL, TANGENT, TEXCOORD_0, JOINTS_0, WEIGHTS_0, COLOR_0}, 'indices', 'regions'}]
    images: [(name, mime, bytes)] in the order basecolor, normal, orm."""
    d = copy.deepcopy(g.doc); old = d['accessors']; views = d['bufferViews']
    blob = bytearray(); d['accessors'] = []; d['bufferViews'] = []; view_cache = {}

    def view(raw, target=None, stride=None):
        key = (bytes(raw), target, stride)
        if key in view_cache: return view_cache[key]
        blob.extend(b'\0' * (-len(blob) % 4)); v = {'buffer': 0, 'byteOffset': len(blob), 'byteLength': len(raw)}
        blob.extend(raw)
        if target: v['target'] = target
        if stride: v['byteStride'] = stride
        d['bufferViews'].append(v); view_cache[key] = len(d['bufferViews']) - 1; return view_cache[key]

    def acc(values, kind, component=5126, target=None, normalized=False):
        fmt = {5126: 'f', 5121: 'B', 5122: 'h', 5123: 'H', 5125: 'I'}[component]
        row = struct.calcsize(fmt) * len(values[0]); stride = (row + 3) // 4 * 4 if target == 34962 else row
        raw = b''.join(struct.pack('<' + fmt * len(r), *r) + b'\0' * (stride - row) for r in values)
        a = {'bufferView': view(raw, target, stride if stride != row else None), 'componentType': component,
             'count': len(values), 'type': kind}
        if normalized: a['normalized'] = True
        if kind != 'MAT4':
            a.update(min=[min(v[i] for v in values) for i in range(len(values[0]))],
                     max=[max(v[i] for v in values) for i in range(len(values[0]))])
        d['accessors'].append(a); return len(d['accessors']) - 1

    copied = {}; seen = {}
    def original(i):
        if i not in copied:
            a = copy.deepcopy(old[i]); v = views[a['bufferView']]; assert 'byteStride' not in v
            a['bufferView'] = view(g.bin[v.get('byteOffset', 0):v.get('byteOffset', 0) + v['byteLength']])
            if a['type'] != 'SCALAR': a.pop('min', None); a.pop('max', None)
            key = json.dumps(a, sort_keys=True)
            if key not in seen: d['accessors'].append(a); seen[key] = len(d['accessors']) - 1
            copied[i] = seen[key]
        return copied[i]
    for a in d['animations']:
        for s in a['samplers']:
            for k in ('input', 'output'): s[k] = original(s[k])
    for skin in d['skins']: skin['inverseBindMatrices'] = original(skin['inverseBindMatrices'])

    d['meshes'] = []
    for lod, m in enumerate(meshes):
        packed = []
        for row in m['attrs']['WEIGHTS_0']:
            q = [int(x * 65535) for x in row]
            for i in sorted(range(4), key=lambda i: -(row[i] * 65535 - q[i]))[:65535 - sum(q)]: q[i] += 1
            packed.append(q)
        aa = {}
        for k, v in m['attrs'].items():
            normalized = k in {'COLOR_0', 'NORMAL', 'TANGENT', 'TEXCOORD_0', 'WEIGHTS_0'}; comp = 5126
            if k == 'COLOR_0': v = [tuple(round(x * 255) for x in r) for r in v]; comp = 5121
            elif k == 'JOINTS_0': v = [tuple(j if w else 0 for j, w in zip(r, q)) for r, q in zip(v, packed)]; comp = 5121
            elif k in {'NORMAL', 'TANGENT'}: v = [tuple(round(max(-1, min(1, x)) * 32767) for x in r) for r in v]; comp = 5122
            elif k == 'TEXCOORD_0': v = [tuple(round(max(0, min(1, x)) * 65535) for x in r) for r in v]; comp = 5123
            elif k == 'WEIGHTS_0': v = packed; comp = 5123
            aa[k] = acc(v, 'VEC' + str(len(v[0])), comp, 34962, normalized)
        big = max(i[0] for i in m['indices']) >= 65536
        d['meshes'].append({'name': m['name'], 'primitives': [{'attributes': aa,
            'indices': acc(m['indices'], 'SCALAR', 5125 if big else 5123, 34963), 'material': 0}],
            'extras': {'vertex_regions': m['regions'], 'lod': lod}})
    node = next(n for n in d['nodes'] if 'mesh' in n); node.update(name=meshes[0]['name'], mesh=0, skin=0)
    for lod in range(1, len(meshes)):
        d['nodes'].append({'name': meshes[lod]['name'], 'mesh': lod, 'skin': 0})
        d['scenes'][d.get('scene', 0)]['nodes'].append(len(d['nodes']) - 1)
    d['extensionsUsed'] = ['KHR_mesh_quantization']; d['extensionsRequired'] = ['KHR_mesh_quantization']
    d['images'] = []; d['textures'] = []
    for name, mime, raw in images:
        if mime == 'image/webp': raw = plain_webp(raw)
        d['images'].append({'name': name, 'mimeType': mime, 'bufferView': view(raw)})
        if mime == 'image/webp':
            d['textures'].append({'extensions': {'EXT_texture_webp': {'source': len(d['images']) - 1}}, 'sampler': 0})
            for key in ('extensionsUsed', 'extensionsRequired'):
                if 'EXT_texture_webp' not in d[key]: d[key].append('EXT_texture_webp')
        else:
            d['textures'].append({'source': len(d['images']) - 1, 'sampler': 0})
    d['samplers'] = [{'magFilter': 9729, 'minFilter': 9987, 'wrapS': 33071, 'wrapT': 33071}]
    team = d['asset']['extras']['team']
    d['materials'] = [{'name': material_name,
        'pbrMetallicRoughness': {'baseColorTexture': {'index': 0, 'texCoord': 0},
                                 'metallicRoughnessTexture': {'index': 2, 'texCoord': 0},
                                 'metallicFactor': 1, 'roughnessFactor': 1},
        'normalTexture': {'index': 1, 'texCoord': 0}, 'occlusionTexture': {'index': 2, 'texCoord': 0},
        'doubleSided': True, 'extras': {'team': team}}]
    d['asset']['copyright'] = credit
    d['asset']['generator'] = 'Legatum fit_source_to_0ad'
    d['asset']['extras'].update(credit=credit, license=license_id,
        lod_meshes=[m['name'] for m in meshes], default_lod=0,
        approved_motion_sha256=hashlib.sha256(g.path.read_bytes()).hexdigest(), **(extras or {}))
    d['buffers'] = [{'byteLength': len(blob)}]; blob.extend(b'\0' * (-len(blob) % 4))
    raw = json.dumps(d, separators=(',', ':')).encode(); raw += b' ' * (-len(raw) % 4)
    out.write_bytes(struct.pack('<4sII', b'glTF', 2, 28 + len(raw) + len(blob)) +
                    struct.pack('<II', len(raw), 0x4e4f534a) + raw + struct.pack('<II', len(blob), 0x004e4942) + blob)
    return out
