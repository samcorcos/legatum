# Barbarians

Part of the Legatum rules reference (index: `index.md`). Answers: How do barbarian camps, raids and the Celtic exceptions work?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

**Barbarians** (toggle, default **on**): camps from turn `FIRST_CAMP_TURN` = 10 (a third of the target at once, then one more every 12 turns; one camp per 120 land tiles), at least 6 hexes from cities and other camps;
each has a guard, a scout, up to `MAX_RAIDERS` = 4 raiders. A scout that sees a city and gets home alive triggers a 3-unit raid (melee, ranged, melee, one mustered per 2 turns) prowling
2 tiles from the city for 12 turns, then a 30-turn cooldown. Raiders capture lone civilians, fight units in the open (never a Celtic unit or civilian: the Celts are spared, below), but
**never assault/capture a city, never heal** (explicit "this game has no pillage" comment). Ranged
raiders also require the shared terrain LOS check before ordering a shot
(`crates/core/src/rules/barbarians.rs::raider_plan`, `crates/core/src/los.rs::ranged_clear`). Clearing
a camp pays `CAMP_CLEAR_GOLD` = 50 Gold.
Raid prowl, scout-return and random roam destinations must be free of units; the camp's
stationary guard is not a valid endpoint (`barbarians.rs::beside,random_tile,raider_plan`).
Movement events store axial `Hex { q, r }`; the stopped-move sentence prints odd-row offset
`column,row` from `Hex::to_offset` (`rules/movement.rs::stop_blocked_moves`).
**Wild camps (Celts, Call the Wild)** are ordinary camps with `Camp::rite = Some(raiser)`: each raise
(`barbarians::call_the_wild`, from `Action::CallTheWild`) puts one only in each Celtic city lacking a camp (`barbarians::rite_plan`)
(`tile.city == city`), on the farthest passable free tile (ties: the lowest hex; never the city's own tile, so
`RITE_MIN_CITY_DISTANCE` = 1, which keeps a young city's one ring usable) and `RITE_MIN_CAMP_DISTANCE` = 2 from any camp, with a guard
and a scout like any camp (`rite_plan` previews the exact tiles; a city with no valid tile gets none, and
the call is refused when no city has one). They raid every empire but the Celts (barbarians never attack them, below), and deny the
tiles under them and their units (above). Differences from natural camps: borders never disperse them,
they are outside the natural camp target (`BarbarianState::natural_camps`), and the raiser's team
gets 0 Gold for clearing one. With barbarians off the rite creates the camp state itself and natural
camps stay off (`spawn_camps` needs `setup.barbarians`). **A Celtic sack raises one too (2026-10-23):**
`barbarians::raise_sack_camp` puts one rite camp, with its guard and scout, on a free tile within
`SACK_CAMP_RADIUS` (3) of the ruins (§4, "Capture"); it follows every rule above and counts for Raise the
Warbands when it lands in the territory of a Celtic city that has no camp yet.
**Barbarians never attack the Celts (designer, 2026-11-02; `RULES_VERSION` 11).** "They view them as neutral or allied", all game, for every camp, natural or Celtic-raised.
`rules::barbarians::spares(state, owner)` (the owner is a Celtic empire; `PlayerView::barbarians_spare_me` for the AI) is checked in
every barbarian decision: `raider_plan` ignores Celtic units (no melee target, no ranged shot, no captive), `scout_goto` never reports a
Celtic city (so no raid is launched at one) and clears a report on a city that has become Celtic, and the raid march ignores a Celtic city. At resolution,
`movement.rs::movement_round` blocks a barbarian step into a tile holding a Celtic unit or city ("barbarians don't attack the Celts", the refusal text in `movement.rs`) and drops
a barbarian step onto a tile a Celtic unit also steps onto that round, so no meeting engagement is ever the barbarians' doing. The rule is one-sided:
the Celts can still attack barbarian units and camps, and clear them (a natural camp pays the usual 50 Gold, a rite camp pays the Celts nothing),
a camp's tile still yields nothing to whoever works it, and rivals clear camps as before, which breaks Raise the Warbands. Scope: the Celtic empire
only. A teammate or ally of the Celts gets no protection (an ambiguity of team play, resolved as "the Celts' own empire only"). Barbarians off changes nothing.
**Barbarians are no threat to the Celts either (designer, 2026-11-03, playtest 8.9.25).** The sparing was one-directional at first: a Celtic city shot at a barbarian beside it and Celtic sleepers woke for one, because `hostile`/`at_war` still read barbarians as enemies. One shared predicate now separates *hostile* (attack orders and targets: the Celts may still fight barbarians) from *threat* (everything automatic or advisory): `rules::is_threat(state, observer, other)` and `PlayerView::is_threat(observer, other)` = at war and not (a barbarian toward the Celts). It decides: a walled city's shot (`battle.rs::resolve_ranged`, `forecast::forecast_city_strike`, `city_strike_target`), the Sleep and Heal wake-ups, the garrison auto-fire target, city heal blocking (`heal_blocked_by_adjacent_enemy`), `settlement::known_threats` (the Threat lens and Settle advice), and in the AI `Ctx::threat_units` (danger, Settler danger, remembered sightings, scout exploration stops, home-defence and escort checks, `purchase_defense::city_attacks`, modernization foes, hollow march). `Ctx::enemy_units` stays the list of attack targets. In the app, `diplomacy.ts::isThreat` / `barbariansSpareMe` drive the city-under-attack alarms and map rings (`siege.ts`), route danger (`enemyReach.ts`), wounded-near-enemies, reinforcement and aftermath-risk notes, the "rival in sight" hint, turn-feedback warnings and the air-wing decision prompt (`airWing.ts::airState`; the Strike picker still lists barbarians). Attack and targeting code still reads `atWar` (`commands.ts`, `unitEra.ts`, `MapCanvas.tsx`, `path.ts` route blocking). No `RULES_VERSION` change (no save field moved).
**Barbarians deny yields** (Civ 6; user 2026-10-06): a tile holding a barbarian camp or a barbarian unit yields nothing to the city that works it, shown in the city breakdown as "Barbarians hold the tile" (`economy.rs::denied_near`, shared with the Byzantium salvage camp ; `app/src/ui/TileCard.tsx`).
