# Unit art audit, 2026-11-14

Trigger (the designer, looking at the Modern era sheet): "you still have some placeholders. biplane and howitzer are just different colors for the same art. can you do a sweep of all the unit art and make sure it meets the requirements?" The rule is in `.claude/skills/legatum-art/SKILL.md`, "A recolour is a placeholder, not art": every unit has its own painting of its own subject, and its own icon and roundel; a recolour or copy never counts, even when a tint moves the distinctness score or the pair is on the accepted list.

## Method

`tools/art/unit-audit.py` (numpy and Pillow). It does not use the distinctness score, which a tint can move.

- **Miniature copies:** for every pair of `*-256.png` masters, the alpha mask is cropped to its bounding box and fitted into a square; the greyscale painting is histogram-matched to the other's inside the shared mask (so tint and grade cannot hide a copy), then compared by pixel correlation (`ncc`) and by edge-map correlation (`edge`). A recolour scores 1.00 on both. Different paintings score `edge` 0.3 or less (the highest non-copy pair in the whole set is Archer against the barbarian Archer at 0.32, a different painting of the same pose idea). Calibration: Biplane/Fighter 0.99, Howitzer/Artillery 1.00, Trench Infantry/Infantry 0.99; nothing else is above 0.32, so the line is not close.
- **Byte copies:** SHA-256 of every `-256 -64 -48 -card -panel -source` PNG across slugs (none identical, the copies were re-graded), and of every roundel and icon drawing with comments, titles and `aria-label` removed. This finds the three identical icon drawings (`unit-biplane` = `unit-fighter`, `unit-howitzer` = `unit-artillery`, `unit-trenchinfantry` = `unit-infantry`: the Machine Age ones are XML-comment-only different).
- **Plan views:** the air-strike sprites compared by eye and by the same structure test; `air/biplane-plan.png` is the Fighter's plan view in tan.
- **Subject, style, size:** every miniature, icon (rendered at 110 px) and roundel looked at; PNG sizes, modes, opaque corners, touching the edge and detached specks checked by script (all 256/64/48/card/512 sizes are right, all RGBA, no opaque corners, nothing touches the edge).
- **Roundels:** per `app/src/assets/units/README.md` roundels are class marks and units of one class share one on purpose. That conflicts with a literal reading of "its own roundel". The shared marks are listed in the table, and the question is put to the designer in the findings.

## Totals

