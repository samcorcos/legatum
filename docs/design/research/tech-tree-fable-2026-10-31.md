# Tech tree pass: Fable's proposal (2026-10-31)

Answers `tech-tree-brief-2026-10-31.md`. Current unlocks were checked against `eras::TECHS`, `data::tech_card_effect_list`, `data::unit_stats`, `data::building_stats`, `resources::stats` and `improvements::upgrade_tech`, not the rules prose alone. Two things the rules reference and the code disagree on: the decided "S5 Fusion +5 Production" is not in code yet (`FusionPower => &[]`), and the one-wonder-per-era rule (decided 2026-10-31) is not built: the three Great Works are still 1-of-3 per empire. Both are treated below as decided-but-unbuilt.

## 1. Diagnosis

The tree's bones are right: seven eras, five techs each, a line unit per class per era, and a resource revealed one era early. The weak spots are of three kinds: (a) techs whose only payoff is one building or one wonder, (b) techs that give a faction nothing (Aztecs and horse techs; Mongols and Masonry before the Medieval era), and (c) the Future, where three of five techs are a yield number or nothing.

| Era | Tech | Verdict | Why |
|---|---|---|---|
| Ancient | Irrigation | Keep | +1 Food everywhere; wanted by all, matters at once |
| Ancient | Bronze Working | Keep | Workshop; the opening's Production building |
| Ancient | Masonry | Tweak | Walls (Pyramids will join). Dead for the Mongols until the Medieval era: give it the Quarry upgrade so stone pays for everyone |
| Ancient | Currency | Keep | Market, Revenue Office, Imperial Diwan |
| Ancient | Writing | Keep | Library, Lyceum, Scriptorium |
| Classical | Mathematics | Keep | Phalangite and Siege tier 1: two things, both wanted |
| Classical | Engineering | Keep | Roadworks and Composite Archer |
| Classical | Iron Working | Tweak | Iron reveal, Forge, Cataphract/Keshig, Deep Mine; the Quarry upgrade moves to Masonry |
| Classical | Code of Laws | Tweak | One rule effect only (ring-2 tile); add Alliances as its second unlock (optional, see §2) |
| Classical | Philosophy | Replace payoff | The brief's placeholder: Great Library and nothing else. Keep the wonder, add +1 Science in every city |
| Medieval | Feudalism | Tweak | Pikeman and Castle. The Castle moves to Chivalry; the Mill comes here (manor mills) |
| Medieval | Machinery | Tweak | Crossbowman and Siege tier 2; add Workshops +1 Production (water-powered machinery) and Hagia Sophia |
| Medieval | Chivalry | Fix | Knight plus the Sparta-only War Shrine: the Aztecs get nothing. Add the Castle |
| Medieval | Education | Keep | University and the Niter reveal; the era's first pick, as the designer plays it |
| Medieval | Crop Rotation | Tweak | Loses the Mill; keeps Irrigated Farm and Ranch, adds Granaries +1 Food |
| Renaissance | Gunpowder | Keep | Musketman and Arquebusier |
| Renaissance | Metallurgy | Keep | Coal reveal, Cuirassier, Siege tier 3; Aztecs still get Coal and Siege |
| Renaissance | Military Engineering | Keep | Star Fort and half-price upgrades |
| Renaissance | Banking | Keep | Bank and the Grand Bazaar |
| Renaissance | Printing Press | Tweak | +1 Science per Library only; add "techs rivals know cost 10% less" (the press spreads knowledge) |
| Industrial | Rifling | Keep | Rifleman: a line unit every faction builds every game is not a placeholder |
| Industrial | Military Science | Fix | Cavalry only: the Aztecs get nothing. Add faster healing in own territory |
| Industrial | Ballistics | Tweak | Field Cannon; the Royal Arsenal moves here from Industrialization (an arsenal is a gun foundry) |
| Industrial | Industrialization | Tweak | Factory/Ruhr Foundry, Oil reveal, Germany's movement relief; loses the Royal Arsenal so the reveal and the wonder are a sequencing choice |
| Industrial | Railways | Tweak | +2 road moves only; add Roadworks reach 12 and Mines +1 Production (steam pumps) |
| Modern | Replaceable Parts | Tweak | Infantry only; add Factories +2 Production (assembly lines) |
| Modern | Combustion | Tweak | Tank only; add Oil Wells +1 Gold (refineries) |
| Modern | Steel | Tweak | Artillery and indirect fire; add the Eiffel Tower |
| Modern | Flight | Keep | Fighter and Bomber; the Future gives them an upgrade |
| Modern | Electricity | Keep | Research Lab and the Silicon reveal |
| Future | Autonomous Systems | Keep | Combat Drone and Autonomous Armour: the only Future tech that already lands |
| Future | Robotics | Replace unit | Humanoid Robot is rejected (designer). Exoskeleton Infantry takes the slot; add automated extraction (+1 on Mines, Oil Wells, Quartz Mines) |
| Future | Fusion Power | Replace payoff | Nothing in code. Build S5 (+5 Production everywhere), free Factories and Research Labs from Coal and Oil, and site the Future wonder here |
| Future | Quantum Computing | Replace | "+4 Science per Research Lab" at the end of the tree is a number nobody feels. Its slot becomes Stealth Aircraft, the aircraft upgrade the designer asked for |
| Future | Recursive AI | Replace payoff | "+2 Science per Library" as the last tech is pointless once Science turns to Gold. Make it the climax: the map revealed, upgrades free |

