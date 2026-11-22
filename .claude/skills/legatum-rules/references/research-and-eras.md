# Research and eras

Part of the Legatum rules reference (index: `index.md`). Answers: What are the nine eras, the tech tree, unlock gates, research costs and game length?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

## Eras and technologies (era overhaul, phase A1, 2026-10-10)

**Nine eras** replace the old four (`crates/core/src/eras.rs`): Ancient, Classical, Medieval, Renaissance, Industrial, **Machine Age** (added 2026-11-14, designer: "the tech jump to modern is too big. tanks + flight is OP"; spec `docs/design/machine-age-proposal-2026-11-14.md`), **World War** (the era first called Modern; display rename only, 2026-11-14), **Modern** (1950 to 2010, added 2026-11-14 after World War; spec `docs/design/ninth-era-proposal-2026-11-14.md`), Future. Five technologies each (45 in all; the Future era is a *living data set* and may grow: the table in `eras::TECHS` is the single source, with a note on how to update it). The old 32-technology tree (Recovery, Consolidation, Integration, Inheritance) is gone: Salvage Protocols, Closed Habitats, Nutrient Synthesis, Precision Industry, Planetary Accord and Legacy Integration were deleted outright; the rest map to the nearest new technology when an old save loads (`TechId::from_name`; saves written with old names load, a plan from the ungated tree is repaired by `cities::normalize_research` the first turn it resolves).

