# Flavour redesign: Aztecs, Japan's goal, Byzantium's drawback

Proposal, 2026-10-10. Nothing here is decided until the user says so. Card: `flavour-redesign-2026-10-10.png`.

Checked against `legatum-rules` (one order per unit or city per turn, simultaneous resolution, §9 absent systems) and the faction method (one broken rule, one drawback that pushes further into the style, a 2 VP goal different in kind, a visible counter, AI on both sides, bend existing systems).

## Summary

| | Rule break | Drawback | 2 VP goal |
|---|---|---|---|
| **Aztecs** (new, 9th) | **Flower Wars:** a kill by an Aztec melee unit takes a captive, +1 population in the nearest Aztec city | **No horses:** no Horseman, Knight or Cavalry | **The New Fire** (the Swarm, moved): offer 30 population in one rite |
| **Japan** | Unchanged dense hive | Unchanged (−10% per hex beyond 4) | **Sakoku:** vow to close the country; 2 VP once closed 30 turns, held while no enemy soldier stands in Japan |
| **Byzantium** | Bought peace (being built) | **Thin levies:** at most one military unit per city, plus one | The Heir (unchanged) |
| | + optional **Hire an army:** a paid call to arms to anyone | | |

---

## A. Aztecs

**Story.** The Mexica of Tenochtitlan, about 1500: a lake city of canals and chinampas, the twin shrines of the Templo Mayor, Eagle and Jaguar warriors. They fought Flower Wars to take captives, not to kill. They had no horses and no wheel.

**Rule break: Flower Wars.** When an enemy empire's military unit dies and an Aztec melee unit took part in the kill (and survived), the nearest Aztec city gains 1 population (ties: lower city id).
- Only melee units take captives, as in hand-to-hand fighting; archers don't. A garrison killed by overflow counts if an Aztec melee unit hit the city that stage.
- Barbarians and Scouts give no captives, so teammates can't farm them (for example a Celts ally calling the wild).
- It reuses the kill attribution Sparta's healing already uses (`rules/battle.rs::heal_sparta_killers`). The new population claims a tile through the normal growth path.
- It scales: +1 population is worth more Food as cities grow.

**Drawback: no horses.** Aztecs can't build Horseman, Knight or Cavalry. Tanks are machines, so they're allowed.
- It pushes them onto foot armies, the melee line that takes the captives.
- Rivals get a readable counter: mounted units beat ranged, so an Aztec army can't catch archers that fall back. Ranged armies and Walls deny them kills.
- Sparta and the Aztecs both profit from kills, but differently. Sparta's kills heal and pay Gold, sustaining the army. Aztec kills grow cities, and the goal then spends that population.

**Goal: The New Fire.** This is the Swarm, moved from Japan and renamed. It's a one-time rite that offers 30 population, taken from the biggest cities first, and every city keeps 1. The rules and AI stay the same.
- **Why it works:** it's "save then burn". War produces the population and the rite spends it, so the Aztecs need wars and then shrink.
- **Calibration:** keep 30 as the starting value. The Aztecs have none of Japan's Food bonuses, so peaceful growth is lower and captives have to make up the gap. That gap is the intended cost. Tune it by playtest (sims check function only until the AI is competent).
- **Presentation, tasteful by design:** no altar or blood imagery. The rite shows the New Fire ceremony: a fire kindled on the hill at night, with torches carried out to every city. The confirm dialog previews each city's loss, as it does now. The goal icon is a torch or fire drill.

**Counter.** Deny kills with ranged armies, Walls and garrisons, and don't send melee units into Aztec land. After the rite, the Aztec cities are smaller and thinner (`swarm_weakness_value` already prices this).

**AI.**
- **As the Aztecs:**
  - Value a kill at the Food the captive's population would cost, like `sparta_kill_heal_value`.
  - Give the killing blow to melee units.
  - Prefer wars against neighbours whose armies are heavy in melee or mounted units.
  - The rite timing is the current `goal_actions.rs` Swarm logic.
