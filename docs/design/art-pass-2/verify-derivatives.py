from pathlib import Path
from PIL import Image
import json
R=Path(__file__).resolve().parents[3];O=R/'docs/design/art-pass-2';changes=json.loads((O/'changed.json').read_text()); errors=[];count=0
for x in changes:
 s=x['slug'];k=x['kind'];checks=[]
 if k=='unit':checks=[(R/'app/src/assets/units'/(s+suffix+'.png'),(n,n)) for suffix,n in [('-source',1254),('-256',256),('-64',64),('-48',48),('-panel',512),('-card',64)]]
 elif k=='portrait':checks=[(R/'app/src/assets/art'/(s+suffix),(n,n)) for suffix,n in [('.png',1024),('-512.png',512),('-360.jpg',360)]]
 elif k=='construction':checks=[(R/'app/src/assets/construction'/(s+suffix+'.png'),(n,n)) for suffix,n in [('',512),('-128',128),('-64',64)]]
 elif k=='city':checks=[(R/'app/src/assets/cities'/(s+suffix+'.png'),(n,n)) for suffix,n in [('',512),('-128',128),('-64',64)]]
 elif k=='era':checks=[(R/'app/src/assets/eras'/(s+suffix),size) for suffix,size in [('.png',(1920,1080)),('-1920.jpg',(1920,1080)),('-1280.png',(1280,720)),('-1280.jpg',(1280,720)),('-portrait.png',(720,1280))]]
 elif k=='crest':checks=[(R/'app/src/assets/heraldry/raster'/(s+suffix+'.png'),(n,n)) for suffix,n in [('',1024),('-512',512)]]
 elif k=='unlock':
  checks=[(R/'app/src/assets/unlocks'/(s+f'-{n}.png'),(n,n)) for n in [256,128,64]]
  checks.append((R/'docs/design/unlock-art/masters'/(s+'.png'),(1254,1254)))
  if s.startswith('improvement-'):checks.extend([(R/'app/src/assets/improvements'/(s.removeprefix('improvement-')+suffix+'.png'),(n,n)) for suffix,n in [('',128),('-64',64)]])
 for p,size in checks:
  count+=1
  try:
   im=Image.open(p);im.load()
   if im.size!=size:errors.append(f'{p}: {im.size} != {size}')
  except Exception as e:errors.append(str(e))
print(count,'image derivatives decoded and dimension-checked');print('\n'.join(errors) if errors else 'PASS')
(O/'derivative-check.txt').write_text(f'{count} image derivatives decoded and dimension-checked\n'+('\n'.join(errors) if errors else 'PASS')+'\n')
if errors:raise SystemExit(1)