- 46 playable unit pieces audited (37 unit kinds plus 9 faction pieces) and 6 barbarian raiders, 52 miniatures in all; 46 icons (the Spartan and Byzantine pieces use `unit-oathbearer` and `unit-legionary`); 46 roundel files drawing 30 distinct marks; 8 air plan views.
- **Copies and recolours: 3 units** (Biplane of Fighter, Howitzer of Artillery, Trench Infantry of Infantry), each across miniature, panel, card, 64, 48 and icon, plus the Biplane's plan view. Roundels of all three are the class marks.
- **Wrong subject: 3.** The Biplane (a monoplane), the Artillery (a WWI-style howitzer standing for a WWII unit; it is the Machine Age Howitzer's rightful picture), the Eagle Warrior's icon (reads as a bomb).
- **Other (minor, not repainted):** the Infantry's WWI kit, the Fighter icon's jet shape, the Varangian Guard's pre-rename icon, the Mechanised Infantry's wider rose base, stray specks on the Combat Drone, War Elephant, Jet Fighter and a barbarian spearman, the accepted-list text that is stale.
- Everything else passes.

## Every unit

Columns: miniature = the 256 painting and its panel, card, 64 and 48 derivatives; icon = `assets/icons/unit-*.svg`; roundel = `assets/units/<slug>-roundel.svg`.

| Unit | Art slug | Miniature | Icon | Roundel | Verdict |
| --- | --- | --- | --- | --- | --- |
| Scout | `scout` | own painting (nearest pikeman: edge 0.17) | own | class-shared with settler | pass |
| Settler | `settler` | own painting (nearest barbarian-pikeman: edge 0.09) | own | class-shared with scout | pass |
| Spearman | `spearman` | own painting (nearest barbarian-archer: edge 0.17) | own | class-shared with infantry, musketman, pikeman, rifleman, spartan-hoplite, tagmatic-soldier, trench-infantry | pass |
| Archer | `archer` | own painting (nearest barbarian-archer: edge 0.32) | own | class-shared with arquebusier, composite-archer, crossbowman | pass |
| Pikeman | `pikeman` | own painting (nearest cavalry: edge 0.21) | own | class-shared with infantry, musketman, rifleman, spartan-hoplite, spearman, tagmatic-soldier, trench-infantry | pass |
| Crossbowman | `crossbowman` | own painting (nearest field-cannon: edge 0.13) | own | class-shared with archer, arquebusier, composite-archer | pass |
| Musketman | `musketman` | own painting (nearest trench-infantry: edge 0.09) | own | class-shared with infantry, pikeman, rifleman, spartan-hoplite, spearman, tagmatic-soldier, trench-infantry | pass |
| Byzantine Heir | `byzantine-heir` | own painting (nearest tagmatic-soldier: edge 0.14) | own | own | pass |
| Horseman | `horseman` | own painting (nearest gaesatae: edge 0.21) | own | own | pass |
| Knight | `knight` | own painting (nearest gaesatae: edge 0.17) | own | own | pass |
| Cavalry | `cavalry` | own painting (nearest pikeman: edge 0.21) | own | own | pass |
| Tank | `tank` | own painting (nearest panzer: edge 0.12) | own | class-shared with autonomous-armour | pass (WWII medium tank). Roundel shared with Autonomous Armour (class mark) |
| Autonomous Armour | `autonomous-armour` | own painting (nearest horseman: edge 0.12) | own | class-shared with tank | pass |
| Rifleman | `rifleman` | own painting (nearest infantry: edge 0.17) | own | class-shared with infantry, musketman, pikeman, spartan-hoplite, spearman, tagmatic-soldier, trench-infantry | pass |
| Infantry | `infantry` | original (its recolour is trench-infantry) (nearest trench-infantry: edge 0.99) | original (copied by unit-trenchinfantry) | class-shared with musketman, pikeman, rifleman, spartan-hoplite, spearman, tagmatic-soldier, trench-infantry | pass as a file; **watch**: Brodie helmet, puttees and a bayonet rifle are WWI kit on a World War (WWII) unit, and the Trench Infantry needs the WWI look. Judged after the Trench Infantry repaint |
| Hunter-Killer | `hunter-killer` | own painting (nearest drone-swarm: edge 0.07) | own | own | pass |
| Stealth Fighter | `stealth-fighter` | own painting (nearest drone-swarm: edge -0.03) | own | own | pass |
| Stealth Bomber | `stealth-bomber` | own painting (nearest jet-bomber: edge 0.08) | own | own | pass |
| Field Cannon | `field-cannon` | own painting (nearest pikeman: edge 0.21) | own | class-shared with artillery, howitzer | pass |
| Artillery | `artillery` | original (its recolour is howitzer) (nearest howitzer: edge 1.00) | original (copied by unit-howitzer) | class-shared with field-cannon, howitzer | **FAIL (subject)**: painted as an early-20th-century spoked-wheel howitzer, which is the Machine Age Howitzer's look; this is the World War unit (Rocketry, after the Howitzer), so it reads a generation too old beside its upgrade path |
| Combat Drone | `drone-swarm` | own painting (nearest rocket-artillery: edge 0.14) | own | own | pass. Other: a 30 px stray speck at the left edge of the 256 master |
| Fighter | `fighter` | original (its recolour is biplane) (nearest biplane: edge 0.99) | original (copied by unit-biplane) | class-shared with biplane, bomber | pass (miniature, panel, plan). Icon is a swept-wing jet silhouette on a propeller fighter: other, minor |
| Bomber | `bomber` | own painting (nearest stealth-bomber: edge 0.09) | own | class-shared with biplane, fighter | pass |
| Phalangite | `phalangite` | own painting (nearest trench-infantry: edge 0.11) | own | own | pass |
| Composite Archer | `composite-archer` | own painting (nearest peltast: edge 0.17) | own | class-shared with archer, arquebusier, crossbowman | pass |
| Cataphract | `cataphract` | own painting (nearest hunter-killer: edge 0.07) | own | own | pass |
| Arquebusier | `arquebusier` | own painting (nearest barbarian-archer: edge 0.26) | own | class-shared with archer, composite-archer, crossbowman | pass |
| Cuirassier | `cuirassier` | own painting (nearest cavalry: edge 0.06) | own | own | pass |
| Trench Infantry | `trench-infantry` | **recolour of infantry** (nearest infantry: edge 0.99) | **copy of unit-infantry** | class-shared with infantry, musketman, pikeman, rifleman, spartan-hoplite, spearman, tagmatic-soldier | **FAIL: copy of Infantry** (miniature at every size, panel, card, icon, roundel). Wrong subject for the era as well: it shows the Infantry painting in sepia |
| Howitzer | `howitzer` | **recolour of artillery** (nearest artillery: edge 1.00) | **copy of unit-artillery** | class-shared with artillery, field-cannon | **FAIL: copy of Artillery** (miniature at every size, panel, card, icon). Roundel is the shared gun mark |
| Early Tank | `landship` | own painting (nearest panzer: edge 0.08) | own | own | pass |
| Biplane | `biplane` | **recolour of fighter** (nearest fighter: edge 0.99) | **copy of unit-fighter** | class-shared with bomber, fighter | **FAIL: copy of Fighter, wrong subject** (a single-wing P-51-style monoplane, not a biplane): miniature, panel, card, icon (a swept-wing jet), air plan view. Roundel is the shared aircraft mark |
| Mechanised Infantry | `mechanised-infantry` | own painting (nearest stealth-bomber: edge 0.11) | own | own | pass. Other: its base is wider and rose-coloured against the set's cream stone |
| Main Battle Tank | `main-battle-tank` | own painting (nearest stealth-bomber: edge 0.15) | own | own | pass |
| Rocket Artillery | `rocket-artillery` | own painting (nearest stealth-bomber: edge 0.22) | own | own | pass |
| Jet Fighter | `jet-fighter` | own painting (nearest stealth-bomber: edge 0.18) | own | own | pass. Other: an 8 px stray speck |
| Jet Bomber | `jet-bomber` | own painting (nearest stealth-bomber: edge 0.08) | own | own | pass |
| Spartan Hoplite | `spartan-hoplite` | own painting (nearest cavalry: edge 0.19) | own | class-shared with infantry, musketman, pikeman, rifleman, spearman, tagmatic-soldier, trench-infantry | pass |
| Keshig | `keshig` | own painting (nearest peltast: edge 0.07) | own | own | pass |
| Tagmatic Soldier (Varangian Guard) | `tagmatic-soldier` | own painting (nearest gaesatae: edge 0.14) | own | class-shared with infantry, musketman, pikeman, rifleman, spartan-hoplite, spearman, trench-infantry | miniature pass. Icon `unit-legionary` is the old pre-rename shield glyph (renamed Varangian Guard, 2026-11-07): other, minor |
| Eagle Warrior | `eagle-warrior` | own painting (nearest barbarian-pikeman: edge 0.30) | own | own | miniature pass. **Icon reads as a bomb** (round body with a fuse): wrong subject |
| Gaesatae | `gaesatae` | own painting (nearest barbarian-archer: edge 0.25) | own | own | pass |
| Peltast | `peltast` | own painting (nearest cavalry: edge 0.14) | own | own | pass |
| War Elephant | `war-elephant` | own painting (nearest gaesatae: edge 0.19) | own | own | pass. Other: 53 px and 2 px stray specks (its known minor flecks) |
| Reislaufer | `reislaufer` | own painting (nearest barbarian-archer: edge 0.14) | own | own | pass |
| Panzer | `panzer` | own painting (nearest tank: edge 0.12) | own | own | pass (own painting). Other: on the accepted list beside Tank (gist 0.93) with a stale "repaint queued" reason |
### Barbarians (miniature only: map only, no panel, card or icon)

| Raider | Slug | Verdict |
| --- | --- | --- |
| Spearman | `barbarian-spearman` | pass (own painting; a 1 px speck) |
| Pikeman | `barbarian-pikeman` | pass (own painting) |
| Musketman | `barbarian-musketman` | pass (own painting) |
| Archer | `barbarian-archer` | pass (own painting; shares the empire Archer's draw by decision, accepted pair `archer`/`barbarian-archer`) |
| Crossbowman | `barbarian-crossbowman` | pass (own painting) |
| Scout | `barbarian-scout` | pass (own painting) |

### Air plan views (`assets/units/air`)

| Plan | Verdict |
| --- | --- |
| `fighter-plan` | pass |
| `bomber-plan` | pass |
| `biplane-plan` | **FAIL: the Fighter's plan view in tan** (a single-wing aircraft with a propeller) |
| `jet-fighter-plan`, `jet-bomber-plan`, `stealth-fighter-plan`, `stealth-bomber-plan`, `drone-plan` | pass |

## Accepted-list entries that cover a recolour or copy (candidates for removal)

`tools/art/distinctness.json` is not edited by this sweep. The entries below pass only because they are listed. Checked by running the full distinctness check on a copy of the config with the first nine rows removed (`landship`/`tank` miniatures kept): 0 failing except that one silhouette pair.

| Set | Pair | Why it is a candidate |
| --- | --- | --- |
| units-miniature | `infantry` / `trench-infantry` | a sepia copy |
| units-miniature | `biplane` / `fighter` | a tan copy |
| units-miniature | `artillery` / `howitzer` | a sand copy |
| units-miniature | `barbarian-crossbowman` / `trench-infantry` | inherited from the Trench Infantry copy |
| units-miniature | `biplane` / `stealth-bomber` | inherited from the Biplane copy |
| unit-icons | `unit-artillery` / `unit-howitzer` | the same drawing |
| unit-icons | `unit-biplane` / `unit-fighter` | the same drawing |
| unit-icons | `unit-infantry` / `unit-trenchinfantry` | the same drawing |
| units-miniature | `landship` / `tank` | **keep, reword**: the reason says "a sepia-tinted copy", which is false (the Landship is its own rhomboid painting); with the entry removed the pair still fails on the silhouette bar alone (gist 0.77, score 0.55), so it needs a true reason: two different paintings, a tall lozenge against a turreted tank |
| unit-icons | `unit-landship` / `unit-tank` | same stale reason; the Landship icon is a hand-drawn rhomboid and no longer flags: remove |
| units-roundel | `biplane+bomber+fighter` / `scout+settler` | stale name: the three aircraft now have three drawings, so no asset has that id any more: remove |
| building-icons, unlock-art | `building-clearinghouse` / `building-stockexchange` | not a unit: the same "Machine Age placeholder" wording, a copy of the neighbouring Bank painting; outside this sweep, reported for the buildings pass |

Not recolours, so not candidates, but with stale reasons: `panzer`/`tank` and `unit-panzer`/`unit-tank` ("Panzer repaint queued": the Panzer IV painting is already shipped, two different paintings of two WWII tanks); the rest (aircraft on one stand, Hoplite/Spearman, barbarian Archer, resources, milestones) are judged exceptions between different paintings.

## Findings for the designer

1. **Roundel ownership.** The README makes roundels class marks on purpose (22 of the 46 pieces share one of six identical drawings: the melee line x8, archers x4, guns x3, aircraft x3, armour x2, civilians x2; the six horse-and-rider marks differ only in small details). Read literally, "its own roundel" would ask for about 20 more drawings. This sweep gives the three failures their own roundels and leaves the class marks alone; say if you want the rest drawn.
2. **Infantry's era.** The Infantry (a World War unit) is painted in WWI kit. It stays until you say; the new Trench Infantry is clearly WWI beside it.

## Phase 2: what was repainted

`distinctness.json` is untouched. Contact sheet: `docs/design/unit-art-sweep-sheet.png` (old beside new, look-alikes, play size and 3x). Prompts are in `app/src/assets/units/prompts/` (`biplane`, `artillery`, `howitzer`, `trench-infantry`).

| Asset | Fix |
| --- | --- |
| Biplane miniature, panel, card, 64, 48 | new Codex painting: a Sopwith Camel-manner biplane, two stacked cream-linen wings, struts, rotary cowling, wooden propeller, on the aircraft stand, no markings |
| `air/biplane-plan.png` | new plan view, an image edit of the new miniature, keyed and fitted by `tools/art/air-plan.py` |
| Biplane icon, roundel | hand-drawn side-view biplane (two wings, struts, wheel) |
| Fighter and Bomber roundels | redrawn as a straight-wing propeller monoplane and a twin-engine bomber (they had shared the Biplane's jet-like mark; the Biplane leaving the group made `bomber+fighter` score 0.81 against `scout+settler`, so they needed their own anyway) |
| Howitzer miniature set, icon, roundel | takes the old Artillery painting (a First World War howitzer, which is what the Machine Age unit is); icon is the old Artillery drawing; roundel is its own |
| Artillery miniature set, icon | new Codex painting of a World War II 105 mm field howitzer (rubber tyres, gun shield, split trail); new icon. Roundel stays the gun class mark with the Field Cannon |
| Trench Infantry miniature set | new Codex painting: a 1916 French poilu in horizon blue, Adrian helmet, in a low crouching run (a striding bayonet charge was rejected: gist 0.78 against the Crossbowman) |
| Trench Infantry icon, roundel | helmet beside an upright bayoneted rifle (a plain helmet scored 0.83 against the Infantry's) |
| Eagle Warrior icon | redrawn as an eagle's head (it read as a bomb) |

Verification: `python3 tools/art/unit-audit.py` now finds no copy (highest `edge` 0.32 for a non-copy, no byte-identical icon, no shared roundel except the class marks); the distinctness check passes with the real `distinctness.json` (0 failing) and, on a config copy with the rows above removed, fails only `landship`/`tank`.

## Still open

- Class-shared roundels (the question above) and the Infantry's WWI kit.
- Artillery's miniature is wide and low (30 px tall at 48 px, the Field Cannon is similar); the rule's 36 px floor was written for figures.
- Minor items in the table (Fighter and Varangian Guard icons, the Mechanised Infantry base, stray specks) were not touched.
- The Trench Infantry's bayonet is a 1 px line at 48 px.
