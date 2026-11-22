# Rule breaks: Byzantium alternatives and a catalogue of breakable mechanics

Design research, 2026-10-17. Nothing here is decided. Decisions go to the design doc's decisions log and the `legatum-factions` skill. Every idea is checked against `.claude/skills/legatum-rules/references/game-rules.md` (one order per unit per turn, simultaneous resolution, section 9's absent systems) and against the faction method (one broken rule, one drawback that pushes further into the style, a 2 VP goal different in kind, a visible counter, AI on both sides, bend existing systems before adding one). Where an idea needs a new piece, it says so.

**The prompt (user, 2026-10-17):** "The Byzantium rule break of forced peace feels off. Can we think about alternatives for the core rule break for Byzantium? ... we should do some creative thinking and research on the core game mechanics that we could break in a fun way. Don't limit yourself to just Byzantium."

Contents:
1. What is off about forced peace
2. Byzantium: eight alternative core rule breaks, ranked, with a recommendation
3. A catalogue of breakable core mechanics
4. Fresh ideas for the other factions, and future factions beyond the Aztecs
5. Questions for the user
6. Sources

---

## 1. What is off about forced peace

The current kit (`game-rules.md` section 5 and 8): while at War, Byzantium may order `PayTribute{to}`; the team **cannot refuse**; Gold moves, Peace begins at the start of that turn's resolution before any attack lands, and neither side may declare war for 10 turns. Alongside it: Hire an army (a paid CallToArms to any met team at Peace), thin levies (at most cities + 1 armed units) and the Heir.

Reading the user's one line against the game's own principles, forced peace fails on six counts. Each alternative below is judged against them.

