# Siege rework: why siege units are a waste today, and what to do about it

Design research, 2026-10-18. Nothing here is decided; decisions go to the design doc's decisions log and the `legatum-design` and `legatum-rules` skills. Every option is checked against `.claude/skills/legatum-rules/references/game-rules.md` (one order per unit per turn, simultaneous resolution, the split counterattack, the city shielding its garrison) and simulated in the combat lab on the real engine (`docs/sim-results/siege/`). No game rule or AI code changed: the levers below are opt-in rule experiments that only the lab switches on.

**The prompt (user, 2026-10-18):** "Siege units seem like a total waste of time. I've never built them and I don't imagine the AI does either... they're weak units that are easily killed by the walls firing back, and they don't appear to do more damage to walls than a handful of archers, which are more versatile... In Civ, it's pretty dang hard to take down walls without siege equipment, so it almost becomes a requirement, but once you have siege equipment, you can take down the walls pretty quick... Walls are not intended to make cities impenetrable, but instead they're intended to make it more costly and slower."

**The user's lead idea (same day):** "Instead of making separate units for siege (catapults, ladders), what if we make 'siege' a support class. Like a 'siege engineer'... If it's next to a melee unit, the walls can be breached by that melee unit... and when sitting next to ranged units, it gives the ranged unit it's next to 'siege' capability so it has no negative siege penalty. It feels like that would lead to less complexity in the game because it doesn't require an entire new unit class or way of fighting; it just affects bonuses and penalties for existing units." And later: "The more I think about it, the more I like the idea of 'military engineer' or 'siege engineer'. They could also do other war-related things like 'build roads' or other kinds of fortification... It changes the tech from 'new types of units available', which adds complexity, to 'the military engineer got this new ability', which is simpler."

**In one paragraph:** the user is right on every count (§1). A Catapult does 7% more wall damage per Production than Composite Archers, dies in nearly every fight, and the AI builds under 0.1 a game because its own value scoring correctly sees a worse archer. The cause is that Walls are a +10 strength number, not a kind of defence, so no unit can be good against them. The fix is a baseline in which Walls halve all damage to the city's HP (§2) and one **Military Engineer** support unit whose neighbours ignore Walls (§6), replacing the Catapult and Bombard. In 517,000 simulated fights (§4) that gives the gap the user described: a four-unit melee army takes a typical walled city **0%** of the time without an Engineer and **44%** with one (today: 35%), while archers alone can still breach late and slowly. The Engineer cannot be shot by the walls, only captured by a sally, which makes the defender's counter active. §7 specifies the AI's "rush open cities, retarget when walls rise, bring an Engineer" as value scoring.

Contents:
1. Diagnosis, with numbers: the user is right, and why
2. What walls should do (the baseline every option needs)
3. The options
4. What the simulations say
5. Recommendation, ranked
6. The Military Engineer, specified
7. AI decision logic, as genes
8. Questions for the user
9. Sources and method

---

## 1. Diagnosis: the user is right, and the cause is structural

### 1.1 What a Catapult does to a wall, against what archers do

The engine has one damage formula for everything: `30 · e^((own − other)/25)`, rolled 80–120%. A walled city of population 3 defends at **26** (City 10 + Population 6 + Walls 10); unwalled, 16. The Catapult's only siege feature is a flat **+12 strength** when it shoots a city. Average rolls, full-HP city:

| Unit (era) | Production | Strength / ranged | Damage to a walled city per shot or hit | **Per 100 Production** | The city's strike on it (per turn) |
|---|---:|---|---:|---:|---:|
| Archer (Ancient) | 66 | 8 / 15 | 19 | 29 | 62 |
| Spearman melee (Ancient) | 40 | 20 | 24 (takes 38 back, split) | 60 | 38 |
| **Catapult** (Classical, +10 Coal, 1 move) | 120 | 10 / 22+12 | **41** | **34** | **57** (dead in two) |
| Composite Archer (Classical) | 84 | 15 / 23 | 27 | 32 | 47 |
| Phalangite melee (Classical) | 55 | 26 | 30 (takes 30 back, split) | 55 | 30 |
| **Bombard** (Renaissance, +Coal, 1 move) | 170 | 14 / 34+16 | **78** | **46** | 48 |
| Arquebusier (Renaissance) | 136 | 29 / 39 | 50 | 37 | 27 |
| Musketman melee (Renaissance) | 105 | 38 | 48 | 46 | 19 |

