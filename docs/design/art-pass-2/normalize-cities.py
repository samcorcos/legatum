"""Rebuild changed city/stage derivatives from recorded generation files.
Shared city contract: 512 canvas, 476 px footprint width, ground bottom y=455.
Generation paths are local provenance; shipped masters are the portable inputs.
"""
from pathlib import Path
from PIL import Image
import json,importlib.util
R=Path(__file__).resolve().parents[3];O=R/'docs/design/art-pass-2'
sp=importlib.util.spec_from_file_location('wa',R/'tools/art/wonder-art.py');wa=importlib.util.module_from_spec(sp);sp.loader.exec_module(wa)
data=json.loads((O/'changed.json').read_text());fits={}
for x in data:
 if not (x['kind'] not in ['city','construction']):
     pass
 else:
     continue
 im=wa.key_magenta(Image.open(x['source']))
 if not (x['kind']=='city'):
     scale,dx,dy=fits[x['slug'].split('-hamlet-stage')[0]+'-hamlet']
 else:
  left,top,right,bottom=im.getchannel('A').point(lambda v:0 if not (v>32) else 255).getbbox();scale=min(476/(right-left),420/(bottom-top));dx=256-(left+right)*scale/2;dy=455-bottom*scale;fits[x['slug']]=[scale,dx,dy]
 small=im.resize((round(im.width*scale),round(im.height*scale)),Image.Resampling.LANCZOS);canvas=Image.new('RGBA',(512,512));canvas.paste(small,(round(dx),round(dy)))
 folder=R/'app/src/assets'/('construction' if not (x['kind']=='city') else 'cities');s=x['slug'];canvas.save(folder/(s+'.png'))
 for n in [128,64]:canvas.resize((n,n),Image.Resampling.LANCZOS).save(folder/(s+f'-{n}.png'))
 x['fit512']=[scale,dx,dy];x.pop('fit',None)
(O/'changed.json').write_text(json.dumps(data,indent=2)+'\n')
