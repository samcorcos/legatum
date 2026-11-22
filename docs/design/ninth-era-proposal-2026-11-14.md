# A ninth era for 1950 to 2010: proposal (2026-11-14)

Status: approved by the user 2026-11-14 ("after modern is fine, go with your names, both in"): the era goes after today's Modern, the names are World War and Modern, Air Defence and Rocket Artillery at range 3 are in. Increment 1 is being built as rules 35.

His request (2026-11-14, after abandoning the 8.9.36 Celts game): "I'm pretty convinced that we still need the extra tech level for the ww2 era tech. can you help me come up with that and do the new techs, cards, units, and stuff?"

## Where the gap is

The era called Modern already holds the Second World War: Infantry (Mass Mobilisation), Artillery (Rocketry), the Tank (repainted as a WWII tank on 2026-11-14), propeller Fighters and Bombers (Air Power), the Research Lab (Electronics). The Future era starts at about 2025 (drones, robots, fusion, stealth, recursive AI). The missing period is 1950 to 2010: jets, guided missiles, main battle tanks, mechanised infantry, computers. So the new era goes after today's Modern, not before it.

Evidence from his Celts tape (`~/LegatumRuns/bug-saves/celts-8936-abandoned-20261114/`, rules 33): eras are even in turns. His entries: Industrial T109, Machine Age T136, Modern T161, Future T191 (27, 25 and 30 turns). Athens entered the Future on T165. The game was on T204 with the last era open for 39 turns; the target length is 250. A ninth era of about 25 turns moves the end toward the target.

## Names

- Era 7, today's Modern, becomes **World War** (display name only; contents unchanged).
- Era 8, new, takes the name **Modern** (1950 to 2010).
- Era 9 is the Future.

Alternatives for era 7: Total War, Atomic Age (Civ's name; we have no nuclear weapons). For era 8: Cold War, Jet Age, Information Age.

## Thesis

The World War era is where air power arrives and dominates. The new Modern era is where the answer arrives: guided missiles defend cities from the air, and ground armies become fast. Bombers are not weakened; they get a counter that the defender has to build.

## Technologies (five, no prerequisites, as every era)

| Tech | Unlocks |
| --- | --- |
| Mechanised Warfare | Mechanised Infantry (Infantry upgrades to it) |
| Composite Armour | Main Battle Tank (Tanks upgrade to it) |
| Guided Missiles | Rocket Artillery (Artillery upgrades to it); Air Defence (city building) |
| Jet Engines | Jet Fighter and Jet Bomber (Fighter and Bomber upgrade to them; the Stealth pair then upgrades from the jets) |
| Green Revolution | Supermarket (third Food building, after Granary and Mill); Apollo Program (world wonder) |

Costs: the new era takes roughly the Future's current costs (650 to 720); the Future moves up one step (about 840 to 930).

## Units

Strengths come from the era curve, about 20% above the World War unit of the same line. The Future units move up one step with their era.

| Class | World War (today's Modern) | Modern (new) | Signature |
| --- | --- | --- | --- |
| Melee | Infantry 89, 2 moves | Mechanised Infantry, about 107 | 3 moves: infantry that keeps up with armour |
| Mounted | Tank 94, 5 moves | Main Battle Tank, about 113, 5 moves | none; the straight upgrade |
| Ranged | Artillery 77/98, range 2 | Rocket Artillery, about 92/118 | range 3, indirect fire; still needs a target its team sees |
| Air | Fighter 103, strike 5, guard 4 | Jet Fighter, about 124, strike 6, guard 4 | none |
| Air | Bomber 114, range 8 | Jet Bomber, about 137, range 9 | none |

**Air Defence** (building, Guided Missiles): intercepts air strikes on its own city, the way a Fighter on guard does, without using a hangar slot. It reuses the interception rule; it is not a new system. Stealth Bombers still pass it (only Stealth Fighters intercept them).

## Knock-ons

- `ERA_COUNT` 9, 45 technologies, `RULES_VERSION` 35 (rules-34 saves will not load).
- Era objective pool and World Age cards for the new era (placeholders from the Future's pool in increment 1, its own in increment 2).
- One more era of objectives means more VP on offer: check the VP target against game length before it ships.
- Faction lines: Germany's Panzer and the other special units need an upgrade target or an end of line in the new era; Byzantium, Aztecs and Mongols rule text checked.
- Art: five unit paintings, the building, the wonder, the takeover painting, a city-art tier. The present-day tank painting that the WWII repaint replaced is the Main Battle Tank's art on day one.
- Hard: upgrades and builds follow the unit tables automatically; deliberate use (Air Defence against a neighbour with Bombers, Rocket Artillery with spotters) is increment 3.
- The Machine Age's increments 2 and 3 are still open (objective pool, Hoover Dam, takeover painting, Hard's use).

## Increments

1. Playable era with placeholder art, after the rules-34 build ships. About a day, wall-clock; the release check (about 25 minutes a build, plus the full verify) is the bottleneck.
2. Art, takeover, its own objective pool and Age cards, Apollo Program, done together with the Machine Age's increment 2.
3. Hard's deliberate use of both new eras, tuned from his games.

## Rulings needed

1. Placement: the new era after today's Modern (this proposal), or a new WWII era before it with today's Tanks and Bombers pushed one era later.
2. Names for eras 7 and 8.
3. Air Defence: in or out.
4. Rocket Artillery at range 3: in or out.