1. **It is a veto on another player's agency.** It is the only place in the game where a rival's submitted orders are voided by someone else's order ("attacks and ranged fire the two teams ordered against each other that turn find no war and lapse"). The design skill already rejected a cousin of this, the Mongol war push-out, as "magic". Every other faction rule changes what *you* can do; this one changes what *they* can do, without their consent.
2. **It is negative space, not a capability.** TI4's breaks are things you do: Nekro steals technology, the Ghosts move through wormholes, Mentak skims trade goods. "Pay to not be attacked" does not put anything new on the map. Pillar 8 wants a strength the faction leans into; there is no play style called "not playing".
3. **It fights the core loop.** Pillar 10: expansion by settling or conquest is the loop and "tall" is deliberately not viable. A faction whose signature stops conquest is tall by decree, and it makes the user's own stated stakes ("early rushes are final; the counter is scouting, then Walls and archers") false for one faction.
4. **Simultaneous turns make it feel worse than it reads.** An attacker stages for ten turns, declares, and before a single shot lands sees a "Peace established" interstitial. The Gold (10% of the attacker's army value, minimum 10) is rarely worth the tempo lost; and the attacker had no decision in it.
5. **It breaks the coalition direction the user set the same day.** "The AIs should commit to war if they're going to declare it." A coalition against Byzantium can be dissolved one member at a time with Gold, so commitment against this one faction is meaningless.
6. **It is not even the Byzantine thing.** Byzantium's tribute bought *time*, never a binding peace: the Avars and Rus took the gold and came back. What the sources celebrate is the other half of the toolkit: subsidising the enemy's enemy ("there was always someone to the enemy's rear in a position to appreciate the emperor's largesse"), intelligence, the walls, hired foreign soldiers, and outlasting everyone. The enforced 10 turns is the un-Byzantine part. Hire an army, the optional extension, is the more Byzantine rule, and it is already built.

The closest precedent in another game is Endless Legend's Drakken, whose affinity "gives the empire the ability to force truce, peace, or alliance on a targeted empire", paired with an army that is "few in number but exceedingly powerful". That is thin levies plus forced peace exactly, and it is the same genus of problem: a faction veto over the table's diplomacy. We should take the pairing (few, elite, bought) and drop the veto.

What forced peace got right, and the alternatives should keep: Gold as Byzantium's weapon, a self-scaling price (bigger armies cost more; repeat payments double), a rich Byzantium reading as a target, and the Heir as the point rivals can take.

---

## 2. Byzantium: alternative core rule breaks

Each entry: historical root; the rule it breaks and how it plays turn by turn; the compensating drawback; counterplay; the pillar it serves; feasibility against the real rules.

Thin levies (`data::levy_cap`) and the Heir are assumed to stay in every variant unless noted. The ranking is at the end of the section.

### B1. Set them against each other (Subsidies)

**Root.** "If the Bulgars threatened, subsidies could be given to the Kievan Rus. A Rus threat could be countered by subsidies to the Patzinaks. If the Patzinaks proved troublesome, the Cumans or Uzes could be contacted." John Tzimiskes in 967 set the Rus, Bulgars, Pechenegs, Magyars and Avars against one another in turn. The emperor "maintained a stable of pretenders to almost every foreign throne". Byzantium's regular army after the seventh century was 120,000 to 140,000 men; "the empire's security depended on activist diplomacy".

**The rule it breaks.** Everyone else can only call an *ally* to arms, into a war they are *already in* (`calls::check_request`: caller allied with the ally, at War with the enemy). Byzantium may pay any team it has met and is at Peace with to declare war on **any third team it has met, whether or not Byzantium is at war with it**. It funds wars it is not in. Hire an army becomes the special case where Byzantium happens to be at war with the target.

**Turn by turn.** Turn N: Byzantium submits `CallToArms{ally: H, enemy: T}` with the fee attached (one diplomatic order per other team per turn, as now). H sees the inbox proposal with the Gold on it (a human gets the existing call prompt; an AI answers by value). Turn N+1: H accepts or refuses. On acceptance at the boundary, the Gold moves, H's war on T is declared, and the `HireLock` (10 turns, no Peace between H and T) is recorded, exactly as hire does today. The event becomes **public** to every team ("Byzantium paid the Mongols 60 Gold to attack Sparta"), where today `ArmyHired` is visible only to the two teams. The replacement for "one call per ally per war" is the lock itself: one subsidy per (H, T) pair while a lock between them stands.

**Price.** Keep one price function (`tribute::quote`: 10% of an army's Production value, rounded to 5, doubling per earlier payment to the same team, minimum 10). Which army prices it is a knob: the hireling's (you pay a big army more to march, as now) or the target's (the hireling is paid for the risk). The doubling per repeat is the self-scaling cost: a Byzantium that leans on one friend pays exponentially.

**Drawback.** Thin levies already push into it: Byzantium cannot win a war of numbers, so it must make other people fight. Add one honest cost that the history insists on: **the paymaster is known.** Because the subsidy is public, the target may treat Byzantium as a belligerent. No rule is needed for that beyond publishing the event; the AI's war valuation adds "funded a war against us" as a grievance term (like the betrayal record, 20 turns, visible).

**Counterplay.** The hireling can refuse (it is an offer, not a veto). The target can declare on the thin-levied paymaster, or offer the hireling a free Alliance against Byzantium (coalition rules already waive the fee when both are at war with the target). Rivals can read the ledger: a Byzantium hoarding Gold is about to buy a war, as a Swiss hoarding is about to score.

**Pillars.** 10: a diplomatic strategy with teeth, counterable by the other diplomatic strategy. 11: a weak Byzantium can bankroll a coalition against the leader, a rubber band with a player's hand on it, which is also what the user asked of coalitions ("attack all at once and open multiple fronts"). 8: it is a true asymmetry (TI4's Hacan negotiates with non-neighbours; the Empyrean sells passage; nobody else funds wars).

**Feasibility.** Smallest of the set and mostly built: in `calls::check_request`, drop the "caller at War with the target" requirement for the Byzantium, key the one-call limit to the lock instead of the war, and make `Event::ArmyHired` public. AI: Byzantium chooses (H, T) pairs by the value of a war between two rivals (both lose Power; the one on Byzantium's border is pinned) against the Gold, through the existing plan layer; hirelings already accept by value; the target's grievance term is new but small. UI: the existing call row, offered for every met team at Peace, with a target picker over met teams, and a public report line.

### B2. The Varangian Guard (every nation's soldiers, for Gold)

**Root.** Basil II took 6,000 Rus from Vladimir in 988 and made them his guard "as they lacked local political loyalties". Later the Guard was Anglo-Saxons who had fled the Normans. The Komnenian army "expanded the employment of foreign mercenary troops" into permanent units; Huns, Cumans, Alans and Turks served as light cavalry, Franks and Varangians as heavy infantry. In Age of Empires IV the Byzantines' whole identity is recruiting other civilizations' units from Mercenary Houses with olive oil.

**The rule it breaks.** A faction can build only its own roster (`data::available_to_faction`, `faction_unavailable_reason`). Byzantium may **buy with Gold** (never train) any military unit that a faction it has met can build, including its unique pieces: a Spartan Hoplite with its phalanx bonus, a Mongol Keshig that shoots or charges, a Celtic Ambusher that hides. The unit keeps the rules of its origin.

**Turn by turn.** `BuyItem` from a city, at the normal 200% of Production cost, one purchase per city per turn (the existing limit); the unit appears next turn. It counts against the levy cap like any armed unit. Upkeep as normal, which matters: bankruptcy disbands the costliest unit first, so the Varangians leave when the pay stops, with no new rule.

**Drawback.** Thin levies are the perfect pair, and they read straight from the history (few, elite, bought). The hire is quality, the cap is quantity. If more cost is wanted: foreign units cannot be upgraded for Gold (you buy the next generation again; "the Varangians were replaced, not retrained"), which keeps Byzantium's Gold flowing out.

**Counterplay.** A mercenary army is only as good as the treasury: bankrupt the paymaster (war costs it upkeep, decadence taxes its luxury), and the best units desert first. The cap means a 4-city Byzantium fields 5 units however good; outnumber it. And you can see exactly what it bought: a Keshig in Byzantine purple is on the map.

**Pillars.** 8 (nobody else can do this), 2 (Civ players love unique units and will enjoy a faction that collects them), 10 (a Byzantium that bought Spartan phalanxes plays differently from one that bought Celtic ambushers; the choice depends on neighbours).

**Feasibility.** Medium. The buy path and the unit art exist. The work is in the rules that key on the *owner's* faction rather than the *unit's* kind: the Sparta melee line ignoring wounds, the Celts hunter concealment, the Keshig's mode choice. A `Unit::origin: Faction` (or per-kind rules) makes them travel with the unit. The AI's composition valuation (`production.rs::composition_bonus`) extends to foreign kinds. UI: the buy list grows by met factions, each item marked with its origin's heraldry.

### B3. Greek Fire

**Root.** A state secret from the seventh to the fourteenth century, "comparable in composition to modern napalm", sprayed from tubes and thrown in grenades; "rival powers tried unsuccessfully to copy the material". It broke the Arab sieges of 674 to 678 and 717 to 718 and the Rus attack of 941. AoE IV's Byzantines fling it from towers and trebuchets to "engulf the area in flames".

**The rule it breaks.** A walled city strikes **one** target a turn (`rules/battle.rs`, city strike, walls only since 2026-10-03). A walled Byzantine city's strike hits **every enemy unit adjacent to it** (the fire spreads along the wall). Strength per target is the normal city strike (or a tuned share; a knob).

**Turn by turn.** Nothing new to order. City strikes resolve in phase 2 with ranged fire, before movement, as now. The forecast shows "Greek fire: N units" on the hover card, and the attack forecast adds the line to every unit standing in the ring.

**Drawback.** Thin levies: the walls are the army, so the army is thin and must stay near them. Optional second line if the fortress reads too safe: Byzantine units themselves get no fortify bonus outside their own territory (the Strategikon's "do not seek pitched battle").

**Counterplay.** It is pure rock-paper-scissors from the class triangle: siege and ranged units strike from range 2 and never stand in the ring, so an attacker brings Catapults and Archers and keeps melee back until the walls are down; Mongols (no siege, half damage to cities) are the worst matchup and should hire help or go around. Melee rushing a walled Byzantine city is punished; melee rushing an unwalled one is unchanged, so the early rush stays final.

**Pillars.** 10 (a readable counter by unit class), 2 (city strike is Civ-familiar, this is "what if the strike were an area"), 8.

**Feasibility.** Small to medium: city-strike target selection in `rules/battle.rs` iterates the ring for the Byzantium; forecasts in `forecast.rs`; the AI's exposure maths (`tactics.rs::city_fires`) must count the ring. Art: a fire ring on the strike animation.

### B4. The Theodosian Walls (the capital does not fall first)

**Root.** The double line of the Theodosian walls was "almost impregnable for any medieval besieger" for a thousand years; the city fell twice, in 1204 to a fleet and in 1453 after the Ottomans had taken everything around it. Byzantium's story is the capital outliving the provinces.

**The rule it breaks.** A city is captured when a melee unit drops it to 0 HP (or kills the garrison through overflow). **A walled Byzantine capital cannot be captured while Byzantium owns any other city.** A melee unit that would capture it is bounced like a blocked move; its HP can still be fought down and its garrison killed, and it can still be starved of yields by units on its tiles. A one-city Byzantium is captured normally, so the user's T18 eliminations are untouched.

**Turn by turn.** Nothing new to order. Attackers see "Constantinople cannot be captured while Nicaea stands" on the city card and plan the campaign around it.

**Drawback.** Thin levies; and the rule itself pushes Byzantium toward settling buffer cities (expansion, pillar 10's loop) rather than turtling in one.

**Counterplay.** Take the provincial cities first (they are thin-levied and often unwalled), then the capital. The capital's rule is a signpost for the attacker, not a wall against him.

**Pillars.** 11 (a late Byzantium is hard to finish but easy to shrink), 10.

**Feasibility.** Small: a guard in `rules/battle.rs::capture_city` and the fused attack step in `movement.rs`; AI targeting (`attack_targets`, `conquest.rs::choose_conquest_city`) skips the capital while the guard holds; a label on the city card.

**Caveat.** This is defensive, like forced peace. The difference is that it vetoes no order, it has an obvious counter, and it rewards the attacker who widens the war. It is here because it is the most Byzantine image we have, but it should not be the only rule: pair it with B1 or B2 if chosen.

### B5. Sponsor a Revolt (the stable of pretenders)

**Root.** Michael VIII funded the Sicilian Vespers of 1282 against Charles of Anjou. Heraclius forged 400 names onto an intercepted Persian execution order and provoked a rebellion. "The emperor maintained a stable of pretenders to almost every foreign throne. These could be given funds and released to wreak havoc if their homeland threatened attack."

**The rule it breaks.** Only the Celts raise barbarian camps, and only on their own land. Byzantium may pay Gold to raise a camp inside a **rival** city's territory, at Peace or at War, having met them.

**Turn by turn.** `Action::SponsorRevolt{city}` (a new variant of the existing one-time-action order, as the rules permit) with the price shown on the city card. At resolution the camp appears on the farthest free tile of that city's territory, at least 2 hexes out, the same placement `CallTheWild` uses. It is an ordinary camp: it denies the tile's yields (general rule since 2026-10-06), it raids, and whoever clears it takes the normal Gold. The event is public ("Byzantium sponsored a revolt in Lugdunum").

**Price.** Scaled with the city's population (say 10 Gold per population, doubling per earlier sponsorship against the same team), so it bites the leader's big cities hardest and is unaffordable as spam. A cooldown per city (20 turns) keeps it from becoming a chore for the victim.

**Drawback.** Thin levies: Byzantium starts fires it cannot follow up. Publicity is the second cost: the victim knows the paymaster.

**Counterplay.** Clear the camp (two units, two or three turns; you get the Gold); garrison the frontier; or declare war on the thin paymaster, which is the decision the user liked in the old squatters rule ("is it worth a war over 1 tile?").

**Pillars.** 11 (aimed at the leader's cities, priced by their size), 10.

**Feasibility.** Medium-small: camp raising, barbarian AI and the Gold path exist; new `Action` variant, validation, a public event, the AI's target valuation (as `wild_camp_approach` for rivals), UI as a city-targeted action with a price preview like `BuyTile`.

**Risk.** It can read as griefing and, for the victim, as busywork; the user rejects chores. The cooldown and the price are the guards. It is distinct in kind from the Celts (own land, for VP) and from Switzerland (Gold hoarded, not spent).

### B6. The Bureau of Barbarians (embassies that stay for years)

**Root.** "Embassies to Constantinople would often stay on for years." The Bureau of Barbarians kept the translators and watched the envoys; the agentes in rebus of the postal system gathered intelligence. In Humankind the Byzantines' trait is "Master of Whispers": influence per Alliance. Endless Legend's Drakken are "Well Connected": they know every capital at the start.

**The rule it breaks.** Only an Alliance shares vision (`visibility.rs::shares_team_sight`). Byzantium sees what every team it is at **Peace** with (and has met) sees, one way.

**Turn by turn.** Passive. The fog lifts over its peaceful neighbours' lands and armies.

**Drawback.** Make it reciprocal and it becomes self-scaling: everyone at Peace with Byzantium sees Byzantine land too ("the sumptuous display": Constantinople showed its riches on purpose). Everyone can count the thin levies. Thin levies stay.

**Counterplay.** War blinds it. Since a surprise war takes effect the same turn, a rival who wants to hide a build-up must be at war already, which is itself a cost.

**Pillars.** 5 and 6 (information, no cheating), 10 (a kingmaker needs to see the board, so this is the natural second line under B1).

**Feasibility.** Tiny: one clause in `shares_team_sight` and the view's visibility mirror. The former TypeScript client's `visibility` mirror follows.

**Caveat.** It changes what Byzantium *knows*, not what it *does*. It is a companion to B1, not a core break on its own.

### B7. The Strategikon (never a pitched battle)

**Root.** The Strategikon of Maurice and the later De velitatione teach shadowing, ambush, refusal of battle and withdrawal into fortresses: "it is a matter of indifference which [means] one uses to reach" victory (Kinnamos). TI4's Naalu Foresight lets a fleet step out of a system when an enemy moves in.

**The rule it breaks.** A unit that is attacked fights. A Byzantine unit ordered to **Hold** in the open evades the first melee attack of the turn: it steps to a free adjacent tile farther from the attacker and no fight happens; the attacker advances into the vacated tile. A unit with no free tile, or in a city, fights normally.

**Turn by turn.** Ranged fire and city strikes still land in phase 2 (routing cannot dodge shots, as the game already says). In the movement rounds, a melee step onto a holding Byzantine unit resolves as the evade instead of the attack. The evader has used its order; it cannot attack next turn while it is still "withdrawing" (a one-turn flag), which is the honest cost of refusing battle.

**Drawback.** Thin levies (you cannot afford to lose a unit, so you refuse battle). Evading units give up fortify.

**Counterplay.** Ranged (the only class that already beats melee cleanly here), encirclement (no free tile), and mounted pursuit (the pursuit rule re-aims a chaser each round; a 4-move unit catches a 2-move evader in the next round).

**Pillars.** 10, 8.

**Feasibility.** Medium-hard, the hardest here: a reaction inside the simultaneous movement rounds (`rules/movement.rs::movement_round`), with the friend-blocking and claim rules to respect; forecasts must say "will evade"; the AI must Hold deliberately and rivals must predict the step. Fun, but fiddly in exactly the place the engine is most delicate.

### B8. Preserver of Learning (research past the gate)

**Root.** Byzantium kept Roman law and Greek learning through the centuries the West lost them; the "Macedonian Renaissance"; Constantine VII's encyclopaedic projects. The user already approved the Imperial Scriptorium on 2026-10-17: techs that rivals know are cheaper for Byzantium.

**The rule it breaks.** The strict era gate (`economy::tech_available`: only your current era's unknown techs). Byzantium may research **any technology a met living empire knows**, whatever its era, at the diffused price. It can never research ahead of everyone.

**Turn by turn.** `SetResearch` accepts the borrowed tech; the schedule shows it. Byzantium's own era advances only through its own era's last tech, so it never *enters* an era by borrowing: it is never Most Advanced through this (the crown counts furthest era entered), and it scores era objectives only for eras it entered. Preservation, not invention.

**Drawback.** Thin levies; and "copyists": Byzantium's techs that nobody else knows cost +25% (a knob; a number, so the weakest part).

**Counterplay.** None direct, and that is the problem: rivals cannot stop diffusion. It is a pure catch-up (pillar 11).

**Feasibility.** Small in `tech_available`, but it touches the era gate's interaction with VP v2 and the user said of the old salvage identity that "Science isn't really their thing". It mostly overlaps the Scriptorium already decided. Ranked last for those reasons.

### Considered and set aside

- **Bribe the garrison** (TI4 Yin "Indoctrination": pay to replace an enemy infantry with yours). Byzantium pays Gold to make an adjacent enemy unit defect. Very Byzantine (wars ended by bribing commanders), but it is the same genus as forced peace: a transaction the victim cannot refuse, resolved before fire. Listed so it is not re-proposed by accident.
- **Mercenary-only armies** (military bought, never trained): rejected 2026-10-10 as overlapping Switzerland. B2 is the version that survives: Byzantium buys *other factions'* units; the Swiss buy Great Works.
- **Can never declare war:** rejected 2026-10-10 (too passive; conquest is the loop).
- **Gold-sweetened, refusable peace offers.** This is not a faction break; it is a general diplomacy feature Civ players expect (Civ 6 peace deals carry Gold). If the user wants Gold diplomacy to survive the removal of forced peace, give every faction a `ProposeTreaty{Peace, gold}` the other side may accept. Byzantium then stands out by B1, not by owning peace.

### Ranking and recommendation

| Rank | Break | Changes what Byzantium *does*? | Counter is a decision for the rival? | New piece? | Feasibility |
| --- | --- | --- | --- | --- | --- |
| 1 | B1 Subsidies | Yes: funds wars it is not in | Yes: refuse, or attack the paymaster | None (relax `check_request`, public event) | Small, mostly built |
| 2 | B2 Varangian Guard | Yes: buys every met faction's units | Partly: bankrupt it, outnumber it | `Unit::origin` so rules travel | Medium |
| 3 | B3 Greek Fire | Changes how its cities fight | Yes: siege from range, bring ranged | None (strike targeting) | Small-medium |
| 4 | B4 Theodosian Walls | Changes how it is conquered | Yes: take the provinces first | None (capture guard) | Small |
| 5 | B5 Sponsor a Revolt | Yes: raises camps on rival land | Yes: clear it, or war on the paymaster | New `Action` variant | Medium-small |
| 6 | B6 Bureau | No: changes what it knows | Yes: war blinds it | None | Tiny |
| 7 | B7 Strategikon | Changes how its units fight | Yes: ranged, encirclement, mounted | Movement reaction | Medium-hard |
| 8 | B8 Preserver | Changes what it researches | No | Gate carve-out | Small, but overlaps the Scriptorium |

**Recommendation: B1, Set them against each other, as the core rule break, with thin levies and the Heir unchanged, and forced peace retired.** Hire an army is kept as the special case, so the change is a relaxation of a rule already in the code, not a new mechanic. It satisfies every count in section 1: it is an offer the hireling can refuse, it is a capability Byzantium exercises on the map (wars start because of it), it feeds the conquest loop rather than stopping it (someone's army marches), it gives the user's coalitions a bankroller, and it is the documented heart of Byzantine statecraft. The drawback stays self-scaling (repeat payments double; the paymaster is public and thin). Its path to 10 VP is clear: keep the neighbours at each other's throats, settle behind them, hold the Heir, take the Largest Economy crown with Gold nobody dares raid because its rivals are busy.

If the user wants Byzantium to feel more like a fortress than a court, B3 Greek Fire is the alternative core, and B4 is a strong second line under it. If the user wants the mercenary identity, B2. B6 is the natural second line under B1 in either case, and costs almost nothing.

Three identities, one choice: the court (B1 + B6), the fortress (B3 + B4), the paymaster of soldiers (B2). The faction method says one broken rule; the current kit already has two (tribute and hire), so a core plus a companion is within precedent.

**Measurement when built:** subsidies paid per game, wars started by subsidy, Gold spent against the Byzantine win rate, how often a human hireling accepts, and the human-seat benchmark; the AI-impact gate as usual.

---

## 3. A catalogue of breakable core mechanics

For each core system: how it works in Legatum today (so a break is a break of *this* game), the breaks worth considering, the game or history that inspired each, and which faction it suits. Breaks that need a system from section 9 of the rules (trade routes, religion, spies, loyalty, navy) are excluded or flagged. "Who" names a current faction or a future one from section 4.

### 3.1 Movement

Today: one order per unit per turn; terrain costs; a unit always takes at least one step; roads cost 1 and give +1 move at the start of a turn; borders are closed at Peace except for Mongol Scouts and civilians, Alliance partners and a Yam Relay holder's land; no zone of control; same-class friends block a tile; Mongols slip past enemies.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Evade** | TI4 Naalu Foresight; the Strategikon | A holding unit steps away from the first melee attack (B7) | Byzantium |
| **Legions lay roads** | Roman roads; the rules say "no unit lays a road" | A melee unit that Holds on an own or unowned tile two turns in a row builds a road tile (`Tile::road`, the existing cost function). Nobody else can build roads tile by tile | Rome (future) |
| **Interior lines** | Persian Royal Road; Prussian railways; TI4 Ghosts | A unit starting its turn in an own city may end it in any other own city linked by road, regardless of distance (one order, no fighting on the way) | Germany (Railways, see 4.3), Persia (future) |
| **The horde does not stop** | Mongol campaigns | Mongol military ending a turn on a road or open ground may take a second move order next turn only if it did not fight (a forced march, then a rest). Needs a per-unit flag; fine under one order per turn | Mongols (balance caution: already strong) |
| **Through the woods** | Celtic forests; Germanic Teutoburg | Celtic units pay 1 move for Forest and Hills; everyone else pays full | Celts |
| **Stack** | TI4 fleets | A faction whose units may share a tile two per class (same-class friends never block). Big engine change; listed for completeness | None yet |

### 3.2 Combat

Today: damage `30·e^((own−other)/25)` with 80 to 120% rolls; melee moves onto the target and both take damage; split counterattack among attackers; ranged has no retaliation, range 2; the class triangle gives +18 (melee beats mounted, mounted beats ranged, ranged beats melee by range); cities shield garrisons, strike only with Walls, and overflow can capture; fortify +3 a turn to +6; promotions are flat and automatic.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Area strike** | Greek fire; AoE IV | A walled city's strike hits every adjacent enemy (B3) | Byzantium |
| **Phalanx** | Decided 2026-10-17 | Hoplites gain strength beside another Hoplite | Sparta |
| **Captives** | Flower Wars; decided | A kill with melee adds population (Aztecs) | Aztecs |
| **Convert the defeated** | TI4 Cabal Devour; Ottoman devshirme | When an Ottoman melee unit kills an enemy unit of its own class, a Janissary appears in the nearest Ottoman city at half HP (counts against upkeep) | Ottomans (future) |
| **Absorb the engineers** | Mongols took Chinese siege engineers; TI4 Nekro | A faction that cannot build a class learns to when it captures a city holding that class's unlock building (Mongols and siege after taking a city with a Workshop). Fixes the Mongol siege misfit (4.2) | Mongols |
| **Ranged retaliation** | English longbow; Agincourt | A faction's ranged units fire back once when attacked in melee (today ranged never retaliates) | England (future) |
| **Fight twice** | Spartan drill; TI4 Sardakk | Not a number: a Spartan melee unit that kills with its attack may be ordered again next round? Breaks one order per turn. Rejected here; noted so it is not re-proposed | None |
| **Shieldwall** | Norse; hoplites | Adjacent units of a faction split damage taken as well as counterattack (the game already splits counterattacks) | Norse (future), Sparta alternative |
| **Hidden in the trees** | Exists | Celtic archers are concealed | Celts |

### 3.3 Cities and founding

Today: Settlers need population 2, cost 75 +20 each, consume 1 population; cities must be more than 3 hexes from every other city; a new city owns ring 1; the capital gets +2 Production; captured cities keep buildings and improvements; Celts raze on capture; no moving cities; Byzantium alone trains the Heir.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Dense settlement** | Japanese castle towns; Root's Marquise | Japan may found cities 3 hexes apart instead of more than 3 (a literal hive; its network bonus already wants it) | Japan |
| **Migration** | Celtic migrations; Greek apoikiai; TI4 Arborec Mitosis | Every time a Celtic city reaches population 5, a Settler appears at no Production cost (the population is consumed as usual). Nobody else gets Settlers from growth. Pushes the anti-conquest faction into the settling loop | Celts (see 4.4), Athens alternative |
| **The capital does not fall first** | Constantinople | B4 | Byzantium |
| **Citizenship** | Rome's extension of citizenship; TI4 L1Z1X Assimilate | Roman captured cities skip resistance and keep their original faction's unique buildings working | Rome (future) |
| **Successor state** | Nicaea, Trebizond after 1204 | When the capital falls, the largest remaining city becomes the capital *with* the +2 Palace bonus and a free Walls (today capital status transfers without the extras). Defensive; pairs with B4 | Byzantium second line |
| **Nomadic capital** | Mongol ordo; Roving Clans' moving cities | Rejected 2026 (moving cities). Not re-proposed | None |
| **Colonies pay home** | Delian League; Venice | A faction's cities beyond the first N send a Gold share to the capital | Athens (4.5) |

### 3.4 Borders and territory

Today: population is territory (each growth claims a tile, nearest ring first by focus); tiles are worked within 3 hexes, claimed beyond; tiles can be bought within 3 hexes at ring prices; barbarian camps and units deny the tile's yields; borders and city ownership are public; no pillage.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **The horde eats the land** | Mongol raiding; "this game has no pillage" | Mongol military units deny the yields of the enemy tile they stand on, as barbarians already do. Raiding without pillage orders, no new order | Mongols (strong already; weigh it against a drawback) |
| **Sponsored revolt** | Pretenders; Sicilian Vespers | A paid camp in a rival's territory (B5) | Byzantium |
| **Buy a rival's tile** | Swiss cantons buying towns and rights; TI4 Xxcha Peace Accords | At Peace, Switzerland may offer Gold for an unworked rival tile within 3 hexes of a Swiss city; the owner accepts or refuses (a `ProposeTreaty` kind, so it is an offer, not a veto) | Switzerland |
| **Grazing claims** | Steppe pastoralism | Unowned tiles a Mongol army stands on at turn end are claimed by the nearest Mongol city, even beyond the growth order | Mongols |
| **Hillforts** | Celtic oppida | A Celtic city founded on Hills owns ring 2 at founding | Celts (map luck; flag) |
| **Reclaim** | TI4 Winnu Reclamation | A faction whose razed or lost city sites can be re-founded within 3 hexes of other cities (the spacing rule waived for the original site) | Byzantium second line, Rome |

### 3.5 Production and buying

Today: Production banked per item; buying at 200% of Production (Swiss 175%); one purchase per city per turn (item or tile); Germany alone runs continuous Industry projects; improvements are built from the city list; Switzerland alone buys Great Works; the Heir cannot be bought.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Buy every nation's soldiers** | Varangians; AoE IV mercenaries | B2 | Byzantium |
| **The bank never closes** | Swiss banking | Switzerland may make any number of purchases per city per turn (everyone else: one). A tempo break, in theme, no new order | Switzerland alternative strength |
| **Reinforcements ride to the front** | TI4 Sol Orbital Drop | A bought Mongol unit may appear on any tile with a Mongol military unit instead of in the city | Mongols |
| **Sankin-kotai** | Daimyo attendance at Edo; TI4 trade goods | A Japanese city may send its turn's Production to the capital or to another city within 5 hexes (a production item "Send to X"), so the hive pools work | Japan second line |
| **Free unit a turn** | TI4 Arborec Mitosis | Sparta's capital trains one Hoplite every N turns at no Production cost (the agoge class), counted against upkeep | Sparta alternative strength |
| **Mit'a** | Inca labour tax | Inca cities have no Gold at all (Gold yields become Production); they cannot buy and pay no upkeep; improvements and roads can be built for *other* cities' tiles | Inca (future) |
| **Cannot build infantry** | TI4 Arborec | Decided for Aztecs with mounted; the pattern "a faction cannot build a class" is one line in `faction_unavailable_reason` | Aztecs, and any future |

### 3.6 Research and eras

Today: seven eras of five techs; the strict gate (only your current era); no prerequisites inside an era; diffusion −10% per living empire that knows the tech, cap 50%; a closed Japan gets no diffusion; the Most Advanced crown counts furthest era entered then techs; one era objective per era; no tech trading.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Preserver** | Scriptorium | Research what rivals know past the gate (B8) | Byzantium |
| **Share the Learning** | Exists | Three free techs to every rival | Athens |
| **No diffusion** | Sakoku | Exists | Japan |
| **Steal on kill** | TI4 Nekro Technological Singularity; Mongols absorbing conquered expertise | When a Mongol unit kills a unit the Mongols cannot yet build, the Mongols gain that unit's unlock tech if it is in their current era (respects the gate) | Mongols (and it funds the siege fix in 4.2) |
| **Cannot research** | TI4 Nekro Propagation | A faction that researches nothing and learns only by diffusion, capture and kills. The old Byzantium salvager was a cousin and the user disliked it for Byzantium; it may fit a steppe faction better. Listed, not recommended | None now |
| **Ignore the gate once per era** | TI4 Jol-Nar Analytical | Athens may research one technology of the next era before finishing its own (the Lyceum's reach) | Athens alternative |
| **Reverse diffusion** | Guild secrecy (Greek fire, Byzantine silk) | A faction's techs do not diffuse to others (rivals get no discount from it knowing them). Weakens the rubber band; flag | Byzantium second line, Japan |
| **Era entry upgrades the army** | Prussian reforms; Meiji | When Germany enters an era, every German unit of the previous era upgrades at half Gold the same turn (one order each is not needed; it is an event) | Germany second line |

### 3.7 Diplomacy

Today: Peace, Alliance or War; Alliance fee 60 Gold (0 in a coalition against the target) and upkeep 1 Gold per city per turn for everyone on both teams; leaving inside 10 turns costs 120 and is a betrayal (20 turns, public); surprise war at Peace takes effect the same turn; war on an ally takes effect next turn; calls to arms need an Alliance and an existing war, one per ally per war; first contact needed for Alliances; coalitions are derived, not ordered; Byzantium alone pays tribute and hires.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Fund wars you are not in** | Byzantine subsidies | B1 | Byzantium |
| **Allies pay you** | Delian League phoros; Hegemony's class economy | Athens pays no Alliance fee or upkeep; its partners pay their upkeep **to Athens** instead of to nobody. Drawback: Athens can never betray (no war on an ally, no leaving inside the window), and its partners leave free at any time (the allies revolted). See 4.5 | Athens |
| **Sell passage** | TI4 Empyrean Aetherpassage; Swiss passes | Switzerland may grant Peace passage through its land to a chosen team for a flat fee per turn (a treaty kind). Note: "traffic tolls" were rejected because no trade routes exist; this tolls units, not trade. Flagged, not recommended, since it is close to the rejected idea | Switzerland |
| **No surprise against the ready** | Spartan readiness | War declared on Sparta takes effect next turn, not the same turn (Sparta always sees it coming). Sparta's own declarations stay surprise. A sequencing exception; the user allows those rarely | Sparta second line |
| **Honour** | Japanese bushido; TI4 Argent Zeal | Japan's wars take effect next turn and are public a turn early (drawback), and in exchange its Alliance upkeep is nothing | Japan alternative drawback |
| **Marriage ties** | Byzantine and Habsburg dynastic marriages; TI4 Mahact Hubris inverted | Byzantium's Alliances need no first contact and cost no fee; partners cannot be called to arms against Byzantium by anyone | Byzantium second line |
| **The kurultai** | Mongol succession councils | Mongol wars must be re-declared every 30 turns or lapse to Peace (the horde goes home to choose a khan). A drawback with history, a timer the whole table can read | Mongols alternative drawback |
| **Refusable Gold peace** | Civ 6 peace deals | A general feature, not a break: `ProposeTreaty{Peace, gold}` for everyone | All |

### 3.8 Victory, crowns and overreach

Today: 10 VP solo (20 for a pair), held 10 turns; three live crowns (Largest Economy, Strongest Army, Most Advanced) with upkeep after 30 turns held; one era objective per era; a Great Work (2); a faction goal (2); decadence cuts every yield in proportion to the lead over the median; victory holds are public with a countdown.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Skim the rich** | TI4 Mentak Pillage; Byzantine silk and the imperial monopoly | Whenever a met empire pays decadence on Gold ("luxury"), Byzantium takes a quarter of the Gold cut as income (the leader's luxuries are bought from Constantinople). A rubber band in a player's hands | Byzantium second line |
| **Outlast** | Dune's Spacing Guild wins if nobody wins; Byzantine longevity | If the game reaches the Future era with nobody holding the target, a faction with the most turns alive without losing a city scores a permanent VP. Overlaps the late-game VP question the user deferred; park it | Byzantium, future |
| **The prophecy** | Dune's Bene Gesserit | Rejected family (predict-the-winner needs a crowd; the proposals doc, pattern 4) | None |
| **Crown without upkeep** | Swiss neutrality | Switzerland pays no crown upkeep on Largest Economy, but pays double on Strongest Army. A number rule; weak | None |
| **Dominance** | Root's dominance cards | A faction may trade its faction goal's 2 VP for a positional condition everyone can see and break (hold three named cities). Our Live goals already are this | Pattern exists |
| **Shared victory** | Pax Pamir's coalitions; the two-human team | Nothing to add: the permanent team is the shared win | Exists |

### 3.9 Upkeep and Gold

Today: 2 free military units, 1 Gold each beyond (Swiss 3); Scouts and civilians free; bankruptcy disbands the costliest unit each turn the deficit lasts; Swiss interest and capture bounty; Alliance upkeep per city; decadence on Gold.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Helots** | Spartan helotage, the uncomfortable truth the lore note names | Spartan military units cost no Gold upkeep; each costs 1 Food a turn from the nearest Spartan city instead. The army eats the countryside; a big army starves growth (self-scaling), and the Food is visible in the city breakdown | Sparta alternative drawback |
| **Themes** | The theme system: soldiers settled on land | Byzantine units standing in own territory pay no upkeep; outside it they pay double. Pushes a thin army home | Byzantium second line |
| **The mercenary export** | Reisläufer; Swiss Guard | Whenever a rival buys a military unit with Gold, Switzerland receives 10% of the price (the Swiss supplied Europe's armies). Mentak-style; a bonus more than a break | Switzerland |
| **No money** | Inca economy | Inca have no Gold at all (3.5 Mit'a) | Inca (future) |
| **Pay in land** | Pronoia | A faction whose units may be paid by granting them a tile: the tile's yields go to nobody while the unit lives (the unit is free). A pronoia ledger is a new system; flag | Byzantium (rejected here as a whole mechanic) |
| **Mercenaries desert at zero** | Carthaginian mercenary revolt | A faction whose entire army disbands the turn its treasury hits 0 (today: one unit a turn). A brutal, readable drawback for a faction that buys armies | Carthage (future) |

### 3.10 Map knowledge and fog

Today: sight by unit and terrain (+1 on hills); only Alliances share vision; Celtic hunters are hidden until they fire; borders, city ownership, Power, army strength, VP and treasuries are public; the Yam Relay's roads are public.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Embassies** | Bureau of Barbarians; Humankind Master of Whispers | Sight shared with everyone at Peace (B6) | Byzantium |
| **Know every capital** | Endless Legend Drakken "Well Connected" | Mongol scouts' yam: the Mongols know every capital's location at turn 1 (fog stays, the marker shows). Tiny, flavourful, helps the rush AI and the human; weigh against the rush strength | Mongols |
| **See their production** | Intelligence | A faction that sees what every met city is building (production is fog-limited today). Espionage-adjacent without spies | Byzantium second line |
| **Darkness** | Celtic forests | Celtic *cities* in Forest are hidden until a unit is adjacent (today cities are public on explored tiles by the user's 2026-10-04 rule). Conflicts with a decided rule; not recommended | None |

### 3.11 Turn order and simultaneity

Today: all orders resolve together; surprise wars and tribute first; ranged before movement; movement in rounds; reinforcing your own city resolves first within a round (one of the "rare deliberate sequencing exceptions"); capture ends orders aimed at the city.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Initiative** | TI4 Naalu Telepathic | A faction whose units move first in every movement round (they take contested tiles, they reinforce first everywhere). Readable but it touches the engine's fairness core; the user allows sequencing exceptions rarely | Mongols (too strong), none recommended |
| **Delayed declarations** | 3.7 Honour, No surprise against the ready | War takes effect next turn for or against a faction | Japan, Sparta |
| **Peace before fire** | What tribute does today | Keep only as the general refusable Gold peace, resolved at the boundary like any treaty | All |

### 3.12 Barbarians, camps and civilians

Today: camps spawn, guard, scout and raid; clearing pays Gold; barbarians capture lone civilians and walk them to camp; Celts raise camps on their own land for VP; Settlers and the Heir are captured in place by a military unit.

| Break | Inspiration | How it works here | Who |
| --- | --- | --- | --- |
| **Raise camps on rival land** | Pretenders | B5 | Byzantium |
| **The tribes join** | Mongol confederation; TI4 Cabal | When a Mongol unit clears a camp, its guard joins the Mongols as a unit instead of paying Gold | Mongols |
| **Ransom** | Byzantine hostage princes | A captured civilian may be released to its owner for Gold (a treaty kind with the unit attached). Needs a transaction shape; small | Byzantium second line |
| **Hostage** | Royal hostages at Constantinople | While Byzantium holds a rival's captured Settler in a city, that rival cannot declare war on Byzantium (the hostage). Another veto; rejected on the section 1 test | None |

### The three most exciting

1. **Who pays whom in an Alliance** (3.7): subsidies for Byzantium and phoros for Athens turn diplomacy from a fee schedule into a lane with winners, and both reuse the treaty shapes we have.
2. **Who may build what** (3.5, 3.2): buying other factions' units (Byzantium), learning a class by capture or kill (Mongols), converting the defeated (Ottomans). The unit roster is the most visible thing on the map, and today it is identical for everyone except a handful of renamed pieces.
3. **Where Settlers come from and where cities may go** (3.3): Settlers from growth (Celts), 3-hex spacing (Japan), legions that lay roads (Rome). The settling loop is the core loop, and only its prices are asymmetric today.

---

## 4. Fresh ideas for the other factions, and future factions

The factions skill names these misfits and open questions: the Mongols' no-Walls and weak-siege rules ("Mongol forces historically conducted major sieges"), Germany's slow units ("real German armies were not inherently slow"), the Athens/Athens redesign, and faction-level rock-paper-scissors. Each proposal follows the method and names what changes.

### 4.1 Switzerland (Switzerland): sound

Interest, Great Works, the capture bounty and the Reserve hang together and the user confirmed the uncapped hoard on 2026-10-17. Nothing to change. Two options if a stronger *capability* is ever wanted: the bank never closes (3.5: unlimited purchases per city per turn) or buying a rival's tile by offer (3.4). The mercenary export (3.9) is a bonus, not a break.

### 4.2 Mongols (Mongols): fix the siege misfit by absorption

Keep +1 move, slip-past, Scout and civilian passage, the Yam and no Walls (nomads). Replace "cannot build siege, half damage to city HP" with **absorb the engineers**: the Mongols cannot build siege units *until they capture a city that holds a Workshop*; from then on they build siege normally, and the half-damage rule ends. The story is exact (Chinese and Persian engineers served the khans), the early rush must still strike before Walls rise (the drawback bites hardest where it should), and a Mongol that has taken one developed city becomes the siege power history says it was. The counter is unchanged and visible: Walls first; now also "never let them take a Workshop city". AI: `faction_unavailable_reason` becomes state-dependent (one flag on the player); rivals' valuation of a Mongol capturing a Workshop city rises. Alternative drawback if the horde still runs away: the kurultai (3.7), wars lapse every 30 turns unless re-declared.

### 4.3 Germany (Germany): re-skin the drawback as the two-front problem

Industry projects fit. The −1 move everywhere reads as "Germans are slow", which is false. Make it **slow abroad, fast at home**: German units move normally inside German territory and on roads (the railway state) and −1 outside it (encircled, far from the depots). Same lever, a true story (Moltke's railways, the Schlieffen fear), and it pushes Germany to lay Roadworks and fight on its own ground. Feasibility: the move-cost function already knows the tile owner. Alternative capability if the user wants one: era entry upgrades the army (3.6), or interior lines by rail (3.1) from Railways onward.

### 4.4 Celts (Celts): add migration, soften the stereotype

Attrition on home ground and hidden archers fit the land-knowledge story and should stay. The rename note worries that automatic razing flattens "several peoples into a stereotype". Replace raze-on-capture with **migration**: every Celtic city that reaches population 5 spawns a Settler at no Production cost (the population is consumed), and Celtic captured cities are kept but lose their Walls family (the oppidum is rebuilt the Celtic way). The faction that punishes invaders becomes the faction that spreads, which is the loop the user wants for everyone, and the goal (Raise the Warbands, one camp per city) scales with it in exactly the inconvenient way the user likes. Counter: snipe Settlers, which the user says is "always worth the war".

### 4.5 Athens (Athens): the Delian League

The open question in the skill. Today Athens is "+6 Science in a Library replacement, 30 Science from ruins, a weaker Market", which is numbers. Proposal: **phoros**. Athens pays no Alliance fee or upkeep; each partner's upkeep (1 Gold per city per turn) is paid to Athens. Drawback: Athens can never betray (no `BreakTreaty` inside the window, no war on an ally, ever), and partners may leave free at any time (Naxos, Thasos, Samos all revolted). Goal unchanged: Share the Learning. Counter: refuse Athens' Alliances, or join and leave when it suits you; a coalition against a rich Athens costs nothing (the free-Alliance rule). Caveat: the user said diplomacy is Byzantium's lane (rejecting a Triple Alliance for the Aztecs). B1 is war by proxy and this is alliance economics; they are different lanes inside diplomacy, but the user should rule on whether two factions may live there. If not, the research break "ignore the gate once per era" (3.6) is the Lyceum's version.

### 4.6 Sparta (Sparta): the drawback could be helots

The kit works (veterans, kill healing, slow rest, desertion, retirements). The lore note says helotage "should not be romanticized"; it can be *mechanised* honestly: Spartan units pay upkeep in Food from the nearest city instead of Gold (3.9). A big Spartan army starves its own growth, which is the real Spartan trap (oliganthropia, the shortage of citizens). Offered only if desertion at peace ever proves unfun; do not stack both.

### 4.7 Japan (Japan): sound after Sakoku

The hive and the vow fit. Dense settlement (3.3, cities 3 hexes apart) is the one break that would make the hive literal; weigh it only if the network bonus alone does not produce the look the user wants.

### 4.8 Aztecs (decided, not yet built)

Flower Wars (captives as population), no mounted class, the New Fire. Sound. One note from the research: the Aztec tribute provinces are the obvious second line if ever needed (captured cities send a population share of Gold each turn, like the Swiss bounty spread over time), but it would overlap Byzantium's Gold lane; leave it.

### 4.9 Future factions beyond the Aztecs

Each in the method's shape: story, one break, one drawback that pushes into it, a counter, and a goal different in kind from the existing nine (hoard, vow, held unit, held building, every-city building, camps, retirements, broadcast, sacrifice).

| Faction | Break (what only they can do) | Drawback | Counter | Goal sketch (2 VP) |
| --- | --- | --- | --- | --- |
| **Rome** | Legions lay roads: a melee unit holding two turns builds a road tile (3.1); captured cities keep working and skip resistance (citizenship) | Bread and circuses: the capital must stay the largest Roman city or every other city loses 10% yields (push to a great capital and roads to feed it) | Cut the roads' cities; make Rome's provinces outgrow the capital by taking it | Pax Romana: hold every road-linked city for 30 turns with no city lost (live) |
| **Persia** | Interior lines by Royal Road: a unit may move between any two own road-linked cities in one order; captured cities keep their faction's unique buildings and Persia may build them (satrapies) | Satrap revolts: a Persian city with no garrison for 10 turns raises a camp in its territory | Pick off ungarrisoned satrapies; cut roads | The King's Peace: every met empire at Peace with Persia for 20 turns (a vow others can break) |
| **Carthage** | Mercenary armies: military is bought, never trained, at 150% (not 200%) | The Mercenary War: the whole army disbands the turn the treasury hits 0 | Bankrupt it: raid, force upkeep, decadence | The Sacred Band: hold 1,000 Gold *and* field 8 units at once (a hoard that must also be spent; inconvenient) |
| **Inca** | Mit'a: no Gold at all (Gold yields become Production); cities build improvements and roads on other cities' tiles; no upkeep | Cannot buy anything, ever; cannot sign Alliances (no fee to pay) | Out-tempo it with purchases; it has no emergency buys | Qhapaq Ñan: every Inca city linked by road to the capital, at least 6 cities (live) |
| **Ottomans** | Devshirme: a melee kill converts the victim into a Janissary at the nearest city at half HP (3.2) | Fratricide: the Ottomans may never train a civilian goal unit and their capital cannot build Walls above tier 1 (the court is the danger) | Ranged armies deny melee kills; strike the open capital | The Conquest: capture a rival capital and hold it 20 turns |
| **Norse** | Raiding: Norse units deny yields where they stand on enemy land and take 2 Gold a turn from the owner (the only pillage in the game) | Things: Norse cities beyond 4 hexes from a Norse city or a road stop growing (a scattered people) | Walls and garrisons; hunt the raiders | Danegeld: receive 500 Gold from rivals (bounties from raids and Gold peace offers), then stop raiding for 20 turns |
| **England** | Longbow: English ranged units fire back once when attacked in melee (3.2) | Island habit: English units cannot enter foreign land at Peace even as allies (no passage), and heal 0 outside own land | Mounted (beats ranged) and siege | The Parliament: hold the Largest Economy crown without the Strongest Army crown for 30 turns |
| **China** | The Great Wall: Walls may be built on border tiles (an improvement that blocks passage and fires), not only in cities | The Middle Kingdom: no Alliances; China's diffusion to others is doubled (everyone learns from China) | Go around; out-tech it with its own gifts | The Examinations: a University in every city (every-city building; overlaps Germany's kind, so prefer a different kind) |

Navy is punted, so Venice, Carthage's fleet and the Norse longships stay on land for now.

---

## 5. Questions for the user

1. **Which part of forced peace is off?** The veto ("cannot refuse"), the output (peace, so passivity), the feel in simultaneous turns (an attack voided before it lands), or the history? Section 1 argues all four, but the answer decides between a court Byzantium (B1), a fortress (B3 + B4) and a mercenary paymaster (B2).
2. **One line or two?** The method says one broken rule; the current kit has two. Is B1 alone enough, or B1 with the Bureau (B6) as its eyes?
3. **Should subsidies be public?** The recommendation makes every hire visible to the whole table so the target can retaliate. The alternative is the current two-team secrecy, which is more Byzantine and less fair.
4. **May Byzantium fund wars against a human's teammate?** Teams are permanent and never at war with each other; a subsidy against one human pins the other too. Fine, or exclude teams with a human member as targets?
5. **Keep thin levies?** Every variant assumes it. B2 (Varangians) makes it a feature; B3 (Greek Fire) could instead take "no fortify abroad".
6. **Gold peace for everyone?** Should refusable Gold-sweetened peace offers become a general diplomacy feature when forced peace goes, so Gold diplomacy survives for all factions?
7. **Germany's drawback re-skin** (slow abroad, normal at home): yes, or leave the mechanic as is and fix the words?
8. **Mongol siege by absorption** (4.2): does fixing the history matter more than the clean rule we have, given the Mongols are still above fair share?
9. **Athens in the diplomacy lane** (4.5): may two factions live in diplomacy if one is war by proxy and the other alliance economics?
10. **Celtic migration** (4.4): is "Settlers from growth" too strong for the settling race, or exactly the pressure the user wants against greedy expansion from a faction that cannot conquer?
11. **Sponsored revolts** (B5): does a camp appearing in your land read as a fun provocation or a chore? The cooldown and price are the guards; the user's chore test decides.
12. **Future factions:** which three of section 4.9 are worth a design pass after the Aztecs land?

---

## 6. Sources

Fetched 2026-10-17 with curl (text extracts; the Stellaris wiki refused automated requests, so Stellaris origins are cited from memory and not relied on):

**This repository**
- `.claude/skills/legatum-factions/SKILL.md`, `.claude/skills/legatum-design/SKILL.md`, `.claude/skills/legatum-rules/references/game-rules.md` (sections 1, 5, 6, 8, 9).
- `docs/design/faction-goals-proposals-2026-10.md` (patterns from Root, Dune, TI4, Cthulhu Wars, Vast, Oath, Here I Stand, Pax Pamir).
- `docs/design/historical-factions/flavour-redesign-2026-10-10.md`, `docs/design/historical-factions/rename-map.md`.
- `docs/learnings.md`, 2026-10-10 (tribute resolution slot), `docs/playtests/human-reasoning.md`, 2026-10-17.

**Twilight Imperium 4** (faction abilities, base and Prophecy of Kings, from the AsyncTI4 data files)
- https://raw.githubusercontent.com/AsyncTI4/TI4_map_generator_bot/master/src/main/resources/data/abilities/base.json
- https://raw.githubusercontent.com/AsyncTI4/TI4_map_generator_bot/master/src/main/resources/data/abilities/pok.json
- Used: Nekro (Propagation, Technological Singularity, Galactic Threat), Ghosts (Quantum Entanglement, Slipstream), Arborec (Mitosis), Naalu (Foresight, Telepathic), Mentak (Pillage, Ambush), Yin (Indoctrination, Devotion), Winnu (Reclamation), Xxcha (Peace Accords), Hacan (Guild Ships), Yssaril (Crafty), Saar (Nomadic, Scavenge), Sol (Orbital Drop), Jol-Nar (Analytical, Fragile), L1Z1X (Assimilate), Sardakk (Unrelenting), Empyrean (Aetherpassage), Mahact (Edict, Imperia, Hubris), Cabal (Devour, Amalgamation), Argent (Zeal), Titans (Awaken).

**Other games**
- Civilization VI Byzantium (Taxis) and Basil II (Porphyrogénnētos, Tagma, Hippodrome): https://civilization.fandom.com/wiki/Byzantine_(Civ6), https://civilization.fandom.com/wiki/Basil_II_(Civ6)
- Humankind Byzantines (Master of Whispers, Varangian Guards, Hippódromos): https://humankind.fandom.com/wiki/Byzantines
- Endless Legend Drakken (Diplomatic Pressure: force truce, peace or alliance; Well Connected), Roving Clans (Make Trade Not War, Nomadic Cities, Keys to the Market), Cultists (one city, converted villages): https://endless-legend.fandom.com/wiki/Drakken, https://endless-legend.fandom.com/wiki/Roving_Clans, https://endless-legend.fandom.com/wiki/Cultists
- Age of Empires IV Byzantines (mercenaries for olive oil, Greek fire, cisterns): https://ageofempires.fandom.com/wiki/Byzantines_(Age_of_Empires_IV)
- Root factions (Lizard Cult, Riverfolk Company): https://root.fandom.com/wiki/Lizard_Cult, https://root.fandom.com/wiki/Riverfolk_Company
- Dune (board game) and Cosmic Encounter (each faction or alien "bends or breaks one of the rules"): https://en.wikipedia.org/wiki/Dune_(board_game), https://en.wikipedia.org/wiki/Cosmic_Encounter
- From memory, not fetched: Stellaris origins (Scion, Necrophage, Void Dwellers), Spirit Island, Inis, Kemet, Oath, Pax Pamir, Hegemony, Old World, Through the Ages, Civilization VII. None of the recommendations depends on them.

**History**
- Byzantine diplomacy (subsidies to the enemy's enemy, Tzimiskes 967, pretenders, Heraclius' forged letter, Sicilian Vespers, embassies, the Bureau): https://en.wikipedia.org/wiki/Byzantine_diplomacy
- Bureau of Barbarians: https://en.wikipedia.org/wiki/Bureau_of_Barbarians
- De Administrando Imperio: https://en.wikipedia.org/wiki/De_Administrando_Imperio
- Strategikon of Maurice: https://en.wikipedia.org/wiki/Strategikon_of_Maurice
- Greek fire: https://en.wikipedia.org/wiki/Greek_fire
- Theme (Byzantine district): https://en.wikipedia.org/wiki/Theme_(Byzantine_district)
- Varangian Guard; Varangians: https://en.wikipedia.org/wiki/Varangian_Guard, https://en.wikipedia.org/wiki/Varangians
- Solidus: https://en.wikipedia.org/wiki/Solidus_(coin)
- Walls of Constantinople: https://en.wikipedia.org/wiki/Walls_of_Constantinople
- Pronoia: https://en.wikipedia.org/wiki/Pronoia
- Byzantine silk (imperial monopoly, silk as payment and diplomacy): https://en.wikipedia.org/wiki/Byzantine_silk
- Byzantine army; Komnenian army (500 regulars in 1091, Cuman allies at Levounion, mercenaries): https://en.wikipedia.org/wiki/Byzantine_army, https://en.wikipedia.org/wiki/Byzantine_army_(Komnenian_era)
- Komnenian restoration: https://en.wikipedia.org/wiki/Byzantine_Empire_under_the_Komnenos_dynasty
- Byzantine bureaucracy and aristocracy (succession, coups): https://en.wikipedia.org/wiki/Byzantine_bureaucracy_and_aristocracy
- Hippodrome of Constantinople: https://en.wikipedia.org/wiki/Hippodrome_of_Constantinople
- Delian League (phoros): https://en.wikipedia.org/wiki/Delian_League
- Liturgy (ancient Greece): https://en.wikipedia.org/wiki/Liturgy_(ancient_Greece)
- Ostracism: https://en.wikipedia.org/wiki/Ostracism
- Crypteia: https://en.wikipedia.org/wiki/Crypteia
- Devshirme: https://en.wikipedia.org/wiki/Devshirme
- Mit'a: https://en.wikipedia.org/wiki/Mit%27a
- Flower war: https://en.wikipedia.org/wiki/Flower_war
- Zollverein: https://en.wikipedia.org/wiki/Zollverein
- Kurultai: https://en.wikipedia.org/wiki/Kurultai
- Oppidum: https://en.wikipedia.org/wiki/Oppidum
