# Byzantine mercenaries, and what to change in the other factions

Design research, 2026-10-17, second round. Follows `rule-breaks-2026-10-17.md`. Nothing here is decided until the user says so; decisions go to the design doc's decisions log and the `legatum-factions` skill.

**The user's reply to round one:** "'Set them against each other' only works in theory. They can't actually force someone to move an army and do something. I kind of like mercenary paymaster, but how would the mechanic work there? ... they can buy units (as mercenaries) for cheaper than normal, but they only last n turns ... Or they can buy the units cheaper but that money goes to another civ? Or maybe they can buy other civs' special units with gold? ... Or they can buy them for cheaper in cash upfront, but then pay more in upkeep? ... it needs to feel like a real rule break. What other faction rules or bonuses do you recommend we change?"

**Lesson recorded (for the factions skill):** a rule break must produce effects the faction itself controls on the map. Anything that routes through another player's choice to act (subsidies, calls to arms, "cannot refuse" treaties) either forces them (a veto, rejected) or asks them (and an AI or human may do nothing, so the break is theoretical). Hire an army fails this test too and should retire with forced peace.

Everything below is checked against `game-rules.md` as it stands: buying at 200% of Production in Gold with one purchase per city per turn (`economy::buy_cost_for`, `buy_blocker`), upkeep of 1 Gold per armed unit beyond two free (`data::FREE_MILITARY_UNITS`, `pays_upkeep`), thin levies at cities + 1 (`data::levy_cap`, `economy::levy_count`, `levy_check`), bankruptcy disbanding the costliest unit (`rules/cities.rs::bankruptcy_victim`), promotions as flat automatic levels (`rules/promotion.rs`), upgrades at 200% of the cost difference (`UPGRADE_COST_PERCENT`), the roster filter (`data::available_to_faction`, `faction_unavailable_reason`), first contact (`DiplomacyState::met`), and simultaneous resolution with one order per unit.

Contents:
1. The mercenary rule break: options, the design, numbers, an example, drawback, counters, AI, UI, code
2. Changes to the other factions, ranked
3. Questions for the user
4. Sources

---

## 1. The mercenary rule break

### 1.1 The four options, each on its own

| Option | What it breaks | What it gets right | Why it is not enough alone |
| --- | --- | --- | --- |
| **Temporary cheap mercenaries** (Civ 6 city-state levy: units for 30 turns at their Production cost in Gold, you pay their upkeep, you cannot delete them) | Units are permanent here; nothing expires | A real break that the whole table can read (a countdown); a surge army on demand, which is the Komnenian story (500 regulars in 1091, Cuman allies for one battle at Levounion) | "Cheap" alone makes Byzantium a better Switzerland (175% buyer). The expiry is only interesting if the units are foreign and the price is sharp; otherwise it is a nuisance |
| **The price goes to another civ** | Gold is destroyed when you buy; nothing moves Gold between empires except tribute, hire fees and the Swiss bounty | A choice Byzantium makes and controls: whom do I enrich? And a choice the rival reads: do I take the income or cut it off? Historically exact (Byzantine solidi paid the Rus, Pechenegs, Normans, Turks) | Permanent bought foreign units with no cap would make thin levies meaningless and Byzantium's army unlimited |
| **Buy met civs' unique units** | The roster is per faction | Visible and fun: a Keshig in purple. The deeper power is borrowing the seller's *era*, as Civ 6 lets you levy units you have not discovered | There are only four unique units today (Hoplite, Keshig, Ambusher, Tagmatic Soldier), so alone it is thin |
| **Cheap upfront, heavy upkeep** | Prices | Pushes toward short wars and against standing armies, which is the thin-levy identity | A pricing number, not a rule; nobody would call it a break |

Each of the four is one face of the same unit. Put together they are one coherent mechanic with one sentence per face.

### 1.2 The design: Mercenary Companies

**Rule (the break).** Byzantium may hire a **Company**: any military unit that a faction it has met and is at Peace with can build right now, bought with Gold in a Byzantine city. A Company does not count toward thin levies. It serves 20 turns and then goes home.

Five lines, in the faction's own words on the sheet:

