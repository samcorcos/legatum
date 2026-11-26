#!/usr/bin/env python3
"""Validate Legatum's real-time unit contract directly from GLB JSON/BIN.

No dependencies. Checks actual geometry, hierarchy, pixels, animation samples and
skinned bounds; aesthetic flatness and recognisability still require visual review.
"""
import argparse, bisect, copy, json, math, struct
from pathlib import Path

CREDIT='0 A.D. art by Wildfire Games, CC BY-SA 3.0; adapted for Legatum'
IDENTITY=[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1]

def multiply(a,b):
    return [sum(a[k*4+r]*b[c*4+k] for k in range(4)) for c in range(4) for r in range(4)]

def point(m,p):
    return tuple(sum(m[c*4+r]*p[c] for c in range(3))+m[12+r] for r in range(3))

def matrix(node):
    if 'matrix' in node:return node['matrix']
    x,y,z,w=node.get('rotation',[0,0,0,1]);sx,sy,sz=node.get('scale',[1,1,1])
    tx,ty,tz=node.get('translation',[0,0,0])
    return [(1-2*y*y-2*z*z)*sx,(2*x*y+2*w*z)*sx,(2*x*z-2*w*y)*sx,0,
            (2*x*y-2*w*z)*sy,(1-2*x*x-2*z*z)*sy,(2*y*z+2*w*x)*sy,0,
            (2*x*z+2*w*y)*sz,(2*y*z-2*w*x)*sz,(1-2*x*x-2*y*y)*sz,0,tx,ty,tz,1]

class GLB:
    def __init__(self,path):
        self.path=Path(path);data=self.path.read_bytes()
        magic,version,size=struct.unpack_from('<4sII',data)
        assert magic==b'glTF' and version==2 and size==len(data),'Invalid GLB header'
        pos=12;chunks=[]
        while pos<len(data):
            n,kind=struct.unpack_from('<II',data,pos);pos+=8
            assert n%4==0 and pos+n<=len(data),'Invalid chunk size'
            chunks.append((kind,data[pos:pos+n]));pos+=n
        assert chunks[0][0]==0x4e4f534a and len(chunks)==2 and chunks[1][0]==0x004e4942
        self.doc=json.loads(chunks[0][1]);self.bin=chunks[1][1];self.cache={}
    def accessor(self,i):
        if i in self.cache:return self.cache[i]
        a=self.doc['accessors'][i];view=self.doc['bufferViews'][a['bufferView']]
        assert 'sparse' not in a,'Sparse accessor unsupported by contract validator'
        fmt={5120:'b',5121:'B',5122:'h',5123:'H',5125:'I',5126:'f'}[a['componentType']]
        count={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4,'MAT4':16}[a['type']]
        step=struct.calcsize(fmt)*count;stride=view.get('byteStride',step)
        offset=view.get('byteOffset',0)+a.get('byteOffset',0)
        assert a['count']==0 or offset+(a['count']-1)*stride+step<=len(self.bin)
        vals=[struct.unpack_from('<'+fmt*count,self.bin,offset+n*stride) for n in range(a['count'])]
        if a.get('normalized'):
            maxval={5120:127,5121:255,5122:32767,5123:65535}[a['componentType']]
            vals=[tuple(max(-1,v/maxval) for v in row) for row in vals]
        assert all(math.isfinite(x) for row in vals for x in row),'Non-finite accessor'
        self.cache[i]=vals;return vals
    def worlds(self,animation=None,time=0):
        nodes=copy.deepcopy(self.doc['nodes'])
        if animation:
            for ch in animation['channels']:
                sampler=animation['samplers'][ch['sampler']];times=self.accessor(sampler['input'])
                values=self.accessor(sampler['output'])
                idx=max(0,min(len(times)-1,bisect.bisect_right([t[0] for t in times],time)-1))
                value=values[idx]
                if idx+1<len(times) and sampler.get('interpolation','LINEAR')=='LINEAR':
                    t=max(0,min(1,(time-times[idx][0])/(times[idx+1][0]-times[idx][0])))
                    a=values[idx];b=values[idx+1]
                    if ch['target']['path']=='rotation':
                        dot=sum(x*y for x,y in zip(a,b))
                        if dot<0:b=tuple(-x for x in b);dot=-dot
                        if dot<.9995:
                            angle=math.acos(min(1,dot));denom=math.sin(angle)
                            value=tuple((x*math.sin((1-t)*angle)+y*math.sin(t*angle))/denom for x,y in zip(a,b))
                        else:
                            v=tuple(x*(1-t)+y*t for x,y in zip(a,b));length=math.sqrt(sum(x*x for x in v));value=tuple(x/length for x in v)
                    else:value=tuple(x*(1-t)+y*t for x,y in zip(a,b))
                nodes[ch['target']['node']][ch['target']['path']]=value
        worlds={}
        def walk(i,parent):
            worlds[i]=multiply(parent,matrix(nodes[i]))
            for child in nodes[i].get('children',[]):walk(child,worlds[i])
        for root in self.doc['scenes'][self.doc.get('scene',0)]['nodes']:walk(root,IDENTITY)
        return worlds
    def vertices(self,animation=None,time=0,exclude=()):
        worlds=self.worlds(animation,time)
        for ni,node in enumerate(self.doc['nodes']):
            if 'mesh' not in node or node.get('name') in exclude:continue
            joint_matrices=None
            if 'skin' in node:
                skin=self.doc['skins'][node['skin']];inv=self.accessor(skin['inverseBindMatrices'])
                joint_matrices=[multiply(worlds[j],inv[k]) for k,j in enumerate(skin['joints'])]
            for prim in self.doc['meshes'][node['mesh']]['primitives']:
                attrs=prim['attributes'];positions=self.accessor(attrs['POSITION'])
                if joint_matrices:
                    joints=self.accessor(attrs['JOINTS_0']);weights=self.accessor(attrs['WEIGHTS_0'])
                    regions=self.doc['meshes'][node['mesh']].get('extras',{}).get('vertex_regions',[])
                    for vi,(p,js,ws) in enumerate(zip(positions,joints,weights)):
                        if any(r['part'] in exclude and r['start']<=vi<r['start']+r['count'] for r in regions):continue
                        transformed=[point(joint_matrices[int(j)],p) for j in js]
                        yield ni,tuple(sum(v[k]*w for v,w in zip(transformed,ws)) for k in range(3))
                else:
                    for p in positions:yield ni,point(worlds[ni],p)

