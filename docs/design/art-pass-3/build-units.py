"""Build reviewed image edits with the existing unit-art pipeline."""
from pathlib import Path
import importlib.util, json, sys
from PIL import Image, ImageChops, ImageFilter
R=Path(__file__).resolve().parents[3]
O=R/'docs/design/art-pass-3'
spec=importlib.util.spec_from_file_location('unit_art',R/'tools/art/unit-art.py')
unit=importlib.util.module_from_spec(spec);spec.loader.exec_module(unit)
rows=json.loads((O/'generations.json').read_text())
standard_key=unit.key_edge
def thin_key(raw):
 # Preserve the painted arrow/pike itself: a 5px minimum filter erases a 4px shaft.
 # Same saturation key, no morphological thickening or erosion; despill only edges.
 r,g,b=raw.convert('RGB').split()
 m=ImageChops.subtract(ImageChops.darker(r,b),g)
 a=m.point(lambda v:255 if v<50 else max(0,int(255-(v-50)*255/40)))
 boundary=ImageChops.subtract(a,a.filter(ImageFilter.MinFilter(3))).point(lambda v:1 if v else 0)
 edge=ImageChops.lighter(boundary,a.point(lambda v:1 if 0<v<255 else 0))
 spill=ImageChops.multiply(m,edge)
 return Image.merge('RGBA',(ImageChops.subtract(r,spill),g,ImageChops.subtract(b,spill),a))
for row in rows:
 s=row['slug']
 if len(sys.argv)>1 and s not in sys.argv[1:]:continue
 im=Image.open(row['path']).convert('RGBA')
 bg=Image.new('RGBA',im.size,(255,0,255,255));bg.alpha_composite(im)
 raw=Path('/tmp/legatum-art-pass-3')/(s+'-selected.png');bg.convert('RGB').save(raw)
 unit.key_edge=thin_key if s in ['barbarian-archer','pikeman','barbarian-pikeman','phalangite','legionary'] else standard_key
 unit.build(str(raw),s,edge_key=True)
