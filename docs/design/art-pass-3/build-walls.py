"""Key generated wall rings and apply each master's SAME transform to its stages."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import importlib.util, json
R=Path(__file__).resolve().parents[3]; O=Path(__file__).resolve().parent
sp=importlib.util.spec_from_file_location('wa',R/'tools/art/wonder-art.py')
wa=importlib.util.module_from_spec(sp);sp.loader.exec_module(wa)
walls=json.loads((O/'walls.json').read_text()); stages=json.loads((O/'stages.json').read_text())
font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',24)
def keyed(path):
    im=Image.open(path).convert('RGBA')
    bg=Image.new('RGBA',im.size,(255,0,255,255));bg.alpha_composite(im)
    return wa.key_magenta(bg.convert('RGB'))
def place(im,fit):
    scale,dx,dy=fit
    im=im.resize((round(im.width*scale),round(im.height*scale)),Image.Resampling.LANCZOS)
    out=Image.new('RGBA',(512,512));out.paste(im,(round(dx),round(dy)))
    return out
for w in walls:
    im=keyed(w['path']);x0,y0,x1,y1=wa.bbox(im)
    scale=476/(x1-x0);fit=[scale,256-(x0+x1)*scale/2,454-y1*scale];w['fit512']=fit
    frames=[]
    for n in [1,2,3,4]:
        row=w if n==4 else next(x for x in stages if x['era']==w['era'] and x['stage']==n)
        stem=w['era']+'-walls'+('' if n==4 else f'-stage{n}')
        folder=R/'app/src/assets'/('cities' if n==4 else 'construction')
        canvas=place(keyed(row['path']),fit);frames.append(canvas)
        canvas.save(folder/(stem+'.png'),optimize=True)
        for size in [128,64]:canvas.resize((size,size),Image.Resampling.LANCZOS).save(folder/(stem+f'-{size}.png'),optimize=True)
        correction=w.get('correctionPrompt','') if n==4 else ''
        (folder/(stem+'.prompt.txt')).write_text((row['prompt']+'\n'+correction).rstrip()+'\n')
    # Phone portrait: stages and finished at 360px, plus exact 128px composites.
    sheet=Image.new('RGB',(720,1520),(230,222,204));draw=ImageDraw.Draw(sheet)
    draw.text((20,10),w['era'].title()+' walls: registered stages',font=font,fill=(45,40,32))
    city=Image.open(R/f"app/src/assets/cities/{w['era']}-city.png").convert('RGBA')
    for i,frame in enumerate(frames):
        x=i%2*360;y=60+i//2*430
        draw.text((x+16,y),['Footings','Half-built','Roofless shell','Finished'][i],font=font,fill=(45,40,32))
        crop=frame.crop((0,160,512,470)).resize((360,218),Image.Resampling.LANCZOS)
        sheet.paste(crop,(x,y+42),crop)
        composite=city.copy();composite.alpha_composite(frame)
        small=composite.resize((128,128),Image.Resampling.LANCZOS);sheet.paste(small,(x+116,y+275),small)
    draw.text((20,960),'Finished beside city / exact 128 px above',font=font,fill=(45,40,32))
    composite=city.copy();composite.alpha_composite(frames[-1]);sheet.paste(composite,(104,990),composite)
    sheet.save(O/(w['era']+'-wall-stages.png'))
(O/'walls.json').write_text(json.dumps(walls,indent=2)+'\n')
print('Wrote 4 wall masters, 12 stages, 32 derivatives and 4 review sheets.')