## 2. The proposed tree

Bold marks a change from today. Wonder = the era's unique world wonder (2 VP, one per era, decided 2026-10-31). Every tech now gives every faction at least one thing it can use; where a faction cannot use an unlock, the companion is named.

| Era | Tech | Units | Buildings and wonder | Rule effects and reveals |
|---|---|---|---|---|
| Ancient | Irrigation | | | +1 Food in every city |
| Ancient | Bronze Working | | Workshop | |
| Ancient | Masonry | | Walls (Alpine Redoubt); **Pyramids (wonder)** | **Quarries +1 Production (Dressed Stone; moved from Iron Working)** — the Mongols' payoff before the Medieval era |
| Ancient | Currency | | Market, Revenue Office, Imperial Diwan | |
| Ancient | Writing | | Library, Lyceum, Scriptorium | |
| Classical | Mathematics | Phalangite | | Siege tier 1 (Mongols: from the Medieval era) |
| Classical | Engineering | Composite Archer | Roadworks | |
| Classical | Iron Working | Cataphract / Keshig (Aztecs: Iron, Forge, Deep Mine) | Forge | Reveals Iron; Mines +1 (Deep Mine) |
| Classical | Code of Laws | | | A new city (and, S3, existing ones) claims its best ring-2 tile; **optional: Alliances become available (Peace and War only before it)** |
| Classical | Philosophy | | Great Library (wonder) | **+1 Science in every city** |
| Medieval | Feudalism | Pikeman / Legionary | **Mill (moved from Crop Rotation)** | |
| Medieval | Machinery | Crossbowman | **Hagia Sophia (wonder)** | Siege tier 2; **Workshops +1 Production** |
| Medieval | Chivalry | Knight (Aztecs: the Castle) | **Castle (moved from Feudalism)**; War Shrine (Sparta) | |
| Medieval | Education | | University | Reveals Niter |
| Medieval | Crop Rotation | | | Irrigated Farm, Ranch; **Granaries +1 Food** |
| Renaissance | Gunpowder | Musketman, Arquebusier | | |
| Renaissance | Metallurgy | Cuirassier (Aztecs: Coal, Siege) | | Reveals Coal; Siege tier 3 |
| Renaissance | Military Engineering | | Star Fort | Upgrades cost half |
| Renaissance | Banking | | Bank; Grand Bazaar (wonder) | |
| Renaissance | Printing Press | | | +1 Science per Library family; **techs a rival knows cost 10 points less (inside the 50% diffusion cap)** |
| Industrial | Rifling | Rifleman | | Industrial units have Siege built in (era rule) |
| Industrial | Military Science | Cavalry (Aztecs: the healing) | | **Units heal 30 a turn in own territory, not 20 (Sparta keeps its 2)** |
| Industrial | Ballistics | Field Cannon | **Royal Arsenal (wonder, moved from Industrialization)** | |
| Industrial | Industrialization | | Factory / Ruhr Foundry | Reveals Oil; Germany moves normally |
| Industrial | Railways | | | +2 moves from a road; **Roadworks link cities within 12 (was 8)**; **Mines +1 Production (Steam Pumps)** |
| Modern | Replaceable Parts | Infantry | | **Factories and the Ruhr Foundry +2 Production** |
| Modern | Combustion | Tank | | **Oil Wells +1 Gold (Refinery)** |
| Modern | Steel | Artillery | **Eiffel Tower (wonder)** | Artillery fires indirectly |
| Modern | Flight | Fighter, Bomber | | |
| Modern | Electricity | | Research Lab | Reveals Silicon |
| Future | Autonomous Systems | Combat Drone, Autonomous Armour | | |
| Future | Robotics | **Exoskeleton Infantry (Infantry upgrades to it; replaces the Humanoid Robot)** | | **Mines, Oil Wells and Quartz Mines +1 (Automated Extraction)** |
| Future | **Stealth Aircraft (replaces Quantum Computing)** | **Stealth Fighter (Fighter upgrades), Stealth Bomber (Bomber upgrades)** | | **Hangars hold 3 aircraft (was 2)** |
| Future | Fusion Power | | **Fusion Reactor (wonder)** | **+5 Production in every city (S5)**; **Factories and Research Labs no longer use Coal or Oil** |
| Future | Recursive AI | | | **The whole map is revealed to your team (terrain, cities, improvements; units still need sight)**; **unit upgrades cost no Gold (Silicon supply still limits them)** |

