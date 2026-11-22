from pathlib import Path
from PIL import Image, ImageDraw, ImageFont, ImageFilter

ROOT = Path(__file__).parent
NAMES = ['wheat', 'cattle', 'stone', 'copper', 'iron', 'coal']
SOURCES = {
    'painted-medallions': ('sources/painted-medallions-sheet.png', 'circle'),
    'natural-objects': ('sources/natural-objects-sheet.png', 'object'),
    'illuminated': ('sources/illuminated-sheet.png', 'panel'),
}

def cell(src, i):
    x, y = (i % 3) * 512, (i // 3) * 512
    im = src.crop((x, y, x+512, y+512)).convert('RGBA')
    mode = list(SOURCES.values())[list(SOURCES.keys()).index(current)][1]
    if mode == 'circle':
        # Keep the full painted field inside the thin medallion rim; clear the surrounding sheet.
        mask = Image.new('L', (512,512), 0)
        ImageDraw.Draw(mask).ellipse((7,7,505,505), fill=255)
        im.putalpha(mask)
    elif mode == 'panel':
        # The manuscript frame defines the miniature; preserve its vellum interior.
        mask = Image.new('L', (512,512), 0)
        ImageDraw.Draw(mask).rounded_rectangle((23,22,489,491), radius=54, fill=255)
        im.putalpha(mask)
    else:
        # The generator supplied a vignetted brown ground. Retain each object and its soft
        # ground shadow inside a generous, feathered silhouette, clearing only the field.
        silhouettes = [
            [(176,18),(252,45),(322,25),(407,81),(447,145),(425,220),(387,285),(358,351),(337,430),(300,484),(194,484),(151,448),(130,372),(98,308),(75,235),(90,161),(126,91)],
            [(51,265),(64,196),(90,134),(156,93),(228,90),(266,67),(286,38),(307,35),(325,74),(359,80),(398,101),(441,145),(450,221),(430,284),(414,390),(392,478),(343,483),(325,395),(298,330),(267,393),(259,475),(208,481),(187,391),(156,335),(131,475),(88,480),(80,389)],
            [(100,158),(153,99),(371,87),(439,130),(448,274),(429,338),(453,414),(405,453),(318,446),(267,427),(184,446),(105,424),(73,363)],
            [(62,278),(81,205),(123,150),(172,111),(224,86),(282,82),(334,100),(390,137),(438,178),(456,254),(423,330),(394,409),(336,457),(258,474),(175,457),(111,417),(68,358)],
            [(28,290),(48,218),(75,160),(127,108),(204,83),(273,110),(329,198),(491,205),(492,267),(377,279),(388,329),(414,363),(398,429),(346,478),(273,474),(214,434),(151,465),(82,440),(43,384)],
            [(45,321),(54,260),(92,210),(152,191),(202,117),(265,73),(330,97),(372,160),(444,170),(478,218),(474,287),(430,343),(456,397),(414,458),(348,481),(280,447),(220,480),(145,455),(92,418),(53,377)],
        ]
        mask = Image.new('L', (512,512), 0)
        ImageDraw.Draw(mask).polygon(silhouettes[i], fill=255)
        mask = mask.filter(ImageFilter.GaussianBlur(7))
        # Crisp-enough alpha at the edge while preserving the generated shadow inside.
        im.putalpha(mask)
    return im

# Build master/derivatives.
for current, (source, _) in SOURCES.items():
    sheet = Image.open(ROOT/source).convert('RGB')
    for i, name in enumerate(NAMES):
        icon = cell(sheet, i)
        # Crop transparent margin to the visual boundary, then square-pad with a small buffer.
        bbox = icon.getchannel('A').getbbox()
        icon = icon.crop(bbox)
        side = max(icon.size)
        square = Image.new('RGBA', (side, side), (0,0,0,0))
        square.alpha_composite(icon, ((side-icon.width)//2, (side-icon.height)//2))
        square = square.resize((256,256), Image.Resampling.LANCZOS)
        target = ROOT/current
        target.mkdir(exist_ok=True)
        square.save(target/f'{name}-256.png', optimize=True)
        for size in (64,32):
            square.resize((size,size), Image.Resampling.LANCZOS).save(target/f'{name}-{size}.png', optimize=True)

# Portrait comparison sheet, based on a real screen-tour map crop.
W = 1080
BG = '#eee4cd'; INK = '#382f24'; MUTED = '#756a57'; RULE = '#c4b596'
font_path = '/System/Library/Fonts/Supplemental/Georgia.ttf'
font_bold = '/System/Library/Fonts/Supplemental/Georgia Bold.ttf'
def font(size, bold=False):
    path = font_bold if bold and Path(font_bold).exists() else font_path
    try: return ImageFont.truetype(path, size)
    except OSError: return ImageFont.load_default()

def label(draw, xy, text, size, fill=INK, bold=False):
    draw.text(xy, text, font=font(size,bold), fill=fill)

# Full source tour screenshot, crop around its playable map rather than UI panels.
back = Image.open(ROOT.parent.parent.parent/'app/test-results/screen-tour/1000x600/05d-eight-empires-first-turn.png').convert('RGB')
map_crop = back.crop((335,160,625,420))
# A six-hex arrangement; positions sit over tiles in the actual map screenshot.
spots = [(105,60),(170,60),(235,60),(72,143),(137,143),(202,143)]
# Use a broad, portrait canvas with a per-direction band.
H = 180 + 3*1041
out = Image.new('RGB',(W,H),BG)
d = ImageDraw.Draw(out)
label(d,(48,34),'RESOURCE ART EXPLORATION',34,INK,True)
label(d,(48,82),'Three directions · six deposits · Atlas map context',20,MUTED)
d.line((48,126,W-48,126),fill=RULE,width=2)
y=150
for current in SOURCES:
    label(d,(48,y),{'painted-medallions':'A  Painted medallions','natural-objects':'B  Naturalistic objects','illuminated':'C  Illuminated miniatures'}[current],30,INK,True)
    label(d,(48,y+42),'256 px masters · reduced here to a 3 × 2 resource set',17,MUTED)
    icons=[Image.open(ROOT/current/f'{name}-256.png').convert('RGBA') for name in NAMES]
    grid_y=y+82
    for i,(name,icon) in enumerate(zip(NAMES,icons)):
        col,row=i%3,i//3
        x=52+col*334; yy=grid_y+row*220
        # a restrained paper tile helps judge silhouette and painterly detail
        d.rounded_rectangle((x,yy,x+308,yy+190),radius=12,fill='#f6efdf',outline=RULE,width=2)
        icon=icon.resize((142,142),Image.Resampling.LANCZOS)
        out.paste(icon,(x+(308-142)//2,yy+4),icon)
        label(d,(x+16,yy+151),name.title(),17,INK)
    map_y=grid_y+455
    label(d,(48,map_y),'NORMAL ZOOM · 36 px art',18,INK,True)
    label(d,(560,map_y),'FAR ZOOM · simple mark tier',18,INK,True)
    # Same authentic screenshot crop for both views; normal retains 36 px paintings.
    crop=map_crop.resize((470,424),Image.Resampling.LANCZOS)
    out.paste(crop,(48,map_y+32))
    normal_icons=[Image.open(ROOT/current/f'{n}-256.png').convert('RGBA').resize((36,36),Image.Resampling.LANCZOS) for n in NAMES]
    for (sx,sy),icon in zip(spots,normal_icons):
        out.paste(icon,(48+sx-18,map_y+32+sy-18),icon)
    # Simulated wider zoom using the same map screenshot downscaled into a larger field.
    far = Image.new('RGB',(470,260), '#e7dcc1')
    far_map=map_crop.resize((280,252),Image.Resampling.LANCZOS)
    far.paste(far_map,(95,4))
    out.paste(far,(560,map_y+32))
    # High-contrast category dots echo resourceIcons.ts's far tier.
    dots=[('#8f8b54','#473c2d'),('#8f8b54','#473c2d'),('#8f8b54','#473c2d'),('#a74732','#473c2d'),('#a74732','#473c2d'),('#a74732','#473c2d')]
    farspots=[(180,58),(215,58),(250,58),(163,120),(198,120),(233,120)]
    fd=ImageDraw.Draw(out)
    for (sx,sy),(fill,edge) in zip(farspots,dots):
        fd.ellipse((560+sx-8,map_y+32+sy-8,560+sx+8,map_y+32+sy+8),fill=fill,outline=edge,width=2)
    y=map_y+472
    d.line((48,y,W-48,y),fill=RULE,width=2)
    y+=32
label(d,(48,H-58),'Concept art only · generated raster exploration · shipped SVG icons unchanged',16,MUTED)
out.save(ROOT/'resource-art-variants.png',optimize=True)
