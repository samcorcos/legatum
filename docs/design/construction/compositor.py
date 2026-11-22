"""Concept frames for the construction proposal, round 2 of the compositor.

A: the Great Library's site at 0, 15, 45, 80 and 100% on a real 2x map frame (Bellinzona, T151), plus the finish beat
   and the standing landmark (real render). The landmark box is found by trying the six neighbour hexes against the
   real render. Stages are MOCKED: the finished painting cut at the construction line with a fresh-course band, and the
   site grammar in ink (stakes and string, stock, ramp, the crane beside the hall, hugging scaffold frames, workers,
   dust). The painted stages (Codex task) replace the cut.
B: Walls built by section round Winterthur (T63): the real wall overlay clipped by angle from the town centre,
   clockwise from the gate, with the string line ahead and a scaffold at the live end. No new art needed for this one.

usage: python3 -I composite2.py STILLS_DIR MASTER_PNG OUT_DIR
"""
import json, math, sys, random, textwrap
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter, ImageFont, ImageChops, ImageOps, ImageStat

stills, master_path, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
out.mkdir(parents=True, exist_ok=True)
INK = (43, 36, 25); INK2 = (91, 80, 63); RED = (154, 51, 34); GOLD = (128, 98, 13); VELLUM = (247, 241, 227); PAPER = (242, 234, 216)
TIMBER = (107, 74, 43); STONE = (214, 198, 160); EARTH = (176, 146, 102); EARTH_D = (132, 104, 66)
DPR = 2
F = '/System/Library/Fonts/Supplemental/Baskerville.ttc'
try:
    FONT = ImageFont.truetype(F, 30); FONT_S = ImageFont.truetype(F, 23); FONT_CAP = ImageFont.truetype(F, 22); FONT_T = ImageFont.truetype(F, 26)
except OSError:
    FONT = FONT_S = FONT_CAP = FONT_T = ImageFont.load_default()
rng = random.Random(7)

def hexpx_of(zoom): return 36 * zoom * DPR
NEIGHBOURS = lambda h: [(h * math.sqrt(3), 0), (h * math.sqrt(3) / 2, h * 1.5), (-h * math.sqrt(3) / 2, h * 1.5), (-h * math.sqrt(3), 0), (-h * math.sqrt(3) / 2, -h * 1.5), (h * math.sqrt(3) / 2, -h * 1.5)]