- **As rivals:** reading the faction raises the value of ranged units and Walls next to an Aztec neighbour. The existing post-rite thin-target value is re-keyed to the Aztecs.

**Anything else?**
- The Japan's rules stay with Japan (inland start, +Food, network bonus, Brood Hall, Hive Spire).
- Optionally, the melee line could be named Eagle Warrior by name only, with ordinary stats, like Sparta's Oathbearer.
- Nothing else is needed. The Aztec identity is "war feeds the temple".

**A 9th faction with 8 seats.**
- **AI seats:** today AIs take `Faction::ALL[(seed % N + i) % N]` and can duplicate the humans' picks. Change it to rotate over the factions the humans didn't pick. With 9 factions, two humans leave 7 for 6 AI seats, so one faction sits out each game, varying by seed. All-AI sims already seat 8 distinct factions out of 9 with the same formula.
- **Hardcoded 8s to fix:** `% 8` in `sim/examples/playtest_bench.rs` and `wasm/examples/app_search_time.rs`, and the `length: 8` test in `app/src/factions.test.ts`.
- **Colours:** no change. Colours belong to seats, not factions (`app/src/theme.ts::PLAYER_COLORS`).
- **Faction picker:** the portrait strip is `repeat(8, …)` (`FactionPicker.css:177`). Make it 9 and check it at 1000x600 with the UI reviewer.
- **Portraits** (`leader-aztecs`, `people-aztecs`, plus heraldry): a man in Eagle-warrior dress (feathered eagle helm, quilted cotton armour, a feathered shield at rest) and a woman in an embroidered huipil with a feather and jade headdress. Behind them, the Templo Mayor and the canals of Tenochtitlan at dawn, about 1500. No weapons raised, no sacrifice imagery.

**Files.**
- **Core:**
  - `types.rs` (`Faction::Aztecs`, `ALL` becomes 9)
  - `goals.rs` (Swarm keyed to the Aztecs, constant renamed `AZTEC_NEW_FIRE_POPULATION`)
  - `rules/battle.rs` (captives)
  - `data.rs` (the mounted exclusion in `faction_excluded_building`/`available_to_faction`, names)
  - `state.rs` and `server/staging.rs` (AI rotation)
  - `ai/goal_actions.rs`, `ai/tactics.rs`, `ai/params.rs::for_faction`
- **App:**
  - `app/src/data.ts` (`FACTION_INFO`, `FACTION_SHEETS`, `FACTION_GOAL`)
  - `gameSetup.ts::aiFaction`
  - mock rules
  - `rules.generated.json`
  - picker, Guide, art
- **Delete:** Japan's "Great Migration" text and every Japan branch of the Swarm (it moves, it isn't duplicated).
- **Dependency:** the drawback only bites once the mounted class lands, so ship the Aztecs with it or after it.

**Risks.**
- **Snowball:** war, then population, then Production, then more war. The checks are the melee-only rule, ranged counters, the rite burning the gains, and imperial overreach.
- **AI quality:** the AI may fight badly and never reach 30. The fix is in the AI, not the threshold.

**Rejected:**
- **Tributary cities:** needs a tribute flow (a new mechanic) and overlaps Byzantine bought peace.
- **Triple Alliance** (cheap Alliances): diplomacy is Byzantium's lane, and it doesn't feed population.
- **Chinampas** (Farms on any lake tile): Food growth duplicates Japan, and it depends on map luck.
- **Captive promotions** (Eagle and Jaguar ranks): overlaps Sparta's promotions.
- **No wheel** (no road bonus): roads are situational, so it's too weak.
- **Peace starves the gods:** copies Sparta's desertion.

---

## B. Japan: Sakoku

**Story.** Tokugawa Japan closed itself for two centuries (1639–1853), with Dejima as its only window. It was prosperous and dense at home and shut to the world.

**Rules unchanged:** inland start, +1 Food, the network bonus within 5 hexes, −10% yields per hex beyond 4 from the capital, Brood Hall.