1. **Foreign soldiers.** The shop lists every military unit each met, peaceful faction can build *now*, with that faction's rules and tech: Spartan Hoplites with their phalanx, a Mongol Keshig once the Mongols have Iron Working, Knights before Byzantium has Chivalry. Not Scouts, Settlers, the Heir or aircraft; not Byzantium's own roster (those are levies). A faction at War with Byzantium sells nothing.
2. **Paid to the seller.** The price is 100% of the unit's Production cost in Gold (everyone else buys at 200%), and the Gold goes to the seller's treasury. A human seller sees "+110 Gold: Byzantium hired a Hoplite company from you".
3. **Outside the levies.** Companies are not armed levies: `levy_count` ignores them. Byzantium's own units stay at cities + 1.
4. **A contract of 20 turns.** A Company appears in the hiring city next turn at promotion level 1, fights as any unit (it can capture cities; it earns XP; it cannot be upgraded), pays 2 Gold a turn upkeep instead of 1, and disbands at the end of its twentieth turn with a public countdown on the unit. There is no renewal order; you hire again.
5. **Unpaid companies turn.** When Byzantium cannot pay its upkeep, the bankruptcy rule takes a Company first, and instead of disbanding it becomes a barbarian unit where it stands (the Catalan Grand Company, hired by Andronikos II in 1303, unpaid, then at war with him for years).

Thin levies stay as the drawback that makes line 3 matter. The Heir stays as the goal. The Tagmatic Soldier (+4 strength, +5 cost) stays as the native core: "the Tagmata hold the cities, the Companies win the battles". The Imperial Scriptorium stays. Bought peace and Hire an army retire.

### 1.3 The key questions

**Do Companies ignore thin levies?** Yes; that is the break. Byzantium's own army is capped at cities + 1 and permanent; its field army is hired, uncapped and temporary. The cap is what makes Byzantium hire, and the hiring is what makes the cap bearable. Without line 3 the design is a discount; with it, Byzantium plays a different game: its military strength is a Gold flow, not a Production stock.

**Where do they appear, with what promotions?** In the hiring city, next turn, through the existing `BuyItem` path (`complete_item` spawns the unit; "bought units act next turn"). One purchase per city per turn holds, so a 4-city Byzantium raises at most four Companies a turn, which is already a surge. They start at level 1 (seasoned professionals; the same spawn rule Sparta uses in `game.rs::spawn_unit`) and promote normally; the XP dies with the contract. Alternative placement considered and set aside: appearing at the border nearest the seller (more flavour, but it needs a placement rule and makes the AI's staging harder; the city is where the money is).

**Can they capture cities?** Yes. Harald Hardrada's Varangians took Sicilian towns for Michael IV; the Normans in Byzantine pay took Messina. Forbidding it would mean the conquest loop runs through cities + 1 native melee units, which makes Byzantium a defender by rule, the thing we are moving away from. If a Company-heavy Byzantium snowballs in sims, "only the Tagmata take possession" is the first lever to try (section 1.9), because it is in theme and reads at a glance.

**What happens to a contract mid-war?** The countdown is on the unit badge for everyone. A Company at 0 at the end of its turn disbands in phase 6 (with heal/fortify), whatever it is doing: standing in a siege ring, garrisoning a city, carrying a wound. That is the Civ 6 tension ("plan accordingly") and the historical one (the Cumans went home after Levounion). A Company inside an enemy city it captured leaves the city empty. Byzantium's answer is to stagger contracts, keep a native garrison, or hire again. There is no "pay to extend" order in v1: an extension is a new hire in a city, so the one-purchase-per-city rule and the Gold flow stay the only limits. If the user wants extension, `Action::RenewContract{unit}` while the unit stands in a Byzantine city is the shape (a unit action like `RetireVeteran`), at the hire price again, resetting to 20.

**Can rivals counter it?**
- *Outlast it.* The countdown is public. A besieger facing three Companies with 4 turns left holds; an attacker strikes the turn after they go home. The Mongol AI that waited 58 turns for a battle in 8.9.2 will find a Byzantine war has a rhythm it can read.
- *Starve the Gold.* Upkeep is 2 a turn per Company; a long war bleeds the treasury; decadence taxes the hoard's income; every Company killed is Gold Byzantium must spend again (Production losses are someone else's). A Byzantium at 0 Gold with three Companies is a Byzantium with three new barbarian bands next to its cities.
- *Deny the roster.* A faction at War with Byzantium sells nothing. The leader whose Knights Byzantium is hiring has a decision: take the Gold, or declare and shut the shop (and face the Knights already bought).
- *Take the income.* The seller profits. A rival next to Byzantium can let it hire, bank the Gold, and out-build it. That is the "rival choice" the user asked for, and it needs no action from the rival, so it passes the lesson.
- *Outbidding* does not exist and should not: an auction is a new system.

