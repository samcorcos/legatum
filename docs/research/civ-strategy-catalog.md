# Civ strategy catalog for Legatum AI behaviours

Researched 2026-10-07 (round 2). Phone summary: `civ-strategy-catalog.png`. Machine-readable: `civ-strategy-catalog.json`. Round 1 is `civ-human-expertise.md`.

**120 strategies**, each with a link to the public source that suggested it, a fit verdict against `.claude/skills/legatum-rules/references/game-rules.md` (including section 9, the systems that do not exist), the factions it suits and a testable AI behaviour. **107 are playable in Legatum** (68 possible as written, 39 adapted) and **13 are impossible** (no religion, trade routes, spies, Great People, pillage, naval units, districts, policies, eurekas or gifts); the impossible ones are listed so nobody re-proposes them.

## Sources and limits

- **Sources.** Public community strategy writing about Civilization V and VI: Reddit threads, Steam guides and CivFanatics forum threads. Each entry links the page that suggested it, and the full list is at the end.
- **Own words.** Every entry is a summary in this project's words, followed by this project's own judgement of whether the idea fits Legatum's rules. No source text is reproduced here.
- **Limits.** The sources skew to Civ 6 single-player on Deity, so much of the advice is tuned against AI mistakes. Evidence from human multiplayer is thin. Every Legatum number comes from the rules reference as it stood on 2026-10-07 and was not re-run; faction names and rules have changed since.

## Counts

| Archetype | Primary | Tagged (incl. secondary) |
|---|---:|---:|
| Rush and conquest | 37 | 53 |
| Defence against a rush | 18 | 31 |
| Wide expansion | 17 | 25 |
| Tall and defensive | 4 | 8 |
| Economic and Gold | 7 | 13 |
| Science | 9 | 10 |
| Diplomatic and coalition | 8 | 12 |
| Late-game closing | 7 | 12 |
| Does not transfer (impossible, incl. naval) | 13 | 13 |

Fit: possible 68, adapted 39, impossible 13. Fit means: **possible** = playable as written under Legatum's rules; **adapted** = the idea works but needs a Legatum-specific substitute (named in the note); **impossible** = depends on a system section 9 says does not exist.

## Three surprising findings

1. **Stay unmet (RC24).** In Legatum contact is mutual and permanent and an Alliance needs it, but War does not. A rusher that scouts the border without being seen never gives the target the chance to ally, and the guide's own trick (see the edge of the border, then hide) works with city sight 3 and unit sight 2.
2. **The first attacker usually loses (DC05).** Experienced Civ 5 multiplayer players say that when a third party stays out and keeps growing, whoever attacks first loses the tempo race. With 8 empires this should gate the AI's early rush openings unless the edge is decisive (force ratio 2 or more).
3. **The Civ 6 AI is science-biased and decouples building from using (SC03, RC13).** A forum find showed a patch pushed every AI toward science and Campus and Theatre districts; players report it became much more vulnerable to military aggression, and that stealing one Worker from a Deity AI throws it off because its unit builder does not notice the loss. For Legatum: target the tech leader's thin garrisons and snipe Settlers, and make sure our own AI does not share the flaw.
Also worth testing: barbarian camps only spawn in fog (DF11), and banked production never decays (DF08), so a one-turn-left defender in every border city is free insurance.

## Top 12 AI experiments

Each is a gene-ready behaviour from the catalog. Per the standing rule (sims are for functionality only until the AI is competent), run them as behaviour checks per faction on scripted Hard, not as balance claims.

1. **DC05 The first attacker usually loses**: Declaration gate: refuse first strikes against a peer when two other strong empires are unengaged, unless force ratio >= 2.
2. **RC04 Quick, unexpected, brutal first strike (20-turn horizon)**: War gate: estimate turns-to-first-capture from forces and distance; refuse wars above a tunable horizon.
3. **DF06 Rush tell: low score on the first turns of the surge**: Treat a one-city neighbour within 10 hexes as a rusher until it expands.
4. **RC05 Stage outside border, declare at the last second**: Staging step: park outside sight radius, then DeclareWar and MoveUnit in the same order batch.
5. **SC03 Hit the science-obsessed AI**: If the tech leader's army share < 0.5x ours and its capital is reachable, add it as a candidate target.
6. **RC08 Pre-build cheap units, then mass-upgrade with Gold**: Stockpile 6+ Soldiers while Ballistics is 1-2 techs away; spend Gold on upgrades the turn it lands.
7. **DF08 Bank production at one turn left**: Border cities keep a Soldier at <= 1 turn remaining while no war is declared.
8. **DF11 Keep land visible so camps do not spawn**: Idle Scouts and Rangers fan out to ring 6-7 around home to deny camp spawns.
9. **WX04 Triangle opening: three cities 3-4 tiles apart**: Japan founds cities at distance 4 in a triangle around the capital before reaching beyond 4 hexes.
10. **EG03 Buy the Settler instead of building it**: When Gold exceeds 2x the Settler cost and a site exists, buy the Settler rather than build.
11. **DC02 Attack the target before its allies**: Target selection removes the target's allies from the plan or isolates it first.
12. **RC13 Capture lone civilians (Settler sniping)**: Scouts shadow enemy Settlers inside 6 hexes; fast Soldiers capture when the escort is absent.

## Strategy catalog by archetype

Format: *Civ 6 civs* (famous with), then what to do, known counter, source, fit, factions, and the AI behaviour to test.

## Rush and conquest (37)

### RC01 Two-mount opening rush (resource-gated)
- **Tags:** rush_conquest. **Civ 6:** Scythia, Mongolia, Sumeria.
- **What to do:** Settle turn 1 beside the rush resource, build Scout then minimal economy, and start pumping the fast unit by turn 22-25 (22-23 is optimal). Attack with 2-3 at once: the guide says 2 horsemen kill a 10-strength city in a turn. Legatum port: no mount resource exists, so use Soldiers (40 Production, 2 move, Mongols 3). A pop-1 city has 200 HP and strength 12; a Soldier hits for about 41, a Ranger for about 34 with no retaliation, so field 6 attackers, not 3 (a walled city needs 8-9).
- **Known counter:** Walls (60 Gold, Fortification 30) plus a fortified garrison; a Scout parked on the rusher; emergency purchase of a defender.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: adapted.** No horse resource and no Great General. Needs ~6 attackers to crack a 200 HP city; city heals +20 a turn only if untouched.
- **Factions:** Mongols, Sparta. Works against: Switzerland, Athens, Byzantium, Japan.
- **AI behaviour to test:** Trigger T22-30 when a neighbour within 12 hexes has at most 2 cities and no Walls; abort the moment Walls or a 2nd defender appear.

