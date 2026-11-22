# Byzantium: Fable's research, candidate kits and recommendation (2026-11-06)

Answers `brief.md`. Rules checked against `.claude/skills/legatum-rules/references/game-rules.md` (§3 economy and buying, §4 walls, siege and capture, §5 diplomacy and Companies, §6 victory, §8 factions, §9 systems that do not exist) and `crates/core/src/data.rs::levy_cap`.

## Ranking

| # | Kit | Rule break (one sentence) | Drawback | Goal | New code | Verdict |
|---|---|---|---|---|---|---|
| 1 | **The Theodosian Walls** | Byzantium may buy Walls, Castles and Star Forts with Gold; nobody else can. | Thin levies, tightened: at most one armed unit per city. | The Heir | Relax one buy blocklist; one AI buy candidate | **Ship.** The one rule everybody feels (Walls are Production-only) broken for one faction; small to build; the AI already treats Walls as the garrison |
| 2 | Impregnable Walls | Siege does not work against Theodosian Walls before the Industrial era: only cannon breached them. | Thin levies | The Heir | `siege_cover` keyed on the owner | Strong second; the escalation if kit 1 reads too mild. Risk: cities nobody can take before T130 |
| 3 | Greek Fire | A walled Byzantine city's strike hits every enemy unit beside it. | Thin levies | The Heir | City-strike targeting, forecast | Good fortress flavour; narrower than 1 or 2 because attackers already stand off at range 2 |
| 4 | Belisarius's Restoration | Cities Byzantium captures suffer no resistance and lose no population. | Thin levies | The Heir | Capture path flag | The only kit that pushes Byzantium into conquest; the identity is Justinian's, not the empire's thousand years |
| 5 | Mercenary Companies, taught | Today's rule unchanged; fix the AI's hiring. | Thin levies | The Heir | AI only | The data says the rule can work (deleting it is +8.8 pp because the AI wastes Gold on it); the designer dislikes it, and it funds rivals |
| 6 | The Thirty Years' Peace | Peace made with Byzantium holds 30 turns, not 10, for both sides. | Thin levies | The Heir | One constant keyed on the pair | Clean and historical, but it only acts when a rival agrees to Peace |
| 7 | The Bureau of Barbarians | A war declared on Byzantium takes effect next turn, never the same turn. | Thin levies | The Heir | Surprise-war check | Pairs well with kit 1; on its own it is a surprise the attacker reads as a bug |
| 8 | The Themes | Every walled Byzantine city keeps one free Spearman-line militia that never leaves its territory. | None needed (the militia is the cap) | The Heir | New unit state (bound to a city) | New mechanic for one faction; the designer rejects those |
| 9 | Constantinople does not fall first | A walled Byzantine capital cannot be captured while Byzantium owns another city. | Thin levies | The Heir | Capture guard | A capture veto; he declined capital protection on 2026-11-01 |

Not a candidate: the sketch's "pay Gold to end a war at once, even when the enemy doesn't want Peace". That is bought peace, removed 2026-10-22 after the designer said forced peace "feels off" (2026-10-17): it vetoes another player's war, and his rule since then is that a break acts on the map by Byzantium's own hand. A price that grows per use does not change what it is. "Set them against each other" (subsidised wars) is rejected on record too ("they can't actually force someone to move an army"). Both stay out.

**Delete or ship?** Ship kit 1. Deletion beats shipping Companies again, but not kit 1: kit 1 is a few dozen lines of core change plus one AI buy candidate, keeps the two Byzantine rules that already pay in the census (the Tagmatic Soldier, −2.2 pp when off; the Heir, −1.95 pp), and gives the game its only medieval Mediterranean faction, the one whose landmark (Hagia Sophia) is already a world wonder on the tech tree. Nine factions also let the eight seats rotate. If the designer wants no defensive identity at all, delete rather than ship another Gold-diplomacy rule.

## Research in brief

