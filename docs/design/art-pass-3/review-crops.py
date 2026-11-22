from pathlib import Path
from PIL import Image, ImageDraw
import json, sys
O=Path(__file__).resolve().parent
rows=json.loads((O/'generations.json').read_text())
start=int(sys.argv[1]); stop=int(sys.argv[2])
sh=Image.new('RGB',(1000,(stop-start)*240),(239,231,212)); d=ImageDraw.Draw(sh)
for i,r in enumerate(rows[start:stop]):
 im=Image.open(r['path']).convert('RGBA');w,h=im.size
 crop=im.crop((0,int(h*.58),w,h));crop.thumbnail((690,210))
 sh.paste(crop,(290,i*240+25),crop);d.text((10,i*240+10),r['slug'],fill='black')
 im.thumbnail((200,200));sh.paste(im,(40,i*240+30),im)
sh.save(O/'crops'/f'bases-{start:02d}-{stop:02d}.png')