**Goal: Sakoku (live, a vow).** Japan may close the country (an action, public). While closed:
- Japan can't declare war or join an Alliance.
- Japan gets no tech diffusion. It learns nothing from abroad.
- No foreign unit may enter Japanese land at Peace, Mongol couriers included.

**Scoring.**
- After 30 turns closed, Japan holds 2 VP while no enemy military unit (Scouts excepted) stands in Japanese territory at the end of a turn. The VP returns when the intruders are gone.
- Japan may reopen at any time and lose the VP. Closing again restarts the 30 turns.
- Wars that others declare are fought normally, and Japan may make peace.

**Why it differs in kind.** It's a standing vow that costs Japan conquest (the fastest economy), allies and diffusion for as long as it holds the VP. Rivals can only take the points by holding ground inside Japan. The compact hive gives Japan a short, defensible border, which makes this the dense identity's goal.

**Counter.** It's public: the race shows "Sakoku: 2 VP in N turns". Rivals declare war and keep a unit alive on Japanese land, and Japan must sally out to clear it. Rivals can also out-research a Japan with no diffusion, or ally against it while it can't ally back.

**AI.**
- **Japan:**
  - Vows when the 2 VP plus 30 turns of lead time still reaches the target with time to hold it (`rite_pays` style), or at about 55% of the game.
  - Only vows when no winning offensive war is running and no Alliance it relies on is worth more than 2 VP.
  - While closed, hunting intruders is its top military task, and it garrisons border cities.
  - It reopens only if losing cities and an Alliance would save it.
- **Rivals:** a held Sakoku feeds `held_goal_denial`. A rival near the leader, or containing it, declares war and parks a unit on the farthest Japanese tile outside city strike range.

**Files.**
- **Core:**
  - `orders.rs` (`Action::Sakoku { close }`)
  - `Player::sakoku_since`
  - `diplomacy.rs` (`check_order` refusals, `passage_allowed`)
  - `economy.rs::tech_cost` (skip diffusion)
  - `goals.rs` (a new Live goal for the Japan)
- **AI:** `ai/goal_actions.rs`, `diplomacy.rs`, `military.rs`.
- **App:** `FACTION_GOAL.Japan`, the race action row, an intruder highlight on the map, the now-retired TypeScript parity client.
- **Edge case:** Japan can't close while it holds a captured Courier Hall, because the open road contradicts the vow.

