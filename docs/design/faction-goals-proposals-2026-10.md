# Faction goal proposals (working doc, 2026-10-06)

Design research and proposals only. Nothing here is decided until the user picks it; decisions go to the design doc's decisions log and the `legatum-factions` skill. Numbers are Standard speed unless marked (Quick is half), and every "N" is a tuning knob, not a measured value (balance waits for a competent AI).

**The test (user's words):** in theme; inconvenient, "not something I would commonly do in any game"; a strategic tradeoff, not a chore; no map luck; not templated (goals differ in kind); visible and counterable; scales from 1v1 to 8 players.

**Already decided:** Switzerland spend 1,000 Gold on a buy-only project; Sparta N military deaths in battles they fight; Japan 8 cities in one live cluster; Celts raze 2 cities taken from rivals.

---

## 1. Research findings: what makes a faction goal thematic and tense

1. **The scoring engine is the identity, and it can trap you.** In Root each faction scores by doing its own thing: the Marquise builds, the Woodland Alliance spreads sympathy, the Eyrie score roosts each Evening. The Eyrie's Decree must keep growing, and failing it throws them into turmoil, losing points. So the identity carries its own risk. *Lesson:* on-identity scoring is Root's main track, which in Legatum is titles and milestones. A faction goal should add the tension that plain play lacks. Sources: https://root.fandom.com/wiki/Root, https://root.fandom.com/wiki/Eyrie_Dynasties, https://en.wikipedia.org/wiki/Root_(board_game)

2. **Swap points for a board position everyone can deny.** Root's dominance cards let a player give up their score track for a condition such as ruling particular clearings. The condition is public, so the whole table piles in to break it. The Vagabond can't win this way; it joins a coalition instead. *Lesson:* a public, positional, losable goal turns rivals into counterplay. That is Japan's live cluster. Sources: https://root.fandom.com/wiki/Root, https://root.fandom.com/wiki/Vagabond

3. **Status-quo and denial wins.** In Dune (1979/2019), the Guild wins if nobody has won by turn 10, and the Fremen win at turn 10 if certain strongholds are empty or Fremen-held. These factions win by preventing things, so rivals must act to stop the clock. *Lesson:* restraint and denial can be real goals, provided someone can break them. Source: https://en.wikipedia.org/wiki/Dune_(board_game)

4. **Prediction makes the table paranoid, but needs a crowd.** Dune's Bene Gesserit secretly predict who will win and on which turn, and win instead if they're right. It works because several players could win. With one opponent it collapses into "predict yourself" or a freebie. Source: https://en.wikipedia.org/wiki/Dune_(board_game)

5. **Faction-shaped thresholds are often freebies.** In TI4 Prophecy of Kings, each faction's commander unlocks on its own condition. Many simply say "do your thing more": Hacan hold 10 trade goods, L1Z1X field 4 dreadnoughts, Nekro own 3 technologies. Players reach these without trying. The memorable ones force a detour or a relationship: the Winnu must fight at Mecatol Rex, the Empyrean must be neighbours with every player, and the Mahact must hold two other factions' command tokens. *Lesson:* this matches our Byzantium 98% freebie. Sources: https://twilight-imperium.fandom.com/wiki/Leaders, https://github.com/AsyncTI4/TI4_map_generator_bot/blob/master/src/main/resources/data/leaders/pok.json

6. **Save, then burn.** TI4's spend objectives (Erect a Monument: spend 8 resources; Centralize Galactic Trade: spend 10 trade goods) cost tempo you can see coming. Cthulhu Wars goes further for one faction: the Crawling Chaos must pay 4 Power, and later 6, as whole actions. Its wiki notes enemies can see it saving. *Lesson:* this is exactly the Switzerland' Endowment. One faction can own it; three factions doing it reads as a template. Sources: https://twilight-imperium.fandom.com/wiki/Objectives, https://cthulhuwars.fandom.com/wiki/Crawling_Chaos

7. **Goals priced in a loss you'd never choose.** TI4's Become a Martyr requires losing a planet in a home system. Cthulhu Wars' Black Goat must eliminate 2 of its own Cultists, and the Yellow Sign must hand another player 3 Doom. In Cosmic Encounter, the Masochist wins by losing all its ships. *Lesson:* self-harm is the purest form of "inconvenient". Sparta's own dead counting, and the Athens giving up their fog below, use it. Sources: https://twilight-imperium.fandom.com/wiki/Objectives, https://cthulhuwars.fandom.com/wiki/Black_Goat, https://cthulhuwars.fandom.com/wiki/Yellow_Sign, https://en.wikipedia.org/wiki/Cosmic_Encounter

8. **Scale with the player count explicitly.** Black Goat must share areas with all enemies and gains 1 Power per enemy. The Discordant Stars Vaden commander needs control tokens "for every 2 players". The Empyrean's "neighbours with all" grows harder with more players. *Lesson:* goals that name "every rival" break at one end of 1v1 to 8. Count your own things, or cap rival counts. Sources: https://cthulhuwars.fandom.com/wiki/Black_Goat, https://github.com/AsyncTI4/TI4_map_generator_bot/blob/master/src/main/resources/data/leaders/ds.json

9. **Location-bound goals are counterable; abstract ones aren't.** The Yellow Sign must desecrate areas marked with particular glyphs, and the Cthulhu Wars wiki calls the faction blockable because its destinations are known. Great Cthulhu's spellbooks need kills in battle, which an opponent can refuse. *Lesson:* tie each goal to a place, a city or a battle, so a rival has a lever. Sources: https://cthulhuwars.fandom.com/wiki/Yellow_Sign, https://cthulhuwars.fandom.com/wiki/Great_Cthulhu

10. **Interlocking goals build in counters.** In Vast, each role has its own path to victory, and each is someone else's threat: the Knight hunts the Dragon, the Goblins hunt the Knight, and the Cave wants to collapse on everyone. *Lesson:* a goal that points at rivals' cities or armies (Celts, Germany, Byzantium below) gives those rivals a reason and a means to resist. Source: https://ledergames.com/products/vast-the-crystal-caverns

11. **Hidden goals add tension but cost readability.** TI4 secret objectives and Oath's Vision cards are private. Oath's reviewers note that its differing win conditions make the board hard to read. *Lesson:* keep every goal public, with a progress readout (pillars 4 and 5). Sources: https://twilight-imperium.fandom.com/wiki/Objectives, https://en.wikipedia.org/wiki/Oath:_Chronicles_of_Empire_and_Exile

12. **Different VP sources per side, same race.** In Here I Stand every power scores in its own ways (piracy, New World exploration, debates, key cities) toward one shared VP track. Pax Pamir scores whoever's coalition dominates, and players switch loyalty to ride it. *Lesson:* asymmetric goals sit well beside shared titles, as long as the totals stay comparable (see §4). Sources: https://en.wikipedia.org/wiki/Here_I_Stand_(boardgame), https://en.wikipedia.org/wiki/Pax_Pamir

**Summary.** The tense goals cost something visible (patterns 6 and 7), sit at a place or target rivals can reach (9 and 10), and are public and losable (2 and 11). They aren't "do your thing more" (5). They count your own things, or cap the rival counts, so they scale (8).

---

## 2. Proposals

Each proposal lists: **Rule** · **Theme** · **Cost** (the inconvenience) · **Seen and countered** · **Feasibility** (what's new) · **Map luck** · **Scaling**.

### Switzerland: tax-and-turtle bankers

**T1. Endowment (decided).** **Rule:** spend 1,000 Gold (500 Quick) in one purchase on the Endowment, a goal project that can only be bought.
- **Theme:** a banking house proves itself by endowing something grand.
- **Cost:** 1,000 Gold not spent on Great Works, defenders or tiles. Saving it parks a fat treasury under the capture bounty.
- **Seen and countered:** treasury and bounty are public, and a progress bar shows Gold against 1,000. Rivals raid a city before the purchase to take a share (the bounty) or force defensive spending.
- **Feasibility:** one new production item, buy-only, through the existing `BuyItem` order; projects already exist for Germany. No new order.
- **Map luck:** none.
- **Scaling:** self-contained. More rivals means more bounty hunters.

**T2. War Chest.** **Rule:** hold 1,500 Gold for 10 consecutive turns.
- **Theme:** the vault as a fortress.
- **Cost:** idle money for 10 turns, with the bounty at its largest.
- **Seen and countered:** the treasury is public. Capturing any Swiss city transfers a share and resets the streak.
- **Feasibility:** existing, plus a streak counter.
- **Map luck:** none.
- **Scaling:** fine.
- **Flag:** this is the old "hold" family, which interest already rewards. Hoarding is natural for them, so it's less inconvenient than spending.

**T3. Toll Gates.** **Rule:** own 3 walled cities, each within 5 hexes of a rival city, for 20 consecutive turns.
- **Theme:** customs posts on the frontier (the Customs Bastion is their Walls).
- **Cost:** a turtle must build forward, paying for three Bastions where raids land first.
- **Seen and countered:** visible cities. Rivals take or besiege one gate, or settle the gap first.
- **Feasibility:** existing, plus a streak counter.
- **Map luck:** medium, since it needs free frontier land or a capture.
- **Scaling:** fine; 8 players has more frontiers.

**T4. Hired Swords.** **Rule:** have 8 military units (not Scouts) at once.
- **Theme:** bankers hire armies rather than raise them.
- **Cost:** 3 Gold per unit beyond 2 is 18 Gold a turn, against their own drawback.
- **Seen and countered:** army strength is public. **Weak counter:** the army can sit at home.
- **Feasibility:** existing.
- **Map luck:** none.
- **Scaling:** fine.

**T5. Bought Land.** **Rule:** own 20 tiles bought with Gold at once.
- **Theme:** buying the land instead of conquering it.
- **Cost:** tile prices escalate with every tile claimed.
- **Seen and countered:** rivals claim or settle first, or capture cities holding bought tiles.
- **Feasibility:** existing.
- **Map luck:** low.
- **Scaling:** fine.
- **Flag:** another Gold burn (same kind as T1), and tile buying is ordinary play.

### Japan: the dense linked hive

**C1. One Hive (decided, confirmed).** **Rule:** 8 cities in one connected cluster, each within 5 hexes of another. It's live: the VP is lost while the cluster is broken. The distance penalty is −10% per hex beyond 4.
- **Theme:** the hive is the faction.
- **Cost:** 8 cities is far past what Japan AIs reach today (1–3 by T130). Holding the shape forbids spreading toward resources and means defending every link.
- **Seen and countered:** visible on the map. Rivals capture a linking city (or Celts razes it) to split the cluster.
- **Feasibility:** live VP already exists for titles, so this behaves like a title. No new order.
- **Map luck:** low.
- **Scaling:** counts only your own cities, but 8 players means less land, so it's harder there. In the 1v1 Small playtest 8.3, the winner had 6 cities by T91, so 8 is a stretch in 1v1 (good).
- **Notes:**
  - A rival city wedged into the gap does not break a 5-hex link by itself; only losing a city does. If wedging should count, the link test needs "no foreign city between".
  - Consider 7 on Small maps (open question 4).

### Celts: the living land's first people

**V1. Scorched Ground (decided, confirmed).** **Rule:** raze 2 cities taken from rivals.
- **Theme:** the land doesn't keep what invaders built.
- **Cost:** the anti-rush faction must march out of its attrition homeland with weak melee (Soldier and Trooper −2) and take walled cities.
- **Seen and countered:** visible siege columns. Walls, garrisons and sallies, fighting off Celts land.
- **Feasibility:** razing is automatic on Celts capture; count it. No new order.
- **Map luck:** none.
- **Scaling:** 1v1 means razing 2 of your only rival's cities: brutal, but also near-winning.
- **Notes:**
  - Needs AI work: the Celts AI currently forgoes city assaults (`crates/ai/src/military.rs`).
  - Count a city the rival had itself captured from a third empire.

### Sparta: "pay the iron price"

**O1. The Iron Price (decided, confirmed with two guards).** **Rule:** N military units die in battles an Sparta unit fights in, theirs and enemies' both counting.
- **Theme:** blood is the price; their own dead count too.
- **Cost:** they need bloody, even fights, not clean kills. Losing units costs the army their healing depends on.
- **Seen and countered:** a public tally. Rivals refuse battle (walls, retreat, peace) to starve it.
- **Feasibility:** battle events exist; add a counter.
- **Map luck:** none.
- **Scaling:** 1v1 has one rival plus barbarians, which is enough.
- **Guards:**
  - **Scouts don't count.** Otherwise throwing 30-Production Scouts with no upkeep at walls is a degenerate chore.
  - **Barbarian deaths count at most N/4.** Otherwise camp farming replaces war.

### Byzantium: the garrison waiting for a sovereign who never comes

Lore (`docs/design/factions-original-three-astra.md`): they keep Legatum "until relieved", issue orders under an absent sovereign's seal, and insist the old order still holds.

**R1. The Long Watch.** **Rule:** keep a Byzantium unit on a tile next to a rival's capital for 40 consecutive turns (20 Quick).
- **Theme:** the garrison keeps its post at the seat of power, waiting to be relieved.
- **Cost:** a unit pinned on the most defended ground. It provokes the one war the Byzantium avoids: at war, a walled capital fires on it every turn, so the post must be reinforced under fire.
- **Seen and countered:** a public clock. The owner declares war and evicts it, or blocks the passable ring-1 tiles with its own units. It's a decision for the rival: "is 2 VP worth a war?"
- **Feasibility:** existing camps and Peace passage, plus a streak counter.
- **Map luck:** low, since every capital has passable neighbours.
- **Scaling:** 1v1 forces the only rival to decide; 8 players lets the Byzantium pick the distracted one (strategy, not luck).

**R2. Reopen the Depots.** **Rule:** camp on 3 rival-owned Iron or Coal deposits at once for 10 turns.
- **Theme:** "every reopened depot" proves the chain of command endures.
- **Cost:** deposit tiles are mediocre salvage, and the units spread out.
- **Seen and countered:** visible. Rivals evict or garrison their deposits.
- **Feasibility:** existing.
- **Map luck:** medium (deposit counts vary; Coal sits in the borderlands).
- **Scaling:** worst in 1v1, where one rival may own only 1–2 deposits.

**R3. Restore the Seat.** **Rule:** capture a rival's original capital and hold it 20 turns.
- **Theme:** "The Astrarchy did not fall. It is waiting."
- **Cost:** a full war against the strongest city, from a faction built to squat, not storm.
- **Seen and countered:** obvious and walled.
- **Feasibility:** needs a "was a capital" record; capture drops capital status today.
- **Map luck:** none.
- **Scaling:** fine.
- **Flag:** conquest kind, overlapping Celts.

**R4. Recall the Legions.** **Rule:** no camps, and 6 Legionary-class units inside your own borders, for 15 consecutive turns.
- **Theme:** the garrison recalled to the colours.
- **Cost:** all salvage Science for 15 turns, which is their entire research.
- **Seen and countered:** visible. **Weak counter:** rivals just enjoy their yields back.
- **Feasibility:** existing.
- **Map luck:** none.
- **Scaling:** fine.

**R5. Imperial Census.** **Rule:** camp on the land of every rival you've met at once.
- **Theme:** the old writ runs everywhere.
- **Cost:** spreads thin.
- **Seen and countered:** visible.
- **Feasibility:** existing.
- **Map luck:** low.
- **Scaling:** broken: near-free in 1v1 (they camp anyway), and very hard with 7 rivals (pattern 8).

### Germany: heavyworld engineers with heavy machines

**K1. Breach.** **Rule:** bring 2 different walled rival cities to 0 HP.
- **Theme:** heavy machines exist to crack walls.
- **Cost:**
  - The Germany never campaigns (its AI defends locally).
  - With −1 move, a slow column is far from its forges for many turns.
  - Siege Engines need Siege Engineering and 10 Coal each.
- **Seen and countered:** a 1-move column is visible for turns. Rivals add Citadels and garrisons, sally to kill engines (10 melee strength), or make peace.
- **Feasibility:** existing combat and city HP; count distinct walled cities reaching 0 HP. Capture isn't required.
- **Map luck:** low (Troopers and Rangers can do it without Coal, just slower).
- **Scaling:** fine. In 1v1 a rival that never walls avoids the goal but invites capture instead.

**K2. Engines at the Gate.** **Rule:** keep a Siege Engine within 2 hexes of a rival capital for 5 consecutive turns.
- **Theme:** a siege park.
- **Cost:** the engine is exposed deep in enemy land, and it needs war.
- **Seen and countered:** the capital fires, and defenders sally.
- **Feasibility:** existing.
- **Map luck:** low (Coal).
- **Scaling:** fine.
- **Flag:** a positional siege, same kind as K1.

**K3. Heavy Colony.** **Rule:** found a city at least 10 hexes from every other city you own.
- **Theme:** weak; heavyworlders aren't explorers.
- **Cost:** a 1-move Settler crawls through no-man's-land, and the colony is defended far from home.
- **Seen and countered:** snipe the slow Settler, settle the gap first, attack the colony.
- **Feasibility:** existing.
- **Map luck:** medium-high in 8 players (no free land 10 hexes out).
- **Scaling:** poor in crowded games.

**K4. Grand Battery.** **Rule:** have 4 Siege Engines at once.
- **Theme:** an artillery park.
- **Cost:** 480 Production, 40 Coal, upkeep.
- **Seen and countered:** **weak counter:** they can sit at home.
- **Feasibility:** existing.
- **Map luck:** medium (Coal).
- **Scaling:** fine.
- **Flag:** effectively a Production burn, which is templated.

**K5. Unbroken.** **Rule:** own 6 cities and lose none for 100 consecutive turns.
- **Theme:** what heavyworlders build stays built.
- **Cost:** walls and garrisons everywhere, and no expendable frontier.
- **Seen and countered:** a strong counter: capture any city.
- **Feasibility:** a streak counter.
- **Map luck:** none.
- **Scaling:** harder with more attackers.
- **Flag:** turtling is already natural for them, so it's the least inconvenient option.

### Mongols: the courier lineage

**G1. Courier's Oath.** **Rule:** go 80 consecutive turns (40 Quick) without declaring war or accepting a call to arms.
- **Theme:** a courier never strikes first; the road stays open to everyone.
- **Cost:** the natural rusher gives up its rush windows, and its offence kit (+1 move, slip-past) idles. Wars declared on it are still fought, without terrain defence.
- **Seen and countered:** a public clock shows exactly when the rush window reopens. Rivals greed (skip Walls, expand) during the oath and time their defences for its end. They can also attack it, knowing it can't open a second front. Medium counter.
- **Feasibility:** existing orders, plus a streak counter.
- **Map luck:** none.
- **Scaling:** self-contained (pattern 8).

**G2. Neutral Ground.** **Rule:** be at Peace with every empire for 60 turns in total (30 Quick), counted after turn 50.
- **Theme:** the courier as everyone's neutral.
- **Cost:** as G1.
- **Seen and countered:** any rival stops the count by declaring war (a strong counter, and a decision for that rival).
- **Feasibility:** existing.
- **Map luck:** none.
- **Scaling:** harder with 7 rivals, any of whom can stall it.

**G3. Waystation.** **Rule:** hold a city closer to a rival's capital than to your own capital for 30 consecutive turns.
- **Theme:** a relay house deep in others' lands, reachable because Mongols Settlers cross borders at Peace.
- **Cost:** a far city with no terrain defence, and a Settler exposed in transit.
- **Seen and countered:** snipe the Settler, claim the land first, besiege the station.
- **Feasibility:** existing, plus a streak counter.
- **Map luck:** medium (free land near rival capitals shrinks over time).
- **Scaling:** fine.

**G4. Ransom Road.** **Rule:** capture 3 rival Settlers.
- **Theme:** highway robbers rather than couriers.
- **Cost:** low.
- **Seen and countered:** escorts.
- **Feasibility:** existing.
- **Map luck:** none.
- **Scaling:** fine.
- **Flag:** normal rusher play (the user calls Settler sniping "pretty much always worth the war"), so not inconvenient.

**G5. Open Roads.** **Rule:** hold an Alliance for 40 consecutive turns.
- **Theme:** the trusted carrier.
- **Cost:** no war on the partner; upkeep.
- **Seen and countered:** the partner leaves (free after 10 turns).
- **Feasibility:** existing.
- **Map luck:** none.
- **Scaling:** broken in 1v1: the only rival must agree, and in a zero-sum duel it never should.

### Athens: the relay maintainers

Lore: "the rightful heir will be whoever makes the silent network answer."

**S1. Open Broadcast.** **Rule:** build the Open Relay. While it stands, every empire sees everything inside your borders. After 40 turns standing (20 Quick), score.
- **Theme:** to make the network answer, you must transmit in the clear.
- **Cost:** no fog over your armies, garrisons, Settlers and wounded cities for 40 turns. Production and buildings stay fog-limited.
- **Seen and countered:** rivals strike with full information, or capture the Relay's city (the Relay is destroyed on capture, resetting the clock).
- **Feasibility:** a new goal building, plus a one-way vision grant that reuses the Alliance shared-sight path (`visibility.rs::shares_team_sight`). No new order.
- **Map luck:** none.
- **Scaling:** fine. With 8 players, seven empires watch you.

**S2. Listening Watch.** **Rule:** see a city of every rival (up to 3) at the end of 10 consecutive turns.
- **Theme:** listening posts.
- **Cost:** units posted forward on high ground. They can't cross borders at Peace, so they need hills, war or an ally's sight.
- **Seen and countered:** rivals push watchers back (war) and keep cities deep.
- **Feasibility:** existing, plus a streak counter.
- **Map luck:** low-medium (hills).
- **Scaling:** capped at 3. Chore risk is moderate: a held posture, not a tour.

**S3. The Forecast.** **Rule:** once per game, publicly name a rival and a title. If that rival holds the title 40 turns later, score.
- **Theme:** reading the signals (Bene Gesserit).
- **Cost:** low.
- **Seen and countered:** the named rival can dump the title, but rarely wants to.
- **Feasibility:** **new order type.**
- **Map luck:** none.
- **Scaling:** **weak in 1v1:** predict the rival's natural title and it's nearly free (pattern 4).

**S4. Silent Channel.** **Rule:** know 5 techs that no living rival knows, at once.
- **Theme:** signals nobody else can read.
- **Cost:** a detour into techs others skip.
- **Seen and countered:** `known_by` counts are public. Rivals research those techs, cheaper for them through diffusion.
- **Feasibility:** existing.
- **Map luck:** none.
- **Scaling:** much harder with 7 rivals.

**S5. Restore the Relay.** **Rule:** complete the Relay Uplink before any rival completes any Great Work.
- **Theme:** strong; it's the Relay.
- **Cost:** rushing a 300-Production Work (plus its Archive) early, at expansion's expense.
- **Seen and countered:** rivals race their own Work; Switzerland can buy one.
- **Feasibility:** existing.
- **Map luck:** none.
- **Scaling:** harder with more racers.
- **Flag:** stacks with the 2-VP Great Work milestone, so one act is worth 4 VP.

---

## 3. Recommendations

| Faction | Recommended | Why, in one line | Runner-up |
| --- | --- | --- | --- |
| Switzerland | **T1 Endowment** | The user's own save-then-burn idea. Templating stops mattering once no other faction burns a currency. | T3 Toll Gates |
| Japan | **C1 One Hive** (confirmed) | A live, public shape the hive must hold, with an obvious counter. | — |
| Byzantium | **R1 The Long Watch** | Makes the squatters hold one post at a rival's seat until someone pays for a war to remove them. | R3 Restore the Seat |
| Germany | **K1 Breach** | The heavy machines' purpose, forcing a slow faction that never campaigns to march out. | K2 Engines at the Gate |
| Celts | **V1 Scorched Ground** (confirmed) | The anti-rush faction must attack and raze: inconvenient and in theme. | — |
| Mongols | **G1 Courier's Oath** | The natural rusher's goal is not to rush: the most inconvenient thing it can do, and self-scaling. | G3 Waystation |
| Sparta | **O1 The Iron Price** (confirmed; no Scouts, barbarians capped) | Blood, its own included. | — |
| Athens | **S1 Open Broadcast** | The listeners must be heard: a self-harm goal (pattern 7) with a clear counter and no map luck. | S2 Listening Watch |

**Do they differ in kind?**

| Faction | Kind | What is counted | Who can stop it, and how |
| --- | --- | --- | --- |
| Switzerland | Sacrifice | Gold spent in one purchase | Any rival: raid the treasury through the bounty before the purchase |
| Japan | Cohesion (live shape) | Your cities' linkage | Any rival: capture or raze a linking city |
| Byzantium | Vigil (hold a post) | Turns at a rival capital | The capital's owner: evict, or block the tiles |
| Germany | Siege | Walled rival cities broken | The target: Citadels, garrisons, sallies |
| Celts | Destruction | Rival cities razed | The target: Walls, garrison |
| Mongols | Restraint | Turns without starting a war | Indirect: exploit the window, attack it |
| Sparta | Attrition | Deaths in their battles | Everyone: refuse battle |
| Athens | Exposure | Turns transmitting in the clear | Any rival: strike with intel, capture the Relay city |

No two share a kind, and only one burns a currency. **Honest caveat:** four goals (Byzantium, Germany, Celts, Sparta) put the faction in conflict with rivals' cities or armies. That's deliberate, since expansion by conquest is the core loop, and they differ in what is counted. Germany's Breach and Celts's raze are the closest pair: both hit cities, but Breach needs no capture and Celts's needs capture plus destruction. If the user wants them further apart, use G3-style encroachment for the Germany instead (a walled forward foundry nearer a rival capital than its own).

**AI implications (method step 8):** all four conflict goals need the AI to pursue them, and rivals' AI to defend against them: evicting the Watch, intercepting siege columns, punishing the Open Broadcast. Celts and Germany currently forgo city campaigns entirely.

---

## 4. Should every goal be worth 2 VP?

Today: Switzerland, Japan, Germany, Celts and Athens 2; Byzantium and Mongols 3; Sparta 4 (`crates/core/src/data.rs::faction_goal_vp`).

**Recommendation: yes, 2 VP for everyone.**
- **Readable (pillars 4 and 5):** "a faction goal is worth 2", the same as a Great Work, with no per-faction footnote.
- **Not decisive:** 4 VP is 40% of a 10-VP win from one condition. That makes the Sparta game "did they get the goal", and pillar 11 wants nobody to know the winner early.
- **Difficulty belongs in N:** a harder goal gets a lower threshold, not more VP.
- **The 3s and the 4 were balance patches** (bundle D era, and Sparta's slump) on goals that are now being replaced. Balance waits for a competent AI (user, 2026-10-04). When it resumes, use the faction-method levers: deepen drawbacks or make weak identities pay off.

If a faction still lags after search-AI baselines, raising its goal VP is a last resort, with the reason recorded.

---

## 5. Open questions for the user

1. **Mongols:** G1 (no war you start; you can still be attacked) or G2 (peace with everyone; any rival can stall it)? G1 scales better; G2 is easier to counter.
2. **Permanent or live:** should only Japan's goal be live, with the rest permanent once reached (my default)?
3. **Sparta:** agree that Scouts don't count and barbarian deaths are capped at N/4? A proposed N: 40 Standard and 20 Quick deaths, before any sims.
4. **Japan:** keep 8 cities on every map size, or 7 on Small?
5. **Athens:** may the Open Relay go in any city (an interior one is safer), or must it be the capital? Is it destroyed on capture (my default) or inherited?
6. **Byzantium:** the Watch only at rival capitals (my default, the hardest), or next to any rival city with a longer clock?
7. **Germany:** is Breach too close to Celts's raze? If so, the fallback is the forward foundry.
8. **Switzerland:** must the Endowment be bought in one 1,000-Gold purchase (my default, so it's visible), or can it be paid in instalments?
9. **Four conflict goals out of eight:** acceptable, given conquest is a core loop?
10. **VP:** move every goal to 2 now, or wait until the new goals land?