- **The strict gate (user, 2026-10-10).** An empire may research only unknown technologies of its **current era** (`eras::current_era`: the first era that still has an unknown technology; `economy::tech_available`). There are **no prerequisites inside an era**, so any of the five can come first. `SetResearch` and `SetResearchQueue` are refused outside the current era ("X is in the Y era: finish the Z era first"); a queue holds only current-era technologies, no duplicates. Finishing the era's last technology enters the next era **that turn**: `eras::learn` (the only way a technology is learned, also for the Athens' free picks) emits `Event::EraEntered { player, era (the era entered, 1-based), first }`, public to every viewer; `first` marks the first empire into that era (`GameState::era_firsts`, shown in `ResearchView.era_firsts`; two empires finishing on the same turn: the lower player id is first). Nothing is granted on entry: the unlocks and the open techs are the reward. The research queue is empty after an era-up, so the player (or the AI, next planning pass) chooses the new era's order; stored Science carries over. The phase C full-screen era-up takeover hooks on `EraEntered`.
- **Public era.** `PlayerPublic.era` (**1-based**: 1 Ancient .. 6 Machine Age .. 7 World War .. 8 Modern .. 9 Future, the VP numbering; `ResearchView.era`, `TechOption.era`, the `eras::*` functions and `era_firsts` keys are 0-based) for every player in the public view. **Stable API for other systems (VP v2, AI, UI):** `eras::ERA_COUNT` (9), `eras::ERA_NAMES`, `eras::era_name(era)`, `eras::current_era(&techs)`, `eras::era_of(&state, player)` / `GameState::era_of(player)`, `eras::era_of_tech`, `eras::era_techs(era)`, `eras::tech_def(tech)`. The Future era is the last era and is never finished into another; an empire that knows all 45 stays in era 8.
- **Cost.** `economy::tech_cost` = the listed cost (the table below), less **tech diffusion** (10% per other living empire that knows it, capped at 50%). Byzantium uses the same research rule and builds an ordinary Library. A closed country (Sakoku) gets no diffusion. The breakdown names each line. Nothing the empire does changes a technology's price.
- **No boosts (user, 2026-10-17).** Every technology once had a boost earned from play (a Civ 6 Eureka: "Own 3 cities", "Build a Walls", 40% off). They were removed entirely (core: the conditions, `Player::boosts`/`tallies`, `Event::BoostEarned`, `TechOption.boost`; AI: the research order's boost patience; the app). The user: "just things that happen randomly and not things that affect strategic decision making". Tech costs were lowered to stand in for the average discount boosts gave, era by era, so era entry turns held (see Pacing).
- **After the last technology (designer decision 2026-10-31).** Once `eras::all_researched` is true at the start of a turn, `SetScienceConversion` chooses Gold, Food or Production empire-wide (default Gold; changeable every turn). Gold adds floor(total empire Science / 2) to the treasury; Food or Production adds floor(each city's own Science / 2) in that city, with its own yield breakdown line (`economy.rs::yields_for_item`, `player_income`; `rules/cities.rs::process_research` stops accruing Science toward research). Stored in `Player::science_conversion` (default Gold). It awards no VP. The research panel forecasts all three choices (`app/src/ui/TechPanel.tsx`).
- **Forecast.** `ResearchView.schedule` gives each queued technology's cost, turns from now and arrival turn at the current Science income, in queue order (pillar 5).
- **Unit unlocks (phase A2, built).** Every technology that used to announce a unit now unlocks it (table below); the placeholder `CardEffect::Soon` is gone. `data::unit_class(kind)` (`UnitClass` Melee/Ranged/Mounted/Air) is read by combat (`combat::class_bonus`) and `data::unit_era_tier(kind)` by the AI's quality ranking.
- **Pacing (Standard, Hard + Search, 4 players, 512 games):** see `docs/sim-results/era-overhaul/no-boosts.md` (median winner enters Classical..Future at T27/63/103/140/174/204 in 4-player games, within a turn or two of the games before boosts were removed (T27/63/103/140/173/200)). Costs are the `cost` field of `eras::TECHS`. Rules 31 (designer, 2026-11-13) had raised the five Modern costs about 30% because Modern was the shortest era in his tapes; rules 33 (designer, 2026-11-14) took that back off, since the Machine Age now does that job: the five technologies that moved down into the Machine Age cost their pre-31 prices again (388 to 422, era total 2,020), and each new Modern technology costs the geometric midpoint of the Machine Age and Future technology in the same slot (sqrt(388 x 654) = 504 .. sqrt(422 x 720) = 551, era total 2,633). There is no cost multiplier in code: the table is the price. The era-entry turns above are pre-Machine-Age sim medians for Industrial and later; recalibrate with the sim's "Era pacing" report (`telemetry.era_entries`).

| Era | Tech (identifier) | Cost (Science) | Unlocks |
|---|---|---:|---|
| Ancient | Irrigation | 24 | **+1 Food in every city** (the Granary needs no tech) |
| Ancient | Bronze Working | 26 | Workshop |
| Ancient | Masonry | 30 | Walls (Production only); Pyramids (world wonder); Quarries +1 Production (Deep Quarry) |
| Ancient | Currency | 32 | Market |
| Ancient | Writing | 38 | Library |
| Classical | Mathematics | 48 | Phalangite (Spearmen upgrade to it); units can buy **Siege** (siege tier 1: ignores Walls; Industrial and later units have it built in) |
| Classical | Engineering | 48 | Roadworks (every faction); Composite Archer (Archers upgrade to it) |
| Classical | Iron Working | 56 | Reveals Iron; Forge; Mine upgrade (Deep Mine); Cataphract (Horsemen upgrade to it) |
| Classical | Code of Laws (`CodeOfLaws`) | 62 | **Every existing city claims its best available ring-2 tile on research; later cities do so on founding** |
| Classical | Philosophy | 68 | Great Library (world wonder); +1 Science in every city |
| Medieval | Feudalism | 120 | Pikeman (`Trooper`; Phalangites upgrade to it); Mill |
| Medieval | Machinery | 126 | Crossbowman (`Marksman`; Composite Archers upgrade to it); Hagia Sophia (world wonder); Star Fort (Walls tier 3, Production only; rules 50, was Military Engineering); +1 Production per Workshop family |
| Medieval | Chivalry | 132 | Knight (Cataphracts upgrade to it); Castle (Walls tier 2, Production only); War Shrine (Sparta) |
| Medieval | Education | 136 | University |
| Medieval | Crop Rotation | 140 | Farm/Pasture upgrade (Irrigated Farm, Ranch); +1 Food per Granary family |
| Renaissance | Gunpowder | 216 | Musketman (`AssaultTrooper`; Pikemen upgrade to it); Arquebusier (Crossbowmen upgrade to it); Niter visible |
| Renaissance | Metallurgy | 224 | Cuirassier (Knights upgrade to it); Siege reaches Star Forts (tier 3). Machinery (Medieval) is tier 2 (Castles) |
| Renaissance | Military Engineering | 230 | **unit upgrades cost half** (the Star Fort moved to Machinery in rules 50) |
| Renaissance | Banking | 238 | Bank; Grand Bazaar (world wonder) |
| Renaissance | Printing Press | 248 | **+1 Science per Library and +1 Gold per Market** |
| Industrial | Rifling | 344 | Rifleman (Musketmen upgrade to it). Every Industrial or later land unit has **Siege built in** (ignores Walls, Castles and Star Forts; `data::built_in_siege`) |
| Industrial | Military Science | 352 | Cavalry (Cuirassiers upgrade to it; Siege built in); land units cost 10% less Production |
| Industrial | Ballistics (`Cannonry`) | 356 | Field Cannon (Arquebusiers upgrade to it; Siege built in); Royal Arsenal (world wonder) |
| Industrial | Industrialization | 368 | Factory; Coal visible |
| Industrial | Railways | 376 | **A unit that starts its turn on a road gets +2 moves (was +1)**; Roadworks link cities within 12 hexes (was 8); Mines +1 Production |
| Machine Age | Replaceable Parts | 388 | Trench Infantry (Riflemen upgrade to it); Factories +2 Production |
| Machine Age | Combustion | 394 | Early Tank (`Landship`; Cavalry upgrade to it); Oil Wells +1 Gold; Oil visible |
| Machine Age | Steel | 402 | Howitzer (Field Cannons upgrade to it); Eiffel Tower (world wonder); Howitzers, Artillery and Combat Drones fire indirectly over terrain at targets their team sees (`data::indirect_fire`, `los::ranged_clear_with_tech`) |
| Machine Age | Flight | 414 | Biplane (aircraft, based in cities) |
| Machine Age | Electricity | 422 | Stock Exchange |
| World War | Mass Mobilisation (`MassMobilisation`) | 504 | Infantry (Trench Infantry upgrades to it) |
| World War | Rocketry | 514 | Artillery (Howitzers upgrade to it) |
| World War | Armoured Warfare (`ArmouredWarfare`) | 525 | Tank (Early Tanks upgrade to it) |
| World War | Air Power (`AirPower`) | 539 | Fighter and Bomber; each Biplane upgrades to either, the player's choice |
| World War | Electronics | 551 | Research Lab |
| Modern | Mechanised Warfare (`MechanisedWarfare`) | 654 | Mechanised Infantry (Infantry upgrades to it; 3 moves) |
| Modern | Composite Armour (`CompositeArmour`) | 670 | Main Battle Tank (Tanks upgrade to it) |
| Modern | Guided Missiles (`GuidedMissiles`) | 686 | Rocket Artillery (Artillery upgrades to it; range 3, indirect fire); Air Defence (city building) |
| Modern | Jet Engines (`JetEngines`) | 702 | Jet Fighter and Jet Bomber (Fighter and Bomber upgrade to them) |
| Modern | Green Revolution (`GreenRevolution`) | 720 | Supermarket (the Food line's third building) |
| Future | Autonomous Systems | 840 | Combat Drone (Rocket Artillery upgrades to it); Autonomous Armour (Main Battle Tanks upgrade to it); Silicon visible |
| Future | Robotics | 862 | Hunter-Killer, a four-legged armed ground robot (Mechanised Infantry upgrades to it) |
| Future | Fusion Power | 884 | +5 Production in every city; Factory and Research Lab no longer use Coal/Oil; ITER Fusion Reactor (world wonder) |
| Future | Stealth Technology | 906 | Stealth Fighter (Jet Fighter upgrade, strike 7, guard 5), Stealth Bomber (Jet Bomber upgrade, strike 10, only Stealth Fighters intercept); both use Silicon; hangars hold 3 |
| Future | Recursive AI (`RecursiveAi`) | 930 | Reveal the whole map to the team (not unit sight); +20% Production in every city |

**Game length:** there is one pace and the data tables hold its real prices and turn counts (no game-speed setting, scale or `Quick`). No default turn limit (`None` = play until victory); an optional limit ends on
highest VP. The game's expected length is `data::EXPECTED_GAME_TURNS` = 300 (forecast horizon and AI pacing); `data::FIRST_THIRD_TURNS` = 100.
