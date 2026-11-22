#!/usr/bin/env python3
"""Inputs for the art reviewer (legatum-art-reviewer): what art a branch changed, side-by-side sheets of it, and the
art the branch's other changes imply should have changed too.

  tools/art/art-review.py changed BASE TIP           # art files added, modified, renamed or deleted
  tools/art/art-review.py sheet BASE TIP OUT_DIR [--with=slug,...]  # one PNG per changed set: old | new | neighbours
                                                     # (--with adds named neighbours: predecessor, class siblings)
  tools/art/art-review.py impact BASE TIP            # code changes that should have come with art changes

`sheet` draws each changed file at the sizes the game shows that set (see SIZES), beside the old version (from BASE)
and up to eight unchanged neighbours from the same folder and size, on parchment and in greyscale, so a newcomer that
breaks the set's palette, footprint, base, ground or finish is visible at a glance. It renders through headless
Chromium (SVG and raster alike), using app/node_modules of the main checkout.

`impact` is a candidate list, not a verdict: the reviewer checks each line. It flags
  - a unit, building, faction or era whose display name changed or was added (crates/core/src/data.rs, eras.rs);
  - a new enum variant of a kind that carries art (UnitKind, BuildingKind, Faction, Resource, Improvement);
  - an art slug wired in app/src (unitArt.ts, unlockArt.ts, ...) with no file behind it;
  - an art file deleted while tools/art/distinctness.json, a README or a prompt still names it;
  - an art file changed with no prompt, README or PROVENANCE line touched beside it;
  - a raster with detached opaque islands (a floating shrub, pouch or speck the key cut loose);
  - a repainted unit whose roundel, icon or older sibling prompts did not change;
  - art changed with no PROVENANCE.md change.
"""
import base64
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ART = re.compile(r"^app/(src/assets|public/(wonders|terrain))/.*\.(png|svg|jpe?g|webp)$")
# Display sizes (px, the long side) per set folder; the first is the size it is judged at.
SIZES = {
    "units": [48, 96], "unlocks": [64, 128], "cities": [64, 128], "construction": [64, 128], "icons": [16, 24, 48],
    "heraldry": [24, 56, 96], "resources": [18, 32, 64], "improvements": [64, 128], "map-features": [64, 128],
    "art": [128, 256], "eras": [320], "rewards": [26, 56], "siege-props": [48, 96], "wonders": [64, 128],
    "terrain": [64, 128], "landmarks": [64, 128],
}
# The file judged for each changed piece, by suffix preference: one sheet per set, not one per size.
MAIN = ("256", "512", "128", ".svg", "64", ".png", ".jpg")
ART_KINDS = ("UnitKind", "BuildingKind", "Faction", "Resource", "Improvement")
NAME_FILES = ("crates/core/src/data.rs", "crates/core/src/eras.rs")
WIRING = re.compile(r"^app/src/(unitArt|unlockArt|resourceIcons|landmarks|portraits|siegeProps|icons|wonders)\.ts$"
                    r"|^app/src/map/(cityArt|landmarks|improvementArt)\.ts$")
NOTES = re.compile(r"(\.prompt\.txt|README\.md|PROVENANCE\.md|distinctness\.json)$")


def git(*args, check=True):
    r = subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True)
    if check and r.returncode:
        sys.exit(f"git {' '.join(args)}: {r.stderr.decode().strip()}")
    return r.stdout if r.returncode == 0 else None


def changes(base, tip):
    """[(status, path, old_path)] for every file BASE..TIP (renames detected)."""
    out = []
    for line in git("diff", "--name-status", "-M", "-C", f"{base}...{tip}").decode().splitlines():
        parts = line.split("\t")
        st = parts[0][0]
        if st in "RC":
            out.append(("R", parts[2], parts[1]))
        else:
            out.append((st, parts[1], parts[1]))
    return out


def art_changes(base, tip):
    return [c for c in changes(base, tip) if ART.match(c[1]) or ART.match(c[2])]


def set_of(path):
    """(set folder name, size key) for an art path: units/archer-256.png -> ('units', '256')."""
    parts = Path(path).parts
    folder = parts[3] if parts[1] == "src" else parts[2]
    m = re.search(r"-(\d+|card|panel|source|roundel|map)\.(png|jpe?g|svg|webp)$", path)
    return folder, (m.group(1) if m else Path(path).suffix)