**How does it scale across eras?** The price is 100% of Production cost, which rises by era with the unit tables (Spearman 40, Pikeman 76, Musketman 105, Rifleman 145, all doubled at Standard). The roster borrows the sellers' eras, so a Byzantium behind in research can still field the period's units, and a Byzantium ahead cannot hire ahead of everyone. Upkeep (2) and the contract (20 turns) are flat; upkeep is flat for everyone, and 20 turns is about half an era at the measured pacing (Classical entered about T30, Medieval T63, Renaissance T97), so a contract covers one campaign in any era. The one number that fades is the level-1 start (+10% of base strength); it is a flavour touch, not the power.

**How does the AI value it?** Byzantium's `desired_army` is clamped to the cap today (`ai/levy.rs`); Companies become the way it closes the gap between the army it wants and the army it may own. The planner prices a Company as a 20-turn asset: value = the campaign or defence it enables over the next 20 turns (the existing `plan.rs` forecast-priced Campaign and `threat.rs` defence), cost = price + 40 upkeep + the Gold given to the seller weighted by that seller's threat to Byzantium (paying the leader is dearer than paying a distant minnow). Seller choice is by unit quality per Gold (`production.rs::composition_bonus` over foreign kinds, with era tier), timed so contracts start when the army is staged, bought in the city nearest the front. Rivals' threat model (`threat.rs`) counts Companies with their expiry (a threat that fades on a known turn); the Campaign planner can wait out a contract; sellers value the income passively; the war valuation against Byzantium gains "shuts its shop" when we are its main seller. Telemetry: Companies hired per game, Gold paid to each seller, Companies alive at expiry, captures by Companies, barbarian turns.

**What UI does it need?** The city buy list gains a Mercenaries section grouped by seller with the origin heraldry, price, "paid to the Celts", contract 20 turns, upkeep 2; the unit badge carries the origin heraldry and a countdown ring visible to everyone (pillar 5); the hover card says "Company (Spartan): 7 turns of contract left, upkeep 2"; the empire panel shows "Companies: 3, upkeep 6, next goes home in 4 turns"; a report line for the seller ("+110 Gold: Byzantium hired a Hoplite company") and for the table when a Company turns ("Unpaid, the Catalan company turns on Byzantium"); the faction sheet and the Guide.

### 1.4 Numbers to start from

| Knob | Start value | Why | Where |
| --- | --- | --- | --- |
| Hire price | 100% of Production cost in Gold | Half the ordinary buy; the cheapness is the only reason to rent | `economy::buy_cost_for` (a mercenary branch) |
| Share paid to the seller | 100% | The sharpest rival choice; 50% if it reads as a gift to the leader | `tribute::transfer` |
| Contract | 20 turns | Half an era; one campaign | `Unit::contract_until`, `data::COMPANY_TURNS` |
| Upkeep per Company | 2 Gold a turn | A 20-turn Company costs price + 40, so over 60 turns renting costs about 1.6x owning: cheap now, dear always | `economy::upkeep_breakdown` |
| Starting level | 1 | Professionals; Sparta's spawn rule | `game.rs::spawn_unit` |
| Upgrade | Not allowed | You hire the next era's Company instead | `upgrade_blocker` |
| Hires per turn | One per city (existing rule) | A natural surge limit without a new cap | `buy_blocker` |
| Bankruptcy | A Company first; it turns barbarian | The Catalan rule | `rules/cities.rs::bankruptcy_victim` |
| Seller eligibility | Met and at Peace or Alliance | At War: no sales | `DiplomacyState::met`, relation |

Worked prices at Standard (costs doubled): a Spartan Hoplite company 80 Gold (vs 160 to buy a Spearman), a Phalangite 110, a Pikeman 152, a Keshig about 150, a Knight 204, a Musketman 210, a Rifleman 290. Early Byzantine income is 1 to 3 Gold a turn (the tribute calibration), so the first Company is a mid-game purchase unless Byzantium hoards; that is intended, and the early rush stays final (two levies and Walls, as the user wants).

### 1.5 A turn-by-turn example

Standard, 8 empires. Byzantium has three cities (Constantinople, Nicaea, Trebizond), so a cap of 4, and fields exactly 4: two Tagmatic Soldiers, an Archer, a Horseman. It has met Sparta (Peace, knows Mathematics), the Celts (Peace) and the Mongols (Peace). The Mongols have 6 units and a Keshig column has been riding toward Nicaea for three turns at Peace. Byzantium's treasury is 430 Gold, public, and the Mongol AI's threat model already counts it as "about to hire".

