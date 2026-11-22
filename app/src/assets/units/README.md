# Legatum unit art

This is the finished unit art set in the user-selected direction A: AI-generated miniatures on neutral coloured stone bases, with the normal-zoom full figure and a simplified far-zoom owner roundel. `app/src/unitArt.ts` selects each unit's figure, panel portrait, thumbnail and roundel.

## Files and layers

For every slug below:

- `<slug>-256.png` is the transparent PNG master used at normal map zoom.
- `<slug>-64.png` and `<slug>-48.png` are downsampled derivatives (the 64 is the unit panel's head badge).
- `<slug>-card.png` is the 64 px build-card thumbnail, cropped to the figure's upper body so a tall figure fills the 30 px card slot (`tools/art/unit-derivatives.py --card`; `--panel` rebuilds the panel portrait from the source).
- `<slug>-panel.png` is a 512 px transparent full-figure portrait crop for the unit panel.
- `<slug>-roundel.svg` is the high-contrast far-zoom mark. Its disc fill and rim use `currentColor` for the owner colour; the class silhouette remains ink coloured. The outer plate is parchment. The silhouette is ink only and clipped inside the rim (scaled 0.86, clip radius 18.3), so no weapon crosses the owner ring.
- `<slug>-source.png` retains the source raster used to make those files. It is kept outside git (archived; see "Where the source rasters live" below): `python3 tools/art/fetch-sources.py app/src/assets/units/<slug>-source.png` brings one back. Spearman, Tank, and Drone Swarm carry over the A exploration raster. Archer and Horseman were regenerated because their study crops contained clipped neighboring fragments. Newly generated sources are retained at full returned size.
- `prompts/<slug>.prompt.txt` records the subject and shared production prompt.

Bases are painted neutral pale stone, with no faction colour baked in. The renderer can apply an owner-colour wash to the base while retaining its painted highlights and stone texture. No separate base layer is required for this convention. The roundel repeats the owner cue with both a coloured disc wash and coloured rim, while its silhouette is independent of colour. Silhouette shapes are deliberately distinct across foot melee, bows, guns, mounted units, siege, aircraft, drones and civilians so class recognition does not depend on colour vision.

## Where the source rasters live

The `*-source.png` files are not in the repository. `tools/art/sources-manifest.json` lists each one (path, byte size, sha256) and `tools/art/fetch-sources.py` downloads them from the archive named by `LEGATUM_ART_SOURCES_URL`, verifying every hash (`--all` fetches every missing file). `unit-derivatives.py --panel` and `air-plan.py drone` fetch a missing source on their own. To add a new unit's master: run `unit-art.py` (it writes the source beside the derivatives), run `python3 tools/art/fetch-sources.py --add app/src/assets/units/<slug>-source.png`, upload the file to the archive at the same path, and commit the manifest, not the file. The app bundles only the derivatives, so a build, the tests and the gates never need a source. The mentions of a `-source` file in the sections below describe how the files were made.

## Subjects

| Slug | Subject / class |
| --- | --- |
| `spearman` | Ancient Greek foot soldier with upright spear and round shield. |
| `pikeman` | Medieval Swiss pikeman in bright steel, braced, pike angled back over the shoulder, no shield. |
| `musketman` | Renaissance musketeer advancing, plug bayonet levelled at the hip, broad plumed hat. |
| `rifleman` | 19th-century rifle infantry in blue wool tunic and kepi holding rifled musket. |
| `trench-infantry` | Machine Age: a First World War French infantryman of 1916 in a horizon-blue greatcoat, Adrian helmet, haversack and rolled blanket, in a low crouching run with his bayoneted rifle carried at the trail (repainted 2026-11-14; it had been a sepia recolour of the Infantry). |
| `infantry` | World War II infantryman (generic GI, no insignia) in an M1 steel helmet with a scrim net, olive-drab field jacket, web belt and pack, canvas leggings and boots, throwing a pineapple grenade straight overhead, an M1 Garand held upright by his side (repainted 2026-11-15; it had been WWI kit: Brodie helmet, puttees, bayonet). |
| `archer` | Ancient Greek foot archer drawing a curved wooden bow. |
| `crossbowman` | Medieval crossbowman kneeling behind a tall striped pavise, crossbow levelled round its edge. |
| `field-cannon` | 19th-century field cannon with two wooden wheels, shown with one crew member. |
| `artillery` | World War II 105 mm field howitzer (M101 manner): rubber tyres on steel disc wheels, a flat gun shield, a split trail, olive drab, no crew (repainted 2026-11-14; it had been an early-20th-century spoked-wheel howitzer). |
| `howitzer` | Machine Age (First World War) heavy howitzer on large spoked wooden wheels, thick barrel raised high on its cradle, box trail, no crew. The painting that was the Artillery's until 2026-11-14 (the Howitzer had been a sand-tinted recolour of it). |
| `drone-swarm` | Three compact realistic quadcopter combat drones in a tight formation. |
| `horseman` | Ancient Greek rider on one complete horse with short upright spear. |
| `knight` | Medieval mail-clad knight on barded horse carrying an upright lance. |
| `cavalry` | 19th-century light cavalry rider on horse with curved sabre. |
| `tank` | World War II medium tank in the Sherman and T-34 family: rounded cast turret, short 75 mm gun, bogie suspension, plain olive drab, no insignia (repainted 2026-11-14; it was a present-day main battle tank). |
| `main-battle-tank` | Modern era (2026-11-14): a Cold War main battle tank in the Soviet T-72 manner, low wide hull, low dome turret, a very long smooth-bore gun raised about 10 degrees, bottle green with stowage on the rear deck. |
| `mechanised-infantry` | Modern era: a generic eight-wheeled armoured personnel carrier in sand and olive camouflage, rear ramp down, with one infantryman stepping off it (repainted 2026-11-22: the old squad of three in front of a tracked vehicle read as a crowd, not one piece). Wheels, not tracks, keep it apart from the tanks; the icon and roundel draw the same: four wheels under a sloped hull, a small turret, a soldier in the rear hatch. |
| `rocket-artillery` | Modern era: a three-axle truck carrying a raised rack of forty launch tubes (BM-21 Grad manner), olive drab. |
| `jet-fighter` | Modern era: an early swept-wing jet (F-86 / MiG-15 manner) in pale blue-grey paint, climbing, on the aircraft display stand. |
| `jet-bomber` | Modern era: a B-52D-style eight-engine strategic bomber in dark wrap-around camouflage, nose to the left, on the aircraft display stand. |
| `autonomous-armour` | Near-future autonomous tracked combat vehicle with unmanned turret. |
| `hunter-killer` | Future melee: an armed autonomous ground robot, not humanoid and not modelled on any maker's product, standing on the cream disc. Repainted 2026-11-22 as an original design: a low, wide armoured hull on four short, thick, crab-set legs, a small turret with a short autocannon and a boxy sensor block under the gun, no head or neck (the first painting, a generic four-legged platform with an autocannon and belt box, sat too close to a commercial quadruped robot). |
| `stealth-fighter` | Future air: a faceted fifth-generation stealth fighter in charcoal-grey absorbent coating, banking on the aircraft display stand. |
| `stealth-bomber` | Future air: a flying-wing stealth bomber with a saw-toothed trailing edge, on the aircraft display stand. |
| `biplane` | Machine Age: a First World War single-seat biplane (Sopwith Camel manner): two stacked cream-linen wings on chunky struts, a round olive cowling, a wooden propeller, spoked wheels, nose to the upper left, on the aircraft display stand. It replaced a tan recolour of the Fighter (2026-11-14). |
| `fighter` | World War II single-engine propeller fighter on small neutral display plinth. |
| `bomber` | World War II twin-engine medium bomber on small neutral display plinth. |
| `settler` | Ancient civilian woman settler with travel bundle and walking staff; no weapon. |
| `scout` | Ancient explorer, one foot on a rock, shading his eyes, forked staff slanted out. |
| `spartan-hoplite` | 5th-century BCE Spartan hoplite with bronze helmet, aspis shield and spear. |
| `keshig` | The Mongol Keshig (the Mongols' Classical mounted unit, in the Cataphract's slot, a mounted unit that can also shoot): a galloping steppe pony, the rider turned back drawing a recurved bow (the Parthian shot). The Mongol Scout is the ordinary `scout`. |
| `byzantine-heir` | Young adult 11th-century Byzantine imperial heir in court mantle and small diadem. |
| `tagmatic-soldier` | Byzantine skoutatos (the Tagmatic Soldier, Byzantium's Pikeman) behind a tall red almond shield, spear raised overhand. |
| `phalangite` | Classical Macedonian phalanx block of three: the front sarissa lowered, the rear two raised at 45 and 70 degrees (a tall fan of pikes). |
| `composite-archer` | Classical Scythian archer kneeling, tall pointed hood, red coat, drawing a short recurved bow. |
| `cataphract` | Classical cataphract on a grey scale-barded horse, kontos lance couched horizontally. |
| `arquebusier` | Renaissance arquebusier firing a matchlock from a forked rest, a puff of smoke at the muzzle. |
| `cuirassier` | Renaissance cuirassier on a black horse in blackened plate, firing a wheel-lock pistol forward. |

## Prompt convention

The prompt files use this common direction: “Refined hand-painted miniature, painterly realism, historically plausible tactile materials, muted parchment-compatible pigments, soft diffuse studio lighting. One complete centered subject with bold silhouette on a small round neutral pale stone base; square transparent background, generous transparent margins. No owner color on the stone base, so the renderer can tint it. No text, watermark, border, scenery or extra figures.” Per-subject equipment and era appear in each prompt file.

## Class and ownership at far zoom

Roundels are class symbols (distinctness pass, 2026-10-11; `tools/art/distinctness.py` found the ranged and melee figures indistinguishable at 16 px, because every roundel shared one body): the melee line (Spearman through Humanoid Robot) is a round shield with a spear, the ranged class (Archer, Crossbowman, Composite Archer, Arquebusier) is a bow with a drawn arrow, the mounted class is a horse and rider, and civilians stay the unarmed figure. Units of one class share a roundel on purpose (owner colour says whose, the shape says which class); `tools/art/distinctness.json` lists those class groups, and any two roundels of different classes must stay distinct. Roundels keep class legibility in monochrome through their object silhouette: pikes and foot weapons, drawn bow/crossbow, horse and rider, tracked hull, siege frame or gun, fixed-wing aircraft, drone rotors, or an unarmed civilian figure. Owner identity is a second cue: a tinted disc and a stronger rim. A player colour-vision playtest is still appropriate before wiring the assets into live maps.

Faction-specific pieces: the Spartan Hoplite (replaces the Spearman), the Mongol Keshig (`keshig`, replaces the Cataphract), the Tagmatic Soldier (`tagmatic-soldier`, Byzantium's Pikeman) and the Byzantine Heir. Unused drafts (a Prussian Legionary, a Celtic Cairn Singer, a spare reconnaissance drone) were removed on 2026-10-11; git history keeps them.

## Classical and Renaissance unit lineage

The five era-specific units now have their own paintings, thumbnails, panel portraits and class roundels: Phalangite (Classical melee), Composite Archer (Classical ranged), Cataphract (Classical mounted), Arquebusier (Renaissance ranged) and Cuirassier (Renaissance mounted). Their prompts are retained under `prompts/`, their class icons remain in `assets/icons/unit-*.svg`, and `app/src/unitArt.ts` maps each kind to its own slug. The `*-source.png` file retains each generated transparent source raster; the other PNGs are transparent, baseline-aligned derivatives. Each bespoke roundel keeps a high-contrast class silhouette on the parchment plate with the owner-coloured disc and rim.

## Future units (2026-11-01, `claude/future-unit-art`)

The Hunter-Killer, Stealth Fighter and Stealth Bomber each have the full set (`-source`, `-256`, `-64`, `-48`, `-card`, `-panel`, roundel, prompt, and the engraved icon in `../icons`). They were generated with Codex image generation (no Gemini key is configured), one reference image per piece (the Autonomous Armour, Fighter or Bomber source on a flat magenta field, used only for camera, finish, lighting and base), then `tools/art/unit-art.py RAW.png <slug>` keys the field, fits the figure and writes every derivative. Rules the three taught:

- Ask for no antennas or other thin protruding parts. `key_magenta` pulls the alpha edge in several pixels, which erodes a 14 px antenna into a floating speck at 256 px.
- Ask for the same plain warm cream disc as the Autonomous Armour and every other unit; the first Hunter-Killer came back on a different base.
- The aircraft sit on the same curved display stand as the Fighter and Bomber, in charcoal grey instead of olive, with silhouettes that cannot be mistaken for a propeller aeroplane (a diamond arrowhead with canted tails; a flying wing with a saw-toothed edge).
- Roundels are drawn by hand from the icon's shapes, ink only, inside the rim. The Hunter-Killer roundel is its own quadruped silhouette (splayed legs, gun up and to the right) rather than the melee shield, because the shield-and-spear mark read as the Infantry's; the stealth roundels keep the same 24 px read as the icons.
- `app/src/unitArt.distinct.test.ts` fails if two unit kinds (or a kind and a faction piece) share an art slug, unless the pair is listed in its `SHARED` table with a reason; it also checks every slug has the whole file set.

## Barbarians

Barbarian camps field six kinds (`crates/core/src/rules/barbarians.rs`: Spearman, then Pikeman, then Musketman as half the living empires learn Feudalism and Gunpowder; Archer, then Crossbowman with Machinery; and the camp's Scout). Each has its own painted raider, so a barbarian never draws as a glyph counter beside the painted empires (2026-11-01):

| Slug | Subject |
| --- | --- |
| `barbarian-spearman` | Iron Age tribal raider in fur cloak, hide oval shield with a red slash, spear held low in a prowling advance. |
| `barbarian-pikeman` | Medieval brigand in a patched gambeson and dented kettle hat, billhook raised overhead to strike. |
| `barbarian-musketman` | Thirty Years' War marauder in a torn buff coat and slouch hat, down on one knee firing a matchlock levelled forward. |
| `barbarian-archer` | Iron Age raider archer under a wolf-fur hood, long self bow drawn high. |
| `barbarian-crossbowman` | Medieval forest outlaw, hooded, foot in the stirrup spanning a heavy crossbow. |
| `barbarian-scout` | Iron Age outrider crouched on one knee, shading his eyes, ox-horn war horn on his back. |

Every raider carries one blood-red cloth accent; the map washes the base charred ink in an ink ring with a blood-red notch at its left end (never a seat colour, and no owner roundel), and the far zoom uses the class roundel washed blood red under a heavy ink ring with a red notch at the upper left. A kind without its own raider (a Company that turned barbarian) keeps its ordinary figure on the barbarian base (`barbarianArtSlug` in `app/src/unitArt.ts`). Only the `-256` master and the `-source` are shipped: barbarians appear on the map only, never in the unit panel or build cards. They were painted by Codex image generation on a flat magenta field with `pikeman-source.png` and `archer-source.png` as style references, keyed to alpha, eroded 2 px and despilled at the edge; prompts are in `prompts/barbarian-*.prompt.txt`.

## Air-strike plan views (2026-11-02, `claude/painted-aircraft`)

`air/<Fighter|Bomber|StealthFighter|StealthBomber>-plan.png` (as `fighter-plan.png` and so on) are the top-down sprites the combat animation flies over a strike's target (`app/src/map/combatFxDraw.ts::drawAircraft`), and `air/drone-plan.png` is one drone of the swarm for the Drone shot. Each plan view was generated with the aircraft's miniature as the reference (same finish, colours and markings), asked for a straight top-down view, nose to the right, no stand or shadow; `tools/art/air-plan.py` trims and fits it to 320 px. The drone is cropped from `drone-swarm-source.png` (`air-plan.py drone`).

## Faction special units (2026-11-14, `claude/special-unit-art`)

Six faction replacements, each with the full set (`-source`, `-256`, `-64`, `-48`, `-card`, `-panel`, roundel, prompt, engraved icon in `../icons`). Codex image generation with a plain unit as the style reference, then `tools/art/unit-art.py`. Icons are named by the piece (`unit-eaglewarrior`, `unit-gaesatae`, `unit-peltast`, `unit-warelephant`, `unit-reislaufer`, `unit-panzer`). The Panzer carries no insignia of any kind. Review sheet: `docs/design/special-units/contact-sheet.png`.

| Slug | Faction, replaces | Subject |
| --- | --- | --- |
| `eagle-warrior` | Aztecs, Spearman | Eagle Warrior in feathered suit and eagle-head helmet, macuahuitl raised, round feathered shield. |
| `gaesatae` | Celts, Phalangite | Bare-chested Gaesatae with woad spirals and gold torc, spear cocked back, tall oval shield. |
| `peltast` | Athens, Composite Archer | Skirmisher side-on, left foot forward, in red chiton, indigo cloak and fox-fur cap: the right hand carries a javelin palm up beside the head, point forward; the left holds the wicker crescent pelte out front by its inner grip with two equal-length spare javelins behind it, points forward (v6: the hand repainted by an image edit of a crop, composited back). |
| `war-elephant` | India, Knight | War elephant in quilted caparison with howdah, an archer-spearman and a mahout. |
| `reislaufer` | Switzerland, Musketman | Reisläufer in slashed parti-coloured dress, plumed beret, pike braced upright. |
| `panzer` | Germany, Tank | Panzer IV silhouette, skirt armour, sand and olive camouflage, no markings. |

Lessons: the first Peltast (fox cap, cream tunic) vanished on tan terrain and the first Gaesatae's overlong spear shrank the figure at 54 px, so both were regenerated with saturated cloth and a spear of about 1.3 heights. Generations that return "no image" in the reply often still saved one under `~/.codex/generated_images/<session id>/`. The generator's grey stone bases were recoloured to the older units' warm cream (hue 0.10, saturation 0.20, lightness kept) on the raw image before keying: flood-fill from the base's bottom over low-saturation pixels of the base's own lightness, then `unit-art.py`. The Reisläufer's cobbled base was already warm and was left.

The stock magenta key punches holes in blue-violet and red-pink cloth; `unit-art.py --edge-key` keys by the field's saturation alone. Only the Peltast uses it so far; the Eagle Warrior, Gaesatae, War Elephant and Panzer have minor flecks (about 0.5-1.2% of interior samples) on red-pink rims, caparison and tusk bands.

## Sweep of 2026-11-14 (`claude/unit-art-sweep`; audit in `docs/design/unit-art-audit-2026-11-14.md`)

`tools/art/unit-audit.py` found three recolours (Biplane of the Fighter, Howitzer of the Artillery, Trench Infantry of the Infantry) and repainted them. The Biplane (miniature and `air/biplane-plan.png`, the plan view an image edit of the miniature), the Artillery and the Trench Infantry are new Codex paintings; the Howitzer takes the Artillery's old painting, which always showed a First World War howitzer. The Biplane, Trench Infantry, Howitzer, Fighter and Bomber have their own hand-drawn roundels (the Fighter and Bomber had shared the Biplane's), and the icons `unit-biplane`, `unit-trenchinfantry`, `unit-artillery` (the Howitzer keeps the old Artillery drawing) and `unit-eaglewarrior` (it read as a bomb) were redrawn. Rule it taught: a recolour is a placeholder even when a tint moves the distinctness score; run `python3 tools/art/unit-audit.py` after adding any unit (a recolour scores 0.99 or more on `edge`, every different painting 0.35 or less).