### RC02 Day-one fast striker (War-Cart)
- **Tags:** rush_conquest. **Civ 6:** Sumeria.
- **What to do:** Train two of the no-tech fast unit immediately, explore and clear camps with them, add a Settler early so a second city doubles output, and at 4+ units ram the nearest neighbour before Pikes and Walls appear. Keep them cheap and spammable; upgrade only when counters show. Legatum port: Soldier is buildable on turn 1 for every faction, so the 'war cart' is Mongols's +1-move Soldier or Sparta's veteran Soldier.
- **Known counter:** Drop everything and build defence: Archers on rough terrain, Walls, and keep a Scout watching so the stack is seen coming.
- **Source:** [Zigzagzigal's Guides: Sumeria (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1905022232)
- **Fit: adapted.** No unique unit. The effect is matched only by Mongols (+1 move) and Sparta (level-1 veterans).
- **Factions:** Mongols, Sparta. Works against: Japan, Switzerland.
- **AI behaviour to test:** Opening brain 'fast Soldier pair at T12-18, second city, 4-unit strike by T30' scored by distance to nearest neighbour.

### RC03 Capital snipe
- **Tags:** rush_conquest. **Civ 6:** Sumeria, Mongolia, Macedon, Cyrus (Persia).
- **What to do:** Skip the border cities and walk the whole stack at the rival's capital (the highest-pop, usually best-garrisoned city). Capturing it removes the rival's +2 Production/+2 Science base and, in Legatum, pays the most: Sparta take population-share of the treasury plus stored production, and Switzerland pay a doubled bounty for their capital. Scout the road first and strike within 2 turns of arrival.
- **Known counter:** Garrison and Walls in the capital, a standing ranged pair, and a Scout picket so the approach is seen two turns out.
- **Source:** [Zigzagzigal's Guides: Sumeria (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1905022232)
- **Fit: possible.** Capital bounty exists for Switzerland (x2) and Sparta capture loot; check Celts razing before sniping a Celts capital.
- **Factions:** Mongols, Sparta, Germany. Works against: Switzerland, Athens.
- **AI behaviour to test:** Campaign target = capital when force ratio >= 2 and path <= 12 hexes; otherwise the weakest border city.

### RC04 Quick, unexpected, brutal first strike (20-turn horizon)
- **Tags:** rush_conquest. **Civ 6:** Sumeria, Mongolia, Zulu.
- **What to do:** Before declaring, ask how many turns the war will take. If the answer is more than about 20 turns or there is no clear edge, pick another target or nobody. Open with everything at once so the defender gives up or loses its field army in the first 2-3 turns; a long siege under fortified units invites a third party to hit you.
- **Known counter:** Trade space for time: Walls, hills, fortified ranged, and a counter-offensive on the attacker's exposed flank.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Turns resolve together, so a surprise strike is native; rescale 20 turns to our game length (verify in sims).
- **Factions:** Mongols, Sparta.
- **AI behaviour to test:** War gate: estimate turns-to-first-capture from forces and distance; refuse wars above a tunable horizon.

### RC05 Stage outside border, declare at the last second
- **Tags:** rush_conquest. **Civ 6:** Cyrus (Persia), Sumeria, Mongolia.
- **What to do:** Gather the stack two tiles outside the rival's border (they see only one tile past it without units), then declare and attack in the final seconds of the turn so the defender has no time to react. Legatum port: a surprise war at plain Peace takes effect at once, city sight is 3 (Hills 4) and unit sight 2, so stage 5+ hexes from the nearest city and walk in the same turn you declare.
- **Known counter:** A Scout picket on the approach, Walls pre-built in border cities, and cities garrisoned before the diplomatic screen shows anything.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Needs the +1 move of Mongols or a short path; check sight deltas so staging does not show on the rival's sighting log.
- **Factions:** Mongols. Works against: Japan, Athens.
- **AI behaviour to test:** Staging step: park outside sight radius, then DeclareWar and MoveUnit in the same order batch.

### RC06 Surround with three and strike together
- **Tags:** rush_conquest. **Civ 6:** Macedon, Mongolia.
- **What to do:** If a city heals, do not trickle attacks. Surround it with about 3 units and attack at once so damage lands in one turn. Legatum port: all damage aimed at a city tile in one stage is summed and overflow beyond the city's HP kills the garrison, so a 3-melee combined assault can break the city, kill the garrison and capture in a single movement round.
- **Known counter:** Keep a healthy garrison behind Walls; city healing stops while attackers are adjacent, so break the stack before it closes.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Native to the overflow rule; ranged first, then melee in the same turn.
- **Factions:** Sparta, Mongols, Germany.
- **AI behaviour to test:** City assault planner requires N>=3 melee-capable units in range before issuing any attack.

### RC07 Escorted siege; buy siege in the captured city
- **Tags:** rush_conquest. **Civ 6:** Macedon, Assyria.
- **What to do:** Fighting walls is hard and humans focus-fire siege, so bring Ram or Siege Tower (never alone, always on a tile with a healthy military unit) and consider buying siege units in a freshly captured city rather than walking them from home. Legatum port: Siege Engine needs Siege Engineering (late), costs 120, moves 1, +12 vs cities, and may stack with a Soldier because the stack limit is per class.
- **Known counter:** Snipe the escorted siege with ranged units first; keep a ranged pair behind Walls.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: adapted.** Siege arrives late; one purchase per city per turn; check whether a just-captured city may buy that turn.
- **Factions:** Germany, Sparta.
- **AI behaviour to test:** After Siege Engineering, assault stacks include 1 Siege per 2 melee; buy one in the captured city when Gold allows.

### RC08 Pre-build cheap units, then mass-upgrade with Gold
- **Tags:** rush_conquest, economic_gold. **Civ 6:** Sumeria, Mali.
- **What to do:** Delay the unit-tech research until you hold a pile of the old unit, then upgrade them in one turn instead of waiting 5-7 turns for new builds ('before I finish Tank research I first make a ton of spare Knights'). Legatum port: stockpile Soldiers, research Ballistics (60) for Trooper (32 strength), then upgrade with the UpgradeUnit order before the strike.
- **Known counter:** Watch rival tech and army count; strike during the pre-upgrade window or add Walls.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: adapted.** UpgradeUnit exists; its Gold price is not in the rules reference (verify). Units never go obsolete in production, so banked Soldiers stay valid.
- **Factions:** Sparta, Switzerland, Germany.
- **AI behaviour to test:** Stockpile 6+ Soldiers while Ballistics is 1-2 techs away; spend Gold on upgrades the turn it lands.

### RC09 Never fight a higher unit tier
- **Tags:** rush_conquest, defence_vs_rush. **Civ 6:** Korea, Mongolia.
- **What to do:** A small difference in unit strength makes a large difference on the field, and promotions widen it. Gate offence on your best unit tier being at least the rival's; a tier below only feeds experience. Legatum port: Soldier 20 vs Trooper 32 is a 12-point gap, which multiplies damage by about 1.6 each way.
- **Known counter:** Beeline the next unit tier and hold defensively until you have it.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Native to the e^(delta/25) damage formula.
- **Factions:** Athens, Germany.
- **AI behaviour to test:** Force-ratio gate multiplies by a tier factor; refuse attacks when enemy tier > own tier.

### RC10 Kill-heal chain
- **Tags:** rush_conquest. **Civ 6:** Tomyris (Scythia), Sparta (Greece).
- **What to do:** Fight so every kill restores HP, then roll straight into the next fight: units that win keep going and units that lose fall away. Pick targets you can finish, focus fire on one unit at a time, and finish with the unit that benefits from the heal. Legatum: Sparta kills restore up to 100 HP shared among survivors, and captures heal units within 2 hexes.
- **Known counter:** Do not feed it wounded units; retreat the damaged, use Walls and fresh ranged units, and avoid fights where the kill is certain for them.
- **Source:** [Domination Victory Tips (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1229036179)
- **Fit: possible.** Native to Sparta (blood is the only medicine). Mutual kills heal nobody.
- **Factions:** Sparta. Works against: Sparta.
- **AI behaviour to test:** Sparta target scoring adds heal value of a likely kill; never attack where a mutual kill is the best case.

### RC11 Perpetual war chain
- **Tags:** rush_conquest. **Civ 6:** Macedon (Alexander), Sparta (Greece).
- **What to do:** Never let the war end: finish one target and immediately pick the next while the army is healthy, because nothing in war costs you amenities or morale. Rotate wounded units into captured cities to heal and keep the main stack moving. Legatum port: Sparta desert after turn 50 (Quick) or 100 (Standard) with no war, so the AI must always hold a live target.
- **Known counter:** End the war early with a city gain or sue for peace; fight inside your own borders so the chain breaks.
- **Source:** [Zigzagzigal's Guides: Macedon (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1902563180)
- **Fit: adapted.** Legatum has no war weariness; the cost is desertion (Sparta) and Gold upkeep beyond 2 units.
- **Factions:** Sparta. Works against: Sparta.
- **AI behaviour to test:** Sparta AI queues the next campaign target before the current one falls.

### RC12 Surprise war with a free move
- **Tags:** rush_conquest. **Civ 6:** Cyrus (Persia).
- **What to do:** Declare with the stack already adjacent so the bonus movement lets slow units strike the same turn (Cyrus gets +2 movement on a surprise war so artillery can move and shoot). Legatum port: surprise war at plain Peace is immediate, and Mongols's +1 move means the stack reaches a capital the turn it declares.
- **Known counter:** Never leave a frontier city ungarrisoned at Peace; buy Walls the turn a stack appears inside your sight.
- **Source:** [Domination Victory Tips (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1229036179)
- **Fit: adapted.** Native timing, but there is no explicit surprise-war move bonus; the faction rule that matches is Mongols +1 move.
- **Factions:** Mongols. Works against: Japan.
- **AI behaviour to test:** Mongols plans war so the stack ends the declaration turn adjacent to the target.

### RC13 Capture lone civilians (Settler sniping)
- **Tags:** rush_conquest, wide_expansion. **Civ 6:** Aztec, Mongolia, Russia (counter).
- **What to do:** Use fast units to capture or threaten enemy Settlers: it forces a defender build, delays their expansion, and gives you a free Settler. Aztec Eagle Warriors capture Builders; the guide's early scout threatens Settlers in transit. Legatum port: civilians are captured, Scouts are free of upkeep, and Mongols's slip-past ignores enemy blockers.
- **Known counter:** Escort every Settler with a Soldier or Scout, move it through your own territory, and never send one alone.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Raiders capture lone civilians in Legatum; verify that a human Scout can capture a Settler.
- **Factions:** Mongols, Byzantium.
- **AI behaviour to test:** Scouts shadow enemy Settlers inside 6 hexes; fast Soldiers capture when the escort is absent.

### RC14 Convert every capture into more force
- **Tags:** rush_conquest. **Civ 6:** Zulu, Aztec.
- **What to do:** Treat each captured city as a base: leave the capturing unit to heal, garrison it, and use it as the next staging point so loyalty or momentum does not stall. The Zulu advantage is that captures turn into promoted corps and armies; the lesson is to snowball per capture. Legatum port: Sparta captures heal units within 2 hexes and take Gold and stored production.
- **Known counter:** Do not let them take cities at all; one lost city feeds their next army.
- **Source:** [Zigzagzigal's Guides: Zulu (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1983949096)
- **Fit: adapted.** Corps and armies do not exist. The matching Legatum rule is Sparta capture-heal and loot.
- **Factions:** Sparta. Works against: Sparta.
- **AI behaviour to test:** After a capture, hold 2 turns in place if units are below 70% HP and the city is not threatened.

### RC15 As many units as possible; second and third armies
- **Tags:** rush_conquest. **Civ 6:** Zulu, Macedon, Mongolia.
- **What to do:** The only correct number of units is as many as possible: your first army goes out, then second and third armies run simultaneous assaults. Fewer losses and more units wins; 'you want to be the carpet'. Legatum: the first two military units are free, extras cost 1 Gold a turn (3 for Switzerland), so budget a tail of Gold before building the third army.
- **Known counter:** Kill the carpet piecemeal at a choke with ranged units behind Walls; deny them cheap captures.
- **Source:** [Domination on Immortal/Deity: a noob's guide (CivFanatics, via web.archive.org)](https://forums.civfanatics.com/threads/domination-on-immortal-deity-a-noobs-guide.547630/)
- **Fit: possible.** Upkeep exists (1 Gold beyond 2 units), so cap the army by income.
- **Factions:** Sparta, Mongols. Works against: Switzerland.
- **AI behaviour to test:** Production share to units rises with surplus Gold; second army starts when the first has left home.

### RC16 First army composition and the five-turn rule
- **Tags:** rush_conquest. **Civ 6:** Macedon, Persia.
- **What to do:** Build 6-8 ranged and 3-5 melee units for the first army, big enough to kill every unit around a city and capture it in about 5 turns. Faster means the army is over-sized; slower means the target is wrong. Legatum port: 4-5 Rangers (50) plus 4 Soldiers (40) against a walled city.
- **Known counter:** A second wall layer, a garrison with HP, and a flank raid.
- **Source:** [Domination on Immortal/Deity: a noob's guide (CivFanatics, via web.archive.org)](https://forums.civfanatics.com/threads/domination-on-immortal-deity-a-noobs-guide.547630/)
- **Fit: adapted.** Rangers have range 2 and no retaliation in Legatum, so the ranged:melee ratio can lean ranged.
- **Factions:** Sparta, Mongols.
- **AI behaviour to test:** Army builder sizes force to a 5-turn kill estimate per target city.

### RC17 Ranged whittles, melee finishes at 40-50%
- **Tags:** rush_conquest. **Civ 6:** Macedon, Persia.
- **What to do:** Fortify melee in front of the city and let ranged units whittle it down; do not send melee in until the city is at about 40-50% HP, because it takes a bucket of damage. Hold ranged units on hills so they fire over cover. Legatum: ranged has no retaliation, resolves before movement, and city HP regenerates only when it was not attacked that turn.
- **Known counter:** Kill or push back the ranged line first; Walls fire back.
- **Source:** [Basic Rush: take out an enemy early on (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=181479636)
- **Fit: possible.** Native: ranged phase, then melee. Walls make the city shoot, so ranged units must stand out of range 2 where possible.
- **Factions:** Sparta, Germany.
- **AI behaviour to test:** Assault stages: ranged shots when ranged <= 2 hexes, then melee only below ~50% city HP.

### RC18 Take the weak flat city first
- **Tags:** rush_conquest. **Civ 6:** Macedon, Persia.
- **What to do:** Take the weakly defended city (flat land, small garrison, no Walls) first and trade the hill city for a peace deal or leave it for later. Attacking a strongly defended city early can cost more units than the war is worth. Legatum: city strength is 10 + 2 x pop + Walls, so a pop-3 flat city is 16 and an unwalled capital 18.
- **Known counter:** Hill cities and Walls on the exposed side.
- **Source:** [Civilization V Military Strategy Guide (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=749815003)
- **Fit: possible.** Native.
- **Factions:** Sparta, Mongols.
- **AI behaviour to test:** Target score = value / (city strength + garrison); prefer the lowest cost per capture.

### RC19 Do not make peace after the win; counterattack
- **Tags:** rush_conquest. **Civ 6:** Persia, Macedon.
- **What to do:** After repelling an invader, the army is damaged and the enemy is weakest: take their nearest cities now, because making peace only invites a stronger return. Do not counter-attack until their units are expelled from your territory. Legatum: peace is free and gives 3 guaranteed turns, so the decision is purely tactical.
- **Known counter:** Break their army first, then take a city quickly; sue for peace only after a gain.
- **Source:** [Civilization V Military Strategy Guide (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=749815003)
- **Fit: possible.** Native; AI should check forecast, not pride.
- **Factions:** Sparta, Mongols.
- **AI behaviour to test:** After a defensive win, if force ratio >= 1.5 and a city is within 8 hexes, go on the offensive.

### RC20 Hit the title, wonder or tech leader
- **Tags:** rush_conquest, late_game_closing. **Civ 6:** Macedon, Rome.
- **What to do:** Choose the target with the most wonders or the tech leader: capturing their best cities pays more than building the same wonders and takes the lead away from a rival. Legatum port: titles are strict-highest 1 VP each (Largest Economy, Most Territory, Strongest Army, Most Advanced, Most Populous), so each capture can swing a title (+1 for you, -1 for them).
- **Known counter:** Stay under the top title thresholds, fortify the wonder city, and stay allied with their neighbour.
- **Source:** [Domination Victory Tips (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1229036179)
- **Fit: adapted.** Titles and Great Works are the Legatum equivalents; whether capturing a Great Work city transfers its 2 VP needs a rules check.
- **Factions:** Sparta, Mongols, Athens, Byzantium. Works against: Athens, Switzerland.
- **AI behaviour to test:** Target value includes the VP swing of titles and Great Works held by the owner.

### RC21 Win one early war against the weakest player, then repeat
- **Tags:** rush_conquest. **Civ 6:** Mongolia, Zulu, Macedon.
- **What to do:** The multiplayer meta is: win an early war against a weaker player, take its cities, snowball, research a mid-game unit, crush the next player, repeat. Choose the weakest reachable neighbour and finish before a third party can exploit the fight. Legatum: 8 empires and 2 humans per team makes a clean first kill decisive.
- **Known counter:** Be the strongest-looking neighbour; keep a deterrent army and Walls.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Native.
- **Factions:** Mongols, Sparta.
- **AI behaviour to test:** First-war target = weakest reachable neighbour by public army share and city count.

### RC22 Clear camps early for Gold and a first army
- **Tags:** rush_conquest, wide_expansion. **Civ 6:** Sumeria, Aztec.
- **What to do:** Pair the first fighter with the warrior or scout to clear a camp, farm the Gold and XP, and carry the army on to the first civ it finds. It also denies the camp's tile yields. Legatum: camps appear from turn 5 (scaled) with a guard, a scout and up to 4 raiders; clearing pays Gold; barbarians never assault cities.
- **Known counter:** Keep land visible so camps do not spawn near you and kill the barbarian scout before it sees a city.
- **Source:** [Reddit: an early game strategy (r/civ)](https://www.reddit.com/r/civ/comments/58tbok/)
- **Fit: possible.** Barbarian scouts triggering raids matches Legatum's camp logic (raid after a scout sees a city).
- **Factions:** Sparta, Mongols, Byzantium.
- **AI behaviour to test:** First two Soldiers clear the nearest camp when guard strength <= own; Scout screens the home city.

### RC23 Park a unit on the rival's best tile
- **Tags:** rush_conquest, economic_gold. **Civ 6:** Vikings (Norway), Mongolia.
- **What to do:** Park and fortify a unit on the best tile of an enemy city (especially outside its attack range), forcing the owner to spend a unit to chase you off while the city loses the yield. Legatum port: Byzantium squatters deny yields and earn Science from the tile; barbarian camps also deny tiles.
- **Known counter:** Build a defender to evict the squatter; keep vision on every ring-1 tile.
- **Source:** [Aggressive Strategies to Dominate your Opponent(s) in Multiplayer (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=517454225)
- **Fit: adapted.** In Legatum only the Byzantium (and barbarians) deny tile yields; other factions park units for position only.
- **Factions:** Byzantium. Works against: Byzantium.
- **AI behaviour to test:** Byzantium AI squats on the highest-yield ring-1 tile of the nearest rival city outside the city's range.

### RC24 Stay unmet: hide the army
- **Tags:** rush_conquest, defence_vs_rush. **Civ 6:** Mongolia, Cyrus (Persia).
- **What to do:** Scout the enemy city's position but avoid actually meeting them: move slowly, see the border edge, confirm it is not a city-state, and withdraw. Mass your military without their demographics showing a surge and without ever triggering contact. Legatum: contact is mutual and permanent (a unit, city or owned tile seen), and an Alliance needs contact but War and Peace do not.
- **Known counter:** Keep a Scout picket on every approach so the contact you never gave is still taken.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: adapted.** Needs a check that sight triggers contact one-way; the army can be hidden by staging beyond sight.
- **Factions:** Mongols, Sparta, Byzantium.
- **AI behaviour to test:** Hide-scout: observe the city from 3 hexes, never step onto an owned tile, then withdraw.

### RC26 Never be weak and aggressive
- **Tags:** rush_conquest, defence_vs_rush. **Civ 6:** Mongolia, Huns.
- **What to do:** An aggressive posture without force is punished: in 274 AI-only Civ 5 games the 'weak and aggressive' Mongolia ranked last. Gate declarations on measured force, not personality; if you are behind, build and expand until the ratio flips. Legatum: coalitions and a 10-turn VP hold mean a failed war costs the war and the lead.
- **Known counter:** Let them over-extend then counter-attack the exposed cities.
- **Source:** [Reddit: 274 AI-only Civ 5 games tier list (r/civ)](https://www.reddit.com/r/civ/comments/1hjdenn/)
- **Fit: possible.** Direct AI-gating lesson.
- **Factions:** Mongols, Sparta.
- **AI behaviour to test:** Aggression gene gated by force ratio and tier; below the gate, build economy instead.

### RC27 Pair ranged shots: wound first, finish second
- **Tags:** rush_conquest. **Civ 6:** Maya.
- **What to do:** Keep ranged units in pairs so the first shot wounds and the second hits a weakened target (injured units lose strength, so the second hit lands harder). Focus both on the same target. Legatum: wound penalty is missing HP / 10 strength points, ranged takes no retaliation, and ranged fire resolves before movement.
- **Known counter:** Fortify a healthy front unit; keep the back line out of range 2.
- **Source:** [Zigzagzigal's Guides: Maya (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2104285340)
- **Fit: possible.** Native (wound penalty applies to units; /20 for cities).
- **Factions:** Germany, Sparta, Athens.
- **AI behaviour to test:** Ranged units are assigned in pairs per target; second shot only after the first reduces HP.

### RC28 Ranged first, melee first only if ranged is much stronger
- **Tags:** rush_conquest. **Civ 6:** any.
- **What to do:** Attack with ranged first so the target loses strength and your melee takes less damage. Exception: when the ranged unit is far stronger than the melee, the damage formula is exponential in strength difference, so a first melee hit can raise the ranged hit; test the numbers. Legatum copies this formula (30 x e^(delta/25) with 80-120% rolls).
- **Known counter:** Keep strong ranged inside walls, or fortify so melee penalties are minimal.
- **Source:** [Reddit: melee versus ranged first, with the math (r/CivVI)](https://www.reddit.com/r/civ/comments/1tsjxhj/)
- **Fit: possible.** The same damage formula and wound penalty are in Legatum; encode as a forecast comparison.
- **Factions:** Germany, Sparta, Mongols.
- **AI behaviour to test:** Forecast both attack orders and pick the one with the larger expected total.

### RC29 Three siege per city plus bait units
- **Tags:** rush_conquest. **Civ 6:** Macedon, Persia.
- **What to do:** Mid-game war works in a narrow window: bring about three siege units per city and extra ranged units to draw the city's attacks, which also levels the siege crew. Strike before the AI reaches the next wall tier. Legatum: Siege Engine (22 vs +12 vs cities) after Siege Engineering; Walls fire at one target a turn, so extra bodies spread the fire.
- **Known counter:** Citadel (Mutual Defense, +5) and a garrison with fresh HP.
- **Source:** [Reddit: war in the mid game (r/civ)](https://www.reddit.com/r/civ/comments/1gvmu1q/)
- **Fit: adapted.** Siege arrives late in Legatum; check how many shots a walled city fires per turn.
- **Factions:** Germany.
- **AI behaviour to test:** Assault stack = 3 Siege + 2 decoy ranged per walled target after Siege Engineering.

### RC30 Cap attackers at six per tile
- **Tags:** rush_conquest. **Civ 6:** any.
- **What to do:** A hex has 6 neighbours, so more than 6 units of one type add nothing; usually a lot fewer than 6 fit the terrain. Plan the number you can actually bring, with a rotating reserve to swap wounded units. Legatum: one unit per class per tile and combined assaults mean the practical cap is the open hexes around the city.
- **Known counter:** Hold the choke so only 2 can engage.
- **Source:** [Zany's Comprehensive Civ 6 Guide (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=926983840)
- **Fit: possible.** Native hex geometry.
- **Factions:** Germany, Sparta.
- **AI behaviour to test:** Assault planner caps attackers at open adjacent hexes; extras queue as reserves.

### RC31 Alpha strike: whoever hits hardest in the first turns wins
- **Tags:** rush_conquest. **Civ 6:** Macedon, Mongolia.
- **What to do:** Fights are decided in the first 1-2 turns of contact. Bring everything, fire first, and only then think about rotation. Defender's counter is to deny first contact by holding terrain. Legatum: simultaneous turns and ranged-before-movement resolution reward the side that commits the larger alpha.
- **Known counter:** Deny first contact: hold the choke, keep ranged behind and let them walk into it.
- **Source:** [Zany's Comprehensive Civ 6 Guide (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=926983840)
- **Fit: possible.** Native.
- **Factions:** Mongols, Sparta.
- **AI behaviour to test:** Engage only with the full stack in range on the same turn.

### RC32 Raid-farm economy (treat rivals as farms)
- **Tags:** rush_conquest, economic_gold, naval. **Civ 6:** Norway (Harald Hardrada), Mongolia.
- **What to do:** Treat every rival as a field: raid its improvements again and again, take peace only to bank income, and never build the economy a peaceful player would. One player held the science lead with a single Campus by raiding coasts. Legatum has no pillage, but the Byzantium converts tile squatting into Science (twice tile yields, plus 5 per tech the landowner has and Byzantium lacks).
- **Known counter:** Evict squatters, keep sight on every rival tile, and learn the techs Byzantium wants to take.
- **Source:** [Reddit: Harald as a raid economy (r/civ)](https://www.reddit.com/r/civ/comments/guu28v/)
- **Fit: adapted.** Pillage does not exist; the faction-level analogue is Byzantium squatting and Salvage Works.
- **Factions:** Byzantium. Works against: Byzantium.
- **AI behaviour to test:** Byzantium AI cycles squatters across rich rival tiles that belong to techs it lacks.

### RC33 Leave a buffer pet state
- **Tags:** rush_conquest, diplomatic_coalition. **Civ 6:** Rome, Macedon.
- **What to do:** Take the capital and best cities of a rival and leave a small byzantium alive: it dilutes everybody else's attention, absorbs rivals' wars, and keeps you from becoming the sole target. Legatum: one team alive wins outright and all AIs treat holders as top threats, so a living buffer keeps others looking elsewhere.
- **Known counter:** Finish the buffer once your own flank is secure.
- **Source:** [Reddit: keeping enemies as pets (r/civ)](https://www.reddit.com/r/civ/comments/14s4tah/)
- **Fit: possible.** Celts razes on capture, so they cannot keep pets.
- **Factions:** Sparta, Mongols, Byzantium.
- **AI behaviour to test:** After taking the capital, stop at 1-2 cities left unless force ratio is above 3.

### RC34 Wipe a small neighbour before it builds Walls
- **Tags:** rush_conquest, wide_expansion. **Civ 6:** Sumeria, Aztec, Scythia.
- **What to do:** If a neighbour has only 2-3 cities, wipe it with ranged-heavy armies before Walls go up; the ranged units gain upgrades for later and the whole continent is yours. The cost is a few turns of expansion. Legatum: one main continent (about 45% land) holds at least 85% of land, so early neighbour wipes are the main route to space.
- **Known counter:** Walls in all cities before turn 25 and a Scout watching the border.
- **Source:** [Reddit: wiping a small neighbour (r/CivVI)](https://www.reddit.com/r/civ/comments/1g18yfm/)
- **Fit: possible.** Native; compare with settling the same land.
- **Factions:** Mongols, Sparta.
- **AI behaviour to test:** If a neighbour has <= 3 cities and no Walls at T30, queue a ranged-heavy strike.

### RC35 Do not over-build before your timing spike
- **Tags:** rush_conquest. **Civ 6:** Gaul, Sumeria.
- **What to do:** Following a good opening and then adding extras (a religion, a wonder) delays the unit spike, and by the time your army shows up the AI has the same tier. Decide the spike turn and skip everything that does not serve it. Legatum: the rival tier unlocks (Ballistics, Combined Arms) are public in the tech race.
- **Known counter:** Time a Walls/Citadel and tier-up to land just before their spike.
- **Source:** [Reddit: the mistake beginners make (r/CivVI)](https://www.reddit.com/r/civ/comments/1pi75k8/)
- **Fit: possible.** Native; needs a build-order score keyed to the spike turn.
- **Factions:** Mongols, Sparta, Germany.
- **AI behaviour to test:** Opening score penalises non-spike items after the declared spike turn.

### RC36 Decoy a civilian to pull units out of position
- **Tags:** rush_conquest. **Civ 6:** any.
- **What to do:** During war, goad enemy units onto irrelevant tiles with a cheap civilian bait: the AI chases it rather than attacking your ranged line. Legatum: raiders and AI units capture lone civilians, so a Settler or civilian on a poor tile can pull a stack.
- **Known counter:** Ignore decoys; keep the line.
- **Source:** [Reddit: Deity AI tricks (r/CivVI)](https://www.reddit.com/r/civ/comments/1ozb0ak/)
- **Fit: adapted.** Needs a sim check that the Legatum AI takes bait; no Great People exist.
- **Factions:** Mongols, Byzantium.
- **AI behaviour to test:** A lone civilian within 3 hexes of an enemy stack pulls it off its target.

### RC38 Rush the rich but slow-to-build empire
- **Tags:** rush_conquest. **Civ 6:** Mali, Netherlands.
- **What to do:** Empires built on Gold and instant purchases are weakest early: they lack production, build walls slowly, and rely on a few fragile trade or income engines. Hit them before the money engine is up. Legatum port: Switzerland and anyone who hoards Gold are the targets; a captured Swiss city pays the captor a population-weighted treasury share, doubled for the capital.
- **Known counter:** Spend the hoard on Walls and units in time, and keep the interest low only if defended.
- **Source:** [Zigzagzigal's Guides: Mali (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1918595516)
- **Fit: possible.** Native; Swiss bounty makes it self-scaling.
- **Factions:** Mongols, Sparta. Works against: Switzerland.
- **AI behaviour to test:** Target score adds expected bounty and reduces defence of Gold-rich empires.

### RC39 Slow siege creep under ranged cover
- **Tags:** rush_conquest. **Civ 6:** Germany, Babylon.
- **What to do:** When your movement is slow, do not rush: advance in a compact stack with ranged units in front of the siege line, fortify each stop, and let ranged fire do the work. Ranged units become very powerful in a trench war. Legatum: Germany has -1 movement, so a Germany assault is a slow siege stack with Siege Engines (+12 vs cities).
- **Known counter:** Early raids before the stack is built; hit the exposed siege on the march.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: adapted.** The Germany movement penalty makes this the natural attack shape; check approach lanes for LOS.
- **Factions:** Germany.
- **AI behaviour to test:** Germany campaigns march at stack speed, ranged first, siege behind, fortify at each stop.

## Defence against a rush (18)

### DF01 Walls early, fortified garrison, ranged behind
- **Tags:** defence_vs_rush, tall_defensive. **Civ 6:** Babylon, Greece, Germany.
- **What to do:** Build Walls as soon as a neighbour is within reach, keep a fortified unit in every border city, and put ranged units behind it so 2 or more cover the likely attack hex. Legatum: Walls cost 60 (Fortification, 30 research), add +10 city strength and make the city fire; only walled cities shoot. Citadel (+5) later upgrades them. Never sally unless the kill is certain.
- **Known counter:** Siege and a mass of attackers; attack the unwalled cities first.
- **Source:** [Zany's Comprehensive Civ 6 Guide (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=926983840)
- **Fit: possible.** Native; the rush defence of record in Legatum.
- **Factions:** Celts, Switzerland, Japan, Athens. Works against: Mongols, Sparta.
- **AI behaviour to test:** Border cities get Walls when a rival army share exceeds ours within 12 hexes; Walls before the second Settler if a neighbour is within 8.

### DF02 Deter with walls and a visible army
- **Tags:** defence_vs_rush. **Civ 6:** Gorgo (Greece), Dido.
- **What to do:** The best defence is preventing the war: keep a decent visible military and build Walls and visible military camps, because attackers pick the weak. Keep your army above the public Strongest Army line of your nearest rival, and keep Scouts alive. Legatum: army share feeds the public Strongest Army title and the AIs' assessment.
- **Known counter:** Out-tech or out-numerate; strike a city that the deterrent army cannot reach.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Public titles make deterrence legible.
- **Factions:** Switzerland, Athens, Celts. Works against: Mongols, Sparta.
- **AI behaviour to test:** Maintain army score >= 50% of the strongest neighbour; keep one unit per border city visible.

### DF03 Keep military at one third to one half of the worrisome neighbour
- **Tags:** defence_vs_rush. **Civ 6:** Shaka (Zulu), Gorgo (Greece).
- **What to do:** As a rule of thumb keep your military strength to at least 1/3 or 1/2 of any worrisome neighbour, with at least one up-to-date unit so the city defence number stays adequate. A Deity player ended up buying almost 1,000 Gold of defenders after a surprise war. Legatum: Soldier 40 / Trooper 70; city strength is a function of Walls and population, not garrison.
- **Known counter:** A surprise declaration before the ratio is repaired.
- **Source:** [How many military units should I build? (CivFanatics, via web.archive.org)](https://forums.civfanatics.com/threads/how-many-military-units-should-i-build.688012/)
- **Fit: possible.** Native; fraction should be a tunable gene.
- **Factions:** Switzerland, Athens, Japan. Works against: Mongols, Sparta.
- **AI behaviour to test:** Defence gene: army ratio floor of a neighbour, rising with its aggression history.

### DF04 Check demographics every turn
- **Tags:** defence_vs_rush, late_game_closing. **Civ 6:** any.
- **What to do:** Read the rivals' public numbers each turn: army size relative to yours, tech level, and production. A low tech count tells you what they cannot do yet; a spike in army tells you what they are about to do. Legatum: titles (Strongest Army, Most Advanced, Largest Economy) and public standings give this for free.
- **Known counter:** Hide the buildup: stage beyond sight, buy rather than build.
- **Source:** [Guide to civ5 multiplayer (CivFanatics, via web.archive.org)](https://forums.civfanatics.com/threads/guide-to-civ5-multiplayer.441979/)
- **Fit: possible.** Public standings and sighting deltas exist.
- **Factions:** Athens, Celts, Switzerland.
- **AI behaviour to test:** Threat tracker updates from title deltas each turn; a doubled army share inside 5 turns raises alert.

### DF05 Scouts as pickets: one per bordering rival
- **Tags:** defence_vs_rush. **Civ 6:** any.
- **What to do:** Have two scouts by turn 30 and then one for each bordering rival; park one next to every dangerous neighbour so stacks are seen two turns out, and when behind scouts are your only defence. Legatum: Scout costs 30, 3 moves, free upkeep, and costs nothing from the 2-unit allowance.
- **Known counter:** Kill the picket (cheaply); approach from fog.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** The user notes parking a Scout is the effective Mongols rush counter.
- **Factions:** Celts, Switzerland, Athens, Japan. Works against: Mongols.
- **AI behaviour to test:** Standing picket per neighbour within 12 hexes; retreat logic keeps it alive.

### DF06 Rush tell: low score on the first turns of the surge
- **Tags:** defence_vs_rush. **Civ 6:** Scythia, Sumeria.
- **What to do:** Military score lags one turn behind real numbers, so a rival that goes from 50 to 120 in 2-3 turns can look harmless the turn before; with two mount resources and one city, assume a rush. Build Walls and upgrade one unit when a close player has a single city and a big production start. Legatum: public army share also trails real production.
- **Known counter:** Pre-build Walls on the exposed side.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: adapted.** Check whether army share lags in Legatum; use sight deltas instead.
- **Factions:** Celts, Switzerland. Works against: Mongols.
- **AI behaviour to test:** Treat a one-city neighbour within 10 hexes as a rusher until it expands.

### DF07 Keep Gold or a small army: emergency purchase
- **Tags:** defence_vs_rush, economic_gold. **Civ 6:** Mali, Netherlands.
- **What to do:** Always keep enough Gold or a small army in hand: money solves a surprise war because you can buy defenders and Walls the turn a stack appears. Legatum: BuyItem is 200% of Production in Gold (175% Switzerland), one purchase per city per turn, so spread the reserve over several threatened cities.
- **Known counter:** Strike all cities at once so one purchase per city cannot cover them.
- **Source:** [Beginners guides: how to counter a surprise war (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1143191647)
- **Fit: possible.** Native; the purchase_defense logic exists.
- **Factions:** Switzerland, Athens. Works against: Mongols.
- **AI behaviour to test:** Reserve floor = cost of one defender per border city; spend when an enemy stack enters sight.

### DF08 Bank production at one turn left
- **Tags:** defence_vs_rush. **Civ 6:** Byzantium (Basil II).
- **What to do:** Leave defenders and Walls one turn from completion in several cities; you pay no upkeep until they finish, and if attacked you have a unit next turn. Legatum: banked production never decays, a unit never goes stale, and upkeep applies only to finished units (2 free, 1 Gold each after).
- **Known counter:** Disrupt production by capture (capture clears the bank).
- **Source:** [Reddit: keep units in the queue at one turn left (r/civ)](https://www.reddit.com/r/civ/comments/vkgbvq/)
- **Fit: possible.** Native: banks are kept indefinitely and cleared on capture.
- **Factions:** Switzerland, Celts.
- **AI behaviour to test:** Border cities keep a Soldier at <= 1 turn remaining while no war is declared.

### DF09 Settler wall: block the path
- **Tags:** defence_vs_rush, wide_expansion. **Civ 6:** Russia (counter), Mali (counter).
- **What to do:** When an enemy Settler is heading for your space, place units on the hills, forest or choke it must cross as a movable wall; it eventually gives up. Also use it to stop boxed-in expansion. Legatum: one unit per class per tile, no zone of control but pathing routes around known hostiles; Mongols slips past, so the wall fails against Mongols.
- **Known counter:** A civilian-slipping faction (Mongols), or another route.
- **Source:** [Reddit: small tips that helped immeasurably (r/civ)](https://www.reddit.com/r/civ/comments/7rix87/)
- **Fit: adapted.** Works against all but Mongols; check that a blocked step fails rather than fights at Peace.
- **Factions:** Japan, Celts. Works against: Mongols.
- **AI behaviour to test:** Wall idle units on the one-hex gap between a rival Settler and a good site.

### DF10 Settle behind a mountain or hilly choke
- **Tags:** defence_vs_rush, tall_defensive. **Civ 6:** Inca, Georgia.
- **What to do:** Place cities so the line of defence runs through terrain: a city behind a mountain or at a hilly choke point slows enemy rushes, and tactical position beats the strongest-looking site when an opponent threatens your existence. Legatum: ridge mountains are about 6% of land and Mountains block sight for everybody.
- **Known counter:** Walk around, or bring ranged units that see over ridges.
- **Source:** [Aggressive Strategies to Dominate your Opponent(s) in Multiplayer (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=517454225)
- **Fit: possible.** Native terrain; no named chokepoint rule.
- **Factions:** Celts, Japan, Switzerland. Works against: Mongols.
- **AI behaviour to test:** Site scoring adds a defensive-choke term when a rival is within 15 hexes.

### DF11 Keep land visible so camps do not spawn
- **Tags:** defence_vs_rush. **Civ 6:** Sumeria (counter).
- **What to do:** Barbarians spawn only in fog, so keep scouts or fortified units evenly spaced outside your borders to stop camps appearing, which also lowers rival aggression and expansion. Legatum: camps spawn from turn 5 (scaled) away from cities, units and camps; verify that seen tiles never spawn them.
- **Known counter:** Fog gaps near the edge.
- **Source:** [Reddit: non-obvious tips (r/civ)](https://www.reddit.com/r/civ/comments/11tllfm/)
- **Fit: adapted.** Check the spawn rule in barbarians.rs; raiders never capture cities, so the stake is yields.
- **Factions:** Celts, Switzerland, Athens.
- **AI behaviour to test:** Idle Scouts and Rangers fan out to ring 6-7 around home to deny camp spawns.

### DF12 Shoot siege and melee first; wait for the unit to move
- **Tags:** defence_vs_rush. **Civ 6:** Poland, Rome.
- **What to do:** When invaded, kill the units that threaten the city: siege first, then melee, and finish weak ones. If a unit is well defended, hold off until it moves, as it cannot stay forever. Do not attack out until they are expelled. Legatum: ranged resolves before movement, so a shot at a unit about to step into the city counts before it arrives.
- **Known counter:** Stack the front with well-promoted defenders; fortify.
- **Source:** [Civilization V Military Strategy Guide (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=749815003)
- **Fit: possible.** Siege arrives late in Legatum; melee first until then.
- **Factions:** Celts, Switzerland, Japan, Germany, Sparta. Works against: Mongols, Sparta.
- **AI behaviour to test:** Defender target priority: siege > melee about to step adjacent > wounded ranged.

### DF13 Fight on your own land
- **Tags:** defence_vs_rush. **Civ 6:** Vietnam, Mapuche, Macedon (counter).
- **What to do:** Fighting in your own territory rather than theirs is cheaper in war weariness and keeps reinforcements one step away. Lure the enemy in, then strike. Legatum: Celts land drains hostile units up to 12 HP a turn (floor 10), the strongest version of this idea.
- **Known counter:** Do not enter; out-expand and let them come out.
- **Source:** [Zigzagzigal's Guides: Macedon (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1902563180)
- **Fit: adapted.** Legatum has no war weariness; the own-land edge is Celts attrition and short supply lines.
- **Factions:** Celts.
- **AI behaviour to test:** Celts AI waits inside its borders and accepts engagements only on its land.

### DF14 Trench war: close every back door
- **Tags:** defence_vs_rush. **Civ 6:** any.
- **What to do:** In a stalemate, ready the defence like in a rush: walls, units, hold hills, and do not leave improvements or civilians exposed. Make sure you cannot be attacked from behind; a single unit behind your lines escaping is a devastating blow to morale and resources. Legatum: raiders capture lone civilians, and barbarian units deny tiles.
- **Known counter:** Slip a fast unit past the line (Mongols) or park on a tile.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: adapted.** No pillage, but behind-the-line units capture civilians and squat tiles.
- **Factions:** Celts, Japan, Switzerland, Germany. Works against: Mongols, Byzantium.
- **AI behaviour to test:** Rear-area patrol: one unit holds each backline tile adjacent to civilians.

### DF17 Befriend the likely attacker first
- **Tags:** defence_vs_rush, diplomatic_coalition. **Civ 6:** Gilgamesh (Sumeria), Gorgo.
- **What to do:** If a player is too close and you are not ready to fight them, try to get a declared friendship early and message them so the deal is noticed. In Legatum that is an Alliance (30 Gold at Quick, 60 at Standard, plus 1 Gold per city per turn); breaking it in the first 10 turns costs 60 and is a visible betrayal.
- **Known counter:** Betrayal after the 10-turn protected window is free, so watch the partner's army.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: adapted.** Alliance replaces friendship; needs first contact.
- **Factions:** Switzerland, Athens, Celts, Japan. Works against: Mongols.
- **AI behaviour to test:** Ally the strongest non-target neighbour once met, when upkeep is affordable.

### DF18 Never forward-settle an AI
- **Tags:** defence_vs_rush, wide_expansion. **Civ 6:** any.
- **What to do:** Moving a Settler toward a nearby AI sets off its alarm and can bring a declaration; know the fog before you settle, found cities behind natural barriers, and expand toward the empty side. Legatum: new cities must be more than 3 hexes from every city, including allies'.
- **Known counter:** Settle at the edge of their territory deliberately to provoke.
- **Source:** [Diplomacy & Alliances in Civ VI (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=792333308)
- **Fit: adapted.** Legatum AI reaction to forward cities is unmeasured.
- **Factions:** Celts, Japan.
- **AI behaviour to test:** City site scoring penalises sites inside 6 hexes of a stronger neighbour.

### WX12 Hold the land choke with troops
- **Tags:** defence_vs_rush, wide_expansion. **Civ 6:** Egypt, Dido.
- **What to do:** A bottleneck of a tile or two between you and your opponent: settle near it or watch it with troops to control the opponent, who may not even find your capital. Legatum: mountain ridges and water create such choke tiles.
- **Known counter:** Take the choke with a larger army.
- **Source:** [Aggressive Strategies to Dominate your Opponent(s) in Multiplayer (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=517454225)
- **Fit: possible.** Native.
- **Factions:** Celts, Switzerland. Works against: Mongols.
- **AI behaviour to test:** Fortify a unit on every 1-2 hex gap between your land and a stronger neighbour.

### DF20 Evict the squatter with a cheap ranged unit
- **Tags:** defence_vs_rush, tall_defensive. **Civ 6:** Vikings (counter), Mongolia (counter).
- **What to do:** When a rival parks a fortified unit on your best tile, build a cheap ranged unit and push it off: the tile is worth more than the unit. Keep vision on every ring-1 tile and shoot squatters from outside the city's range. Legatum: Byzantium squatters deny your yields and earn Science from them; ranged fire resolves before movement and takes no retaliation.
- **Known counter:** A squatter group too large to evict; take the war to their home.
- **Source:** [Aggressive Strategies to Dominate your Opponent(s) in Multiplayer (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=517454225)
- **Fit: possible.** Native; the Byzantium counter-play of record ('evict them or tolerate the loss').
- **Factions:** Switzerland, Japan, Celts, Athens. Works against: Byzantium.
- **AI behaviour to test:** Any hostile unit on a ring-1 tile of our city is shot by the nearest ranged unit within 2 turns.

## Wide expansion (17)

### WX01 Settle on turn 1
- **Tags:** wide_expansion. **Civ 6:** Mongolia, Sumeria.
- **What to do:** Found the capital on turn 1 about 95% of the time; wait a turn only for a clearly superior hill-river-resource spot. Wandering costs a turn of everything. Legatum: no starting city, the Settler founds on turn 1, the capital gets +2 Production, and the first Settler costs full price (75).
- **Known counter:** Settle your own spot first.
- **Source:** [Reddit: what am I doing wrong in my early game (r/civ)](https://www.reddit.com/r/civ/comments/xzho6o/)
- **Fit: possible.** Native.
- **Factions:** Mongols, Sparta, Japan, Germany, Celts.
- **AI behaviour to test:** Found city T1 unless a site with at least +4 yield is one tile away.

### WX02 Settler before the first culture building
- **Tags:** wide_expansion. **Civ 6:** any.
- **What to do:** The first Settler matters more than the first monument: expand first. Typical order is Scout, Slinger, Settler, then a second Settler. Legatum: a Settler needs pop 2, first growth costs 65 Food at every speed, and the Granary (+2 Food, 50 Production) is the only turn-1 building.
- **Known counter:** Pillar: settle first.
- **Source:** [Reddit: what am I doing wrong in my early game (r/civ)](https://www.reddit.com/r/civ/comments/xzho6o/)
- **Fit: adapted.** Granary replaces monument as the early build.
- **Factions:** Japan, Mongols, Sparta.
- **AI behaviour to test:** Scout then Granary then Settler; queue the Settler when pop reaches 2.

### WX03 Two scouts then two settlers for three cities
- **Tags:** wide_expansion. **Civ 6:** any.
- **What to do:** Open with two scouts followed by two settlers to take three cities early: the scouts find sites, ruins and neighbours, the settlers claim the space. Then add defenders. Legatum: cities need more than 3 hexes between them, a Settler costs 75 + 20 per trained, and you must reach pop 2 first.
- **Known counter:** Rush the unprotected expansion.
- **Source:** [Reddit: strategies I use to beat Deity (r/CivVI)](https://www.reddit.com/r/civ/comments/1dl8s0l/)
- **Fit: possible.** Native; watch Settler escalation of +20 each.
- **Factions:** Japan, Mongols, Athens.
- **AI behaviour to test:** Opening: Scout, Scout, Settler, Settler before the first defender unless a neighbour is within 8 hexes.

### WX04 Triangle opening: three cities 3-4 tiles apart
- **Tags:** wide_expansion. **Civ 6:** Lincoln, Germany.
- **What to do:** Settle three cities in a close triangle, 3-4 tiles apart, and put the production into builders and markets. A close triangle is the strongest opening because every city shares the work and the defence. Legatum: cities need more than 3 hexes of distance (so 4), and a city works every tile it owns in 3 hexes, so overlap is a cost.
- **Known counter:** Wedge a fourth city into the centre or raid the single exposed side.
- **Source:** [Reddit: Deity AI opening weaknesses (r/CivVI)](https://www.reddit.com/r/civ/comments/1ff1y71/)
- **Fit: adapted.** Minimum distance is 4 hexes; Japan wants this density.
- **Factions:** Japan.
- **AI behaviour to test:** Japan founds cities at distance 4 in a triangle around the capital before reaching beyond 4 hexes.

### WX05 Escort every Settler
- **Tags:** wide_expansion, defence_vs_rush. **Civ 6:** any.
- **What to do:** Do not send your settler alone unless you are absolutely sure no barbarian or rival can capture it: pair it with a scout or fighter. Use your own scout to threaten rival Settlers. Legatum: raiders capture lone civilians.
- **Known counter:** Hunt unescorted Settlers.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Native.
- **Factions:** Japan, Mongols, Celts.
- **AI behaviour to test:** Settler moves with an escort when any hostile or camp is within 7 hexes.

### WX06 Building a few Settlers is the whole trick
- **Tags:** wide_expansion. **Civ 6:** Lincoln.
- **What to do:** Ten cities by turn 100 is not about perfect play or the perfect civ: you just need to actually build the Settlers. Keep a Settler in production while growth allows. Legatum benchmarks should compare against sim medians (first city T1, second ~T25 at Standard).
- **Known counter:** Cut off the land they need.
- **Source:** [Reddit: 10 cities by turn 100 (r/CivVI)](https://www.reddit.com/r/civ/comments/19endem/)
- **Fit: possible.** Benchmark from sims, not Civ 6.
- **Factions:** Japan, Mongols, Sparta, Celts.
- **AI behaviour to test:** Settler share of early production never drops below a floor while sites remain.

### WX07 Spam three Settlers when a close neighbour exists
- **Tags:** wide_expansion, defence_vs_rush. **Civ 6:** any.
- **What to do:** If a neighbour is close, spam three Settlers right after the opener so you are not forward settled and boxed in. Expansion density is the defence. Legatum: Settler cost rises 20 each, so the third costs 115.
- **Known counter:** Out-settle first, or wedge.
- **Source:** [Reddit: how to compete in Civ 5 FFA multiplayer (r/civ5)](https://www.reddit.com/r/civ/comments/avs26i/)
- **Fit: possible.** Native.
- **Factions:** Japan, Switzerland, Celts.
- **AI behaviour to test:** Expand to 3 cities by T40 when a neighbour is within 10 hexes.

### WX08 Infinite city sprawl (ICS) and how to spot it
- **Tags:** wide_expansion. **Civ 6:** Shaka, Attila.
- **What to do:** Many one-size cities early is a heavy-production sprawl that goes military early. Spot it by its trade-screen resources and city count. Legatum: ICS is capped by the 4-hex spacing, pop-2 Settlers and rising cost, so dense 4-hex grids are the Legatum form.
- **Known counter:** Punish the thin garrisons.
- **Source:** [Guide to civ5 multiplayer (CivFanatics, via web.archive.org)](https://forums.civfanatics.com/threads/guide-to-civ5-multiplayer.441979/)
- **Fit: adapted.** Spacing and Settler cost cap it; Japan wants it.
- **Factions:** Japan, Sparta.
- **AI behaviour to test:** Detect rival sprawl from public city count growth; raise rush priority against it.

### WX09 Compact versus dispersed empires
- **Tags:** wide_expansion. **Civ 6:** Germany, Kongo, Khmer.
- **What to do:** Compact empires (cities 3-4 tiles apart) maximise adjacency and area bonuses; dispersed ones (5-6 apart) suit wonders and unique improvements. Decide the spacing before the second Settler. Legatum: a city works every owned tile within 3 hexes, so tiles overlap at 4; Japan gets 7% Food/Production per other Japan city within 5 hexes.
- **Known counter:** Cut a dense empire with a wedge.
- **Source:** [Zigzagzigal's Guides: Sumeria (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1905022232)
- **Fit: adapted.** No districts or adjacency; the compact case is Japan.
- **Factions:** Japan.
- **AI behaviour to test:** Site scoring: Japan prefers spacing 4 inside 5 hexes of the capital; others 5.

### WX10 Close the chokepoint behind you
- **Tags:** wide_expansion, defence_vs_rush. **Civ 6:** Inca, Nubia.
- **What to do:** Identify chokepoints quickly and settle cities to close off the areas behind them for your opponents, so you are not boxed in and the rest is yours. Legatum: one continent holds at least 85% of land; mountain ridges are the natural walls.
- **Known counter:** Settle through the choke first.
- **Source:** [Reddit: small tips that helped immeasurably (r/civ)](https://www.reddit.com/r/civ/comments/7rix87/)
- **Fit: possible.** Native map.
- **Factions:** Japan, Celts.
- **AI behaviour to test:** Scout the map for 1-2 hex gaps; first Settlers claim the gap.

### WX11 Surround the enemy with new cities
- **Tags:** wide_expansion, rush_conquest. **Civ 6:** Rome, Persia.
- **What to do:** In a slow war with a slight edge, settle new cities to surround your enemy and gain map control; ranged units become very powerful. Do not overextend or be surprised by 2-to-1 unit counts. Legatum: borders grow per city and population is territory.
- **Known counter:** Trade your edge for tempo before they surround you.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Native.
- **Factions:** Mongols, Japan.
- **AI behaviour to test:** When ahead by 20%+ army, found forward cities on the enemy's frontier instead of marching.

### WX13 Take the good spots around the rival
- **Tags:** wide_expansion. **Civ 6:** Mali, Brazil, Russia.
- **What to do:** Deny a rival's preferred sites: take non-desert spots around Mali so it is stuck with a lack of food and production; settle near rainforest so Brazil cannot use its rainforest ability. Legatum: settlers need distance 4 from cities, so a single city can wreck a rival's grid.
- **Known counter:** Expand away from them.
- **Source:** [Zigzagzigal's Guides: Mali (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1918595516)
- **Fit: possible.** Native.
- **Factions:** Japan, Celts.
- **AI behaviour to test:** Site scoring adds a denial term for rival best sites within 8 hexes.

### WX14 Buy the tiles next to their best district
- **Tags:** wide_expansion, economic_gold. **Civ 6:** Korea (counter).
- **What to do:** Use Gold to buy tiles next to a rival's key improvements before it can use them. Legatum: BuyTile exists and Civic Networks makes bought tiles 25% cheaper; new cities claim ring one and population claims more.
- **Known counter:** Buy first; hold the border with units.
- **Source:** [Zigzagzigal's Guides: Korea (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1928634264)
- **Fit: adapted.** Check that tile buying works against rival-adjacent tiles at Peace.
- **Factions:** Switzerland. Works against: Athens.
- **AI behaviour to test:** Spare Gold buys border tiles that flip a rival ring-1 tile.

### WX15 Expand by conquest instead of Settlers
- **Tags:** wide_expansion, rush_conquest. **Civ 6:** Sumeria, Macedon.
- **What to do:** Ram your fast units into a neighbour's cities and enjoy a good-sized empire without having to train any Settlers. Legatum: the design pillar is that expansion by settling or by conquest is the core loop.
- **Known counter:** Hold the border; offer Peace after the first loss.
- **Source:** [Zigzagzigal's Guides: Sumeria (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1905022232)
- **Fit: possible.** Pillar 10.
- **Factions:** Mongols, Sparta.
- **AI behaviour to test:** If settling room is under 3 sites and a weak neighbour exists, switch to conquest.

### WX16 Outpost cities for single resources
- **Tags:** wide_expansion. **Civ 6:** any.
- **What to do:** Small outpost cities to secure single resources are feasible: drop a city in an otherwise useless tile solely to connect or deny a strategic resource. Legatum: Iron (Hills/Tundra) and Coal (Hills/Forest) are placed on the map; settle where a resource is claimed.
- **Known counter:** Take the outpost.
- **Source:** [Reddit: going wide is always better (r/civ)](https://www.reddit.com/r/civ/comments/57axu6/)
- **Fit: adapted.** Check which Legatum units need Iron or Coal.
- **Factions:** Germany, Japan.
- **AI behaviour to test:** Found outposts on contested Iron/Coal hexes when a rival is within 8.

### WX17 Wide beats tall
- **Tags:** wide_expansion. **Civ 6:** Rome, Germany.
- **What to do:** The old tall-versus-wide penalty to wide empires is gone in the community's view: more cities mean more production, more population and more targets for victory points. Plan for 4+ cities as the baseline. Legatum: the FourCities milestone gives 1 VP and 'tall' is deliberately not viable.
- **Known counter:** Let them overextend, then hit the weakest new city.
- **Source:** [Reddit: going wide is always better (r/civ)](https://www.reddit.com/r/civ/comments/57axu6/)
- **Fit: possible.** Pillar 10 agrees.
- **Factions:** Japan, Sparta, Mongols, Celts.
- **AI behaviour to test:** Settling floor of 4 cities before the first war unless the target is within 8 hexes.

### WX18 Scout early for ruins
- **Tags:** wide_expansion, science. **Civ 6:** Poland, Persia.
- **What to do:** Early scouting beats late: ancient ruins are taken quickly, and they can give a free citizen, Gold or Science. Legatum: ruins are 4+ tiles apart (about 1 per 60 land tiles) and pay 40 Gold, 25 Science, +1 pop or a free Scout; Athens always get 30 Science, and Byzantium doubles Gold and Science.
- **Known counter:** Claim them first; keep scouts alive.
- **Source:** [Advanced Multiplayer Strategies (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=229432166)
- **Fit: possible.** Native.
- **Factions:** Athens, Byzantium, Mongols.
- **AI behaviour to test:** Scouts prioritise ruins in the first 25 turns; Athens and Byzantium weight them x2.

## Tall and defensive (4)

### TD01 Few cities, big cities, wonders first: the target on the weak
- **Tags:** tall_defensive, rush_conquest. **Civ 6:** Egypt, Maya.
- **What to do:** The easiest players to weed out are the ones that only build wonders early because they build no units to defend themselves. Pick such targets (and avoid becoming one). Legatum: Great Works are a late race; early 'wonder' equivalents are Archive and Laboratory.
- **Known counter:** Never skip the garrison.
- **Source:** [Advanced Multiplayer Strategies (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=229432166)
- **Fit: possible.** Native.
- **Factions:** Mongols, Sparta. Works against: Athens, Switzerland.
- **AI behaviour to test:** Target score adds a bonus for rivals with low army per city.

### TD02 Home-ground strength bonus
- **Tags:** tall_defensive, defence_vs_rush. **Civ 6:** Maya, Greece.
- **What to do:** Defensive units fight better at home: within six tiles of the capital Hul'che go from 28 to 38 strength. Settle compactly around the capital and fight there. Legatum: Celts land attrition and Japan's yield loss beyond 4 hexes both push the same shape.
- **Known counter:** Do not fight inside their zone.
- **Source:** [Zigzagzigal's Guides: Maya (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2104285340)
- **Fit: adapted.** No home-ground combat bonus; the Legatum equivalent is Walls, hills and Celts attrition.
- **Factions:** Celts, Japan.
- **AI behaviour to test:** Defensive AI stays within 5 hexes of the capital for the first fights.

### TD03 Dense cities with huge population
- **Tags:** tall_defensive, wide_expansion. **Civ 6:** Kongo, Germany.
- **What to do:** Pack cities so each can grow huge: Kongo's Mbanza practically removes the limit on population, and Germany's cities develop among the best of all civs. Legatum: population is territory (a city works every tile it owns within 3 hexes), so dense cities add Food and Production by network (Japan 7% per Japan city within 5 hexes).
- **Known counter:** Wedge between the cities; raid the edges.
- **Source:** [Zigzagzigal's Guides: Kongo (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2533397288)
- **Fit: adapted.** No Mbanza; Japan's Brood Hall and network are the matching rules.
- **Factions:** Japan, Germany.
- **AI behaviour to test:** Japan cities cluster at spacing 4 and add Brood Halls before Markets.

### TD05 Fight from woods and jungle you know
- **Tags:** tall_defensive, defence_vs_rush. **Civ 6:** Vietnam, Mapuche, Maya.
- **What to do:** Turn the forest into a weapon: units in woods and rainforest get better defence and extra mobility, and the defender cancels the enemy's own terrain bonus there. Settle and fight where the cover is yours and the aggressor must walk in. Legatum: Celts hostile units lose up to 12 HP a turn on Celts land, its Ranger and Marksman are hidden until they fire (strategic ambiguity), and Hills and Forest block sight beyond themselves.
- **Known counter:** Do not invade; out-expand; keep sight on revealed hunters and mind last-seen markers.
- **Source:** [Zigzagzigal's Guides: Vietnam (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2589592137)
- **Fit: adapted.** Celts is the matching faction; the hidden first strike and attrition are Legatum-only rules.
- **Factions:** Celts. Works against: Celts.
- **AI behaviour to test:** Celts hunters stay hidden until a target enters 2 hexes in forest, then fire first and relocate.

## Economic and Gold (7)

### EG01 Gold over production: instant delivery anywhere
- **Tags:** economic_gold. **Civ 6:** Portugal, Mali, Netherlands.
- **What to do:** Gold buys the thing you want, in any city, on time, and cannot be stolen away by a better production city; production cities must spend their output on more production. Build Markets early and buy the Settler, Walls or defender where it is needed. Legatum: Exchange (35) then Market (60, +3 Gold), BuyItem at 200% Production in Gold (175% Switzerland), one purchase per city per turn.
- **Known counter:** Raid the treasury (captures pay loot), and deny the early production that starts the money engine.
- **Source:** [Reddit: gold beats production (r/civ)](https://www.reddit.com/r/civ/comments/1ecoobt/)
- **Fit: possible.** Native; Athens Markets give +1 only, so they should not copy it.
- **Factions:** Switzerland, Athens.
- **AI behaviour to test:** Economy opener Exchange then Market in the capital; the Gold budget buys the second and third Settlers.

### EG02 Steal the treasury, not the production
- **Tags:** economic_gold, rush_conquest. **Civ 6:** Mali, Netherlands.
- **What to do:** You can steal gold but not production: capture the richest empire's city and the treasury moves with it. Legatum: Sparta capture takes a population share of the owner's Gold plus stored production, kills pay Gold, and a captured Swiss city pays the captor a treasury share, doubled for the capital.
- **Known counter:** Spend the treasury before you are in reach, or defend the capital.
- **Source:** [Reddit: gold beats production (r/civ)](https://www.reddit.com/r/civ/comments/1ecoobt/)
- **Fit: possible.** Native in Sparta and Swiss rules.
- **Factions:** Sparta, Mongols. Works against: Switzerland.
- **AI behaviour to test:** Target score adds expected treasury loot; pick the richest reachable capital.

### EG03 Buy the Settler instead of building it
- **Tags:** economic_gold, wide_expansion. **Civ 6:** any.
- **What to do:** Buy Settlers where Gold is available and keep the cities producing the long builds; nobody convinced the poster that building a Settler beats buying one unless the housing limit is hit anyway. Legatum: a Settler needs pop 2 to be bought, and cost escalates by 20 each; one purchase per city per turn.
- **Known counter:** Starve their Gold or take the capital.
- **Source:** [Reddit: the Feudalism wave (r/civ)](https://www.reddit.com/r/civ/comments/5aneb7/)
- **Fit: possible.** Native; compare 200% price against the escalating cost.
- **Factions:** Switzerland, Athens, Japan.
- **AI behaviour to test:** When Gold exceeds 2x the Settler cost and a site exists, buy the Settler rather than build.

### EG04 Instant-purchase economy with a production penalty
- **Tags:** economic_gold. **Civ 6:** Mali, Netherlands.
- **What to do:** Build the game around Gold (and faith) instant purchases instead of production: weak start, poor production, but you can purchase buildings and units anywhere. Protect the start with a Slinger then a Settler. Legatum: Switzerland earn interest (+1 per 50 Gold held at Quick, capped at 5 per turn) and tolls (pop/2).
- **Known counter:** Rush them early: they are slow to train units and walls.
- **Source:** [Zigzagzigal's Guides: Mali (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1918595516)
- **Fit: possible.** Native for Switzerland; interest rewards holding Gold.
- **Factions:** Switzerland. Works against: Mongols, Sparta.
- **AI behaviour to test:** Swiss AI holds the interest threshold while Walls and border defenders are bought just-in-time.

### EG05 Buy every Great Work you can handle
- **Tags:** economic_gold. **Civ 6:** Portugal, Kongo.
- **What to do:** Use Gold to buy the prestige items. Legatum: only Switzerland may buy Great Works, at 175% of Production cost; RelayUplink, PlanetaryFoundry and WorldExchange cost 300 Production each, one of three per empire, worth 2 VP; their cities need an Archive, Autoworks or two Clearinghouses.
- **Known counter:** Take the Great Work city (it moves VP) or knock the treasury down.
- **Source:** [Reddit: gold beats production (r/civ)](https://www.reddit.com/r/civ/comments/1ecoobt/)
- **Fit: adapted.** Great Works exist as a national race, not as artworks; buying is Swiss-only.
- **Factions:** Switzerland. Works against: Mongols, Sparta.
- **AI behaviour to test:** Swiss AI buys a legal Work when it preserves the pending treasury award.

### EG06 Fund the army: do not neglect Gold in a long campaign
- **Tags:** economic_gold, rush_conquest. **Civ 6:** Macedon, Zulu.
- **What to do:** A multi-front domination campaign needs a ridiculously large economy to pay army upkeep and upgrades; you cannot neglect gold production. Legatum: the first 2 military units are free, extras cost 1 Gold a turn (3 for Switzerland), so a 12-unit army costs 10 Gold a turn.
- **Known counter:** Starve income; take the treasury.
- **Source:** [Zany's Comprehensive Civ 6 Guide (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=926983840)
- **Fit: possible.** Native.
- **Factions:** Sparta, Mongols, Germany.
- **AI behaviour to test:** Army size cap = (income - reserve) / 1 Gold per unit beyond 2.

### EG07 A standing army is the price of peace
- **Tags:** economic_gold, defence_vs_rush. **Civ 6:** Scythia, Gorgo.
- **What to do:** You need a decent standing army to keep AIs from declaring on you constantly; the cheapest army is the one that you never lose units from, so fortify and trade space for time. Legatum: units beyond the first two cost 1 Gold a turn, so keep a fixed deterrent, not a growing one.
- **Known counter:** Out-tech the standing force, or hit it when it is out of position.
- **Source:** [Reddit: early armies in Civ 6 (r/civ)](https://www.reddit.com/r/civ/comments/593yja/)
- **Fit: possible.** Upkeep exists, so size is a decision.
- **Factions:** Switzerland, Celts, Athens.
- **AI behaviour to test:** Deterrent army sized to the neighbour's army share, no more.

## Science (9)

### SC01 Science cluster, then a defensive tech to keep the lead
- **Tags:** science, defence_vs_rush. **Civ 6:** Korea, Arabia, Maya.
- **What to do:** Build science buildings first and expect neighbours to get nervous: a defensive tier-up (Hwacha for Korea) is rushed so a lead is safe. Legatum: Laboratory (+3, Signal Theory 40), College (+4, Academies 100), Archive (+3); Ballistics (60) gives Trooper; Walls and Citadel hold it.
- **Known counter:** Take the science cluster (it is concentrated) before the tier-up lands.
- **Source:** [Zigzagzigal's Guides: Korea (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1928634264)
- **Fit: possible.** Native; Athens' Laboratory is +6.
- **Factions:** Athens. Works against: Mongols, Sparta.
- **AI behaviour to test:** Athens research order: science buildings then the first defence tier-up before turn 60.

### SC02 Beeline the one key production or military tech
- **Tags:** science, rush_conquest. **Civ 6:** Babylon, Hammurabi.
- **What to do:** Skip economy techs and rush the single tech that unlocks your power spike: Babylon's reliable opener is rushing Industrialisation to unlock Factories; another path is mines for Apprenticeship and Men-at-Arms plus a Ram. Legatum has no eurekas or prerequisite skipping, but a beeline is still real: Fabrication, Machine Tools (90, Machine Shop +3 Production), or Ballistics (60).
- **Known counter:** Strike before the spike lands.
- **Source:** [Zigzagzigal's Guides: Babylon (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2387413001)
- **Fit: adapted.** No eurekas; beelining by research order only.
- **Factions:** Germany, Athens.
- **AI behaviour to test:** Research order queues a spike tech with its prerequisites only.

### SC03 Hit the science-obsessed AI
- **Tags:** science, rush_conquest. **Civ 6:** Korea (counter), Babylon (counter).
- **What to do:** When AIs over-prioritise science, they under-defend: players noted the Civ 6 AI plays like a Civ 5 science AI and is a lot more vulnerable to military aggression. Target the tech leader's thin garrisons first. Legatum: Most Advanced is a title, so the strike also steals 1 VP.
- **Known counter:** Keep army parity even when ahead in tech.
- **Source:** [Reddit: three lines of AI code (r/civ)](https://www.reddit.com/r/civ/comments/oe4yqg/)
- **Fit: possible.** Applies to Athens and any science-leaning AI.
- **Factions:** Mongols, Sparta, Byzantium. Works against: Athens.
- **AI behaviour to test:** If the tech leader's army share < 0.5x ours and its capital is reachable, add it as a candidate target.

### SC04 Read the rival's research before you engage
- **Tags:** science, late_game_closing. **Civ 6:** Australia, Korea.
- **What to do:** Check the rival's tech tree progress: if it is at or near the next tier, avoid it until you have the same units; if it has skipped the military branch, you can attack it with a tier advantage. Legatum: Most Advanced and sightings show tier; tech diffusion cuts costs 10% per empire that knows a tech (max 30%).
- **Known counter:** Hide your tech with distance.
- **Source:** [Zigzagzigal's Guides: Australia (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2542999669)
- **Fit: possible.** Native.
- **Factions:** Athens, Mongols, Sparta, Byzantium.
- **AI behaviour to test:** Tier comparison before every declaration; avoid targets within one tier of ours.

### SC05 Conquest as research
- **Tags:** science, rush_conquest. **Civ 6:** Alexander (Macedon), Tomyris.
- **What to do:** Alexander gets a free boost for tech and science for every city captured: each capture accelerates the next. Treat war as a research source. Legatum has no eurekas, but the Byzantium earn Science from squatting (2x tile yields, +5 per tech the landowner has) and Salvage Works adds more.
- **Known counter:** Deny the captures: fortify the border city and retake it.
- **Source:** [Domination Victory Tips (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1229036179)
- **Fit: adapted.** Only the Byzantium converts war presence into Science.
- **Factions:** Byzantium. Works against: Byzantium.
- **AI behaviour to test:** Byzantium squats on the tile of whichever rival knows the most techs it lacks.

### SC06 Ask how science boosts the army
- **Tags:** science, rush_conquest. **Civ 6:** Babylon, Korea.
- **What to do:** Stop treating science, culture and religion as separate victories: ask how each one boosts your military. Win the fights that matter first; spend science on unit tier. Legatum has a VP victory so science is only a milestone (SixTechs) and the Most Advanced title.
- **Known counter:** Force the war before the tier arrives.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Native; Legatum has no science victory.
- **Factions:** Athens, Germany.
- **AI behaviour to test:** Research queue prefers unit-tier techs unless Most Advanced is within 1 tech.

### SC07 Great Library rush: only with the production lead
- **Tags:** science, economic_gold. **Civ 6:** Egypt, Babylon.
- **What to do:** Rushing the early wonder is very bad most of the time: others can forward settle you while you build it, and the person with the highest production usually wins. Do it only after checking demographics and land. Legatum: the Great Work race (300 Production, one of three per empire, 2 VP) is the equivalent.
- **Known counter:** Forward settle the builder or take the city.
- **Source:** [How to Rush the Great Library (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=957628210)
- **Fit: adapted.** Great Works are a national race; rush only with a production lead and a defended city.
- **Factions:** Germany, Switzerland, Athens.
- **AI behaviour to test:** Start the Great Work only when the production city has Autoworks (or equivalent) and a defender.

### SC08 Three to four cities for a tall science game
- **Tags:** science, tall_defensive. **Civ 6:** Maya, Babylon.
- **What to do:** A tall civ needs about three to four cities to compete scientifically with a wide one, with the full social-policy tree. Early scouting is the way to find that fourth city. Legatum: FourCities is a milestone (1 VP) and pop-based Science (+pop/2 per city) rewards a few large cities.
- **Known counter:** Wide expansion outgrows it; raid the lone capital.
- **Source:** [Advanced Multiplayer Strategies (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=229432166)
- **Fit: adapted.** Pillar 10: tall is not viable; four cities is the floor, not the plan.
- **Factions:** Athens. Works against: Mongols.
- **AI behaviour to test:** Athens AI stops expanding at 5-6 cities if a defensive posture is needed.

### SC09 Split research with your teammate
- **Tags:** science, diplomatic_coalition. **Civ 6:** Babylon.
- **What to do:** On a team, split branches and let research flow: when your teammate finishes a technology you get its boost, which is why Babylon is OP in team games. Legatum: tech diffusion cuts costs 10% per empire that knows the tech; no trading.
- **Known counter:** Hit the weaker partner.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: adapted.** No tech trading; diffusion is the shared effect. Check whether teammates count for diffusion.
- **Factions:** Athens.
- **AI behaviour to test:** Team AI avoids researching what the teammate has researched or queued.

## Diplomatic and coalition (8)

### DC01 Ally the friendly AI the turn you meet it
- **Tags:** diplomatic_coalition. **Civ 6:** Gilgamesh (Sumeria), Cyrus.
- **What to do:** Befriend the easy-going leader on first contact: it nearly always accepts and stays a friend all game, giving trade safety and a shared war. Legatum: an Alliance needs first contact (a unit, city or owned tile seen), costs 30 Gold at Quick (60 Standard) plus upkeep per city, and shares vision.
- **Known counter:** Betray in the first 10 turns costs 60 Gold and 20 turns of no re-alliance with the victim.
- **Source:** [Reddit: small tips that helped immeasurably (r/civ)](https://www.reddit.com/r/civ/comments/7rix87/)
- **Fit: adapted.** No 'friend' AI personalities in Legatum; evaluate by neighbours' threat.
- **Factions:** Switzerland, Athens, Celts, Japan.
- **AI behaviour to test:** On first contact with the weakest neighbour who is not a target, offer an Alliance if Gold allows.

### DC02 Attack the target before its allies
- **Tags:** diplomatic_coalition, rush_conquest. **Civ 6:** Sumeria, Macedon.
- **What to do:** Hit the target directly before attacking its allies: allies have some incentive to protect their friends but not the reverse, and an ally of the target adds strength to its side. Legatum: an Alliance means the two teams cannot attack each other, share vision and may be called to arms.
- **Known counter:** Alliance chains: ally the neighbour of your likely attacker.
- **Source:** [Zigzagzigal's Guides: Sumeria (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1905022232)
- **Fit: possible.** Native; avoid triggering CallToArms by timing.
- **Factions:** Mongols, Sparta.
- **AI behaviour to test:** Target selection removes the target's allies from the plan or isolates it first.

### DC03 Call a joint war
- **Tags:** diplomatic_coalition, rush_conquest. **Civ 6:** Sumeria, Gilgamesh.
- **What to do:** Send a joint-war proposal early so you fight on the same side, share the spoils and avoid being hit by the ally. Legatum: CallToArms is a treaty kind; an Alliance lets either side call the other.
- **Known counter:** Peace separately; keep the target alive.
- **Source:** [Zigzagzigal's Guides: Sumeria (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1905022232)
- **Fit: possible.** Native treaty.
- **Factions:** Sparta, Mongols, Switzerland.
- **AI behaviour to test:** When an ally is at war with X and X is reachable, call it to arms before declaring yourself.

### DC04 Second and third place gang up on the early leader
- **Tags:** diplomatic_coalition, late_game_closing. **Civ 6:** any.
- **What to do:** Players who fall behind should be expected to team up against any civ that takes an early lead; the leader should plan for a coalition. Legatum: a coalition target is the VP-hold leader; Alliances between two teams that are both at war with the target cost no signing fee.
- **Known counter:** Take the lead late and hold only the 10 turns needed.
- **Source:** [Advanced Multiplayer Strategies (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=229432166)
- **Fit: possible.** Native coalition rule (free alliance fee against the target).
- **Factions:** Switzerland, Athens, Japan, Sparta. Works against: Athens, Switzerland.
- **AI behaviour to test:** Coalition scoring ranks holders first; join when the target's VP is >= target - 3.

### DC05 The first attacker usually loses
- **Tags:** diplomatic_coalition, defence_vs_rush. **Civ 6:** any.
- **What to do:** Among experienced players the first one to attack usually loses, as long as a third party stays out and keeps growing: the two who fight wear each other down. So attack only with a clear edge and a quick finish, or wait to be attacked. Legatum: 8 empires make the third-party effect the norm.
- **Known counter:** Strike the second fighter when it is weakened.
- **Source:** [Advanced Multiplayer Strategies (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=229432166)
- **Fit: possible.** Direct AI rule; contradicts early-rush openings unless the edge is decisive.
- **Factions:** Celts, Switzerland, Athens, Japan. Works against: Mongols, Sparta.
- **AI behaviour to test:** Declaration gate: refuse first strikes against a peer when two other strong empires are unengaged, unless force ratio >= 2.

### DC06 Secure the rear before a mid-game war (2 against 1)
- **Tags:** diplomatic_coalition, rush_conquest. **Civ 6:** Macedon, Persia.
- **What to do:** Before attacking, find a two-versus-one setup or assurance that you will not be attacked from behind; it is easiest against the point leader everyone wants displaced. Captured cities cost happiness and loyalty so do not overreach. Legatum: peace with the rest is free and Alliances cost Gold only.
- **Known counter:** Pull the third party in.
- **Source:** [Advanced Multiplayer Strategies (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=229432166)
- **Fit: possible.** Native.
- **Factions:** Sparta, Mongols.
- **AI behaviour to test:** Attack only with Peace or Alliance with all other bordering empires.

### DC07 Avoid war with several teams at once
- **Tags:** diplomatic_coalition, defence_vs_rush. **Civ 6:** any.
- **What to do:** Do not get into wars with multiple teams: if a teammate is far away, turtle up and feed them gold. Legatum: a concurrent-war penalty belongs in the AI's war score.
- **Known counter:** Pile on.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Native; Legatum AI should carry a per-war penalty (open item).
- **Factions:** Celts, Switzerland.
- **AI behaviour to test:** War score: -X per concurrent war; no new declaration above two.

### DC09 Stop the snowballer
- **Tags:** diplomatic_coalition, late_game_closing. **Civ 6:** Tomyris, Alexander.
- **What to do:** Take cities or attack a runaway rival to stop it from snowballing or winning the game. Legatum: a team that holds the VP target for 10 turns wins, so breaking a hold means taking a title or a city.
- **Known counter:** Hold margins in titles; garrison the cities that carry VP.
- **Source:** [Domination Victory Tips (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1229036179)
- **Fit: possible.** Native.
- **Factions:** Mongols, Sparta, Athens, Byzantium. Works against: Switzerland, Athens.
- **AI behaviour to test:** When any team has a VP hold going, spend the army on its cheapest VP source.

## Late-game closing (7)

### LG01 Convert the lead with the best unit tier
- **Tags:** late_game_closing, rush_conquest. **Civ 6:** Poland, Mongolia.
- **What to do:** The meta is snowball, research a mid-to-late-game unit (like cavalry), then crush another player. In the closing phase switch research to the tier that wins fights. Legatum: Trooper (32) then Marksman (26 ranged) and Assault Trooper (44).
- **Known counter:** Walls and Citadel; draw them into a long siege.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Native.
- **Factions:** Sparta, Mongols, Germany.
- **AI behaviour to test:** After 4 cities and Peace on other fronts, research order goes to the next unit tier.

### LG02 Time the war in the narrow window before the next wall tier
- **Tags:** late_game_closing, rush_conquest. **Civ 6:** Macedon, Persia.
- **What to do:** There is a narrow window between getting siege units and the rival reaching the next wall tier: be ready with the right stack and go the turn it opens. Legatum: Walls (Fortification) then Citadel (Mutual Defense, +5); Siege Engineering is the siege unlock.
- **Known counter:** Reach the next wall tier before they reach siege.
- **Source:** [Reddit: war in the mid game (r/civ)](https://www.reddit.com/r/civ/comments/1gvmu1q/)
- **Fit: possible.** Native tech timing.
- **Factions:** Germany, Sparta.
- **AI behaviour to test:** Compare rival Citadel tech to our siege tech; strike if the gap is >= 1 tech.

### LG03 Conquer the wonder owner instead of racing it
- **Tags:** late_game_closing, rush_conquest. **Civ 6:** Egypt, Rome.
- **What to do:** It is safer to conquer the civilization that has the wonders than to try to build them against a stronger AI. Legatum: Great Works are 300 Production each and give 2 VP; the owner's city is a VP source.
- **Known counter:** Garrison and wall the wonder city; keep it away from neighbours' reach.
- **Source:** [Domination Victory Tips (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1229036179)
- **Fit: adapted.** Check whether a captured Great Work keeps its VP for the captor.
- **Factions:** Mongols, Sparta, Byzantium. Works against: Switzerland, Athens.
- **AI behaviour to test:** Value a city by Great Work VP when it holds the owner's only Work.

### LG04 The leader expects a coalition: plan for it
- **Tags:** late_game_closing, diplomatic_coalition. **Civ 6:** any.
- **What to do:** Players A and B may faithfully work together to eliminate a stronger C; the leader should assume it and arrange peace on the other fronts first. Legatum: every AI treats a VP holder as the top threat, and the winner must survive 10 turns at the target.
- **Known counter:** Strike the leader early in the hold, not late.
- **Source:** [Advanced Multiplayer Strategies (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=229432166)
- **Fit: possible.** Native.
- **Factions:** Mongols, Sparta, Switzerland, Japan, Celts, Germany, Byzantium, Athens.
- **AI behaviour to test:** Leaders keep Peace/Alliance with every neighbour and garrison VP cities during a hold.

### LG05 When behind, scout a lot and bleed the weak
- **Tags:** late_game_closing. **Civ 6:** any.
- **What to do:** If you are behind, catch up on economy and scout a lot: scouts are your defence, and weak neighbours are the cheap gains. Legatum: a Scout is 30 Production with free upkeep.
- **Known counter:** Take the economy early.
- **Source:** [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **Fit: possible.** Native.
- **Factions:** Mongols, Sparta, Switzerland, Japan, Celts, Germany, Byzantium, Athens.
- **AI behaviour to test:** Trailing AI increases scout count and targets the weakest neighbour.

### LG07 Limited wars for profit
- **Tags:** late_game_closing, rush_conquest. **Civ 6:** Gorgo, Cyrus.
- **What to do:** Fight smaller contained wars as part of a non-conquest plan: raze or take one city, then stop. Legatum: VP victories mean every war is a limited war toward a title or a city.
- **Known counter:** Peace after your gain.
- **Source:** [Reddit: limited wars (r/CivVI)](https://www.reddit.com/r/civ/comments/1j1hxmd/)
- **Fit: possible.** Native.
- **Factions:** Mongols, Sparta.
- **AI behaviour to test:** War plans end at the first VP gain; do not continue past target.

### LG08 Fortify the VP city that someone will come for
- **Tags:** late_game_closing, defence_vs_rush. **Civ 6:** Macedon (counter).
- **What to do:** Always make sure your wonder cities are either far away from the aggressor or heavily fortified with defensive buildings and units. Legatum: the Great Work cities and the capital are the cities a coalition will target in a hold.
- **Known counter:** Take the Walls first.
- **Source:** [Zigzagzigal's Guides: Macedon (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1902563180)
- **Fit: possible.** Native.
- **Factions:** Switzerland, Athens, Japan.
- **AI behaviour to test:** Garrison + Walls + Citadel in VP-source cities before declaring the target.

## Does not transfer: impossible in Legatum (13)

Each depends on a system in game-rules section 9. The Legatum-side substitute is in the text.

- **NV01 Invade the coast-hugging empire from the sea** (Australia (counter), England, Norway): Australian cities settle on the coast with no naval bonuses until someone declares war on them, so they are vulnerable to a naval invasion; use ships to land a stack. Legatum has no naval units, no ships and no amphibious assault, so this is a Civ-6 only strategy. Source: [Zigzagzigal's Guides: Australia (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2542999669)
- **NV04 Barbarian caravels razing a coast** (any): Early barbarian caravels can raze a coastal city that has only archers. Legatum: barbarian raiders never assault or capture cities and there are no ships. Source: [Reddit: barbarian caravels (r/civ)](https://www.reddit.com/r/civ/comments/7wy94v/)
- **IM01 Pillage-funded war** (Mongolia, Persia (counter), Norway): Pillage farms for healing and gold; one pillage is worth hundreds of Gold in the guide's examples. No pillage exists in Legatum ('this game has no pillage'), so use camp clearing, kills and capture loot instead. Source: [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **IM02 Great General aura** (Mongolia, Macedon): Get a Great General to buff the stack in a balanced war. Legatum has no Great People. Source: [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- **IM03 Spy and loyalty flipping of a friend's city** (Eleanor (France)): Two spies in a city force it to revolt; then take it as a free city with no warmonger penalty. Legatum has no spies and no loyalty. Source: [Reddit: conquering a friend's cities (r/civ)](https://www.reddit.com/r/civ/comments/8idx7j/)
- **IM04 Beat one city to zero for a peace deal** (any): Surround a single city and beat it to 0 repeatedly with ranged units; the AI eventually offers to hand over cities for peace. Legatum Peace is free and carries no terms. Source: [Reddit: Deity AI tricks (r/CivVI)](https://www.reddit.com/r/civ/comments/1ozb0ak/)
- **IM05 Gifts, delegations and favour trading to build trust** (Gilgamesh, Pericles): Send delegations quickly and gift to build a web of friendships and alliances. Legatum has no gifts, delegations or diplomatic favour; treaties cost Gold. Source: [Reddit: small tips that helped immeasurably (r/civ)](https://www.reddit.com/r/civ/comments/7rix87/)
- **IM06 Trade-route economy** (Portugal, Mali, Netherlands): Stack trade routes for gold: Portugal with the right pantheon makes it trivial to win any way you prefer. Legatum has no trade routes; Swiss tolls (pop/2 Gold) are the nearest idea. Source: [Reddit: the most broken setups in Civ 6 (r/civ)](https://www.reddit.com/r/civ/comments/pmsv35/)
- **IM07 Sell strategic resources and take loans from the AI** (any): Sell resources and favour to the AI for gold. Legatum has no resource trading or loans. Source: [Reddit: strategies I use to beat Deity (r/CivVI)](https://www.reddit.com/r/civ/comments/1dl8s0l/)
- **IM08 Stall research to farm boosts** (Babylon): Do not research unless you have to; wait for boosts. Legatum has no eurekas. Source: [Reddit: don't research unless you have to (r/civ)](https://www.reddit.com/r/civ/comments/kqw33u/)
- **IM09 Government and policy-card slots** (Greece, Rome): Wildcard slots and card swaps for flexibility (Greece). Legatum has no governments or policy cards. Source: [Zigzagzigal's Guides: Greece (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1980899859)
- **IM10 District adjacency stacking** (Korea, Germany, Netherlands): Place districts for adjacency (Seowon, Hansa, Polder) to out-produce neighbours. Legatum has no districts; production comes from building families. Source: [Zigzagzigal's Guides: Germany (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2536738340)
- **IM11 Trade-screen and spy peeks at a rival's builds** (any): Open the trade screen even at war to see how much iron a rival has, or use a spy list view to read its tech. Legatum has no trade screen or spies; public titles and sight give a weaker version. Source: [Guide to civ5 multiplayer (CivFanatics, via web.archive.org)](https://forums.civfanatics.com/threads/guide-to-civ5-multiplayer.441979/)

## Faction playbooks

Mapping uses the faction table in `.claude/skills/legatum-factions/SKILL.md` (state of 2026-10-07: goals redesigned, all worth 2 VP). Each playbook gives the Civ 6 civs it resembles, the catalog strategies that suit it, how rivals beat it, and what it must not copy.

### Mongols

**Rules in one line:** Courier lineage: +1 move, slip past enemy units, Scouts and civilians cross borders at Peace; no terrain defence; goal: the Courier Hall (2 VP, capturable, opens its holder's land and builds roads).

**Resembles in Civ 6:** Mongolia (cavalry that slips past defences and strikes first; the closest Civ 6 match to +1 move with no terrain defence); Sumeria (the no-tech War-Cart, a fast striker on turn 1); Persia (Cyrus) (a surprise war that arrives with bonus movement); Scythia (mobility over staying power).

**Strategies that suit it:** RC02 Day-one fast striker (War-Cart), RC01 Two-mount opening rush (resource-gated), RC05 Stage outside border, declare at the last second, RC12 Surprise war with a free move, RC03 Capital snipe, RC13 Capture lone civilians (Settler sniping), RC24 Stay unmet: hide the army, RC34 Wipe a small neighbour before it builds Walls, RC38 Rush the rich but slow-to-build empire, DC02 Attack the target before its allies, RC21 Win one early war against the weakest player, then repeat.

**Natural counter-strategies (used against it):** DF01 Walls early, fortified garrison, ranged behind, DF05 Scouts as pickets: one per bordering rival, DF06 Rush tell: low score on the first turns of the surge, DF07 Keep Gold or a small army: emergency purchase, DF17 Befriend the likely attacker first, DF03 Keep military at one third to one half of the worrisome neighbour, DC05 The first attacker usually loses, RC31 Alpha strike: whoever hits hardest in the first turns wins. A parked Scout watching their cities (the user's own rush counter), Walls plus a fortified garrison, and keeping Gold for an emergency purchase. They get no terrain defence, so fight them at your cities, not in the open. A settler wall (DF09) fails because they slip past blockers.

**Must not copy:** Long sieges (no early siege, short supply); a goal that asks them to stop rushing was rejected by the user.

**AI experiments to run first:** RC05 (Stage outside border, declare at the last second); RC03 (Capital snipe); RC04 (Quick, unexpected, brutal first strike (20-turn horizon)).

### Sparta

**Rules in one line:** Veterans from level 1, XP +50%, 6th Iron Guard level; kills heal (up to 100 HP shared) and pay Gold, captures take treasury and stored production; +50% on non-military production; desertion with no war after turn 50 (Quick) or 100 (Standard); goal: retire 3 veterans at a War Shrine.

**Resembles in Civ 6:** Sparta (Gorgo) (culture and strength from kills, veteran-first); Scythia (Tomyris) (heal on kill, the exact model for blood as the only medicine); Macedon (Alexander) (war without penalty and constant campaigns); Zulu (Shaka) (every capture feeds the next army).

**Strategies that suit it:** RC10 Kill-heal chain, RC11 Perpetual war chain, RC14 Convert every capture into more force, RC15 As many units as possible; second and third armies, RC16 First army composition and the five-turn rule, RC08 Pre-build cheap units, then mass-upgrade with Gold, RC06 Surround with three and strike together, RC03 Capital snipe, EG02 Steal the treasury, not the production, EG06 Fund the army: do not neglect Gold in a long campaign, DC03 Call a joint war, WX15 Expand by conquest instead of Settlers.

**Natural counter-strategies (used against it):** DF01 Walls early, fortified garrison, ranged behind, DF12 Shoot siege and melee first; wait for the unit to move, DC05 The first attacker usually loses, DC04 Second and third place gang up on the early leader, RC31 Alpha strike: whoever hits hardest in the first turns wins, DF02 Deter with walls and a visible army. Walls and ranged shooters that do not feed them kills; retreat wounded units rather than letting a kill heal them; coalitions; and wait out desertion at Peace (no war after turn 50 or 100 shrinks their army). Avoid mutual-kill trades that heal nobody but cost you.

**Must not copy:** Settler-heavy expansion (+50% on civilians): use conquest (WX15) instead of WX03/WX06.

**AI experiments to run first:** RC11 (Perpetual war chain); RC10 (Kill-heal chain); RC14 (Convert every capture into more force).

### Switzerland

**Rules in one line:** Interest on held Gold (+1 per 50 at Quick, capped at 5), tolls pop/2 per city, only they buy Great Works (175%); 3 Gold upkeep per unit beyond 2; a captured city pays the captor a treasury share (doubled for the capital); goal: hold 500 Gold (1,000 at Standard).

**Resembles in Civ 6:** Mali (instant purchases instead of production; weak start); Netherlands (Gold and production from compact rich cities); Portugal (wealth that buys everything); Kongo (buying Great Works).

**Strategies that suit it:** EG01 Gold over production: instant delivery anywhere, EG04 Instant-purchase economy with a production penalty, EG03 Buy the Settler instead of building it, EG05 Buy every Great Work you can handle, DF07 Keep Gold or a small army: emergency purchase, DF08 Bank production at one turn left, DF01 Walls early, fortified garrison, ranged behind, DF02 Deter with walls and a visible army, DC01 Ally the friendly AI the turn you meet it, WX14 Buy the tiles next to their best district, EG07 A standing army is the price of peace, DC07 Avoid war with several teams at once.

**Natural counter-strategies (used against it):** RC38 Rush the rich but slow-to-build empire, EG02 Steal the treasury, not the production, RC03 Capital snipe, DC04 Second and third place gang up on the early leader, DC09 Stop the snowballer, LG03 Conquer the wonder owner instead of racing it, RC01 Two-mount opening rush (resource-gated). Conquest pays: rush them while the money engine is young (Mali is incredibly vulnerable to early rushes), take the capital for the doubled bounty, and knock the treasury below the goal.

**Must not copy:** Standing armies and perpetual war (3 Gold per extra unit): do not copy RC15 or RC11.

**AI experiments to run first:** EG03 (Buy the Settler instead of building it); DF07 (Keep Gold or a small army: emergency purchase); DF08 (Bank production at one turn left).

### Japan

**Rules in one line:** +1 Food per city, Brood Hall (+2 Food) from turn 1, +7% Food and Production per Japan city within 5 hexes; -10% of all yields per hex beyond 4 from the capital; goal: the Swarm (sacrifice 30 population).

**Resembles in Civ 6:** Kongo (dense cities that grow huge, housing is the engine); Germany (compact city clusters with strong production); Netherlands (rivers and tight spacing); Khmer (dense, defended core).

**Strategies that suit it:** WX04 Triangle opening: three cities 3-4 tiles apart, WX09 Compact versus dispersed empires, TD03 Dense cities with huge population, WX08 Infinite city sprawl (ICS) and how to spot it, WX07 Spam three Settlers when a close neighbour exists, WX03 Two scouts then two settlers for three cities, DF10 Settle behind a mountain or hilly choke, DF01 Walls early, fortified garrison, ranged behind, WX06 Building a few Settlers is the whole trick, WX17 Wide beats tall.

**Natural counter-strategies (used against it):** WX10 Close the chokepoint behind you, WX11 Surround the enemy with new cities, RC03 Capital snipe, RC18 Take the weak flat city first, RC05 Stage outside border, declare at the last second, DF05 Scouts as pickets: one per bordering rival. Wedge a city into the cluster, close the chokepoint behind their expansion, snipe the hub capital, and hit the edge cities where the distance loss weakens them.

**Must not copy:** Far-flung cities and conquest campaigns beyond 4 hexes (the yield loss makes them pay twice).

**AI experiments to run first:** WX04 (Triangle opening: three cities 3-4 tiles apart); WX07 (Spam three Settlers when a close neighbour exists); TD03 (Dense cities with huge population).

### Celts

**Rules in one line:** Hostile military loses up to 12 HP a turn on Celts land; Ranger and Marksman are hidden until they fire (strategic ambiguity); razes conquered cities (expand only by settling); never rushes; goal: Call the Wild (a camp in every city).

**Resembles in Civ 6:** Vietnam (woods and rainforest as a weapon, ambush defence); Mapuche (drive out aggressors, strong at home); Maya (strongest within six tiles of the capital); Brazil (terrain-dependent).

**Strategies that suit it:** TD05 Fight from woods and jungle you know, DF13 Fight on your own land, TD02 Home-ground strength bonus, DF01 Walls early, fortified garrison, ranged behind, DF10 Settle behind a mountain or hilly choke, DF11 Keep land visible so camps do not spawn, DF08 Bank production at one turn left, WX06 Building a few Settlers is the whole trick, WX07 Spam three Settlers when a close neighbour exists, WX17 Wide beats tall, WX01 Settle on turn 1, DC05 The first attacker usually loses, DC07 Avoid war with several teams at once, EG07 A standing army is the price of peace.

**Natural counter-strategies (used against it):** WX06 Building a few Settlers is the whole trick, WX13 Take the good spots around the rival, WX10 Close the chokepoint behind you, DF05 Scouts as pickets: one per bordering rival, DF20 Evict the squatter with a cheap ranged unit, DC05 The first attacker usually loses. Do not invade; out-expand and take the spots they need; keep sight on spotted hunters and mind last-seen markers; let them come out to you.

**Must not copy:** Conquest (they raze, so every capture is lost): no RC strategies except defensive counter-attacks on their own ground.

**AI experiments to run first:** DF13 (Fight on your own land); TD05 (Fight from woods and jungle you know); DF11 (Keep land visible so camps do not spawn).

### Germany

**Rules in one line:** Production converted to Gold or Science projects; -1 movement for all units; goal: an Engine in every city (at least 4), live.

**Resembles in Civ 6:** Germany (industrial production beating the neighbours on city development); Babylon (beelining the production tech and building tall industry); Assyria (slow, siege-heavy conquest); Mapuche (patient preparation for one decisive strike).

**Strategies that suit it:** SC02 Beeline the one key production or military tech, RC39 Slow siege creep under ranged cover, RC07 Escorted siege; buy siege in the captured city, RC29 Three siege per city plus bait units, RC17 Ranged whittles, melee finishes at 40-50%, RC27 Pair ranged shots: wound first, finish second, RC28 Ranged first, melee first only if ranged is much stronger, RC30 Cap attackers at six per tile, LG01 Convert the lead with the best unit tier, LG02 Time the war in the narrow window before the next wall tier, SC07 Great Library rush: only with the production lead, WX16 Outpost cities for single resources, TD03 Dense cities with huge population.

**Natural counter-strategies (used against it):** RC01 Two-mount opening rush (resource-gated), RC05 Stage outside border, declare at the last second, RC03 Capital snipe, RC20 Hit the title, wonder or tech leader, DC09 Stop the snowballer, RC31 Alpha strike: whoever hits hardest in the first turns wins. Early raids before the stack is built; take a city off a four-city Germany (the goal then breaks until the Engine is rebuilt); hit them while the -1 move leaves the siege train exposed.

**Must not copy:** Fast rushes and chases (their units are one hex slower).

**AI experiments to run first:** SC02 (Beeline the one key production or military tech); RC39 (Slow siege creep under ranged cover); LG02 (Time the war in the narrow window before the next wall tier).

### Byzantium

**Rules in one line:** Units squat on rivals' tiles, denying yields and gaining twice tile yields as Science, +5 Science per tech the landowner knows and Byzantium does not; Salvage Works; no city Science; ordinary buildings cost 120%; goal: train the Heir (2 VP, rival-capturable).

**Resembles in Civ 6:** Norway (Harald) (raid-farm economy: treat rivals as farms); Mongolia (the strength of being near rival cities); France (Catherine) (taking what the leader learned); Sumeria (camps and ruins as free rewards).

**Strategies that suit it:** RC32 Raid-farm economy (treat rivals as farms), RC23 Park a unit on the rival's best tile, SC05 Conquest as research, WX18 Scout early for ruins, RC13 Capture lone civilians (Settler sniping), RC36 Decoy a civilian to pull units out of position, RC24 Stay unmet: hide the army, SC04 Read the rival's research before you engage, RC20 Hit the title, wonder or tech leader, LG03 Conquer the wonder owner instead of racing it.

**Natural counter-strategies (used against it):** DF20 Evict the squatter with a cheap ranged unit, DF11 Keep land visible so camps do not spawn, DF14 Trench war: close every back door, DF05 Scouts as pickets: one per bordering rival. Evict them with a cheap ranged unit (the tile is worth more than the squatter), keep sight on every ring-1 tile, and tolerate the loss where eviction costs more than the tile; capture the Heir to take the 2 VP.

**Must not copy:** Science-building plans (no city Science accrual) and Settler-heavy sprawl at 120% building cost.

**AI experiments to run first:** RC23 (Park a unit on the rival's best tile); RC32 (Raid-farm economy (treat rivals as farms)); SC04 (Read the rival's research before you engage).

### Athens

**Rules in one line:** Ruins always give 30 Science; Listening House +6 Science; Markets +1 Gold instead of +3; goal: Share the Signal (every rival gains 3 free techs).

**Resembles in Civ 6:** Korea (a compact science engine plus a defensive unit tier to keep it); Babylon (a tech lead used as the weapon); Maya (small, well-defended, science-heavy); Arabia (science first, army second).

**Strategies that suit it:** SC01 Science cluster, then a defensive tech to keep the lead, SC04 Read the rival's research before you engage, SC06 Ask how science boosts the army, SC08 Three to four cities for a tall science game, WX18 Scout early for ruins, DF01 Walls early, fortified garrison, ranged behind, DF07 Keep Gold or a small army: emergency purchase, DF17 Befriend the likely attacker first, DC01 Ally the friendly AI the turn you meet it, LG08 Fortify the VP city that someone will come for, RC27 Pair ranged shots: wound first, finish second, RC09 Never fight a higher unit tier.

**Natural counter-strategies (used against it):** SC03 Hit the science-obsessed AI, RC20 Hit the title, wonder or tech leader, RC01 Two-mount opening rush (resource-gated), RC05 Stage outside border, declare at the last second, DC09 Stop the snowballer, DC04 Second and third place gang up on the early leader. The AI over-prioritises science, so strike the science leader's thin garrisons (a lot more vulnerable to military aggression); Markets give only +1 Gold so they cannot buy emergency defenders. Take the Most Advanced title and break the hold.

**Must not copy:** Gold-economy plans (Markets +1) and wide conquest.

**AI experiments to run first:** SC01 (Science cluster, then a defensive tech to keep the lead); DF07 (Keep Gold or a small army: emergency purchase); DC04 (Second and third place gang up on the early leader).

### Rock-paper-scissors summary (pillar 10)

| Archetype | Beats | Loses to |
|---|---|---|
| Rush and conquest (Mongols, Sparta) | beats thin science and gold empires, thin garrisons and sprawl | loses to Walls plus a scout picket plus an emergency buy, to coalitions, and to the 'first attacker loses' effect when two peers sit out |
| Defence and deterrence (Celts, Switzerland, Athens behind Walls) | beats unprepared rushes and attrition | loses VP to wide expansion and titles (Most Territory, Most Populous) and to patient siege |
| Wide dense growth (Japan) | outgrows defenders and gains titles | loses to a wedge city, a chokepoint, and a capital snipe |
| Siege and industry (Germany) | beats walled defenders | loses to early raids and fast strikes because every unit is one hex slower |
| Raid and squat (Byzantium) | beats sprawl and tech leaders by taking their yields | loses to cheap ranged units that evict squatters |
| Coalition against the leader | beats any VP holder in the 10-turn hold window | costs the members tempo: the first to hit a peer also pays |

The strongest cross-cutting rule from the sources: the first attacker loses tempo if a peer sits out (DC05), so the rush archetypes need a decisive edge, while the defence archetypes need a plan for slow VP growth, because Walls and scouts beat rushes but not wide expansion.

## Sources

- [CreaM's Guide to Civilization Multiplayer (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2493352644)
- [Zigzagzigal's Guides: Sumeria (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1905022232)
- [Domination Victory Tips (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1229036179)
- [Zigzagzigal's Guides: Macedon (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1902563180)
- [Zigzagzigal's Guides: Zulu (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1983949096)
- [Domination on Immortal/Deity: a noob's guide (CivFanatics, via web.archive.org)](https://forums.civfanatics.com/threads/domination-on-immortal-deity-a-noobs-guide.547630/)
- [Basic Rush: take out an enemy early on (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=181479636)
- [Civilization V Military Strategy Guide (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=749815003)
- [Reddit: an early game strategy (r/civ)](https://www.reddit.com/r/civ/comments/58tbok/)
- [Aggressive Strategies to Dominate your Opponent(s) in Multiplayer (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=517454225)
- [Reddit: 274 AI-only Civ 5 games tier list (r/civ)](https://www.reddit.com/r/civ/comments/1hjdenn/)
- [Zigzagzigal's Guides: Maya (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2104285340)
- [Reddit: melee versus ranged first, with the math (r/CivVI)](https://www.reddit.com/r/civ/comments/1tsjxhj/)
- [Reddit: war in the mid game (r/civ)](https://www.reddit.com/r/civ/comments/1gvmu1q/)
- [Zany's Comprehensive Civ 6 Guide (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=926983840)
- [Reddit: Harald as a raid economy (r/civ)](https://www.reddit.com/r/civ/comments/guu28v/)
- [Reddit: keeping enemies as pets (r/civ)](https://www.reddit.com/r/civ/comments/14s4tah/)
- [Reddit: wiping a small neighbour (r/CivVI)](https://www.reddit.com/r/civ/comments/1g18yfm/)
- [Reddit: the mistake beginners make (r/CivVI)](https://www.reddit.com/r/civ/comments/1pi75k8/)
- [Reddit: Deity AI tricks (r/CivVI)](https://www.reddit.com/r/civ/comments/1ozb0ak/)
- [Zigzagzigal's Guides: Mali (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1918595516)
- [How many military units should I build? (CivFanatics, via web.archive.org)](https://forums.civfanatics.com/threads/how-many-military-units-should-i-build.688012/)
- [Guide to civ5 multiplayer (CivFanatics, via web.archive.org)](https://forums.civfanatics.com/threads/guide-to-civ5-multiplayer.441979/)
- [Beginners guides: how to counter a surprise war (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1143191647)
- [Reddit: keep units in the queue at one turn left (r/civ)](https://www.reddit.com/r/civ/comments/vkgbvq/)
- [Reddit: small tips that helped immeasurably (r/civ)](https://www.reddit.com/r/civ/comments/7rix87/)
- [Reddit: non-obvious tips (r/civ)](https://www.reddit.com/r/civ/comments/11tllfm/)
- [Diplomacy & Alliances in Civ VI (Steam)](https://steamcommunity.com/sharedfiles/filedetails/?id=792333308)
- [Reddit: what am I doing wrong in my early game (r/civ)](https://www.reddit.com/r/civ/comments/xzho6o/)
- [Reddit: strategies I use to beat Deity (r/CivVI)](https://www.reddit.com/r/civ/comments/1dl8s0l/)
- [Reddit: Deity AI opening weaknesses (r/CivVI)](https://www.reddit.com/r/civ/comments/1ff1y71/)
- [Reddit: 10 cities by turn 100 (r/CivVI)](https://www.reddit.com/r/civ/comments/19endem/)
- [Reddit: how to compete in Civ 5 FFA multiplayer (r/civ5)](https://www.reddit.com/r/civ/comments/avs26i/)
- [Zigzagzigal's Guides: Korea (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1928634264)
- [Reddit: going wide is always better (r/civ)](https://www.reddit.com/r/civ/comments/57axu6/)
- [Advanced Multiplayer Strategies (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=229432166)
- [Reddit: gold beats production (r/civ)](https://www.reddit.com/r/civ/comments/1ecoobt/)
- [Reddit: the Feudalism wave (r/civ)](https://www.reddit.com/r/civ/comments/5aneb7/)
- [Reddit: early armies in Civ 6 (r/civ)](https://www.reddit.com/r/civ/comments/593yja/)
- [Zigzagzigal's Guides: Babylon (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2387413001)
- [Reddit: three lines of AI code (r/civ)](https://www.reddit.com/r/civ/comments/oe4yqg/)
- [Zigzagzigal's Guides: Australia (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2542999669)
- [How to Rush the Great Library (Steam, Civ 5)](https://steamcommunity.com/sharedfiles/filedetails/?id=957628210)
- [Reddit: limited wars (r/CivVI)](https://www.reddit.com/r/civ/comments/1j1hxmd/)
- [Zigzagzigal's Guides: Kongo (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2533397288)
- [Reddit: barbarian caravels (r/civ)](https://www.reddit.com/r/civ/comments/7wy94v/)
- [Reddit: conquering a friend's cities (r/civ)](https://www.reddit.com/r/civ/comments/8idx7j/)
- [Reddit: the most broken setups in Civ 6 (r/civ)](https://www.reddit.com/r/civ/comments/pmsv35/)
- [Reddit: don't research unless you have to (r/civ)](https://www.reddit.com/r/civ/comments/kqw33u/)
- [Zigzagzigal's Guides: Greece (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=1980899859)
- [Zigzagzigal's Guides: Germany (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2536738340)
- [Zigzagzigal's Guides: Vietnam (GS) (Steam, Civ 6)](https://steamcommunity.com/sharedfiles/filedetails/?id=2589592137)