def image_dimensions(data):
    if data[:8]==b'\x89PNG\r\n\x1a\n':return (*struct.unpack_from('>II',data,16),data[24])
    if data[:4]==b'RIFF' and data[8:12]==b'WEBP':
        kind=data[12:16]
        if kind==b'VP8X':  # extended header (Blender writes it); 24-bit sizes minus one
            assert b'ANIM' not in data[:64],'Animated WebP not allowed'
            return 1+int.from_bytes(data[24:27],'little'),1+int.from_bytes(data[27:30],'little'),8
        if kind==b'VP8L':
            bits=int.from_bytes(data[21:25],'little');return 1+(bits&0x3fff),1+((bits>>14)&0x3fff),8
        assert kind==b'VP8 ','Expected a still WebP'
        width,height=struct.unpack_from('<HH',data,26);return width&0x3fff,height&0x3fff,8
    assert data[:2]==b'\xff\xd8','Invalid PNG/JPEG/WebP'
    offset=2
    while offset<len(data):
        assert data[offset]==255,'Invalid JPEG marker'
        while data[offset]==255:offset+=1
        marker=data[offset];offset+=1
        size=struct.unpack_from('>H',data,offset)[0]
        if marker in {0xc0,0xc1,0xc2}:
            bits,height,width=struct.unpack_from('>BHH',data,offset+2);return width,height,bits
        offset+=size
    raise ValueError('JPEG dimensions missing')

# Contract limits. Per-unit test rounds may raise them only through explicit CLI
# flags (agreed with the map thread); the defaults are the shipped contract.
LIMITS={'lod0':10000,'lod1':3000,'lod_total':None,'bytes':2_000_000,'joints':40}