- **T60.** Byzantium orders `BuyItem` in Nicaea: a Spartan Phalangite company (110 Gold to Sparta), and in Trebizond a Celtic Ambusher company (152 Gold to the Celts). Treasury 168. Sparta's report: "+110 Gold". Nicaea's Walls are 20 turns old.
- **T61.** Both Companies appear at level 1, contract until T81, upkeep +4 (Byzantium now pays 2 + 4 for 6 armed units beyond the two free). The Ambusher walks into the forest east of Nicaea and hides (Celtic rule, carried by its origin). Everyone sees two purple units with a 20-turn ring.
- **T63.** The Mongols declare (surprise war, same turn). Their Keshig shoot the Phalangite on the wall; the Ambusher reveals itself with a +3 shot; Nicaea's walls fire.
- **T64 to T72.** The Mongols cannot take a walled city that fires, with a phalanx in it and a hidden archer beside it; they lose two Horsemen and withdraw. Byzantium does not pursue: its Companies are for the wall, and the Horseman is its only pursuer.
- **T75.** The Mongol AI reads the countdown ("6 turns") and offers Peace. Byzantium's choice: accept now, or hire again at T78 so that the second pair overlaps the first. It has 168 + 15 turns of income (about 300) and takes Peace.
- **T81.** The Companies go home, in the open and at the same time; the Mongol AI had a second strike pencilled for T82 and now has Peace instead. Byzantium is back at 4 levies, 262 Gold lighter, and Sparta and the Celts are 262 Gold richer.

The offensive mirror: at T120 a 6-city Byzantium with 900 Gold hires three Knight companies (612 Gold to Germany, which has Chivalry) in its three cities nearest a 4-population Athenian border city, declares at T122, and has 20 turns to take it before the Knights go home. Athens reads the countdown, walls up, and makes the siege last 19 turns.

### 1.6 The drawback that pushes further into the style

Thin levies already do most of it: Byzantium cannot build a standing army, so its wars are hired, short and expensive. Two things sharpen it without a new system:

- **Unpaid companies turn** (line 5). A Byzantium that over-hires and loses its income gets barbarians at its gates. Self-scaling: more Companies, more risk. Visible: `EconomyView.overreach_dry_in` already warns five turns ahead.
- **Gold leaves the empire.** Every hire enriches a rival. A rich Byzantium is dangerous to itself in a way the Swiss hoard is not: the Swiss hoard is a target, the Byzantine hoard is a subsidy to whoever Byzantium buys from.

Not added: a surcharge on Byzantine buildings (the retired 120%), a Company count cap (the Gold is the cap), or "Companies cannot garrison" (it would make the native cap bind in a fiddly way).

### 1.7 Counterplay, as rock-paper-scissors

- Byzantium beats **greedy expanders** and **rushers who dawdle**: a surge of foreign shooters and a phalanx appear in two turns, without a Production ramp.
- **Attrition** beats Byzantium: long wars drain the Gold, contracts expire, and bankruptcy arms the barbarians.
- **Patience** beats Byzantium: wait out the contract (public countdown) and strike the turn after.
- **War** shuts the shop: the faction whose units Byzantium hires can refuse by declaring, which is a decision with a cost.
- **Greed** profits from Byzantium: a seller can take the Gold and out-build it, especially a Swiss seller whose interest compounds it.

### 1.8 Code it touches