def blob(rev, path):
    data = git("show", f"{rev}:{path}", check=False)
    return data


def data_url(data, path):
    mime = "image/svg+xml" if path.endswith(".svg") else "image/jpeg" if re.search(r"\.jpe?g$", path) else "image/png"
    return f"data:{mime};base64,{base64.b64encode(data).decode()}"


def neighbours(tip, path, changed, n=8):
    folder = str(Path(path).parent)
    _, key = set_of(path)
    names = git("ls-tree", "--name-only", f"{tip}:{folder}", check=False)
    if not names:
        return []
    same = [f"{folder}/{x}" for x in names.decode().splitlines()
            if ART.match(f"{folder}/{x}") and set_of(f"{folder}/{x}")[1] == key and f"{folder}/{x}" not in changed]
    step = max(1, len(same) // n)
    return same[::step][:n]


def stem(path):
    return re.sub(r"-(\d+|card|panel|source|roundel|map)$", "", Path(path).stem)


def main_files(items):
    """One file per changed piece: its judged size (MAIN order), so a repaint written at five sizes is shown once."""
    by = {}
    for st, path, old in items:
        by.setdefault(stem(path), []).append((st, path, old))
    out = []
    for files in by.values():
        files.sort(key=lambda f: next((i for i, k in enumerate(MAIN) if f[1].endswith(k + ".png") or f[1].endswith(k)),
                                      len(MAIN)))
        out.append(files[0])
    return out


def sheet(base, tip, out_dir, extra=()):
    """EXTRA: slugs to show as neighbours too (the piece's predecessor, its class siblings), found in the same folder."""
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    groups = {}
    for st, path, old in art_changes(base, tip):
        if st == "D":
            continue
        groups.setdefault(str(Path(path).parent), []).append((st, path, old))
    if not groups:
        print("no art changed")
        return 0
    changed = {p for g in groups.values() for _, p, _ in g}
    pages = []
    for folder, items in sorted(groups.items()):
        items = main_files(items)
        key = set_of(items[0][1])[1]
        sizes = SIZES.get(set_of(items[0][1])[0], [64, 128])
        nb = neighbours(tip, items[0][1], changed)
        listing = (git("ls-tree", "--name-only", f"{tip}:{folder}", check=False) or b"").decode().splitlines()
        for slug in extra:
            hit = [f"{folder}/{x}" for x in listing if stem(x) == slug and set_of(f"{folder}/{x}")[1] == key]
            nb = hit[:1] + [n for n in nb if n not in hit]
        rows = []
        for st, path, old in items:
            cells = [("new", path, blob(tip, path))]
            prev = blob(base, old) if st in "MR" else None
            if prev:
                cells.insert(0, ("old", old, prev))
            rows.append(cells)
        rows.append([("set", p, blob(tip, p)) for p in nb])
        html = ["<html><body style='margin:0;background:#efe6cf;font:12px sans-serif;color:#3b2a1a'>"]
        html.append(f"<div style='padding:8px;font-weight:bold'>{folder} [{key}] at {', '.join(map(str, sizes))} px"
                    " (colour, then greyscale); first rows: changed files (old then new), last row: unchanged set"
                    " neighbours</div>")
        for grey in (False, True):
            filt = "filter:grayscale(1);" if grey else ""
            for cells in rows:
                html.append("<div style='display:flex;flex-wrap:wrap;gap:10px;padding:6px 8px;align-items:flex-end'>")
                for tag, p, data in cells:
                    if not data:
                        continue
                    url = data_url(data, p)
                    imgs = "".join(f"<img src='{url}' style='max-width:{s}px;max-height:{s}px;{filt}'>" for s in sizes)
                    html.append(f"<div style='text-align:center'><div style='display:flex;gap:6px;align-items:flex-end"
                                f";color:currentColor;color:#3b2a1a'>{imgs}</div>"
                                f"<div>{tag}: {Path(p).name[:34]}</div></div>")
                html.append("</div>")
        html.append("</body></html>")
        name = re.sub(r"[^a-z0-9]+", "-", f"{folder}-{key}".lower()).strip("-")
        pages.append((name, "".join(html)))
    return render(pages, out)


def render(pages, out):
    common = git("rev-parse", "--path-format=absolute", "--git-common-dir").decode().strip()
    modules = os.environ.get("LEGATUM_ART_NODE_MODULES") or os.path.join(os.path.dirname(common), "app", "node_modules")
    tmp = Path(tempfile.mkdtemp(prefix="art-review-"))
    jobs = []
    for name, html in pages:
        (tmp / f"{name}.html").write_text(html)
        jobs.append({"html": str(tmp / f"{name}.html"), "png": str(out / f"{name}.png")})
    script = tmp / "shoot.cjs"
    script.write_text(
        "const { chromium } = require(process.argv[2] + '/playwright');\n"
        "(async () => { const b = await chromium.launch(); const p = await b.newPage({ viewport: { width: 1400, "
        "height: 400 } });\n for (const j of JSON.parse(process.argv[3])) { await p.goto('file://' + j.html); "
        "await p.waitForLoadState('load'); await p.screenshot({ path: j.png, fullPage: true }); console.log(j.png); }\n"
        " await b.close(); })().catch(e => { console.error(e); process.exit(1); });\n")
    r = subprocess.run(["node", str(script), modules, json.dumps(jobs)], capture_output=True, text=True)
    sys.stdout.write(r.stdout)
    sys.stderr.write(r.stderr)
    return r.returncode


def fragments(tip, path):
    """Opaque islands of a raster at TIP that are at least 2% of the largest one but not it (a floating piece), and the
    count of tiny specks; None for SVG or opaque art."""
    data = blob(tip, path)
    if not data or path.endswith(".svg"):
        return None
    from io import BytesIO
    from PIL import Image
    im = Image.open(BytesIO(data))
    if im.mode != "RGBA":
        return None
    a = im.getchannel("A").resize((256, 256)) if max(im.size) > 256 else im.getchannel("A")
    w, h = a.size
    px = a.load()
    seen, sizes = set(), []
    for y in range(h):
        for x in range(w):
            if px[x, y] > 40 and (x, y) not in seen:
                stack, n = [(x, y)], 0
                seen.add((x, y))
                while stack:
                    cx, cy = stack.pop()
                    n += 1
                    for nx, ny in ((cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)):
                        if 0 <= nx < w and 0 <= ny < h and (nx, ny) not in seen and px[nx, ny] > 40:
                            seen.add((nx, ny))
                            stack.append((nx, ny))
                sizes.append(n)
    if len(sizes) <= 1:
        return None
    sizes.sort(reverse=True)
    real = [n for n in sizes[1:] if n >= max(4, sizes[0] * 0.003)]  # ignore dust a downscale leaves
    return len(real) or None


def added_removed(base, tip, path):
    diff = git("diff", "-U0", f"{base}...{tip}", "--", path, check=False)
    plus, minus = [], []
    for line in (diff or b"").decode(errors="replace").splitlines():
        if line.startswith("+") and not line.startswith("+++"):
            plus.append(line[1:])
        elif line.startswith("-") and not line.startswith("---"):
            minus.append(line[1:])
    return plus, minus


def enum_variants(rev, kind):
    src = ""
    for f in (git("ls-tree", "-r", "--name-only", rev, "crates/core/src", check=False) or b"").decode().splitlines():
        if f.endswith(".rs"):
            src += (blob(rev, f) or b"").decode(errors="replace")
    m = re.search(r"pub enum " + kind + r"\s*\{(.*?)\n\}", src, re.S)
    if not m:
        return set()
    body = re.sub(r"//[^\n]*|#\[[^\]]*\]", "", m.group(1))
    return {v for v in re.findall(r"^\s*([A-Z][A-Za-z0-9]*)\b", body, re.M)}


def impact(base, tip):
    lines = []
    all_changes = changes(base, tip)
    paths = {c[1] for c in all_changes}
    # 1. Display names added or changed.
    for f in NAME_FILES:
        plus, minus = added_removed(base, tip, f)
        old = {m.group(1): m.group(2) for ln in minus for m in [re.search(r"(\w+(?:::\w+)?)\s*=>\s*\"([^\"]+)\"", ln)] if m}
        for ln in plus:
            m = re.search(r"(\w+(?:::\w+)?)\s*=>\s*\"([^\"]+)\"", ln)
            if not m:
                continue
            key, name = m.groups()
            if key in old and old[key] != name:
                lines.append(f"renamed {key}: \"{old[key]}\" -> \"{name}\" ({f}): does its art still show a {name}? "
                             "(a renamed unit gets new art, legatum-art)")
            elif key not in old:
                lines.append(f"new name {key} = \"{name}\" ({f}): check it has its own art in every set it appears in")
    # 2. New enum variants of kinds that carry art.
    if any(p.startswith("crates/core/src/") for p in paths):
        for kind in ART_KINDS:
            new = enum_variants(tip, kind) - enum_variants(base, kind)
            for v in sorted(new):
                lines.append(f"new {kind}::{v}: needs its own art (unit: miniature/card/panel/roundel/icon; building: "
                             "unlock miniature + icon; faction: people portrait, seal, glyph, crest)")
    # 3. Slugs wired in app code with no file behind them.
    files_at_tip = set((git("ls-tree", "-r", "--name-only", tip, "app/src/assets", "app/public", check=False)
                        or b"").decode().splitlines())
    stems = {re.sub(r"-(\d+|card|panel|source|roundel)\.(png|svg|jpe?g)$|\.(png|svg|jpe?g)$", "", Path(f).name)
             for f in files_at_tip}
    for p in sorted(paths):
        if WIRING.match(p):
            plus, _ = added_removed(base, tip, p)
            for ln in plus:
                for slug in re.findall(r"['\"]((?:building|effect|improvement|unit|ui|mark|people|seal|glyph|crest)"
                                       r"-[a-z0-9-]+)['\"]", ln):
                    if slug not in stems:
                        lines.append(f"{p} now names '{slug}' but no art file has that stem")
    # 4. Deleted art still referenced; 5. changed art with no note beside it.
    notes_touched = {str(Path(p).parent) for p in paths if NOTES.search(p)}
    dj = (blob(tip, "tools/art/distinctness.json") or b"").decode()
    for st, path, old in all_changes:
        if not (ART.match(path) or ART.match(old)):
            continue
        stem = re.sub(r"-(\d+|card|panel|source|roundel)$", "", Path(old).stem)
        if st == "D" and stem in dj:
            lines.append(f"deleted {old} but tools/art/distinctness.json still names '{stem}'")
        if st in "AMRC" and str(Path(path).parent) not in notes_touched and not path.endswith(".svg"):
            lines.append(f"{path} changed with no prompt, README or PROVENANCE change in its folder: is its prompt "
                         "and record current?")
        if st in "AMRC":
            n = fragments(tip, path)
            if n:
                lines.append(f"{path}: {n} detached opaque island(s) besides the main painting (a floating shrub, pouch "
                             "or speck?): look at a crop")
    # 6. A repainted unit whose icon or roundel did not change; sibling prompts the repaint left behind.
    for st, path, old in all_changes:
        m = re.match(r"^app/src/assets/units/([a-z0-9-]+)-256\.png$", path)
        if m and st == "M":
            slug = m.group(1)
            if f"app/src/assets/units/{slug}-roundel.svg" not in paths:
                lines.append(f"unit {slug} repainted but its roundel is unchanged: does the roundel's class mark still "
                             "fit the new painting?")
            icon = "app/src/assets/icons/unit-" + slug.replace("-", "") + ".svg"
            if icon not in paths:
                lines.append(f"unit {slug} repainted but {icon} is unchanged: does the icon still show the same thing?")
            prompts = (git("ls-tree", "--name-only", f"{tip}:app/src/assets/units", check=False) or b"").decode()
            for pr in prompts.splitlines():
                if pr.startswith(slug + ".") and pr.endswith(".prompt.txt") and f"app/src/assets/units/{pr}" not in paths:
                    lines.append(f"unit {slug} repainted; app/src/assets/units/{pr} is unchanged: still true, or stale?")
    if any(ART.match(c[1]) for c in all_changes) and "app/src/assets/PROVENANCE.md" not in paths:
        lines.append("art changed but app/src/assets/PROVENANCE.md did not: are the class rows and date ranges current?")
    print("\n".join(dict.fromkeys(lines)) if lines else "no art implied by the other changes")
    return 0


def main(argv):
    if len(argv) >= 3 and argv[0] == "changed":
        for st, path, old in art_changes(argv[1], argv[2]):
            print(f"{st}\t{path}" + (f"\t(from {old})" if old != path else ""))
        return 0
    if len(argv) >= 4 and argv[0] == "sheet":
        extra = [a.split("=", 1)[1] for a in argv[4:] if a.startswith("--with=")]
        return sheet(argv[1], argv[2], argv[3], [x for e in extra for x in e.split(",")])
    if len(argv) >= 3 and argv[0] == "impact":
        return impact(argv[1], argv[2])
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
