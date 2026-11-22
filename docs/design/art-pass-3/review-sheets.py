"""Portable review sheets: before images come from the recorded calendar commit."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import importlib.util, io, json, subprocess, textwrap
R=Path(__file__).resolve().parents[3];O=Path(__file__).resolve().parent
BASE='a6ae2f29b';U=R/'app/src/assets/units'
sp=importlib.util.spec_from_file_location('ua',R/'tools/art/unit-art.py');ua=importlib.util.module_from_spec(sp);sp.loader.exec_module(ua)
rows=json.loads((O/'inventory.json').read_text());font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',22);small=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',14)
paper=(234,227,211);ink=(44,39,32)
def before(row):
    path=row['source']
    return Image.open(io.BytesIO(subprocess.check_output(['git','show',BASE+':'+path],cwd=R))).convert('RGBA')
def paste(sheet,im,xy): sheet.paste(im,xy,im)
# Actual 48px rasters, no enlargement, all 54 on one portrait sheet.
sheet=Image.new('RGB',(600,950),paper);d=ImageDraw.Draw(sheet);d.text((16,12),'All 54 units — actual 48 px',font=font,fill=ink)
for i,row in enumerate(rows):
    s=row['slug'];x=i%6*100;y=60+i//6*98
    paste(sheet,Image.open(U/(s+'-48.png')).convert('RGBA'),(x+26,y))
    d.multiline_text((x+4,y+51),'\n'.join(textwrap.wrap(s.replace('-',' '),13)),font=small,fill=ink,spacing=0)
sheet.save(O/'all-units-48.png')
# Nine short phone pages; six units per page, enlarged review and actual map sizes.
for page in range(9):
    sh=Image.new('RGB',(720,1550),paper);d=ImageDraw.Draw(sh)
    d.text((20,14),f'Unit bases / figures — {page+1} of 9',font=font,fill=ink)
    d.text((154,48),'Before',font=font,fill=ink);d.text((464,48),'After',font=font,fill=ink)
    for j,row in enumerate(rows[page*6:(page+1)*6]):
        s=row['slug'];y=90+j*242;d.text((20,y),s.replace('-',' ').title(),font=font,fill=ink)
        old=before(row);new=Image.open(U/(s+'-source.png')).convert('RGBA')
        for k,im in enumerate([old,new]):
            large=ua.fit(im,185,200);paste(sh,large,(85+k*330,y+32))
            tiny=ua.fit(ua.fit(im,232,256),42,48);paste(sh,tiny,(285+k*330,y+130))
        d.line((20,y+238,700,y+238),fill=(202,192,171))
    sh.save(O/f'before-after-{page+1:02d}.png')
# Final contact verification: source feet/tracks/base, six pieces per portrait page.
for page in range(9):
    sh=Image.new('RGB',(720,1430),paper);d=ImageDraw.Draw(sh)
    for j,row in enumerate(rows[page*6:(page+1)*6]):
        s=row['slug'];im=Image.open(U/(s+'-source.png')).convert('RGBA');box=im.getchannel('A').getbbox();bottom=box[3]
        crop=im.crop((box[0],max(box[1],bottom-round((box[3]-box[1])*.30)),box[2],bottom));crop.thumbnail((690,195))
        y=j*238;d.text((16,y+3),s,font=font,fill=ink);paste(sh,crop,((720-crop.width)//2,y+35))
    sh.save(O/'crops'/f'final-contacts-{page+1:02d}.png')
print('54 map sprites, 9 before/after pages, 9 contact-crop pages.')