So at equal Production a Catapult does **7% more** wall damage than Composite Archers (34 against 32 per 100), moves half as fast, needs Mathematics and 10 Coal, cannot shoot anything else well (22 ranged against a Phalangite's 26 defence), and dies to the city's own strike in two turns because its defence is 10. Against a Castle (pop 5, strength 35) the ratio is 1.07 again. The Bombard is 25% better than the Arquebusier per Production, still not "required". Per Production, **plain melee is the best wall-breaker in the game** (55–60 per 100), which is the opposite of Civ and of history.

The city's strike always picks the lowest-HP military unit in range 2 with line of sight. A siege engine stands at range 2 to fire, so it is always in range; once wounded it is the lowest-HP unit and eats every following strike.

### 1.2 Why: Walls are a number, not a kind of defence

Walls add +10 strength and let the city fire. Nothing in the rules distinguishes an arrow, a spear or a stone ball hitting a wall, so there is nothing a siege engine can be *good at*. In Civ 6 walls are a separate outer-defence pool that melee damages at 15% and ranged at 50%, while siege does full damage: the wall is a different problem, and siege is the tool for it. Here the siege class has a seat in the data (`UnitClass::Siege`, indirect fire, the flat city bonus) but no job. The +12 is worth a factor `e^(12/25) = 1.6`, which the Catapult's 43% higher price and worse survival eat.

### 1.3 The AI never builds them, and for the same reason

- Telemetry (`hard_vs_human` `strategic_built`, stockpile study of 2026-10-18 in `docs/learnings.md`): **under 0.1 siege units per empire per game**, against 1.8 Pikemen, "because the AI's production values never ask for them."
- `production.rs::composition_bonus` scores only Melee, Ranged and Mounted; siege returns 0. The siege kind bonus (+500) is gated by `t.siege * 5 >= t.army` (one per five units), a reachable enemy city, three melee first and no threat.
- `walled_quota` (on since 2026-10-18) is the AI's actual answer to walls: **four shooters a tier ahead of the walls**, the Golden Horde pattern that took Aarau. It asks for the ranged class, not siege, and that is the right call under today's rules.
- The siege forecast (`forecast_siege`, `tactics.rs`) treats a Catapult as a shooter with +12; nothing in the plan layer says "this city has Walls, bring siege".

So the AI's behaviour is not a bug; it is the correct reading of rules in which siege is a worse archer. Fixing the AI without fixing the rules would make it build a unit the user is right to ignore.

### 1.4 What the combat lab already showed about walls

Stage 2 (`docs/sim-results/combat-lab/results.md`, live defender, 41,580 situations): with the city in sight, a storm takes an **unwalled** city in 35% of situations and a **walled** one in 26%; against a 4–6 unit army, Walls move the storm from 87% to 73% (empty city) and from 51% to 33% (shooter garrison), at roughly 1.3x the Production lost. Walls are a ~10-point deterrent and a price, not a wall. The user's "a strong deterrent that forced me to change tactics" (8.4) is the +10 and the strike at work against a small army.

---

## 2. What walls should do: the baseline

The user's intent, in one line: **without siege, walls make a city much slower and costlier to take, but not impossible; with siege, the walls stop mattering much and the fight is the garrison.** The AI logic that follows ("rush open cities; when walls go up, move to the next open city or bring siege") only emerges if those two states are far apart.

The cheapest rule that creates the gap, in the engine as it stands:

> **Walls: attacks on this city's HP do half damage, except from units beside a Military Engineer** (or: except from siege engines).

One sentence on the Walls card. Melee and ranged are treated alike (Civ 6 punishes melee harder, 15% against 50%; one number is simpler and the sims below test 50% and 25%). The Walls' +10 strength and the city strike stay: the wall still hurts attackers with siege, which is the "costlier" half of the user's intent. Unwalled cities are untouched, so the rush era (user, 2026-10-03: "before Walls, units counter rushes") is unchanged.

Why the city HP pool rather than a separate wall pool as in Civ 6: a second HP bar is a new system (UI, forecast, AI, art) and §9 of the rules forbids systems that do not earn their place. Halving damage into the existing pool has the same effect on time-to-breach and reuses the breakdown line players already read ("Walls: ×½").

What the halving does by the formula (pop-3 walled city, full HP): a Composite Archer falls from 27 to 13 per shot; a Phalangite from 30 to 15, while still taking the full 30 counterattack and the strike. Four Phalangites need about five turns to break 200 HP instead of two to three, under fire the whole time. Six archers at range need four turns and lose one to the strike each turn. Archers *can* still breach; it costs them.

---

## 3. The options

All keep one order per unit, simultaneous resolution, and the Civ-6 city shield. "Lifts the penalty" means attacks from the covered unit do full damage to the walled city's HP.

### Option A. The Military Engineer (the user's idea; lead option)

A support unit, not a combatant. **"Your units next to a Military Engineer fight a city as if it had no Walls."** Catapult and Bombard retire. Its effect scales with the units it stands beside, so it needs no era versions; later technologies give the same unit new abilities instead of new unit kinds (§6.3), and the Mongols' siege ban becomes "no Military Engineers until they capture a city with a Workshop". Full specification in §6. Two readings are simulated: **A1** the Engineer lifts the damage halving only ("attack a walled city at full strength"); **A2** it also leaves the Walls' +10 out of the city's defence against covered units, while the city still strikes ("as if it had no Walls").

*Historical root:* the Roman army carried its engineers and built its rams, towers and ramps on site (Masada, Jotapata, Alesia); medieval sieges hired master engineers (Master Bertram for Edward I at Stirling, Urban's bombard crew for Mehmed II at Constantinople). The engineer, not the engine, was the scarce asset.

*What it costs the attacker:* one more unit to build and keep alive, a turn to bring it into place (adjacency is read when a fight is planned, so an Engineer moving up this turn covers next turn's attack), and exposure: it can be captured by a sally.

*Defender's counters:* the Walls' +10 and the strike still work; the garrison can sally and capture the Engineer (it is a civilian: a melee unit stepping onto it alone takes it, as with a Settler); a mounted garrison unit reaches one three tiles out. A ranged garrison cannot shoot it (lone civilians are not ranged targets), so the counter is active, not passive.

### Option B. Civ 6 split with real engines

Walls halve non-siege damage; **siege engines do double damage to a city's HP** (replacing the flat +12/+16; "Catapult: double damage to a city's HP"), no Coal, and optionally one of two protections: **B2** the city strike never targets siege engines (crews behind mantlets), or **B3** +5 defence. A Catapult then does 52 per shot against a Composite Archer's 13: four times the wall damage per unit, 2.8x per Production, Civ's ratio. Keeps two unit kinds and their art, upgrades and the Coal/Iron economy; keeps the "siege dies to the strike" problem unless B2 or B3 is taken.

### Option C. Ranged-only penalty

Walls halve ranged damage only; melee storms as today. The Engineer (or engines) lifts it for archers. The smallest change, and it keeps the user's 8.4 experience exactly (melee pays through walls with losses). It does not create the "nearly required" feel for melee rushes, so the AI's retarget logic has less to bite on.

### Option D. Rams and towers as two support units (Civ 6 literally)

Battering Ram (Masonry): adjacent melee do full damage to walled city HP. Siege Tower (Mathematics): adjacent melee fight the garrison directly, ignoring the city's HP. Two units, two rules, the Tower's rule interacts badly with the overflow and shielding rules (a Tower plus three Spearmen captures a full-HP walled city in one turn if the garrison dies). The Engineer is this collapsed into one unit with one rule; D is not simulated separately.

### Option E. Out-range the wall

Siege engines fire at range 3, beyond the city strike's 2. Historically apt (trebuchets and bombards outranged wall engines) and it fixes the fragility problem without a protection rule, but range 3 needs a spotter (sight is 2), so a second rule arrives with it. Parked; it would be a Bombard-era flavour if engines survive.

### Levers that apply to any option

- **Remove the Coal cost** on Catapults (wood, rope and sinew; Coal is a Renaissance input). The stockpile study found Coal piling up unused. Under Option A the question disappears with the unit.
- **Era scaling.** Multiplicative rules (half, double) scale through the exponential formula on their own; flat bonuses do not. The Engineer scales through the units it stands beside.
- **Walls tiers.** Castle (+5) and Star Fort (+5) keep adding strength; they do not deepen the halving. The Bombard era (gunpowder ending castle walls) is represented by the Musketman's and Arquebusier's higher strength against a Castle's +15, not by a special rule.

---

## 4. What the simulations say

Lab: `crates/sim/examples/combat_lab.rs --siege` on the cloud box, 3,232 situations x 32 dice seeds x 5 attacker policies = **517,120 fights in 30 s** (about $0.09 of spot time). One city of population 3 at 200 HP on flat ground, the attackers two hexes out, a live Hard defender with 300 Gold, real city orders and real tactics (combat lab stage 2). Axes: era (Ancient, Classical, Renaissance), attacker (generic, Sparta), army (3, 4 or 6 units: all melee; half archers; one melee and archers; half siege engines; one siege engine), Walls or not, garrison (none, 1 melee, 1 shooter, 2 shooters), and ten rule variants. The Engineer is a real Settler marching with the army (same class and capture rule as the proposed unit), moved by a policy that keeps it beside the units about to attack. Full tables: `docs/sim-results/siege/results.md`; charts `options.png`, `cost.png`, `archers.png`, `engineer.png` (portrait, phone-readable). Capture figures take each cell's better forced policy (storm now, or shooters first); the AI's own column is shown separately because the AI does not know the new rules.

| Variant | Rule under test |
|---|---|
| base | today's rules |
| r50 | archers do half damage to a walled city's HP |
| a50 | archers and melee both do half |
| a50_cat2 | a50, and siege engines do double damage to city HP (replacing +12/+16) |
| a50_cat2_spare / _mantlet | a50_cat2, and the city strike never targets engines / engines get +5 defence |
| a50_eng (A1) | a50, and units beside the Engineer ignore the halving |
| a50_eng2 (A2) | a50_eng, and they also ignore the Walls' +10 |
| a25_eng | both classes at 25% without an Engineer; the Engineer lifts it |
| r50_eng (C) | only archers penalised; the Engineer lifts it |

### 4.1 Today's rules: the diagnosis in the lab

| Classical, walled city | Army cost | Capture | Turns | Production lost | Engines lost | Production per 100 HP of wall damage |
|---|---:|---:|---:|---:|---:|---:|
| 4–6 melee | 238 | 45% | 3.3 | 182 | – | – |
| half archers | 296 | 45% | 2.7 | 216 | – | – |
| one melee + archers | 335 | 27% | 3.5 | 274 | – | **115** |
| melee + one Catapult | 340 | 63% | 3.0 | 254 | **1.0 of 1** | 144 |
| half Catapults | 368 | 51% | 2.6 | 256 | 1.6 of 2 | 177 |

- **The Catapult dies in essentially every fight** (1.0 lost per fight with one engine; 1.6 of 2). Per Production it is the worst wall-breaker in the game (177 Production per 100 HP of damage against the archers' 115). The Bombard: 238 against 145.
- A Catapult still raises a melee army's capture rate (45% to 63%) because its shots take no counterattack and it draws fire from the melee; so does an archer, cheaper. It is a 120-Production consumable, not a siege weapon.
- **The city strike is not what kills it.** `a50_cat2_spare` (the strike never targets engines) is identical to `a50_cat2` to the decimal: the strike shoots the lowest-HP unit in range, which is a wounded melee unit; engines die to the garrison's shooters and to sallies after the melee are gone, 1 move away from safety. +5 defence changes nothing either. Protection rules are not the fix.
- **Walls today cost 10 to 16 capture points and 20–30% more losses** (Classical melee 61% open to 45% walled; Renaissance 85% to 71%). Renaissance armies take a pop-3 walled city at 71–78% because the +10 does not grow with the era: a Musketman at 38 against a 26 city. Castle (+5) and Star Fort (+5) are the era answer, and a pop-3 city with Ancient Walls in the Renaissance should fall.

### 4.2 The option grid

Walled cities, pooled over eras, factions, army sizes and garrisons (296 cells each; Engineer variants 216, since engines are not run under them):

| Variant | Capture | Turns | Attacker Production lost | Production lost per capture | Breach (city reached 0 HP) | Defender spent |
|---|---:|---:|---:|---:|---:|---:|
| base | 52% | 2.9 | 244 | 473 | 73% | 123 |
| r50 | 44% | 3.0 | 264 | 598 | 64% | 118 |
| a50 | 28% | 3.5 | 301 | 1,084 | 43% | 111 |
| a50_cat2 (B) | 29% | 3.4 | 299 | 1,044 | 44% | 112 |
| a50_eng (A1) | 32% | 3.5 | 266 | 820 | 54% | 114 |
| **a50_eng2 (A2)** | **45%** | 3.2 | 246 | 547 | 72% | 118 |
| a25_eng | 24% | 3.8 | 286 | 1,204 | 42% | 110 |
| r50_eng (C) | 41% | 3.2 | 250 | 613 | 62% | 117 |

Capture by army, same pooling:

| Variant | melee | mixed | shooters | half engines | one engine |
|---|---:|---:|---:|---:|---:|
| base | 49% | 50% | 37% | 64% | 75% |
| a50 (walls alone) | 20% | 22% | 14% | 57% | 44% |
| a50_cat2 (B) | 20% | 22% | 14% | 61% | 47% |
| a50_eng (A1) | 36% | 35% | 27% | | |
| **a50_eng2 (A2)** | **52%** | **49%** | **34%** | | |
| r50_eng (C) | 49% | 46% | 27% | | |

**The typical cell: four units against a walled city with one shooter inside.**

| Variant | melee | mixed | shooters | half engines | one engine |
|---|---:|---:|---:|---:|---:|
| base | 35% | 34% | 35% | 73% | 66% |
| a50 (walls, no siege) | **0%** | 4% | 15% | 56% | 23% |
| a50_cat2 (B) | 0% | 4% | 15% | 77% | 25% |
| a50_eng (A1) | 23% | 16% | 33% | | |
| **a50_eng2 (A2)** | **44%** | 31% | 40% | | |
| r50_eng (C) | 39% | 30% | 34% | | |

Reading it against the user's intent:

1. **Halving all non-siege damage (a50) makes walls what the user described.** A four-unit melee or mixed army goes from a one-in-three chance to 0–4%; attacker Production lost per capture more than doubles (473 to 1,084); the city is breached in 43% of fights instead of 73%. Walls are now a reason to go elsewhere.
2. **The Engineer closes the gap, and A2 closes it fully.** With the Engineer beside them, four melee take the typical walled city 44% of the time (A2), slightly above today's 35%, because the +10 is gone for covered units while the city still strikes. A1 (halving lifted, +10 kept) recovers only to 23%: the Walls' +10 is half the deterrent, so if the Engineer is to make walls "not matter much", it must negate both. A2 pooled (45%, 547 Production per capture) sits just under today's walled numbers (52%, 473), which the user judged "strong but beatable" in 8.4. The Engineer's own cost (a unit, a setup turn, and 75 Production in these figures) is the difference.
3. **The one-turn setup is real and visible in the numbers.** Turn-1 attacks are never covered (the Engineer must already stand beside the attacker when the fight is planned); A2 still reaches base because the +10 removal pays for the lost turn. That is the Civ 5 "siege sets up for a turn" feel with no extra rule.
4. **Archers alone can still breach, slowly and late.** Shooter armies against walls under a50: Ancient breach 5% (capture 0%), Classical 27% (capture 1%), Renaissance 75% (capture 43%). With an Engineer (A2): 41% / 73% / 99% breach, 0% / 24% / 79% capture. Without one, arrows alone need many turns and lose about four units doing it; with one, the archers-first siege is the strongest Renaissance plan. Walls are not impenetrable; they are a wall of cost.
5. **Better engines (B) do not rescue the engine.** Doubling engine damage lifts the two-engine army from 57% to 61% (77% in the typical cell) but leaves the one-engine escort at 47% and the engines still die every fight; a melee army without engines is at 20%. B makes walls a two-Catapult problem, A makes them a one-Engineer problem that any army can solve. A also removes two unit kinds, their art, Coal, upgrades and AI special cases.
6. **Ranged-only (C) is a half measure.** r50_eng leaves melee storms at today's 49%, so walls do not change the rush decision; it only penalises the archers-first plan.
7. **25% is too far.** a25_eng recovers only to 27% melee; the Engineer lag then dominates. 50% is the right first number.
8. **The Engineer was captured in under 1% of fights.** The lab's defender AI does not know to sally for it, so these are upper bounds on the attacker; the defence spec in §7 adds the sally. The exposure is real: in the traces the Engineer stands on the city ring, one step from the garrison.
9. **Sparta** gains most from A2 (33% under walls alone to 55% with the Engineer, against the generic 23% to 36%): wound-immune Hoplites next to an Engineer are the rush faction's late answer to walls, which fits its identity.
10. **The AI's own tactics** under A2 take the Classical melee cell 23% against the forced policy's 53%: it does not know the rule, and its siege forecast still prices the halving. §7's forecast term is not optional.

What the lab cannot say yet: whether a human defender who sallies for the Engineer, or a Castle, changes the picture (no Castles in this grid); how often an AI rusher actually switches targets when walls rise (a war-scenario question, §7); and the Engineer's right price (75 here, 60 proposed).


---

## 5. Recommendation, ranked

1. **The Military Engineer with A2 rules (recommended).** Walls halve all damage to the city's HP; units beside a Military Engineer fight the city as if it had no Walls (no halving, no +10; the city still strikes); Catapult and Bombard retire; the Engineer is a 60-Production civilian unlocked at Mathematics (the Catapult's slot), no Coal. It gives exactly the three states the user asked for: open city, rush it (unchanged); walled city without an Engineer, nearly hopeless for a small army and slow for a large one (0–20%); walled city with an Engineer, roughly today's fight (44–52%), with the garrison and the strike as the remaining price. One new unit, two one-sentence rules, two unit kinds removed, and a unit whose later abilities can grow with technology instead of new unit kinds. The defender's counter is a sally, not a passive strike.
2. **The same with A1 rules** (the Engineer lifts the halving only, Walls keep their +10 against everyone) if the user wants walls to keep more bite against a prepared attacker: melee 36% pooled, 23% in the typical cell. A fair second choice; the wording "attack at full strength" is the plainest of all, but the sims say walls then still beat a prepared four-unit army two times in three.
3. **Civ 6 split with real engines (B)**, if the user prefers to keep Catapults and Bombards as pieces: halve non-siege damage, engines do double, drop the Coal. Works for two-engine armies, leaves the engine a consumable and keeps two unit kinds, era upgrades and AI special cases. Choose only if the engine's art and flavour are wanted for their own sake.
4. **Ranged-only penalty (C):** rejected; it does not move the rush decision.
5. **25% baseline:** rejected at this stage; retune later if walls prove too weak against Engineers in playtests.

Era note: Walls (+10) do not scale; the Castle and Star Fort tiers do. Under A2 the Engineer negates the Walls line only. Whether it should also negate Castle and Star Fort lines, and at which tech, is the one scaling decision left (§6.3 proposes Metallurgy for the Bombard moment).


---

## 6. The Military Engineer, specified

Everything here is checked against the engine as it stands. The first version (the skateboard) is the siege ability alone; §6.3 is the roadmap the user asked for, each later ability one sentence with the technology that grants it.

### 6.1 The unit and the siege ability

| Question | Answer | Why |
|---|---|---|
| What is it | A civilian-class unit (`military: false`, 0 strength, 2 moves), like a Settler without the Found City order | Civilians already exist: the city strike ignores them (`is_military` filter), ranged fire cannot target a lone civilian, a melee unit stepping onto a lone hostile civilian captures it, and a civilian may share a tile with its own empire's military unit (`movement.rs`: "different classes may stack only within a permanent team"). No new class of movement or combat. |
| The rule (A2) | **"Your units next to a Military Engineer fight a city as if it had no Walls."** One sentence on its card; the Walls card says "attacks on this city's HP do half damage, except from units beside a Military Engineer". | Two breakdown lines players already read: the Walls' +10 on the city's defence is dropped for a covered attacker, and the "Walls: ×½" line is dropped. The city's strike is the Walls' own action, so it is unaffected; the forecast card says "the city still fires". A1 is the same unit with the plainer card "attack a walled city at full strength" and the +10 kept. |
| Next to, or under | Both: the Engineer covers every own unit on its tile and the six around it. | The user said "next to"; the engine allows stacking under own military, and a stacked Engineer is protected by the unit above it, which is a legitimate choice for the attacker (safer, covers fewer tiles). Simulated as "distance ≤ 1". If the user wants "next to" only, drop the tile itself; the sims barely change since the lab's Engineer mostly stands on the ring. |
| Does it need to see or touch the city | No. Adjacency to the attacking unit, at the moment the fight is planned (start of the ranged phase or of that movement round). | Simultaneous turns: a unit that moves up this turn does not count for this turn's fight. That is the "setup turn" Civ 5 gave its siege, free of a rule. |
| Can the city strike shoot it | No: the strike targets military units only. | This is the point: the old "siege dies to the walls" problem is gone. |
| Can a ranged garrison shoot it | No (lone civilians are not ranged targets). | Keeps the counter active: someone has to step out. |
| How is it lost | Captured by an enemy military unit stepping onto its tile when no own military unit stands there; a captured Engineer belongs to the captor (as a Settler does). Destroyed with the city if its tile is in a razed city? n/a (it never stands in an enemy city). | Capture, not kill, is the historical and Civ-familiar fate of siege works, and it gives the sally a prize. |
| One Engineer covers | Every adjacent own unit; several Engineers do not stack. | Binary rule, nothing to tune. |
| Cost, tech | 60 Production at **Mathematics** (Classical, the Catapult's slot; about nine turns after Walls at Masonry), no strategic resource, 1 Gold upkeep like a military unit so it is not free to hoard. Alternative: Masonry, with Walls. | Mathematics keeps two settled decisions intact: "once most cities have Walls, rushing cities is over" and "early rushes are final; the counter is scouting, then Walls and archers" (user, 2026-10-03 and 2026-10-17). In the Ancient era Walls end the rush (four Spearmen: 4%); from the Classical era a walled city is costlier and slower (an Engineer, a setup turn, the strike) but open. Masonry would make Walls never end anything. See the questions. |
| Era scaling | Automatic: it lifts a percentage penalty on whatever stands next to it. | No era versions, no upgrades, one art asset. |
| Factions | Mongols: cannot build it; gain it when they capture a city with a Workshop (the user's rule). Their existing half damage to city HP becomes the walls question in §8. Sparta: counts as a war build (exempt from the +50% civilian premium, as the Walls family is). Celts: ordinary. Swiss: ordinary; their Customs Bastion (15) stays stronger. | Reads the class, as the other faction unit rules do (one line in `faction_unavailable_reason`). |
| UI | A unit card line, a breakdown line in every city fight and forecast, and a lens/badge on tiles an Engineer covers (so "the next action is obvious": park it where the glow touches your attackers). | Pillar 5 and 9. |
| Name | **Military Engineer** (the user's working name), one unit for the whole game. Era flavour goes in the art and the research cards ("Military Engineers can now…"), not in renamed units; "Siege Engineer" would stop being true once it lays roads. | One unit, one name, one card. |
| Saves | Existing Catapults and Bombards in old saves stay as legacy units until they die or are disbanded (`unit_stats` keeps the kinds, nothing can build them); no conversion. | The wire format and saves stay stable, as every other rename did. |

Defender's play: build Walls and a ranged unit as now; add one melee or mounted unit whose job is the sally, and use the city's strike on the attackers the Engineer is standing beside (they are adjacent to the city). An Engineer caught alone is a 60-Production gift and a siege delayed by the turns it takes to bring another.

### 6.2 What changes on the tech tree

| Technology | Today's card | Proposed card |
|---|---|---|
| Masonry (Ancient) | Walls | Walls (unchanged; the Walls card gains its one sentence) |
| Mathematics (Classical) | Catapult; Phalangite | **Military Engineer**: units next to it fight a city as if it had no Walls; Phalangite |
| Metallurgy (Renaissance) | Bombard (Catapults upgrade to it); Cuirassier | Cuirassier, plus the Engineer's siege-guns ability if adopted (§6.3) |
| Military Engineering (Renaissance) | Star Fort; unit upgrades cost half | unchanged |

Retired: Catapult (`SiegeEngine`) and Bombard, their Coal costs and upgrade line, the flat "Siege against city HP" bonus (`data::siege_city_bonus` keeps the Bomber's +20), the `UnitClass::Siege` build gate for the Mongols (it becomes the Engineer gate), the AI's siege kind bonus and the proxies' Catapult counts. The research cards and unlock art already show "what you got"; each later Engineer ability is one more such line on an existing card, which is the simplification the user named.

### 6.3 Roadmap: abilities the same unit gains later (not in the first version)

Each is one sentence on the research card, checked against the decisions it touches. None is needed for the siege rework to work; they are listed so the unit's shape is decided once.

| Technology | Sentence on the card | Decisions it touches |
|---|---|---|
| Engineering (Classical) | "A Military Engineer that spends its turn on a tile lays a road there." | **Roads today come only from the Mongol Courier Hall and the Roadworks building; there are no Builders.** A one-tile-a-turn military road is logistics, not a yield improvement, and it is slow enough that the Hall's instant network and Roadworks' 8-hex links stay special. It needs a decision (question 6) because it bends the "roads only from the Hall" rule and touches the Mongol identity. It must never lay anything but roads, so it is not a Builder back door. |
| Feudalism (Medieval, with the Castle) | "A Military Engineer that spends its turn on a tile builds a Fort there: units on it defend as on hills." | A new tile feature (map state, art, one defence line). Fortify already exists; a Fort is a permanent, placeable version of the hills bonus. Historical (castra, motte-and-bailey); Civ-familiar (the Fort). Defensive counterpart to the siege ability, so a turtle has something to build with the same unit. |
| Metallurgy (Renaissance, the Bombard's slot) | "Units next to a Military Engineer do double damage to a city's HP." | The gunpowder moment (Urban's guns at Constantinople) as an upgrade to the same unit instead of a Bombard. It also answers the Castle (+5) without a special negation rule. Scales through the exponential formula like every multiplicative rule. |
| Military Science (Industrial) | "A Military Engineer that spends its turn digs entrenchments: units on its tile and next to it get +6 defence until they move." | Optional; only if Industrial sieges need a defensive counterweight to the gun ability. A per-unit state, so it costs more than it looks; park it. |

Not proposed: airfields (air is based in cities), bridges (rivers are passable fords in this game), repairing walls (city HP heals on its own), or any tile improvement with a yield (that is the city's production list by decision).

---

## 7. AI decision logic, as genes

The user's logic for a rush faction: (1) rush cities without walls; (2a) when walls rise, shift to other cities without walls; (2b) or bring siege for the walled one. All by value scoring from public information (walls are public on explored tiles: `CityView.walls`), never a rule.

| Step | Where | Spec |
|---|---|---|
| **Walls as a price in target scoring** | `conquest.rs::choose_conquest_city`, `public_softness`; `campaign_forecast`/`forecast_siege` | The capture forecast models the Walls halving and the Engineer's lift, so a walled city without an Engineer in the army forecasts about twice the turns and losses. Targets compete on `value − cost`, where cost = forecast turns and losses (already) plus the Engineer's Production if one is needed and absent. The hard rule "Sparta skips walled cities in the first third" (`choose_conquest_city`) and `sparta_walled_prize` become unnecessary: an open city simply forecasts better. Gene `walls_as_cost` (0/1) switches the new forecast term on. |
| **Retarget when the target walls up** | same scorer, every turn | `choose_conquest_city` is re-evaluated each turn from the view. When the current target's `walls` flips to true mid-campaign, its forecast doubles; another open city within the campaign's reach (the existing 7–10 hex radius) now scores higher and the campaign re-aims. Existing hysteresis (plan switch reasons) stops dithering. No new code beyond the forecast term; add the switch reason "target walled up" to the plan log so the coach can see it. |
| **Bring an Engineer for walled targets** | `production.rs::walled_quota_value`, new term | Gene `engineer_for_walls` (0/1, default 1 once built): while our plan's war marches on a walled city and the army has no Engineer (or it is more than N turns behind the front), one Engineer is worth what `walled_quota` gives a shooter (counter-class value), so it is built in the city nearest the front. Cap one per campaign; a second when the army has 6+ units. Keep `walled_quota`'s four shooters: a shooter line that is covered does full damage from range, which is the Golden Horde pattern made cheaper. |
| **Move it with the army** | `military.rs` staging and assault passes | The Engineer is assigned to the campaign's melee group and ordered each turn to the tile adjacent to the most units that will attack next turn (the lab's `engineer_orders` is the reference policy: count of covered attackers first, then farther from the city, then nearest). Never end a turn where an enemy military unit could reach it unescorted; prefer a tile under an own melee unit when a mounted enemy is within 4 hexes. Reuse the Settler escort logic (`settler_life`, `settler_raid` defence). |
| **Commit timing** | `city_assault_succeeds`, the ≤3-turn commit | Unchanged threshold; the forecast now includes the Engineer, so the AI waits the one setup turn (Engineer in place) and then commits, exactly the storm-with-engineer policy the lab measures. |
| **Defend against it** | `threat.rs::plan_threat_defense`, `tactics.rs` | A walled city with an enemy Engineer in sight is not "done": value a melee or mounted sally when the capture forecast says the sallying unit survives the next strike round (`Engineer cost + turns it adds to the siege` against the unit's risk, as `settler_raid` already prices a Settler capture). Gene `sally_engineer` (0/1). Captured Engineers are kept, not disbanded. |
| **Mongols** | `faction_unavailable_reason`, goals | Until they hold a Workshop city, Mongols's scorer sees every walled city at the halved forecast, which is the designed drawback; capturing a Workshop city is worth the Engineer's value to it. |
| **Measurement** | war scenarios, human-seat benchmark | Add three war scenarios: an open city walls up mid-campaign (does the AI retarget), a walled city with the Engineer available (does it bring one and take the city), a defender facing an Engineer (does it sally). Gate as usual: seat-only not below −2 pp, Golden Horde not below 100%. |

---

## 8. Questions for the user

1. **A1 or A2?** Should the Engineer only lift the halving (A1: "attack a walled city at full strength"; walls keep their +10 against everyone, four melee 23%) or make covered units fight as if the city had no Walls (A2: the recommendation, 44%)? The sims say A2 gives the gap you described; A1 keeps walls meaner against a prepared attacker.
2. **Half for both melee and ranged?** The recommendation halves both (a50). Civ 6 punishes melee harder (15%) than ranged (50%); the sims say ranged-only (r50) does not change the rush decision and 25% is too far. Is 50% for both the right first number?
3. **Mathematics or Masonry?** Mathematics (recommended) keeps "Walls end the Ancient rush" and makes walls costlier-and-slower from the Classical era on. Masonry gives the rush an answer at once and makes Walls never decisive. Which do you want the early game to feel like?
4. **Next to, or under as well?** The engine lets a civilian stand under its own military unit. Should an Engineer under a unit count as "next to" (recommended: yes, it is the safer, narrower placement), or must it stand on a neighbouring tile?
5. **Mongols.** Today: no siege units and half damage to city HP. Proposed: no Military Engineers until they capture a city with a Workshop (your rule). Keep their half damage too (then a quarter against walls, half against open cities), or let the Engineer ban be the whole drawback? I lean to dropping the half damage and measuring, since "no Engineers" already says "couriers, not sappers".
6. **Roads from Engineers (roadmap).** Should the Engineer lay roads at Engineering, one tile a turn? It bends "roads only from the Courier Hall and Roadworks" and touches the Mongol identity. If yes, is a Hall-holding Mongol still special enough (its network is instant and map-wide; the Engineer's is a footpath)?
7. **Forts (roadmap).** Do you want a placeable Fort at Feudalism (defence as on hills), given it adds a tile feature and art?
8. **A captured Engineer.** The captor keeps it, as with a Settler (recommended), or it is destroyed on capture?
9. **Price.** 60 Production and 1 Gold upkeep (a Walls' worth), or 80 as a real commitment? The sims priced it at 75.
10. **Do Catapult and Bombard go entirely,** including their art, or does the Bombard survive as the Metallurgy flavour (a unit again) rather than the Engineer's siege-guns sentence? The recommendation is to remove both.


---

## 9. Sources and method

- Rules: `.claude/skills/legatum-rules/references/game-rules.md` §4 (formula, city shield, overflow, split counterattack, the city strike), unit table, §9.
- Engine: `crates/core/src/combat.rs` (damage table, `city_defense_with_fort`, `city_tile_defender`), `crates/core/src/rules/battle.rs` (`plan_fight_on`, `resolve_ranged`), `crates/core/src/data.rs` (`unit_stats`, `siege_city_bonus`), `crates/core/src/rules/movement.rs` (stacking, civilian capture).
- AI: `crates/ai/src/production.rs` (`composition_bonus`, `walled_quota_value`, the siege kind bonus and its gates), `crates/ai/src/conquest.rs` (`choose_conquest_city`, `public_softness`), `docs/learnings.md` 2026-10-18 (siege built per game).
- Lab: `crates/sim/examples/combat_lab.rs` (`--siege`, `VARIANTS`, `engineer_orders`), `tools/sim/siege-lab-report.py`, results in `docs/sim-results/siege/`. Rule levers: `crates/core/src/experiments.rs` (`walls-blunt-ranged`, `walls-blunt-melee`, `siege-city-pct`, `siege-defense`, `city-strike-spares-siege`, `siege-engineer-proxy`), all off unless a sim sets them. The Engineer is simulated as a Settler travelling with the army (same class, same capture rule), so the only approximation is its 75-Production price in the cost column (60 is proposed) and that the lab's defender AI does not yet know to sally for it.
- Civ 6: walls as an outer-defence pool, melee 15% and ranged 50% damage to walls, siege full; Battering Ram (Masonry) and Siege Tower (Construction) as stackable support units; cities strike at range 2 and catapults die to them, so players screen them. Civ 5: siege must set up for a turn; Artillery range 3 with indirect fire. Old World: siege units lower city defence before assault. Humankind: siege weapons built on site during a siege. Total War: rams, towers and ladders built at the siege over turns. History: Roman legionary engineers at Masada and Jotapata; Edward I's engineers at Stirling (1304); Urban's bombard at Constantinople (1453), after which the trace italienne (Star Fort) answered gunpowder.