def caption_panel(frame, centre, size, title, sub):
    W, H = size
    crop = frame.crop((centre[0] - W // 2, centre[1] - H // 2, centre[0] + W // 2, centre[1] + H // 2)).convert('RGB')
    lines = textwrap.wrap(sub, 82)[:2]
    band = Image.new('RGB', (W, 112), PAPER)
    d = ImageDraw.Draw(band)
    d.line((0, 0, W, 0), fill=INK2, width=2)
    d.text((22, 12), title, font=FONT, fill=INK)
    for i, ln in enumerate(lines): d.text((22, 52 + i * 27), ln, font=FONT_S, fill=INK2)
    p = Image.new('RGB', (W, H + 112), PAPER)
    p.paste(crop, (0, 0)); p.paste(band, (0, H))
    return p

def sheet(panels, title, cols, name):
    pw, ph = panels[0].size
    gap = 16
    rows = math.ceil(len(panels) / cols)
    W = cols * pw + (cols + 1) * gap; H = rows * ph + (rows + 1) * gap + 64
    s = Image.new('RGB', (W, H), VELLUM)
    d = ImageDraw.Draw(s)
    d.text((gap, 16), title, font=FONT_T, fill=INK)
    for i, p in enumerate(panels):
        r, c = divmod(i, cols)
        s.paste(p, (gap + c * (pw + gap), 64 + gap + r * (ph + gap)))
    s.save(out / name)
    print('wrote', out / name, s.size)

def dust(d, cx, cy, n=3, r=10, alpha=150):
    for i in range(n):
        x = cx + rng.uniform(-r * 2, r * 2); y = cy + rng.uniform(-r * 0.6, r * 0.6)
        rr = r * rng.uniform(0.6, 1.1)
        d.ellipse((x - rr, y - rr * 0.7, x + rr, y + rr * 0.7), fill=PAPER + (alpha,), outline=INK + (110,), width=1)

def workers(d, n, cx, cy, spread=60):
    for i in range(n):
        x = cx + rng.uniform(-spread, spread); y = cy + rng.uniform(-6, 6)
        d.line((x, y, x, y - 6), fill=INK + (255,), width=2)
        d.ellipse((x - 1.5, y - 9, x + 1.5, y - 6), fill=INK2 + (255,))

def frame_scaffold(d, x, y_ground, h, w=58, side=1, alpha=255):
    """A short timber frame hugging a wall: three poles, two ledgers, a brace. `side` +1 grows to the right of x."""
    xs = [x, x + side * w / 2, x + side * w]
    for px in xs: d.line((px, y_ground + 3, px, y_ground - h), fill=TIMBER + (alpha,), width=3)
    for k in (1, 2): d.line((xs[0], y_ground - h * k / 2.2, xs[2], y_ground - h * k / 2.2), fill=TIMBER + (int(alpha * 0.9),), width=2)
    d.line((xs[0], y_ground, xs[2], y_ground - h), fill=TIMBER + (int(alpha * 0.6),), width=2)

def stakes_and_string(d, pts, alpha=255, stakes=True, dash=14):
    for i in range(len(pts)):
        a, b = pts[i], pts[(i + 1) % len(pts)]
        L = math.hypot(b[0] - a[0], b[1] - a[1]); n = max(2, int(L / dash))
        for k in range(0, n, 2):
            x0 = a[0] + (b[0] - a[0]) * k / n; y0 = a[1] + (b[1] - a[1]) * k / n
            x1 = a[0] + (b[0] - a[0]) * min(n, k + 1) / n; y1 = a[1] + (b[1] - a[1]) * min(n, k + 1) / n
            d.line((x0, y0, x1, y1), fill=RED + (alpha,), width=2)
    if stakes:
        for p in pts:
            d.line((p[0], p[1], p[0], p[1] - 16), fill=INK + (alpha,), width=3)
            d.line((p[0] - 3, p[1] - 16, p[0] + 3, p[1] - 14), fill=INK + (alpha,), width=3)

# ============================== A: the Great Library ==============================
before = Image.open(stills / 'wonder151-before.png').convert('RGBA')
after = Image.open(stills / 'wonder151-after.png').convert('RGBA')
meta = json.loads((stills / 'wonder151.json').read_text())
master = Image.open(master_path).convert('RGBA')
hexpx = hexpx_of(meta['cam']['zoom'])
S = int(round(hexpx * 1.75))
city = (meta['screen']['x'] * DPR, meta['screen']['y'] * DPR)
ma = master.split()[3]; mb = ma.getbbox(); scale = S / master.width
mh = mb[3] - mb[1]

def box_for(off):
    cx, cy = city[0] + off[0], city[1] + off[1]
    return (int(round(cx - S / 2)), int(round(cy - 0.58 * S)))

def paste_sprite(frame, im, b): frame.alpha_composite(im.resize((S, S), Image.LANCZOS), b)

# Find the landmark's neighbour hex: the one where the master composited on `before` best matches `after`.
best = None
for off in NEIGHBOURS(hexpx):
    b = box_for(off)
    test = before.copy(); paste_sprite(test, master, b)
    reg = (b[0], b[1], b[0] + S, b[1] + S)
    m = ImageStat.Stat(ImageChops.difference(test.crop(reg).convert('L'), after.crop(reg).convert('L'))).mean[0]
    if best is None or m < best[0]: best = (m, b, off)
print('landmark box', best[1], 'score', round(best[0], 1), 'offset', [round(v) for v in best[2]])
box = best[1]
ground_y = box[1] + mb[3] * scale
top_y = box[1] + mb[1] * scale
fcx = box[0] + (mb[0] + mb[2]) / 2 * scale
fhw = (mb[2] - mb[0]) / 2 * scale
GROUND_SHARE = 0.16

def cut_sprite(f):
    """The master cut at fraction f of its height above the ground patch; a fresh-course band at the cut."""
    im = master.copy()
    cut = int(mb[3] - mh * GROUND_SHARE - mh * (1 - GROUND_SHARE) * f)
    if f >= 1: return im, mb[1], []
    a = im.split()[3]
    mask = Image.new('L', im.size, 0); ImageDraw.Draw(mask).rectangle((0, cut, im.width, im.height), fill=255)
    im.putalpha(ImageChops.multiply(a, mask))
    d = ImageDraw.Draw(im)
    row = master.split()[3].crop((0, cut + 2, im.width, cut + 3)).tobytes()
    runs, start = [], None
    for x, v in enumerate(row):
        if v > 128 and start is None: start = x
        if v <= 128 and start is not None: runs.append((start, x)); start = None
    if start is not None: runs.append((start, im.width))
    for s0, s1 in runs:
        if s1 - s0 < 6: continue
        d.rectangle((s0, cut, s1, cut + 7), fill=STONE + (255,))
        d.line((s0, cut + 7, s1, cut + 7), fill=INK + (140,), width=2)
        d.line((s0, cut, s1, cut), fill=(236, 226, 196, 255), width=2)
    return im, cut, runs

def earth_ground():
    """The master's own ground patch as bare earth (its shading kept), for the staked plot."""
    im, cut, _ = cut_sprite(0.0)
    g = ImageOps.colorize(im.convert('L'), black=(104, 80, 48), white=(206, 180, 134)).convert('RGBA')
    g.putalpha(im.split()[3])
    return g

def plot_outline():
    """Eight points round the ground patch (from the master's alpha), for the stakes and string."""
    im, cut, _ = cut_sprite(0.0)
    a = im.split()[3]
    pts = []
    bb = a.getbbox()
    cx = (bb[0] + bb[2]) / 2; cy = (bb[1] + bb[3]) / 2
    for k in range(8):
        ang = math.radians(k * 45 + 22)
        # walk out from the centre until the alpha ends
        r = 0
        while r < 700:
            x = cx + r * math.cos(ang); y = cy + r * math.sin(ang) * 0.9
            if x < 0 or y < 0 or x >= a.width or y >= a.height or a.getpixel((int(x), int(y))) < 60: break
            r += 4
        pts.append((box[0] + x * scale, box[1] + y * scale))
    return pts

def block_piles(d, n, ox=0.62, oy=0.02):
    x0 = fcx + fhw * ox; y0 = ground_y - mh * scale * (0.1 + oy)
    w, h = 15, 8; k = 0
    for r in range(3):
        for c in range(4):
            if k >= n: return
            x = x0 + c * (w + 2) - r * 4; y = y0 - r * (h + 1)
            d.rectangle((x, y, x + w, y + h), fill=STONE + (255,), outline=INK + (230,), width=1); k += 1

def timber_stack(d, n, ox=0.85, oy=0.3):
    x0 = fcx + fhw * ox; y0 = ground_y - mh * scale * oy
    for i in range(n):
        y = y0 - i * 4
        d.line((x0 - 20 + (i % 2) * 2, y, x0 + 20 - (i % 2) * 2, y), fill=TIMBER + (255,), width=3)
        d.ellipse((x0 + 18, y - 2, x0 + 23, y + 3), fill=STONE + (255,), outline=INK + (200,))

def ramp(d, f):
    top = ground_y - mh * scale * (GROUND_SHARE + (1 - GROUND_SHARE) * f) * 0.97
    base_y = ground_y - mh * scale * 0.08
    x_wall = fcx - fhw * 0.62; x_foot = x_wall - 64
    d.polygon([(x_foot, base_y), (x_wall, base_y), (x_wall, top)], fill=EARTH + (180,), outline=INK + (150,))
    for k in range(1, 6):
        t = k / 6
        d.line((x_foot + (x_wall - x_foot) * t, base_y - (base_y - top) * t, x_foot + (x_wall - x_foot) * t + 4, base_y - (base_y - top) * t + 3), fill=TIMBER + (255,), width=2)

def shear_legs(d, where, f):
    """Two lashed poles, a rope and a block: beside the hall on its far side from the city, or on the roof corner."""
    H = mh * scale * (0.5 + 0.35 * f)
    if where == 'beside':
        bx = fcx - fhw * 1.08; by = ground_y - mh * scale * 0.2; lean = -1
    else:
        bx = fcx - fhw * 0.55; by = ground_y - mh * scale * 0.68; H *= 0.5; lean = -1
    tip = (bx - H * 0.36 * lean, by - H)
    d.line((bx - 10, by, tip[0], tip[1]), fill=TIMBER + (255,), width=4)
    d.line((bx - 32, by - 4, tip[0], tip[1]), fill=TIMBER + (255,), width=4)
    d.line((bx - 21, by - 2, tip[0] - H * 0.08, tip[1] + 6), fill=INK + (120,), width=1)
    ly = tip[1] + H * (0.3 + 0.3 * rng.random())
    d.line((tip[0], tip[1], tip[0], ly), fill=INK + (200,), width=2)
    d.rectangle((tip[0] - 8, ly, tip[0] + 8, ly + 11), fill=STONE + (255,), outline=INK + (230,), width=1)

def hugging_frames(d, runs, cut, h, alpha=255):
    """Short frames at the ends of the walls at the working line: outside the leftmost and rightmost opaque runs."""
    if not runs: return
    cy = box[1] + cut * scale
    xl = box[0] + runs[0][0] * scale; xr = box[0] + runs[-1][1] * scale
    frame_scaffold(d, xl - 6, cy + 6, h, side=-1, alpha=alpha)
    frame_scaffold(d, xr + 6, cy + 6, h, side=1, alpha=alpha)

def name_label(d, text, y, color, alpha):
    w = d.textlength(text, font=FONT_CAP); x = fcx - w / 2
    d.line((x - 30, y + 14, x + w + 30, y + 14), fill=GOLD + (alpha,), width=1)
    d.text((x, y - 12), text, font=FONT_CAP, fill=color + (alpha,))

A = []
pc = (int(fcx), int((ground_y + top_y) / 2) + 10); psize = (900, 620)

# 0: chosen: the plot staked and strung, stock arriving
fr = before.copy(); paste_sprite(fr, earth_ground(), box); d = ImageDraw.Draw(fr)
stakes_and_string(d, plot_outline())
block_piles(d, 4); timber_stack(d, 4)
workers(d, 3, fcx, ground_y - mh * scale * 0.1, 80)
A.append(caption_panel(fr, pc, psize, 'Chosen · 0 of 600 Production · the plot', 'The site appears the turn the city chooses the Great Work: the ground cleared, stakes and a string line, the first stone and timber beside it.'))

# 1: footing 15%
fr = before.copy(); im, cut, runs = cut_sprite(0.15); paste_sprite(fr, im, box); d = ImageDraw.Draw(fr)
block_piles(d, 9); timber_stack(d, 5); ramp(d, 0.15); shear_legs(d, 'beside', 0.15)
workers(d, 4, fcx, box[1] + cut * scale + 10, 110); dust(d, fcx + 30, box[1] + cut * scale, 2, 9)
A.append(caption_panel(fr, pc, psize, 'About 15% · footing', 'The footprint as low courses with a flat top and a mortar line, a ramp, the era\'s shear-legs crane standing beside the hall (never over it), blocks stacked.'))

# 2: columns 45%
fr = before.copy(); im, cut, runs = cut_sprite(0.45); paste_sprite(fr, im, box); d = ImageDraw.Draw(fr)
hugging_frames(d, runs, cut, 44)
block_piles(d, 6); timber_stack(d, 3); ramp(d, 0.45); shear_legs(d, 'beside', 0.45)
workers(d, 5, fcx, box[1] + cut * scale - 2, 120); dust(d, fcx - 50, box[1] + cut * scale - 6, 3, 9)
A.append(caption_panel(fr, pc, psize, 'About 45% · columns', 'Walls and column drums to half height with flat tops; scaffold frames hug the wall ends being raised; the crane hoists the next block; the stock is going down.'))

# 3: roofline 80%
fr = before.copy(); im, cut, runs = cut_sprite(0.8); paste_sprite(fr, im, box); d = ImageDraw.Draw(fr)
_, _, eaves = cut_sprite(0.5)   # the frames stand at the walls' outer ends, reaching up to the roofline
hugging_frames(d, [eaves[0], eaves[-1]], cut, 30)
block_piles(d, 2); shear_legs(d, 'roof', 0.8)
workers(d, 3, fcx, box[1] + cut * scale - 2, 90); dust(d, fcx + 60, box[1] + cut * scale - 4, 2, 8)
A.append(caption_panel(fr, pc, psize, 'About 80% · roofline', 'Up to the eaves with the roof still open; scaffold on the upper storey only; the crane has moved onto the roof corner; one pile left, no ramp.'))

# 4: the finish beat
fr = before.copy(); im, _, _ = cut_sprite(1.0); paste_sprite(fr, im, box); d = ImageDraw.Draw(fr)
_, cut, runs = cut_sprite(0.9)
hugging_frames(d, runs, cut, 22, alpha=90)
dust(d, fcx - 80, ground_y - 28, 3, 12, 120); dust(d, fcx + 90, ground_y - 34, 3, 12, 120)
workers(d, 4, fcx, ground_y - 26, 130)
name_label(d, 'THE GREAT LIBRARY', ground_y + 26, GOLD, 210)
A.append(caption_panel(fr, pc, psize, 'Finished · the finish beat, about 2.5 s', 'The last stage is laid in along the working line, the scaffold is struck, the dust settles, the name is lettered under it over a gilt hairline. No burst, no ring, no pop.'))

# 5: standing (real render)
fr = after.copy(); d = ImageDraw.Draw(fr)
name_label(d, 'THE GREAT LIBRARY', ground_y + 26, INK2, 170)
A.append(caption_panel(fr, pc, psize, 'Every turn after · a landmark', 'The Great Work stands on its own tile beside Bellinzona for the rest of the game (the real render), its name under it at near zoom, a gilt badge when zoomed out.'))

for i, p in enumerate(A): p.save(out / f'great-library-panel-{i}.png')
sheet(A, 'The Great Library at Bellinzona, from the turn it is chosen to the turn it stands. Concept mock on a real 2x map frame; stages are the painting cut at the working line, the painted stages replace the cut.', 3, 'great-library-storyboard.png')

# Overview: the site ringed on the whole frame.
ov = before.copy().convert('RGB'); d = ImageDraw.Draw(ov)
d.ellipse((fcx - fhw * 1.15, top_y + mh * scale * 0.35, fcx + fhw * 1.15, ground_y + 12), outline=RED, width=5)
ov.resize((1440, 900), Image.LANCZOS).save(out / 'overview-site-on-map.jpg', quality=88)

# ============================== B: Walls by section ==============================
wb = Image.open(stills / 'found63-before.png').convert('RGBA')
wa = Image.open(stills / 'found63-after.png').convert('RGBA')
cities = json.loads((stills / 'found63.cities.json').read_text())
wint = next(c for c in cities if c['name'] == 'Winterthur')
wc = (wint['screen']['x'] * DPR, wint['screen']['y'] * DPR)
hx2 = hexpx_of(2.2)
reg = (int(wc[0] - hx2 * 1.6), int(wc[1] - hx2 * 0.28), int(wc[0] + hx2 * 1.6), int(wc[1] + hx2 * 1.6))  # below the banner's meter
diffm = ImageChops.difference(wb.convert('RGB'), wa.convert('RGB')).convert('L').point(lambda v: 255 if v > 28 else 0).filter(ImageFilter.MaxFilter(7))
regmask = Image.new('L', wb.size, 0); ImageDraw.Draw(regmask).rectangle(reg, fill=255)
diffm = ImageChops.multiply(diffm, regmask)
bb = diffm.getbbox(); print('walls diff bbox', bb)
ring_c = ((bb[0] + bb[2]) / 2, (bb[1] + bb[3]) / 2 + 10)
rx, ry = (bb[2] - bb[0]) / 2 + 14, (bb[3] - bb[1]) / 2 + 10

def wedge(k):
    m = Image.new('L', wb.size, 0)
    if k <= 0: return m
    if k >= 1: ImageDraw.Draw(m).rectangle((0, 0, wb.width, wb.height), fill=255); return m
    R = 900
    ImageDraw.Draw(m).pieslice((ring_c[0] - R, ring_c[1] - R, ring_c[0] + R, ring_c[1] + R), start=90, end=90 + 360 * k, fill=255)
    return m

def on_ring(deg, grow=1.0):
    a = math.radians(deg)
    return (ring_c[0] + rx * grow * math.cos(a), ring_c[1] + ry * grow * math.sin(a))

def walls_panel(k, title, sub):
    fr = wb.copy()
    if k > 0:
        m = ImageChops.multiply(diffm, wedge(k))
        fr = Image.composite(wa, fr, m)
    d = ImageDraw.Draw(fr)
    # The string line and stakes ahead of the work, round the rest of the circuit.
    start = 90 + 360 * k
    pts = [on_ring(start + t, 1.06) for t in range(0, int(360 * (1 - k)) + 1, 18)]
    if k < 1 and len(pts) > 1:
        for i in range(len(pts) - 1):
            a, b = pts[i], pts[i + 1]
            d.line((a[0], a[1], (a[0] + b[0]) / 2, (a[1] + b[1]) / 2), fill=RED + (230,), width=2)
        for p in pts[::3]:
            d.line((p[0], p[1], p[0], p[1] - 13), fill=INK + (255,), width=3)
    if 0 < k < 1:
        # the live end: a scaffold frame, dust, workers, a block pile beside it
        e = on_ring(start, 1.0)
        side = 1 if math.cos(math.radians(start)) >= 0 else -1
        frame_scaffold(d, e[0] + side * 8, e[1] + 10, 40, w=46, side=side)
        dust(d, e[0], e[1] - 8, 3, 9)
        workers(d, 3, e[0], e[1] + 6, 30)
        s = on_ring(60, 1.3)
        for i in range(5):
            x = s[0] + (i % 3) * 15 - (i // 3) * 4; y = s[1] - (i // 3) * 9
            d.rectangle((x, y, x + 13, y + 8), fill=STONE + (255,), outline=INK + (230,), width=1)
    if k == 0:
        for i in range(6):
            s = on_ring(60, 1.3); x = s[0] + (i % 3) * 15 - (i // 3) * 4; y = s[1] - (i // 3) * 9
            d.rectangle((x, y, x + 13, y + 8), fill=STONE + (255,), outline=INK + (230,), width=1)
    pc2 = (int(ring_c[0]), int(ring_c[1]) - 50)
    return caption_panel(fr, pc2, (760, 560), title, sub)

B = [
    walls_panel(0.0, 'Chosen · 0 of 60 Production · the circuit staked', 'The wall line staked and strung round the town the turn Walls are chosen; stone stacked by the gate.'),
    walls_panel(0.35, 'About 35% · the first sections', 'The era\'s painted wall overlay clipped by angle from the town centre, built clockwise from the gate; scaffold and dust at the live end; the string line runs ahead.'),
    walls_panel(0.7, 'About 70% · round the circuit', 'Each turn the live end moves on by that turn\'s Production. No new art: the circuit is the real overlay, cut where the work has reached.'),
    walls_panel(1.0, 'Finished · the ring closes', 'The last section meets the first; the scaffold is struck; the real walled town from here on (real render).'),
]
for i, p in enumerate(B): p.save(out / f'walls-panel-{i}.png')
sheet(B, 'Walls built section by section round Winterthur (T63, real 2x frame): the wall overlay clipped by angle, clockwise from the gate.', 2, 'walls-storyboard.png')
print('done')