def validate(path,limits=None):
    L=dict(LIMITS,**(limits or {}))
    g=GLB(path);d=g.doc;errors=[];warnings=[]
    def require(test,message):
        if not test:errors.append(message)
    require(Path(path).stat().st_size<L['bytes'],f"GLB must be under {L['bytes']/1e6:g} MB ({L['bytes']} bytes)")
    require(d['asset']['version']=='2.0','Must be glTF 2.0')
    require(len(d.get('buffers',[]))==1 and 'uri' not in d['buffers'][0],'Must be self-contained')
    extras=d['asset'].get('extras',{})
    # The 0 A.D. rig and clips are always credited; a fitted third-party model adds its own credit.
    require(CREDIT in d['asset'].get('copyright','') and CREDIT in extras.get('credit',''),'Missing exact credit')
    require(extras.get('license')=='CC-BY-SA-3.0','Missing licence in asset.extras')
    require(extras.get('up_axis')=='Y' and extras.get('forward')=='+Z' and extras.get('units')=='metres','Missing axis/metre declarations')
    require(Path(path).with_name('PROVENANCE.md').is_file(),'Missing adjacent PROVENANCE.md')
    require(Path(path).with_name('LICENSE-ART').is_file(),'Missing adjacent LICENSE-ART')
    nodes=d['nodes']
    skinned=[i for i,n in enumerate(nodes) if 'skin' in n]
    require(len(skinned)==2 and len(d.get('skins',[]))==1,'Must have exactly two skinned LODs sharing one skeleton')
    require(all(nodes[i]['skin']==0 for i in skinned),'Both LODs must use skin 0')
    require('MSFT_lod' not in d.get('extensionsUsed',[]) and not any('MSFT_lod' in n.get('extensions',{}) for n in nodes),'MSFT_lod is forbidden')
    joints=set(j for skin in d.get('skins',[]) for j in skin['joints'])
    require(all(len(skin['joints'])<=L['joints'] for skin in d.get('skins',[])),f"More than {L['joints']} joints")
    require(all(len(set(skin['joints']))==len(skin['joints']) for skin in d.get('skins',[])),'Duplicate joint index')
    tris=0;team_regions=[];lod_triangles={}
    require(len(d.get('meshes',[]))==2,'Must have exactly 2 meshes')
    require({m.get('name') for m in d['meshes']}=={'spearman_LOD0','spearman_LOD1'},'Expected named LOD0 and LOD1 meshes')
    require({nodes[i].get('name') for i in skinned}=={'spearman_LOD0','spearman_LOD1'},'Expected named LOD0 and LOD1 nodes')
    require(len(d.get('materials',[]))==1,'Must have exactly 1 material')
    draw_calls=sum(len(d['meshes'][n['mesh']]['primitives']) for n in nodes if 'mesh' in n)
    require(draw_calls==2 and all(len(m['primitives'])==1 for m in d['meshes']),'One primitive per LOD, two packaged draws; runtime selects one')
    for ni,node in enumerate(nodes):
        require(all(abs(v)>1e-6 for v in node.get('scale',[1,1,1])),'Zero-scaled node')
        if 'mesh' not in node:continue
        for prim in d['meshes'][node['mesh']]['primitives']:
            require(prim.get('mode',4)==4,'Only triangle primitives supported')
            count=d['accessors'][prim['indices']]['count'] if 'indices' in prim else d['accessors'][prim['attributes']['POSITION']]['count']
            tris+=count//3
            name=d['meshes'][node['mesh']]['name'];lod_triangles[name]=lod_triangles.get(name,0)+count//3
            require(prim.get('material')==0,'Both LODs must share material 0')
            require('TEXCOORD_0' in prim['attributes'],'Missing albedo UVs')
            require('COLOR_0' in prim['attributes'],'Missing team mask COLOR_0')
            if 'skin' in node:
                weights=g.accessor(prim['attributes']['WEIGHTS_0'])
                require(all(abs(sum(w)-1)<.001 for w in weights),'Weights do not sum to one')
    if L['lod_total']:require(sum(lod_triangles.values())<L['lod_total'],f"LODs together must be under {L['lod_total']} triangles: {sum(lod_triangles.values())}")
    for name,budget in [('spearman_LOD0',L['lod0']),('spearman_LOD1',L['lod1'])]:
        require(lod_triangles.get(name,0)<=budget,f'{name} triangle budget exceeded: {lod_triangles.get(name)}')
    rigid_counts={}
    # Cloak trim is required only when the unit has a cloak (a fitted model may not).
    has_cloak=any(r['part']=='cloak' or r.get('team')=='cloak trim' for m in d['meshes'] for r in m.get('extras',{}).get('vertex_regions',[]))
    required_team={'shield rim','crest'}|({'cloak trim'} if has_cloak else set())
    for mesh in d['meshes']:
        lod_team_regions=set()
        regions=mesh.get('extras',{}).get('vertex_regions',[])
        require(bool(regions),'Missing auditable vertex_regions')
        for prim in mesh['primitives']:
            attrs=prim['attributes'];count=d['accessors'][attrs['POSITION']]['count']
            colors=g.accessor(attrs['COLOR_0']) if 'COLOR_0' in attrs else []
            weights=g.accessor(attrs['WEIGHTS_0']);js=g.accessor(attrs['JOINTS_0'])
            for semantic in ['NORMAL','TANGENT']:
                require(semantic in attrs,f'Missing {semantic}')
                if semantic in attrs:require(all(abs(sum(x*x for x in row[:3])-1)<.002 for row in g.accessor(attrs[semantic])),f'{mesh["name"]} {semantic} vectors must be unit length')
            require(all(len(g.accessor(v))==count for v in attrs.values()),'Vertex attribute count mismatch')
            require(all(0<=c[0]<=1 and c[1:]==(0,0,1) for c in colors),'Invalid COLOR_0 team data')
            covered=[]
            for region in regions:
                start=region['start'];end=start+region['count']
                require(0<=start<end<=count,'Invalid vertex region range')
                if not 0<=start<end<=count:continue
                covered.extend(range(start,end))
                bone=region.get('bone');part=region['part']
                expected={'spear':'weapon_R','aspis':'shield_arm','Aspis back':'shield_arm','Aspis handgrip':'shield_arm','helmet':'helmet','crest':'helmet','crest_base':'helmet','Greek head':'head'}.get(part) or bone
                if expected:
                    require(bone==expected,f'{part} must attach to {expected}')
                    valid=True
                    for i in range(start,end):
                        active=[int(j) for j,w in zip(js[i],weights[i]) if w>1e-6]
                        valid &= len(active)==1 and max(weights[i])==1
                        valid &= all(j<len(d['skins'][0]['joints']) and nodes[d['skins'][0]['joints'][j]].get('name')==expected for j in active)
                    require(valid,f'{part} must have single-bone unit weights to {expected}')
                    rigid_counts[part]=rigid_counts.get(part,0)+end-start
                if region.get('team'):
                    team_regions.append(region['team']);lod_team_regions.add(region['team'])
                    require(bool(colors) and all(colors[i][0]==1 for i in range(start,end)),f'{part} team mask must be one')
                elif colors:require(all(colors[i][0]==0 for i in range(start,end)),f'{part} non-team mask must be zero')
            require(sorted(covered)==list(range(count)),'vertex_regions must partition every vertex exactly once')
        require(required_team<=lod_team_regions,f'{mesh["name"]} team mask must cover {", ".join(sorted(required_team))}')
    # LOD1 (normal zoom) must keep every part LOD0 has, with most of its surface:
    # decimation once silently dropped every garment from a LOD1.
    lod_area={}
    for mesh in d['meshes']:
        prim=mesh['primitives'][0];P=g.accessor(prim['attributes']['POSITION']);I=[i[0] for i in g.accessor(prim['indices'])]
        owner={}
        for r in mesh.get('extras',{}).get('vertex_regions',[]):
            for v in range(r['start'],r['start']+r['count']):owner[v]=r['part']
        area={}
        for k in range(0,len(I),3):
            a,b,c=(P[I[k+j]] for j in range(3))
            u=[b[i]-a[i] for i in range(3)];w=[c[i]-a[i] for i in range(3)]
            cr=[u[1]*w[2]-u[2]*w[1],u[2]*w[0]-u[0]*w[2],u[0]*w[1]-u[1]*w[0]]
            part=owner.get(I[k]);area[part]=area.get(part,0)+math.sqrt(sum(x*x for x in cr))/2
        lod_area[mesh.get('name')]=area
    if {'spearman_LOD0','spearman_LOD1'}<=set(lod_area):
        a0,a1=lod_area['spearman_LOD0'],lod_area['spearman_LOD1']
        missing=sorted(p for p in a0 if p not in a1)
        require(not missing,f'spearman_LOD1 lacks parts that LOD0 has: {missing}')
        # Skin under clothing may be dropped from LOD1, so 'body' needs only 30%.
        thin=sorted(p for p in a0 if p in a1 and a0[p]>1e-4 and a1[p]<(.3 if p=='body' else .5)*a0[p])
        require(not thin,f'spearman_LOD1 keeps under half the surface of: {thin}')
    require({'spear','aspis','helmet'}<=set(rigid_counts),'Missing required rigid prop regions')
    require(required_team<=set(team_regions),f'team must cover {", ".join(sorted(required_team))}')
    require(extras.get('team',{}).get('attribute')=='COLOR_0' and extras.get('team',{}).get('channel')=='r','Missing team mask semantics')
    images=d.get('images',[]);require(len(images)==3,'Exactly base colour, normal and ORM images required')
    texture_size=None;texture_images=[]
    require(extras.get('texture_master_size')==2048,'Textures must declare 2048 authoring masters')
    for im in images:
        require('bufferView' in im and im.get('mimeType') in {'image/png','image/jpeg','image/webp'},'Atlas must be embedded PNG, JPEG or WebP')
        if 'bufferView' in im:
            view=d['bufferViews'][im['bufferView']];off=view.get('byteOffset',0)
            png=g.bin[off:off+view['byteLength']]
            width,height,depth=image_dimensions(png);texture_size=(width,height);require(texture_size==(1024,1024),'Atlas must be 1024 x 1024')
            texture_images.append({'name':im.get('name'),'pixels':list(texture_size),'bytes':len(png),'bit_depth':depth,'mime_type':im['mimeType']})
    for mat in d.get('materials',[]):
        require('normalTexture' in mat and 'occlusionTexture' in mat,'Normal and AO textures required')
        pbr=mat.get('pbrMetallicRoughness',{});orm=pbr.get('metallicRoughnessTexture',{});ao=mat.get('occlusionTexture',{})
        require(orm.get('index') is not None and orm.get('index')==ao.get('index'),'Occlusion and metallic/roughness must share packed ORM')
        maps=[pbr.get('baseColorTexture',{}),mat.get('normalTexture',{}),orm,ao]
        require(all(t.get('texCoord',0)==0 for t in maps),'All maps must use UV0')
        require(len({t.get('index') for t in maps})==3,'Base colour, normal and ORM must be distinct textures')
        require('baseColorTexture' in mat.get('pbrMetallicRoughness',{}),'Every material needs the atlas')
        require('KHR_materials_unlit' not in mat.get('extensions',{}),'Material must accept runtime lighting')
        require(not any(mat.get('emissiveFactor',[0,0,0])),'No emission-projection material')
    animations=d.get('animations',[]);require(sorted(a.get('name') for a in animations)==sorted(['idle','walk','attack','hit','death']),'Exactly five named clips required')
    clips={}
    roots=set(d['scenes'][d.get('scene',0)]['nodes'])
    for a in animations:
        duration=0;loop_error=0
        for ch in a['channels']:
            sam=a['samplers'][ch['sampler']];times=[t[0] for t in g.accessor(sam['input'])];vals=g.accessor(sam['output'])
            require(sam.get('interpolation','LINEAR')=='LINEAR','Clips must be linearly sampled')
            require(abs(times[0])<1e-5 and all(abs(t*30-round(t*30))<1e-4 for t in times),'Animation samples must lie on 30 fps grid')
            require(all(abs(b-a-1/30)<1e-5 for a,b in zip(times,times[1:])),'Animation must include every 30 fps sample')
            duration=max(duration,times[-1])
            if ch['target']['node'] in roots:
                require(all(max(abs(x-y) for x,y in zip(v,vals[0]))<1e-5 for v in vals),'Root motion is forbidden')
            if ch['target']['path']=='scale':require(all(min(v)>1e-6 for v in vals),'Animated zero scale')
            diff=max(abs(x-y) for x,y in zip(vals[0],vals[-1]))
            if ch['target']['path']=='rotation':diff=min(diff,max(abs(x+y) for x,y in zip(vals[0],vals[-1])))
            loop_error=max(loop_error,diff)
        if a['name'] in {'idle','walk'}:require(loop_error<1e-4,f'{a["name"]} loop endpoint mismatch: {loop_error}')
        clips[a['name']]={'duration_s':round(duration,6),'fps':30,'loop_endpoint_max_error':round(loop_error,7)}
    if any(len(skin['joints'])!=d['accessors'][skin['inverseBindMatrices']]['count'] for skin in d.get('skins',[])):
        errors.append('Inverse-bind count must equal joint count')
        return {'valid':False,'errors':errors}
    idle=next((a for a in animations if a['name']=='idle'),None)
    pts=[p for _,p in g.vertices(idle,0,exclude=('spear',))]
    minimum=[min(p[i] for p in pts) for i in range(3)];maximum=[max(p[i] for p in pts) for i in range(3)]
    height=maximum[1]-minimum[1]
    require(1.65<=height<=1.95,f'Figure must be about 1.8m: {height:.3f}')
    bodypts=[p for ni,p in g.vertices(idle,0) if ni in skinned]
    sole_height=min(v[1] for v in bodypts)
    feet=[p for p in bodypts if p[1]<sole_height+.035]
    feet_centre=[(min(p[i] for p in feet)+max(p[i] for p in feet))/2 for i in range(3)]
    require(abs(min(p[1] for p in feet))<.02,'Feet must sit at y=0')
    require(abs(feet_centre[0])<.15 and abs(feet_centre[2])<.15,'Origin must be at feet centre')
    # Mesh toes establish +Z, independently of metadata (front-most low vertex).
    require(max(p[2] for p in feet)>abs(min(p[2] for p in feet)), 'Toe geometry must face +Z')
    warnings.append('Painted quality, surface detail, exact team coverage, collisions and game feel require visual review; runtime must display only one LOD.')
    return {'valid':not errors,'glb':str(Path(path).resolve()),'bytes':Path(path).stat().st_size,'triangles':tris,'lod_triangles':lod_triangles,'joints':len(joints),'skinned_meshes':len(skinned),'atlas_px':texture_size,'texture_images':texture_images,'figure_height_m':round(height,4),'feet_centre_m':[round(v,4) for v in feet_centre],'meshes':len(d['meshes']),'materials':len(d['materials']),'packaged_primitives':draw_calls,'draw_calls_per_lod':1,'team_regions':sorted(set(team_regions)),'lod_part_area_m2':{k:{p:round(v,4) for p,v in a.items()} for k,a in lod_area.items()},'rigid_vertices':rigid_counts,'clips':clips,'errors':errors,'review_required':warnings}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('glb',type=Path);p.add_argument('--output',type=Path)
    p.add_argument('--lod0-max',type=int,help='test-round override of the LOD0 triangle budget (contract: 10000)')
    p.add_argument('--lod-total-max',type=int,help='test-round cap on LOD0+LOD1 triangles')
    p.add_argument('--max-bytes',type=int,help='test-round GLB size cap (contract: 2000000)')
    p.add_argument('--max-joints',type=int,help='test-round joint cap (contract: 40)')
    args=p.parse_args()
    limits={k:v for k,v in {'lod0':args.lod0_max,'lod_total':args.lod_total_max,'bytes':args.max_bytes,'joints':args.max_joints}.items() if v}
    result=validate(args.glb,limits);result['limits']=dict(LIMITS,**limits);out=json.dumps(result,indent=2);print(out)
    if args.output:args.output.write_text(out+'\n')
    raise SystemExit(0 if result['valid'] else 1)