Notes on the choices:

- **Wonder and reveal never share a tech.** Today Industrialization holds both Oil and the Royal Arsenal, and Hagia Sophia would naturally land on Education beside Niter. The designer's favourite decision is "get the reveal first"; if the wonder sits on the same tech there is no decision. So the Arsenal goes to Ballistics and Hagia Sophia to Machinery (treadwheel cranes raised the great domes), and each era asks "reveal first, or race the wonder first?"
- **Castle to Chivalry, Mill to Feudalism, Granary +1 to Crop Rotation** is one chain of three data moves whose only purpose is to leave no Medieval tech empty for the Aztecs and no tech with a single yield line. Castles and knights are one historical package; manor mills are feudal.
- **Philosophy +1 Science per city** mirrors Irrigation's +1 Food: flat, visible, valued by the AI already (`research.rs::income_change` handles Irrigation the same way).
- **Printing Press and diffusion** reuses the Scriptorium code path (`SCRIPTORIUM_DIFFUSION`) as a general per-player discount; Byzantium stacks to 20 points inside the same cap. It is a catch-up tool with the right story, and the user keeps diffusion as the rubber band.
- **Code of Laws and Alliances** is the one change that touches diplomacy. Civ 6 gates alliances behind a civic; we have no civics, so a Classical tech is the place. It is optional because allied defensive wars shape early coalitions; gate it on the war scenarios and the human-seat benchmark, and drop it if wars-without-battles or the Sparta rush balance move.
- **Military Science healing** is the Industrial-era medicine story (Crimea, field hospitals) and the first universal effect the Aztecs get from a horse tech. Sparta's 2 HP rule stands.
- **Where we differ from Civ 6:** no civics tree, so civic-like unlocks (Alliances, Philosophy's Science) sit on techs; no Atomic era and nothing nuclear (Silicon over Uranium, decided); the air line is Fighter/Bomber then Stealth, not Biplane/Fighter/Jet, because our Modern era already spans the twentieth century; the map reveal is on Recursive AI rather than a Satellites tech, since the Future era is the near future and a superintelligence that fuses every sensor is the designer's own item.

## 3. Modern and Future in detail

**Modern.** The era is already structurally right: three line units, aircraft, the Research Lab, the Silicon reveal. Its problem is thinness, not shape. Each unit tech gains one economic line (Factory +2, Oil Well +1 Gold, the Eiffel Tower) so the order you take them in is a choice between the army and the economy, as in the earlier eras. Siege is built in, the triangle has ended, so Modern war is decided by strength, cost, moves and air: the Tank's 5 moves and the Bomber's 8 range are the era's identity and stay.

**The aircraft line.** Design numbers unchanged, ridden up the era curve like every other unit (`data::era_scaled`):

| Unit | Era | Cost | Strike / vs city | Strike range | Guard | Upgrades from |
|---|---|---|---|---|---|---|
| Fighter | Modern | 400 | 103 | 5 | 4 | — |
| Bomber | Modern | 460 | 114 (+36) | 8 | — | — |
| **Stealth Fighter** | Future | ~550 | ~139 | **7** | **5** | Fighter |
| **Stealth Bomber** | Future | ~635 | ~153 (+48) | **10** | — | Bomber |

Both use Silicon (the era rule; an upgrade switches the plane from Oil to Silicon, as a Pikeman switches to Niter), complete into hangars, and keep the one-order-a-turn model, so no new order type and no map art. Hangars grow to 3 under Stealth Aircraft so a Future airbase is a real target. Stealth is in service today (B-2, F-35) and in production (B-21), so it is the plausible near-future step; drones already name the ground ranged unit, so the air line keeps "Stealth".

**What counters aircraft.** The designer ruled (2026-10-27) that Bombers stay decisive against Walls and that air superiority is the counter, so nothing here cuts air damage. The counters, all existing: a Fighter on guard intercepts the strike (Stealth Fighters guard to 5, so they cover more); taking or razing the hangar city destroys every plane in it, and a 3-plane hangar is worth a campaign; Oil and later Silicon supply, since an unsupplied plane strikes 25% weaker and a deposit taken is a wing grounded; and Autonomous Armour's 6 moves, which reach an airbase before it can rebase twice. One optional addition, not in the first increment: an **Air Defence** building (Electricity, radar-directed guns) that intercepts one strike a turn like a Fighter on guard. It reuses `air::STAGE_AIR`, gives a city without planes an answer, and is how real wars work; but it brushes the 10-27 ruling, so it goes to the designer as a question, not into the build.

**The Future as climax.** Today the Future is one good tech and four numbers. The proposal makes each of its five techs change the board in a way everyone sees:

1. **Autonomous Systems:** the fastest army in the game (6-move armour, 3-move drones). Unchanged.
2. **Robotics:** the strongest infantry (134), and every extraction tile pays more. Aztecs and everyone get both.
3. **Stealth Aircraft:** range 10 bombers from 3-plane bases. The air war reaches across the map.
4. **Fusion Power:** +5 Production everywhere, and Factories and Research Labs drop their Coal and Oil, which frees that supply for Tanks, Artillery and the planes that still burn it. Energy independence makes the army bigger, which is what the Future should do. The Fusion Reactor is the wonder (ITER is real and under construction; a tokamak dome is good art).
5. **Recursive AI:** the fog lifts for your team and your whole army re-equips for free, Silicon permitting. The first empire there sees every rival's position and fields a Future army at once; tech diffusion (capped at 50%) is the rubber band that brings the others in behind it.

With Science turning to Gold at 50% once the tree is done (decided), the last era's arc is: enter the Future, see the three cards, race the reveal, the wonder and the two military techs in the order your situation demands, then convert the research economy into Gold and buy the final war. No research is a path to victory; everything the Future gives is a way to take cities or hold points. The Future's living-data-set rule holds: Stealth Aircraft, Exoskeleton Infantry and Fusion are all engineering in flight today, nothing humanoid, nothing nuclear.

## 4. Cost to build, and the skateboard

**Rules (Rust core), about a day of agent time.** Nearly everything is a data row or a one-line effect on an existing lever: the `TECHS` and `tech_card_effect_list` tables; `requires` for Castle, Mill and the Arsenal; `upgrade_tech` for the Quarry, Mine, Oil Well and Quartz Mine steps; three new unit kinds (Exoskeleton Infantry with a serde alias from HumanoidRobot, Stealth Fighter, Stealth Bomber as `UnitClass::Air` with their own constants); `AIR_BASE_CAP` and `ROADWORKS_RANGE` as per-player values; the Printing Press discount through the Scriptorium path; the heal constant; `building_need` returning None for Factory and Research Lab once Fusion is known; a map reveal that marks every tile explored for the team; a zero upgrade price after Recursive AI. Wonders are a separate decided task (four new building kinds, uniqueness, tile sites); this proposal only names their techs. The optional Alliance gate and Air Defence are the two items that are new rules rather than new values.

**AI, about a day.** Research valuation (`crates/ai` research order) already prices units, buildings and yields; it needs terms for the map reveal, free upgrades, the heal rate, the diffusion discount and the hangar cap. `hold_or_upgrade` handles the aircraft upgrades once `upgrades_to` is set. The known hazard is aircraft supply: aircraft on Oil cost Hard about 10 benchmark points in 2026-10-23, so every change here goes through the same-seed master-vs-branch benchmark the designer set for the three revived rules.

**Art, the long pole (days, bounded by image generation and review).** Four wonder landmarks and cards (Pyramids, Hagia Sophia, Eiffel Tower, Fusion Reactor); one map sprite (Exoskeleton Infantry); two card paintings (Stealth Fighter, Stealth Bomber, no map art since aircraft live in hangars); upgraded-improvement variants can reuse the Deep Mine treatment at first.

**UI, half a day.** Tech cards already list effects from the core tables; new strings only. The supply panel must say that Fusion freed the buildings; the reveal gets the ink moment that `ResourcesRevealed` already has.

**Skateboard (one session, then the benchmark gate):** the data-only pass with no art and no new mechanic: Philosophy +1 Science; the Medieval chain (Castle, Mill, Granary); the Quarry step to Masonry; Workshop +1 at Machinery; Royal Arsenal to Ballistics; Printing Press diffusion; Military Science healing; Railways reach and Steam Pumps; Factory +2 and Oil Well +1; Fusion S5 and the Coal/Oil release; Recursive AI's reveal and free upgrades; Humanoid Robot renamed Exoskeleton Infantry on the Infantry sprite; Quantum Computing renamed Stealth Aircraft with Stealth Fighter and Stealth Bomber as Fighter and Bomber clones at Future numbers. That is playable end to end the same day. Scooter: the wonder system lands and the four wonders take their techs. Bicycle: art, the Alliance gate decision, the Air Defence question.

## 5. If only three changes were allowed

1. **Rebuild the Future era** as in §3: Stealth Aircraft replacing Quantum Computing (the aircraft upgrade he asked for), Exoskeleton Infantry replacing the Humanoid Robot, Fusion with S5 plus the Coal/Oil release and the wonder, and Recursive AI as the reveal-and-re-equip climax. This is the era he said does not land.
2. **Give Philosophy +1 Science in every city**, and move the Royal Arsenal to Ballistics so no tech anywhere is only a wonder, and no era puts its reveal and its wonder on the same tech.
3. **Make every horse tech pay the Aztecs**: the Castle to Chivalry (with the Mill to Feudalism and Granaries +1 Food to Crop Rotation behind it), and faster healing in own territory on Military Science. After this, no faction has a tech it researches only to leave the era.