**History (what made it survive, what made it distinct).**
- The land walls, 413 (prefect Anthemius): a double line and moat that held against the Avars and Persians in 626, the Arabs in 674–678 and 717–718, Bulgars and Rus; in 1204 the Crusaders came over the sea walls on the Golden Horn, and only Mehmed's cannon broke the land walls on 29 May 1453 ([The Collector](https://www.thecollector.com/what-are-the-theodosian-walls-of-constantinople/), [World History Encyclopedia, 1453](https://www.worldhistory.org/article/1180/the-fall-of-constantinople-in-1453/), [Medievalists.net](https://www.medievalists.net/2025/05/medieval-sieges-constantinople/)). In 447 an earthquake threw down 57 towers with Attila approaching; the prefect and the circus factions rebuilt the walls in about two months ([447 Constantinople earthquake](https://en.wikipedia.org/wiki/447_Constantinople_earthquake), [Livius](https://www.livius.org/articles/place/constantinople-istanbul/constantinople-photos/constantinople-theodosian-walls)). The state paid for walls the way other states paid for armies: that is the hook kit 1 is built on.
- Diplomacy by Gold: subsidies to Attila, the Avars and Persia; Constantine VII's *De Administrando Imperio* (948–952) opens with eleven chapters on keeping the Pechenegs friendly so that neither the Rus nor the Magyars can attack ([Szeged, Pechenegs in the imperial space](https://ojs.bibl.u-szeged.hu/index.php/chronica/article/view/47843), [GlobalSecurity, Byzantine diplomacy](https://www.globalsecurity.org/military/world/europe/byz-diplomacy.htm)). Long treaties were bought: the Fifty-Year Peace with Persia (562) and the thirty-year treaty with Bulgaria (927) are the models for kit 6.
- The army: themes (soldier-farmers settled on the land from the 7th century, split small so no general could revolt) and the tagmata, Constantine V's professional regiments at the capital; the Varangian Guard from the 6,000 Rus Basil II received in 988; growing reliance on foreign mercenaries after the themes collapsed in the 11th century, ending in the Catalan Company (hired 1303, ravaging Thrace by 1305) ([Wikipedia, Byzantine army](https://en.wikipedia.org/wiki/Byzantine_army)). Thin levies and kits 5 and 8 read from this.
- Greek fire (Kallinikos, about 672), a state secret that broke both Arab sieges ([Wikipedia, Greek fire](https://en.wikipedia.org/wiki/Greek_fire)). The solidus held its weight for about 700 years and was debased only from the 11th century ([Wikipedia, Solidus](https://en.wikipedia.org/wiki/Solidus_(coin)), [World History Encyclopedia, Byzantine coinage](https://www.worldhistory.org/Byzantine_Coinage/)). Belisarius took Africa (533–534) and Italy (from 535) with small armies for Justinian; the reconquest could not be held (kit 4).
- Why it fell: Manzikert 1071 lost Anatolia, the recruiting base; 1204; trade concessions to Venice and Genoa hollowed the customs; civil wars; the Ottomans ([Wikipedia, Decline](https://en.wikipedia.org/wiki/Decline_of_the_Byzantine_Empire), [EBSCO, Manzikert](https://www.ebsco.com/research-starters/military-history-and-science/battle-manzikert)). The empire lost provinces for a thousand years and kept the capital: "the walls are the garrison, the army is the expedition" is the whole identity in one line.

**Other games.**
- Civ 5 Byzantium (extra belief, Cataphract, Dromon): rated weak and boring, a single contested roll (founding a religion) with nothing after T100 ([CivFanatics](https://forums.civfanatics.com/threads/does-byzantium-need-a-boost.514169/page-5)). Lesson: a bonus that fires only if a race is won is not an identity.
- Civ 6 Basil II (Taxis, Tagma, Hippodrome): a religious-cavalry snowball, polarising, Byzantine in name only ([Fandom](https://civilization.fandom.com/wiki/Byzantine_(Civ6)), [CivFanatics](https://forums.civfanatics.com/threads/apparently-popular-opinion-has-turned-against-basil-ii.673099/latest)). Lesson: the defensive-diplomatic empire reskinned as a crusader splits players.
- Humankind: the Byzantines' emblematic unit is the Varangian Guard, chosen over Cataphracts because it links to another culture ([CivFanatics dev thread](https://forums.civfanatics.com/threads/humankind-byzantines-discussion-thread.657474)). CK3 *Roads to Power*: the empire's problem (ambitious theme governors) is the game, "Very Positive" ([GGRecon](https://www.ggrecon.com/reviews/crusader-kings-3-roads-to-power-review)). EU4 1444: a deliberate fight for survival, loved only after the ideas and missions were reworked to pay off ([Paradox dev diary](https://admin-forum.paradoxplaza.com/forum/developer-diary/europa-universalis-iv-development-diary-26th-of-september-2023-byzantium.1600100/page-17)). Lesson: where Byzantium works, the empire's problem is the gameplay, and survival pays.
- AoE2 Byzantines: buildings +10/20/30/40% HP by age, cheap counter units, Greek fire, Logistica: a generalist that never loses cheaply. AoE4 Byzantines: olive oil buys other civilizations' mercenaries from a Mercenary House, cisterns, Greek fire ([Age of Empires](https://ageofempires.com/games/age-of-empires-iv/civilizations/byzantines), [AoE forums](https://forums.ageofempires.com/t/byzantines-need-buff-in-early-game/247843)). Lesson: a dedicated currency for foreign units is concrete but makes a late civ; this is today's Companies.
- TI4: the Xxcha (peaceful, political cancels), Hacan (trade), Mentak (raiding); each is legible from a sentence. Stellaris admitted tributaries were rarely better than conquest until redesigned ([Paradox](https://admin-forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-247-new-ways-to-rule.1516799)). Lesson: tribute and buy-peace mechanics read as a penalty unless the payer gets something concrete on the map.

## Shared pieces (every kit unless stated)

- **The Heir (held 2 VP)** stays: it pays (−1.95 pp when off), the designer called it fine (2026-10-22), and a stealable 2 VP civilian is exactly the "purple-born" succession the empire fought civil wars over. It is the one goal that gives rivals a lever against a fortress faction.
- **Thin levies, tightened to one armed unit per city (`data::LEVY_EXTRA` 1 → 0).** Today's cities + 1 is dead only because Hard's lean war army is cities + 1 (`army_war_extra` 1), so the cap never binds the AI (learnings 2026-11-06). At cities + 0 it binds every AI and every human from the second city, and the ablation can measure it. The opening is Settler, Scout and Soldier with no city, and losing a city never disbands, so the first Soldier is legal; the price is that the capital gets no second defender until city two, which is the fortress's honest cost (and the designer's rule: rushes are final, the counter is scouting and Walls). Fallback if it reads too harsh: keep cities + 1 and accept it as human-binding, as he did for Sparta's desertion on 2026-11-06.
- **The Tagmatic Soldier** stays where no other unique piece replaces it (pays, −2.2 pp).

## 1. The Theodosian Walls (ship)

**Sheet.** "Byzantium may buy Walls, Castles and Star Forts with Gold; no other empire can." Drawback: "Byzantium fields at most one armed unit per city." Unique: the Tagmatic Soldier. Goal: the Heir.

**History.** The walls were the treasury's army: Theodosius II's prefect built them in 413, and in 447, with Attila on the Danube, the city paid the circus factions to rebuild 57 fallen towers in two months. Justinian's Procopius lists hundreds of fortresses the state paid for. The field army was small and professional; the provinces were held by stone bought with gold.

**When it pays.** Walls cost 100 Production, so 200 Gold at the ordinary 200% rate (§3 Buying). Nobody has that before about T30–40; from then on every new Byzantine city can be walled the turn a rival's army is sighted, and all game the Gold reserve is an instant fortification anywhere. It never helps at T18, so the designer's final early rushes and "no special early escape for Byzantium" (2026-10-17) hold.

**How it changes the competition.** Byzantium's decision every turn is Gold into walls or Gold into anything else: one purchase per city per turn, and 200 Gold a wall means a six-city empire that walls everything has bought nothing else. Rivals' lever is tempo and information: take a new city before the treasury refills (the treasury is public arithmetic from the Largest Economy crown and what it just bought), bring Siege (a Gold upgrade that ignores the walls, §4), and go for the public Heir while the thin army is at the front. It pushes Byzantium further into its style because the cap means it cannot answer a sighted army with units, only with walls, so it settles behind walls and sends its whole small army out while the walls hold home (Heraclius 626).

**Systems used.** `BuyItem` and the Walls-line blocklist (`economy::is_unbuyable`, `buy_check`; §3 "Walls, Castle, Star Fort and the Swiss Alpine Redoubt cannot be bought"), one purchase per city per turn, Masonry for Walls, Walls as garrison (`walls_garrison`), the public treasury. New: one exception in the blocklist for the Byzantium faction; `data::LEVY_EXTRA` 0; a "buy Walls" candidate in `production.rs::plan_buy` beside `buy_frontline_defender`, priced by the same defence-per-Gold the Walls garrison gene already uses.

**Distinct from Switzerland.** Switzerland earns Gold (tolls, interest), hoards it (5,000-Gold goal), buys wonders and Great Works at 175%, and cannot buy its Redoubt. Byzantium earns Gold like everyone, spends it on the one thing the Swiss cannot, and is poor in units where the Swiss are expensive in units. The Swiss hoard is a target; Byzantine Gold is stone. Not Sparta (war), Mongols (mobility, no early Walls), Celts (sack), Germany (Production), Athens (Science), India (density), Aztecs (kills).

**AI.** The Hard default already counts Walls as the garrison for every faction and buys a frontline defender when a city is threatened; for Byzantium the buy candidate is Walls instead of a unit, valued by the defence it adds over the units the cap forbids. Rivals need nothing new: their assault forecast reads the wall tier (`city_assault_succeeds`), their Heir hunt exists, and the levy edge odds already price a Byzantium at its cap. Telemetry: walls bought per game, Gold spent on walls, cities lost before and after walls.

**Strongest case against.** It is a fortress kit for a game whose core loop is expansion by settling or conquest, and the designer wants no zero-conquest winners; kit 1 wins by economy, Hold and Spend cards, the Heir and a wonder more than by capitals. The answer is that Switzerland has the same tension and he wants it to conquer anyway, and that thin levies send the whole army out because the walls hold home. The second risk is that it is "defensive like forced peace": it is not a veto on any order, it costs Byzantium Gold every time, and the counter (tempo, Siege, the Heir) is a decision the rival makes.

## 2. Impregnable Walls

**Sheet.** "Theodosian Walls replace Walls: Siege does not work against them before the Industrial era, because only cannon breached them." Thin levies, the Heir.

**History.** Rams, towers and mines failed at the land walls for a thousand years; Mehmed's bombards of 1453 did in 53 days what no earlier army could.

**When it pays.** From the first Walls (about T40) until rivals reach Industrial units (median about T130), then it is an ordinary wall. Bombers (Modern) ignore walls anyway.

**Competition.** Attackers lose their tool against Byzantine cities for a hundred turns: they must bring twice the shooters (walls halve damage and keep full strength, §4), take unwalled new cities first, or wait for cannon. Byzantium's decision is which cities get Walls first (100 Production each cannot be bought here). Pushes into the style because every walled city is safe from everything but mass, so Byzantium walls early and settles outward behind them.

**Systems used.** `data::siege_cover`, `combat::city_defense_covered`, `wall_damage_percent`, `built_in_siege` from Industrial. New: the cover check reads the city owner's faction and the attacker's era; the forecast line "Siege does not cover Theodosian Walls". Cost 100 like Walls (the Swiss Redoubt's +20 is for +5 strength; this piece is stronger, so 120 is a knob).

**Distinct.** The Swiss Redoubt is +5 strength for +20 Production; this is a different kind of wall, not a bigger number. Nothing else touches Siege.

**AI.** Rivals' assault maths reads the cover, so they will mass shooters or go elsewhere; Byzantium's AI needs nothing beyond valuing Walls, which it does. The Mongols (no Siege before Medieval anyway) are unaffected.

**Case against.** It may make a walled Byzantium unconquerable before T130 in AI hands, exactly when the designer wants wars to take cities; the census will say (cities lost per Byzantine seat, failed assaults). If so it is the escalation for kit 1, not a kit of its own.

## 3. Greek Fire

**Sheet.** "A walled Byzantine city's strike hits every enemy unit beside it." Thin levies, the Heir.

**History.** Kallinikos's fire, sprayed from siphons, broke the Arab sieges of 674–678 and 717–718 and the Rus in 941; the formula was a state secret for seven centuries.

**When it pays.** From the first Walls to the end; strongest in the pre-Modern eras when melee units must stand adjacent to capture.

**Competition.** Attackers keep melee back until the city is at 0 HP and shoot from range 2 (the triangle already says so), so the rule punishes the careless and the Mongol charge, not the disciplined siege. Byzantium's decision is unchanged from any Walls faction. It pushes into the fortress because the walls kill as well as hold.

**Systems used.** City strike (walled cities only, §4), the ranged phase, `forecast.rs`, AI exposure maths (`tactics.rs::city_fires`). New: ring targeting for one faction's cities, a fire-ring animation.

**Distinct.** No other faction changes how a city fights. Not the Swiss Redoubt (strength), not Celtic Ambushers (unit range).

**AI.** Rivals' assault staging must count the ring; Byzantium's AI needs nothing.

**Case against.** Narrow: good attackers rarely stand two melee units beside a walled city, so the rule moves few games and the ablation may call it dead. Best as the unique piece under kit 1 if the Tagmatic Soldier is ever cut.

## 4. Belisarius's Restoration

**Sheet.** "Cities Byzantium captures suffer no resistance and lose no population: the Romans have returned." Thin levies, the Heir.

**History.** Belisarius took Carthage in 533 and Rome in 536 with a few thousand men, welcomed as the lawful empire returning; Nikephoros Phokas retook Crete (961) and Antioch (969); the Komnenoi recovered the coasts after Manzikert. The empire's story is reconquest with small armies as much as walls.

**When it pays.** The first capture (AI median first war about T60–80) and every capture after: a taken city works at once instead of six turns of zero output and −1 population (§4 Capture).

**Competition.** Byzantium's decision becomes where to spend its few units for a capture, since each capture is worth more to it than to anyone else and the cap keeps the army small. Rivals' lever: walls (the small army cannot grind them), retaking the city (Byzantium holds it with one unit), the Heir. Pushes into the style of the short, surgical expedition.

**Systems used.** `rules/battle.rs::capture_city` (resistance turns, the population drop). New: a faction branch on capture. Capital VP and Fight cards unchanged.

**Distinct.** Sparta heals on capture, the Celts burn, the Aztecs grow from kills; this is "the city is yours at once". Not Swiss at all.

**AI.** Capture valuation already counts a city's output over 40 turns; remove the resistance discount for Byzantium and the Campaign plan prices it higher. Thin levies mean the AI must pick unwalled targets, which the Sparta guidance already encodes.

**Case against.** It is a conquest buff on a faction famous for outliving conquerors, and with thin levies it may never fire in AI hands (the census would call it dead). It is the kit to build if the designer wants Byzantium to feel like a conqueror; it does not use the Theodosian Walls he liked.

## 5. Mercenary Companies, taught

**Sheet.** Unchanged: "Byzantium hires any armed unit a met empire at Peace can field, paying its Production cost in Gold to that empire; the Company serves 30 turns and sits outside the levies." Thin levies, the Heir.

**History.** The Varangians, the Komnenian mercenary regiments, the Catalan Company.

**When it pays.** First hire median T64 in 85% of AI seats today.

**Competition and the data.** The rule off is +8.8 pp (11.3% → 20.2%), which means the AI's hires lose it games: it spends Gold that enriches a seller and gets a unit that leaves. The counterplay is sound on paper (outlast the countdown, bleed the 2-Gold upkeep, shut the shop by declaring war). The fix is AI: hire only for a priced campaign that ends inside 30 turns, never from the leader, never below ten turns of Gold runway; and make the seller's Gold a term in rivals' valuation. That is the `ai/company.rs` planner, not a rule.

**Case against.** The designer called the rule weird and forced, it is the most complex sheet in the game (seller, contract, upkeep, barbarian turning), and it funds rivals by design. A simplified "buy at 100%, the unit leaves after 30 turns" with no seller overlaps Switzerland's 175% buying, which is why mercenary-only armies were rejected on 2026-10-10. Keep this only if he prefers a paymaster to a fortress; otherwise kit 1.

## 6. The Thirty Years' Peace

**Sheet.** "Peace made with Byzantium holds for 30 turns, not 10, for both sides." Thin levies, the Heir.

**History.** The Fifty-Year Peace with Persia (562), bought with 30,000 solidi a year; the thirty-year treaty with Bulgaria (927); Byzantium bought long peaces and kept them.

**When it pays.** Every war that ends in Peace: Byzantium gets 30 turns to wall and settle, and the rival gets 30 turns with that front closed.

**Competition.** A rival deciding on Peace with Byzantium knows it locks that front for 30 turns, which also protects its flank while it fights elsewhere; a rival who wants Byzantium's cities must refuse Peace and finish the war. Byzantium's own offensives are bound the same way. Pushes into the style of the long, bought calm.

**Systems used.** `PEACE_TRUCE_TURNS` (§5), already priced by the AI's plan layer. New: a per-pair truce length.

**Distinct.** Nothing else touches the truce.

**Case against.** It acts only when a rival accepts Peace, which is the designer's "depends on another player choosing" test, even if softer than a veto. And it is invisible in most games (how often is a 10-turn truce the thing that mattered?). A companion rule, not a core.

## 7. The Bureau of Barbarians

**Sheet.** "A war declared on Byzantium takes effect next turn, never the same turn." Thin levies, the Heir.

**History.** The Bureau of Barbarians and the postal agents watched every embassy; Constantinople was the best-informed court in the world, and it used the warning to buy walls and allies in time (447).

**When it pays.** Every war declared on it: one planning turn to buy a defender, move the army, or (with kit 1) buy Walls.

**Competition.** Rivals lose the surprise turn against Byzantium only; they still stage at the border and strike the turn after. With kit 1 it is the 447 story in rules: see the army, buy the wall, the turn before the war. Byzantium's decision is what to do with the warning.

**Systems used.** Surprise war (§5 "War"), `declare_and_strike`, the AI's declaration forecast. New: a faction check in the declaration's effect turn.

**Case against.** The designer's trap rule: a human declaring war on Byzantium expects the same-turn effect, and "it didn't happen" reads as a bug unless the declare dialog names it. One turn is also too little against a Mongol rush. A companion to kit 1 at most.

## 8. The Themes

**Sheet.** "Every walled Byzantine city keeps one free Spearman-line militia that never leaves its territory." No separate drawback: the militia is bound to its land, and Byzantium's field army is only what it trains beyond that.

**History.** Soldier-farmers paid in land, raised by the theme's strategos and kept close to home so no general could march on the capital.

**When it pays.** From the first Walls; every walled city is garrisoned for free.

**Competition.** Rivals face a garrison in every walled city and an empty field; Byzantium's decision is Walls (which now also mean a soldier) against anything else. Pushes into the fortress.

**Systems used.** Walls, upkeep (§3), garrison rules. New: a unit state "bound to city X" with movement refused outside its territory, spawning and replacement rules, art. That is a mechanic for one faction, which the designer rejects (Mongol contracts, traffic tolls).

**Case against.** The new mechanic, and overlap with "Walls count as garrison", which already gives every faction most of this for free.

## 9. Constantinople does not fall first

**Sheet.** "A walled Byzantine capital cannot be captured while Byzantium owns another city." Thin levies, the Heir.

**History.** 1453: everything else was taken first.

**Competition.** Attackers take the provinces first; the capital is the last city.

**Case against.** It is a capture veto, the genus the designer rejected with forced peace, and on 2026-11-01 he removed every capital protection ("all cities razable", "don't re-propose capital protection"); the +1 capital VP makes a capital the target he wants late wars to have. Listed so it is not re-proposed.

## What I would ship, and the build

Kit 1, with kit 7's one-turn warning considered as a companion only after kit 1 is measured on its own. Build: the buy blocklist exception, `LEVY_EXTRA` 0, the AI buy candidate, the sheet, Guide and forecast text ("Byzantium may buy Walls"), `game-rules.md` §3 and §8, then `tools/sim/ablate.sh byzantium` on the two rules and the Heir. Success is both rules moving the share in the right direction and Byzantium between 8% and 15% a seat; if walls bought per game is under one, the AI candidate is wrong before the rule is.