**Implemented (2026-10-10, branch `claude/sakoku`; user: "Sure let's try it").** As specified, with these choices: the count is `Player::sakoku_turns` (turn ends closed) rather than a `sakoku_since` turn; the 30 turns are `SAKOKU_TURNS_QUICK` 15 speed-scaled (30 at Standard); an intruder takes the VP without resetting the count, and only reopening resets it; "enemy" means at war with Japan, barbarians included (a unit stranded at Peace is not); Alliances are between teams, so a team with a closed member signs none (its permanent teammate's war and other diplomacy are untouched); the vow is refused while in an Alliance or holding a captured Courier Hall, and may not ride with a declaration or an Alliance signature; the Swarm is kept whole as the goal rule `GoalRule::Swarm` for the Aztecs. Rules: game-rules.md §8 "Sakoku". The AI numbers are assumptions (value scoring; no balance sims yet).

**Risks.**
- **Too cheap for a peaceful builder Japan:** diffusion is the cost that bites a builder. If playtests show it's still cheap, lengthen the 30 turns.
- **Intruders as chores:** if hunting them feels like busywork, require the intruder to stand inside for 2 consecutive turns.

**Rejected:**
- **Plain Sakoku** (no intruder for 30 turns): free in any game where nobody attacks (Byzantium's 98% problem), and a one-unit poke resets it (the Long Watch problem).
- **8-city cluster:** already rejected for map luck.
- **Sword Hunt** (disband the army): the same kind as Sparta's retirements.
- **Himeji in every city:** the same kind as Germany's Engines.
- **One great metropolis:** "do your thing more".
- **Peace with every neighbour:** a freebie for a builder, and close to the Courier's Oath.

---

## C. Byzantium: thin levies

**Story.** After the plague and the loss of Anatolia, Byzantium never had the men for big armies. It survived on walls, Gold and other people's armies: it paid the Pechenegs to fight the Rus and the Cumans to fight the Pechenegs.

**Drawback: thin levies.** Byzantium may have at most one military unit per city, plus one (Scouts excepted).
- Training or buying past the cap is blocked with the reason ("Levies full: 5 of 5"), as `heir_check` does.
- Losing a city never disbands units. It only blocks new ones.
- **It pushes into the style:** Byzantium can't win wars of numbers, so it must pay (bought peace) or hire (below). Bought peace now changes how Byzantium plays, because it's the main answer to a bigger army, not a side option.
- **It scales itself:** the cap grows only by settling cities.
- **It's a decision for rivals:** "they can't outnumber me, and attacking them pays tribute". Byzantium's answer is to hire a third party.
- **It's visible:** the cap and the count are public, on the faction sheet and the army strength.
- **Delete:** the interim 120% building cost (`data.rs::building_stats_for`, the `* 120 / 100` line).
- **Keep:** the Heir. A thin army makes him a juicier target, which strengthens the goal's counter.

**Optional extension: hire an army.**
- **The rule:** Byzantium may send a call to arms to any empire it has met, Alliance or not, with Gold attached. The price is the bought-peace price for that empire, so it uses one price function. Accepting declares war on the target and takes the Gold. The hired empire can't make peace with the target for 10 turns, using the same enforcement record as bought peace, pointed the other way.
- **Unchanged:** Byzantium must be at war with the target, the hired empire may refuse, and there's one call per empire per war.
- **The play it enables:** hire the Rus against the Pechenegs, then buy peace with the Pechenegs while the Rus keep fighting.
- **Why it's small:** it bends `CallToArms` and reuses bought peace's Gold transfer and timer. No new system is needed.

**AI.**
- **Byzantium:**
  - Spends its slots on the best unit of each class.
  - Disbands obsolete units to free slots.
  - When outnumbered, compares the bought-peace price, a hire against the attacker's own enemy, and fighting on.
- **Rivals:** a Byzantium with a public army cap reads as a target when we outnumber it, and the bought-peace income already values attacking it. As a hired empire, an AI accepts when the Gold plus the war's value beats the risk (the existing war valuation).
- **A human receiving a hire offer:** it uses the existing call-to-arms prompt, so nobody waits.

**Files.**
- **Core:** `economy.rs` (the levy cap blocker), `data.rs` (the constant, deleting the 120%), `diplomacy/calls.rs` (paid calls without an Alliance), the bought-peace module (shared price and timer).
- **AI:** `ai/production.rs`, `ai/diplomacy.rs`.
- **App:** the faction sheet, the Guide, mock mirrors.

**Risks.**
- **Early rush:** a 1-city Byzantium has only 2 units. It can buy peace from the 60 starting Gold, and the "plus one" can be tuned.
- **Too passive:** watch the turn of its first war and the share of its wars ended by tribute.

**Rejected:**
- **Mercenary-only armies** (military bought with Gold, never trained): good tension with tribute, but Gold-paid armies overlap Switzerland's upkeep and mercenary history.
- **Can never declare war:** too passive, and conquest is the core loop.
- **Succession crisis without the Heir** (−yields): makes the goal mandatory, a freebie.
- **Military units +50% cost:** a number premium, not a rule.

## Recommendation

Approve all three, with "Hire an army". Build order:
1. **Byzantium's levy cap:** small. It lands with or after bought peace.
2. **Japan's Sakoku:** one action and one Live goal.
3. **The Aztecs:** biggest. Faction plumbing plus art, shipped with the mounted class so "no horses" bites.

Each lands as a playable increment and gets the UI review at 1000x600.