- **Core state:** `Unit::origin: Option<Faction>` (the seller; rules that key on the owner's faction read `origin.unwrap_or(owner faction)`: the Sparta wound rule, Celts concealment and the hunter kind test, the Keshig's mode) and `Unit::contract_until: Option<u32>`, both serde-default so old saves load.
- **Roster:** `data::available_to_faction` gains a hire table: for the Byzantium, each met peaceful seller's buildable military kinds (seller's techs via `economy::tech_available` on the seller, seller's faction exclusions, `pays_upkeep` kinds only, no Heir, no aircraft).
- **Buying:** `economy::buy_cost_for` returns the 100% mercenary quote with the seller; `buy_blocker` keeps one purchase per city per turn; resolution moves the Gold with `tribute::transfer` and `complete_item` spawns the unit at level 1 with `origin` and `contract_until = turn + 20`.
- **Levies:** `economy::levy_count` filters `contract_until.is_none()`.
- **Upkeep:** `economy::upkeep_breakdown` charges 2 for a contracted unit, with its own line.
- **Expiry:** a step in `resolve_turn` phase 6 removes Companies at 0 with a public event (`Event::CompanyDeparted`, or `UnitDisbanded` with a reason), mirroring `sparta_restlessness`'s removal path.
- **Bankruptcy:** `rules/cities.rs::bankruptcy_victim` prefers a Company and `apply` converts it to a barbarian unit (owner = the barbarian player, as camp guards are) instead of removing it.
- **Upgrades:** `upgrade_blocker` refuses contracted units.
- **Diplomacy:** nothing new; eligibility reads `met` and the relation. Delete `PayTribute`, `tribute::pay_tributes`, the hire fee in `calls.rs`, `HireLock`, and their UI (bought peace and Hire retire), keeping `tribute::transfer` and `quote`'s rounding helpers.
- **View and UI:** `UnitView.origin`, `UnitView.contract_left`; the buy list's Mercenaries section; the badge ring; report lines; `FACTION_INFO`, the sheet, the Guide; the now-retired TypeScript parity client.
- **AI:** `ai/levy.rs` (desired army includes Companies), `production.rs::plan_buy` (hire decisions by plan value, seller choice), `threat.rs` (fading threats), `diplomacy.rs` (seller income and "shut the shop" terms), `tribute.rs` deleted.
- **Rules reference:** `game-rules.md` sections 3, 5, 8 and 9 in the same change.

### 1.9 Why this beats the alternatives, and the levers if it misbehaves

It is one mechanic that a Civ player already knows (levying) with four faces the user named, each needed: *foreign* gives the roster and the era borrowing; *paid to the seller* gives the rival choice without asking the rival to do anything; *temporary* gives the public counter (the countdown) and the war rhythm; *outside the levies* is the rule that is actually broken, and the levy cap is what makes Byzantium need it. It passes the lesson: every effect is Byzantium's own unit on the map, and the only thing other players "do" is bank Gold or go to war. It keeps the strengths of the old kit (Gold as the weapon, self-scaling price, a rich Byzantium being dangerous to itself) and drops the veto.

Against the round-one alternatives: Greek Fire and the Theodosian Walls are defensive and would need a second line to give Byzantium a verb; the Varangian Guard (permanent foreign units) made thin levies meaningless; subsidies routed through others' choices. This is the Varangian Guard with the three corrections the user's options supply.

