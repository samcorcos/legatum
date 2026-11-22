from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
import json,math
R=Path(__file__).resolve().parents[3];O=R/'docs/design/art-pass-2';data=json.loads((O/'changed.json').read_text());inv=json.loads((O/'inventory.json').read_text())
FONT=next(str(p) for p in [Path('/System/Library/Fonts/Supplemental/Arial.ttf'),Path('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf')] if p.exists())
def f(n):return ImageFont.truetype(FONT,n)
bg=(239,231,212)
def paste(sheet,path,box):
 im=Image.open(path).convert('RGBA');im.thumbnail((box[2],box[3]),Image.Resampling.LANCZOS);sheet.paste(im,(box[0]+(box[2]-im.width)//2,box[1]+(box[3]-im.height)//2),im)
def target(e):return O/'vector-after'/(e['slug']+'.png') if e['kind']=='icon' else R/e['target']
def neighbors(e):
 kind=e['kind'];s=e['slug']
 if kind in ['unit','portrait']:
  xs=[(x['slug'],R/('app/src/assets/units/'+x['slug']+'-256.png') if kind=='unit' else R/x['source']) for x in inv if x['slug'].startswith('people-')==(kind=='portrait')]
 elif kind=='construction':
  era=s.split('-hamlet-stage')[0];xs=[(x['slug'],target(x)) for x in data if x['kind']=='construction' and x['slug'].startswith(era+'-')]+[(era+'-hamlet',R/'app/src/assets/cities'/(era+'-hamlet.png'))]
 elif kind=='city':xs=[(p.stem,p) for p in sorted((R/'app/src/assets/cities').glob('*.png')) if p.stem.endswith(('hamlet','town','city','metropolis'))]
 elif kind=='unlock':xs=[(p.name.removesuffix('-256.png'),p) for p in sorted((R/'app/src/assets/unlocks').glob('*-256.png'))]
 elif kind=='era':xs=[(x,R/'app/src/assets/eras'/(x+'.png')) for x in ['ancient','classical','medieval','renaissance','industrial','machine','modern','contemporary','future']]
 elif kind=='crest':xs=[(p.stem,p) for p in sorted((R/'app/src/assets/heraldry/raster').glob('crest-*.png')) if not p.stem.endswith('-512')]
 else:xs=[(x['slug'],target(x)) for x in data if x['kind']=='icon']
 idx=next((i for i,x in enumerate(xs) if x[0]==s),0);return [xs[(idx+i)%len(xs)] for i in range(1,4)]
(O/'pairs').mkdir(exist_ok=True)
for n,e in enumerate(data):
 sheet=Image.new('RGB',(560,390),bg);d=ImageDraw.Draw(sheet);d.text((12,8),f'{n+1:02d}  {e["slug"]}',font=f(20),fill='#2b2419');d.text((12,36),'BEFORE'+(' (fallback)' if e.get('before','').startswith('previous') else ''),font=f(14),fill='#555');d.text((292,36),'AFTER',font=f(14),fill='#555');paste(sheet,O/'before'/(e['slug']+'.png'),(12,56,256,256));paste(sheet,target(e),(292,56,256,256));d.text((12,325),'48 px',font=f(13),fill='#444');paste(sheet,target(e),(70,325,48,48));
 for i,(slug,path) in enumerate(neighbors(e)):
  paste(sheet,path,(190+i*120,325,48,48));d.text((180+i*120,375),slug[:18],font=f(9),fill='#444')
 sheet.save(O/'pairs'/(e['slug']+'.png'))
for start in range(0,len(data),8):
 group=data[start:start+8];sh=Image.new('RGB',(560,45+390*len(group)),bg);d=ImageDraw.Draw(sh);d.text((12,10),f'Art pass 2 — group {start//8+1} / {math.ceil(len(data)/8)}',font=f(22),fill='#2b2419')
 for j,e in enumerate(group):sh.paste(Image.open(O/'pairs'/(e['slug']+'.png')),(0,45+j*390))
 sh.save(O/f'contact-{start//8+1:02d}.png')
# index: thumbnail and change identifier; detailed findings live in Markdown
sh=Image.new('RGB',(560,100+len(data)*76),bg);d=ImageDraw.Draw(sh);d.text((14,12),'Legatum · art pass 2',font=f(27),fill='#2b2419');d.text((14,49),f'{len(data)} changed pieces · 256 px pairs + 48 px neighbour rows',font=f(16),fill='#444')
for i,e in enumerate(data):
 y=95+i*76;paste(sh,target(e),(14,y,64,64));d.text((93,y+7),f'{i+1:02d}  {e["slug"]}',font=f(18),fill='#2b2419');d.text((93,y+33),f'{e["kind"]} · contact-{i//8+1:02d}.png',font=f(14),fill='#555')
sh.save(O/'index.png')
# actual-size review, each figure in consecutive row with neighbours
for kind in ['unit','portrait']:
 xs=[x for x in inv if x['slug'].startswith('people-')==(kind=='portrait')];sh=Image.new('RGB',(560,70+90*math.ceil(len(xs)/6)),bg);d=ImageDraw.Draw(sh);d.text((12,10),f'All {len(xs)} {kind}s · native 48 px',font=f(22),fill='#2b2419')
 for i,x in enumerate(xs):
  y=60+(i//6)*90;xx=8+(i%6)*92;path=R/x['source'] if kind=='portrait' else R/'app/src/assets/units'/(x['slug']+'-256.png');paste(sh,path,(xx+18,y,48,48));d.text((xx,y+53),x['slug'].replace('people-','')[:18],font=f(9),fill='#444')
 sh.save(O/f'all-{kind}s-48.png')
# SVG reviewer: actual 16,24,48 and 128 for geometry
icons=[x for x in data if x['kind']=='icon'];sh=Image.new('RGB',(560,80+130*len(icons)),bg);d=ImageDraw.Draw(sh);d.text((12,10),'Vector replacements · 16 / 24 / 48 / 128',font=f(21),fill='#2b2419')
for i,x in enumerate(icons):
 y=60+i*130;d.text((12,y),x['slug'],font=f(16),fill='#2b2419')
 for n,xx in [(16,20),(24,70),(48,130),(128,300)]:paste(sh,target(x),(xx,y+25,n,n))
sh.save(O/'icons-review.png')
print(len(data),'changed',len(inv),'figures')