Levers, in order, if sims or playtests show a problem:
1. Byzantium snowballs by hiring: Companies cannot capture ("only the Tagmata take possession"), then the price to 125%.
2. Byzantium is too weak: contract 30 turns (Civ 6's number), then upkeep 1.
3. Hiring from the leader feels like feeding it: seller share 50%, or no sales from an empire holding its VP target.
4. The countdown makes wars too readable: hide the exact turn from rivals (show "contract" without the number); pillar 5 argues against this, so last.

---

## 2. Changes to the other factions, ranked

The evidence since round one: the 10-17 AI-only win rates (Byzantium 19.7%, Athens 18.6%, Sparta 17.6%, Germany 11.5%, Switzerland 11.1%, Celts 10.9%, Mongols 7.6%, Japan 2.9%; spread 6.8x against the 1.5x target; ±3 pp), which are confounded by AI quality (Japan's is mostly a targeting bug; Sparta's AI was fitted to one map); the 8.9.2 Sparta T87 review (AI Sparta's one-city target list is hard-coded to Japan or a nearby Byzantium, it makes peace within five turns of a capture, and defenders swap Walls in progress for a shooter: all AI, not rules); and the 8.9.2 Swiss review (interest dodged decadence, now being fixed; no AI ever aimed at the hoard; the threat model is blind to an economic leader until about 45% of the target).

The test for a rule break here is the **verb test**: what does the faction *do* that nobody else can? Switzerland buys Great Works and earns interest; the Mongols slip past and ride through; Sparta heals by killing; Germany converts Production; the Celts hide and bleed invaders; Byzantium (new) hires. Japan and Athens have no verb: their breaks are percentages. That is the first thing to fix, ahead of balance, because balance waits for a competent AI (user, 2026-10-04).

Ranked by how much the change improves the game on four counts: the break feels real (a verb), story fit, pillar 8 asymmetry, pillar 10 counters.

### 2.1 Byzantium: Mercenary Companies (section 1). Replaces bought peace and Hire an army.

Verb: hires. Story: exact. Asymmetry: the only Gold-flow army. Counter: wait, starve, shut the shop. Also deletes the one veto in the game.

### 2.2 Athens: the Delian League replaces the Science numbers

Today: Lyceum +6 Science (a Library replacement), +1 Gold on Markets, 30 Science from ruins, and Share the Learning. The 18.6% win rate says the numbers are fine; the verb test says there is no verb. With Byzantium leaving diplomacy for the mercenary lane, the diplomacy lane is free, and Athens has the best claim to it in history: the Delian League, 150 to 330 cities paying phoros to Athens.

**Rule:** Athens pays no Alliance fee and no Alliance upkeep; each partner's Alliance upkeep (1 Gold per city per turn, today paid to nobody) is paid **to Athens**. Shared sight and passage as any Alliance. **Drawback:** Athens can never betray: no `BreakTreaty` inside the protected window, no `DeclareWar{breach}` on an ally, ever; and its partners may leave at any time without cost (Naxos, Thasos and Samos revolted; the League became an empire because Athens held them by force, which we do not let it do). **Goal:** Share the Learning stays (teaching the allies is the League's other face). **Counter:** refuse Athens' offers, or join, bank the sight, and leave when it suits; a coalition against a rich Athens costs nothing (the free-Alliance rule). **Verb:** collects tribute from friends. **Feasibility:** `TreatyKind::fee`/`upkeep_for_cities` and `economy::treaty_upkeep` take a payee; `diplomacy::check_order` refuses Athenian breaches; `leave_cost` is 0 for leaving Athens. AI: Athens offers Alliances early (`alliance_acquaintance_turns` lower), values partners by city count; rivals weigh the Gold they would pay against the sight and the security (`alliance_security` already prices it). Keep the Lyceum as a plain Library replacement with a small Science edge so the name survives.

If the user prefers Athens to stay a Science faction, the research verb is "the Lyceum reaches ahead": Athens may research one technology of the *next* era before finishing its own (TI4 Jol-Nar), with the drawback that Share the Learning fires automatically when Athens enters an era first (the agora cannot keep a secret).

### 2.3 Japan: dense settlement makes the hive literal

Today: +1 Food, +7% Food and Production per other Japanese city within 5 hexes, −10% yields per hex beyond 4 from the capital, the Rice Storehouse, Sakoku. The 2.9% is mostly the AI (the targeting bug named in the learnings), so do not read it as a balance call; fix the AI first. But the break is a percentage network, and the user's own picture of Japan is castle towns packed around a centre.

**Rule:** Japanese cities may be founded **3 hexes apart** (everyone else: more than 3; `MIN_CITY_GAP`). The network bonus and the distance penalty stay as they are, so the extra density is paid for in the usual way (a 3-hex city shares tiles with its neighbours under population-is-territory, which is the cost: fewer worked tiles each, more cities). **Counter:** unchanged, wedge into the cluster; and now a Japanese cluster has more, smaller cities to take, each worth less. **Verb:** packs cities. **Feasibility:** `MIN_CITY_GAP` read through a faction function in `rules/units.rs::found_city_blocker` and `settlement::analyze_sites`; the AI's site scorer already values linked sites (`civilians.rs::choose_site`); the settle lens shows the 3-hex ring for Japan. Watch the Sakoku interaction (more cities, more border to patrol): that is a cost in the right direction.

### 2.4 Mongols: absorb the engineers, and keep the Keshig at Classical

Today: +1 move, slip past, Scout and civilian passage, the Yam; no Walls, no siege class, half damage to city HP, no terrain defence. The Keshig is moving to the Classical mounted slot (user, today). The 7.6% AI-only rate sits beside the user's experience that the Mongol rush is "almost too easy" to stop with a Scout and Walls, and that the 8.9.2 Mongol siege was the first competent AI assault: both say the Mongols are fine early and have no mid-game. The rename note flags the siege rules as unhistorical.

**Rule:** the Mongols cannot build siege units and deal half damage to city HP **until they capture a city that holds a Workshop (or Forge)**; from that turn they build siege and hit cities normally. The first developed city they take is their siege train, which is what Chinese and Persian engineers were for the khans. No Walls stays (nomads). **Counter:** unchanged early (Walls before the rush, watch with a Scout), and a new one that reads: do not let the Mongols take a Workshop city; Walls on the workshop towns first. **Verb:** learns siege by conquest. **Feasibility:** `faction_unavailable_reason` and `combat`'s Mongols city-damage rule read a per-player flag (`Player::siege_learned`) set in `capture_city`; the view publishes it; the AI's conquest valuation adds the unlock value to Workshop cities. Reserve: if the rush is still too strong for humans, the kurultai (wars lapse to Peace after 30 turns unless re-declared) is a drawback with a public clock.

### 2.5 Germany: the two-front problem replaces "Germans are slow"

Today: Ruhr Foundry +3 Production, new building families at 90%, continuous Industry projects (the verb: converts), Every City a Workshop; drawback: every unit −1 move. The Industry verb is good. The drawback's story is false (the rename note says so) and it also makes Germany the easiest raid target, which is why its AI "defends locally and never campaigns".

**Rule:** German units move at full speed **inside German territory and on roads** (the railway state) and −1 **outside it** (encircled, far from the depots). Same lever, true story (Moltke's timetables; the Schlieffen fear), and it pushes Germany toward Roadworks and fighting on its own ground, which is also where the Foundries are. **Counter:** unchanged (raid), plus "fight them abroad". **Feasibility:** `Map::move_cost` already knows the tile owner for passage; the Germany branch in `unit_stats_for` moves into the per-step cost. Optional second line later: era entry upgrades German units of the previous era at half Gold the same turn (the Prussian reforms), if Germany needs a mid-game verb.

### 2.6 Celts: keep the land, soften the razing, give them a settling engine

Today: attrition on Celtic land, hidden Ambushers (+3 ranged), Raise the Warbands; drawbacks: captured cities razed, melee −2, Ambusher +10 cost and −1 move. Attrition and concealment are real verbs (bleeds, hides) and should stay. Two problems: automatic razing flattens "several peoples into a stereotype" (the rename note), and a faction that cannot keep conquests has no expansion engine, while expansion is the loop.

**Rule:** replace raze-on-capture with **migration**: a Celtic city that reaches population 5 spawns a Settler at no Production cost (the population is consumed as usual), and captured cities are kept but lose their Walls family (the oppidum is rebuilt the Celtic way). **Counter:** snipe the Settlers ("always worth the war", the user) and settle the gaps first. **Verb:** spreads. **Feasibility:** a growth hook in `rules/cities.rs::grow` for the Celts, `capture_city` drops the raze and removes Walls/Castle/Star Fort instead; the AI's civilian planner already escorts and sites Settlers. Lower than 2.5 because the Celts' verbs already exist; this fixes the drawback's story and the missing loop.

### 2.7 Switzerland: no rule change; aim the AI at the hoard

Interest (uncapped, now under decadence), the Reserve, Great Works, the capture bounty: coherent, and the user confirmed it on 2026-10-17. The Swiss review's findings are AI findings: nobody targeted Basel (`victory.rs::hoard_denial_value`, `conquest.rs` target choice), the threat model missed an economic leader until 45% of the target. Fix those; do not touch the rules. The only rule worth a sim is the capital's bounty share (doubled today): if the hoard is still never attacked once the AI values it, raise the capital to 3x so the signature city is the obvious target.

### 2.8 Sparta: no rule change; the AI must convert the rush

Veterans, kill healing, 2 HP rest, desertion at peace, Honour the Veterans: the user plays it as intended (7 cities by T87) and the AI cannot (1.1 cities), for reasons the T87 review pins on `conquest.rs`'s hard-coded one-city target list, peace within five turns of a capture, and `threat.rs::plan_threat_defense` swapping Walls for a shooter in the *defenders*. Rules are not the lever. In reserve only: helots (upkeep in Food from the nearest city instead of Gold) if desertion ever proves unfun, never both.

### 2.9 Aztecs (not built): no change to the decision

Flower Wars, no mounted class, the New Fire. Ship with the mounted class as decided. One note from section 1: an Aztec unit hired by Byzantium brings Flower Wars with it (a kill grows the nearest *Byzantine* city if `origin` carries the rule), which is fine and funny; decide whether origin rules that touch the *owner's* cities (Flower Wars, Sparta's kill Gold) travel or stay home. Recommendation: combat rules travel (phalanx, wounds, concealment, Keshig mode), economic rules stay home (captives, kill Gold, capture heal).

### Summary table

| Rank | Faction | Change | Break feels real | Story | Asymmetry | Counter | Size |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Byzantium | Mercenary Companies; retire bought peace and Hire | Verb: hires | Exact | Only Gold-flow army | Wait, starve, shut the shop | Medium |
| 2 | Athens | Delian League (allies pay Athens; Athens never betrays) | Verb: collects | Exact | Only paid Alliances | Refuse, or join and leave | Small-medium |
| 3 | Japan | Cities 3 hexes apart | Verb: packs | Castle towns | Only dense founding | Wedge; many small cities | Small |
| 4 | Mongols | Siege by capturing a Workshop city; Keshig at Classical | Verb: learns by conquest | Fixes the misfit | Only conditional class | Wall the workshop towns | Small |
| 5 | Germany | −1 move only outside own land and roads | Same verb, true story | Fixes the misfit | Unchanged | Fight them abroad | Small |
| 6 | Celts | Migration Settlers; keep captures without Walls | Verb: spreads | Softens the stereotype | Only free Settlers | Snipe Settlers | Small-medium |
| 7 | Switzerland | None; AI targets the hoard | — | — | — | — | AI |
| 8 | Sparta | None; AI converts the rush | — | — | — | — | AI |

---

## 3. Questions for the user

Only the user can answer these; everything else above has a recommended default.

1. **Does the hire price go to the seller in full** (recommended: the sharpest choice), **half, or not at all?** If it goes to the seller, does it bother you that Byzantium can bankroll the leader by hiring its Knights?
2. **Contract 20 turns or Civ 6's 30?** And should Byzantium be able to **renew** a Company in a city for the hire price (a new unit action), or only hire again?
3. **May Companies capture cities** (recommended: yes), or should only Byzantium's own levies take possession ("the Tagmata")?
4. **Unpaid Companies turn barbarian** (the Catalan Company, recommended) or simply desert?
5. **Should Hire an army retire with bought peace** (recommended: it fails the same test), or stay as a minor cousin?
6. **Early game:** Byzantium can hire nothing until it has met someone at Peace, so its first 20 to 30 turns are two levies and Walls. Acceptable, or should the shop open with a generic "Pecheneg" Horseman and Spearman from turn 1?
7. **Diplomacy lane:** with Byzantium in the mercenary lane, may Athens take the Delian League, or do you want Athens to stay a Science faction (then "the Lyceum reaches ahead")?
8. **Japan's density:** 3 hexes apart for Japan only, or is a literal hive too much map? (The AI bug comes first either way.)
9. **Mongol siege by capture:** worth the extra rule to fix the history, or keep the clean "no siege" line?
10. **Celtic razing:** keep it (clean, works) and only reword the lore, or replace it with migration?
11. **Origin rules:** should a hired Spartan company heal Byzantium's units by killing, or only fight as a phalanx (recommended: combat rules travel, economic rules stay home)?

---

## 4. Sources

- This repository: `game-rules.md` (sections 1, 3, 5, 8, 9), `crates/core/src/economy.rs::levy_count,levy,levy_check,buy_cost_for`, `crates/core/src/data.rs::levy_cap,pays_upkeep,FREE_MILITARY_UNITS,UPGRADE_COST_PERCENT`, `crates/core/src/rules/cities.rs::bankruptcy_victim`, `crates/core/src/rules/mod.rs::sparta_restlessness`, `crates/core/src/state.rs::Unit`; `docs/sim-results/2026-10-17-faction-winrates-1017.md`; `docs/learnings.md` (2026-10-17 entries); `.claude/skills/legatum-coach/SKILL.md` (8.9.2 Swiss and Sparta T87 findings); `docs/design/research/rule-breaks-2026-10-17.md`.
- Civilization VI city-state levying (30 turns at Standard, Gold equal to the units' Production cost, you pay upkeep, units you have not discovered, cannot delete levied units): https://civilization.fandom.com/wiki/City-state_(Civ6)
- Age of Empires IV Byzantines (mercenaries from other civilizations bought with olive oil at the Mercenary House): https://ageofempires.fandom.com/wiki/Byzantines_(Age_of_Empires_IV)
- Byzantine army (Komnenian era): 500 regulars in 1091, Cuman allies at Levounion, foreign mercenaries as permanent units: https://en.wikipedia.org/wiki/Byzantine_army_(Komnenian_era), https://en.wikipedia.org/wiki/Byzantine_army
- Varangian Guard (Basil II's 6,000 Rus, Anglo-Saxons after 1066): https://en.wikipedia.org/wiki/Varangian_Guard
- Byzantine diplomacy (tribute, subsidies, hostages): https://en.wikipedia.org/wiki/Byzantine_diplomacy
- Delian League (phoros, the revolts of Naxos, Thasos, Samos): https://en.wikipedia.org/wiki/Delian_League
- The Catalan Grand Company in Byzantine service (1303 to 1305) and its war on the empire: from general knowledge, not fetched; check before quoting in the Guide.
