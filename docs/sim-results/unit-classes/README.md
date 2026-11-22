# Unit classes: one unit per class in every era

Branch `claude/era-units` (phase A2), 2026-10-10. Functionality and design-target checks of the class bonus and the era-by-era lineage, not a balance pass (the AI is not competent enough yet to balance on; `legatum-no-balance-sims`).

Regenerate with the release `combat_scenarios` example:

```
cargo build --release -p legatum-sim --example combat_scenarios
target/release/examples/combat_scenarios --eras --games 12  > eras.txt
target/release/examples/combat_scenarios --triangle         > triangle.txt
target/release/examples/combat_scenarios --responses        > responses.txt
target/release/examples/combat_scenarios --keshig --games 8  > keshig.txt
```

## The reports

- [mounted.md](mounted.md) (2026-10-31): the mounted fix (riders at 130% of the melee line, the class bonus as 40% of strength, -33% charging a city), with the variant grid it was chosen from (`combat_scenarios --mounted`, a `combat_lab` subset). The class-bonus numbers in the older reports below are for the flat +18.

- [eras.txt](eras.txt): the user's targets for the era steps, at equal Production. **Same class, newer era over older** (four of the newer unit against the same Production of the older, a fractional count blended between the two whole counts, `+1` to `+3` eras ahead), and **a counter-class unit one era behind against the class it counters**. Win share of the newer (or counter) side, Hard policy / Human policy, open ground, mean of starts 2 to 5 hexes apart. The Human policy (focus fire, advance and strike) is the reference; the Hard policy is the scripted AI on both sides and plays these small arena duels very timidly (the newer, dearer army rarely commits), so its numbers are far lower and are shown for completeness.
- [triangle.txt](triangle.txt): per era, line / gunline / horse doctrines of the era's three units at equal Production (budget 8 times the melee cost), on open ground and with a Forest between.
- [responses.txt](responses.txt): for each era the best compositions (melee/ranged/mounted counts) against four archetypes, and whether any single mix beats them all.

## Targets and results (Human policy)

| Target (user and lead, 2026-10-10) | Result |
|---|---|
| Same class, one era ahead, equal Production: about 60 to 65%, never 85%+ | melee 61 to 70%, mounted 61 to 67%, ranged 61 to 64% (Combat Drone over Artillery 43%) |
| Two eras ahead: about 75 to 85% | melee 81 to 91%, mounted 71 to 82%, **ranged about 50%** |
| Three or more eras ahead: lopsided | melee 94 to 98%, mounted 88 to 98%, ranged 57 to 65% |
| A counter-class unit one era behind still beats or roughly ties what it counters | spear (e-1) over cavalry (e): 100% in every era; cavalry (e-1) over ranged (e): 100% through Industrial, 85% Modern, 76% Future |
| No composition is best everywhere | no mix beats every archetype at 60%+ in any of the seven eras |

How it was tuned: strength steps (melee 6, mounted 7, ranged attack 8 and ranged defence 7 per era) and cost ratios (melee 1.38, mounted 1.36, ranged 1.27 from a base of 66) were chosen from a scan with the simulator, not from a formula. The class bonus is 18 strength (`combat::CLASS_BONUS`), larger than an era's step on purpose. The ranged base cost is higher than the melee base cost: with a cheaper ranged line the late-era equal-Production shooters outnumbered everything, previous-era cavalry stopped beating them, and a 2/7/0 mix beat every archetype.

## Limits and honest findings

- **Ranged era steps stay weak two eras ahead (about 50%).** Fire is unanswered, so an older, larger swarm of shooters keeps pace with fewer newer ones; the calibration that keeps the +1 step at 62% cannot also lift +2 without making ranged cheap enough to dominate the triangle. A follow-up if it matters: a steeper ranged attack step with a higher ranged base cost.
- **Armies beyond about ten a side saturate the arena** (the older swarm overwhelms newer units whatever the stats), so the era-gap duels use four units of the newer kind.
- **A screened gunline beats horse in every era** (`horse > gunline` is 0%): the melee screen kills the riders (class bonus 18). Horse answers *unscreened* ranged and siege, which the counter-class rows measure.
- **Late-era ranged beats melee at equal Production (Industrial onward 100%, Human policy).** That is the user's triangle (ranged beats melee), but the margin is wide; a gentler ranged line from Industrial is one number per unit in `data::unit_stats`.
- These are equal-Production skirmishes on a small open arena, one order per unit per turn, no cities or walls. The 139-case grid (`combat_scenarios --games N`) covers cities, walls and the other goals and was not re-run.
- **AI smoke only** (`unit_matchups --mode campaign`, 8 four-player Hard + Search games, before the last ranged cost change): engaged Production was melee 41,600, ranged 98,000, mounted 3,200 (2% of the engaged force; 0.5% before the war-mode fix), siege 340; mounted units dealt 1.08 damage per Production against melee's 1.25 and ranged's 3.55; 9 aircraft were lost (Fighters and Bombers are built and fought). The AI buys and fires all three classes and upgrades idle units when Gold allows, but it still under-builds mounted units (the class bonus makes them a cheap answer to an enemy shooting line the AI does not yet read well): a tuning follow-up for when the AI is competent. No balance simulations were run for it.

## The Keshig (Mongol horse archer, 2026-10-17)

**The Keshig is the Mongols' Classical mounted unit** (it replaces the Cataphract, from Iron Working; user, 2026-10-17: "the Mongol horde was in the 1200s so it feels premature to give it right at the start"). [keshig.txt](keshig.txt) is the Classical report (8 seeds per cell): Mongol riders against that era's units at equal Production, open ground and a Forest band, plus two town assaults (450 Production of riders against a Phalangite garrison with a Phalangite and a Composite Archer beside it). Columns: the Keshig, the same Mongol rider at the Cataphract's 75 Production that never shoots (the counterfactual), and a Japan Cataphract.

**Chosen: 82 Production, the Cataphract's 25 melee, 18 ranged at range 2** (`data::KESHIG_KIND`, `KESHIG_COST`, `KESHIG_RANGED`, `KESHIG_RANGE`): the Ancient design (below) scaled by the era steps, mounted cost ×1.36 (60 → 82) and the ranged line's +8 (10 → 18). It keeps the Ancient profile: a little better than the bowless rider in the mirror (42 against 34), much better against a mixed spear-and-bow army (30 against 0 in the open, 51 against 4 in forest) and against walled towns (47 against 20), and it still loses to the spear line (0% against six Phalangites in the open).

Classical sweep (open ground; Keshig / bowless Mongol Cataphract):

| Cost / ranged / range | vs 6 Cataphracts | vs 3 Phalangites + 3 Comp. Archers | vs 6 Phalangites | TakeCity walled |
|---|---|---|---|---|
| 75 / 18 / 2 | 86 / 34 | 60 / 0 | 0 / 0 | 66 / 20 |
| 78 / 18 / 2 | 66 / 34 | 46 / 0 | 0 / 0 | 57 / 20 |
| 82 / 16 / 2 | 40 / 34 | 23 / 0 | 0 / 0 | 34 / 20 |
| **82 / 18 / 2** | **42 / 34** | **30 / 0** | **0 / 0** | **47 / 20** |
| 82 / 20 / 2 | 49 / 34 | 38 / 0 | 0 / 0 | 54 / 20 |
| 86 / 18 / 2 | 20 / 34 | 21 / 0 | 0 / 0 | 38 / 20 |
| 90 / 20 / 2 | 3 / 34 | 21 / 0 | 0 / 0 | 58 / 20 |

### First version: the Ancient Keshig (replaced the same day)

The Ancient report (since replaced by the Classical one above): Mongol riders against six opponents at equal Production (open ground and a Forest band, starts 2 to 5 hexes apart, both arena sides, Human policy) and two city assaults (330 Production of riders against a Spearman-garrisoned town with a Spearman and an Archer beside it, unwalled and walled). Three columns: the Keshig (the Human policy picks the best of every charge it can make and every shot in range by forecast: expected damage dealt plus kill chance, minus damage taken and loss chance), the same Mongol rider at the Horseman's 55 Production that never shoots (the counterfactual: what the Mongols had before), and a Japan Horseman for reference. The Mongol rider has 5 moves and no terrain defence.

**Chosen then: 60 Production, the Horseman's 18 melee, 10 ranged at range 2.** The aim (user and lead): better than a Horseman, not dominant, and the triangle still holds. At 60/10/2 the Keshig is at least as good as the bowless rider in all 14 rows, much better against a mixed spear-and-bow army (34% against 1% in the open, 60% against 3% in forest) and against walled towns (13% against 2%), and still loses to the spear line (0% in the open; 3% against Spearmen with Horsemen), which is the class that counters it.

Sweep (8 seeds per cell; open ground; Keshig / bowless Mongol Horseman; the bowless rider takes the towns 75% unwalled and 4% walled):

| Cost / ranged / range | vs 6 Horsemen | vs 3 Spear + 3 Archer | vs 6 Spearmen | TakeCity unwalled / walled |
|---|---|---|---|---|
| 55 / 10 / 2 | 82 / 35 | 70 / 0 | 0 / 0 | 91 / 25 |
| 60 / 8 / 2 | 37 / 35 | 27 / 0 | 0 / 0 | 90 / 8 |
| **60 / 10 / 2** | **41 / 35** | **37 / 0** | **0 / 0** | **96 / 13** |
| 60 / 12 / 2 | 45 / 35 | 40 / 0 | 0 / 0 | 88 / 23 |
| 65 / 10 / 2 | 6 / 35 | 15 / 0 | 0 / 0 | 99 / 2 |
| 70 / 14 / 2 | 2 / 35 | 15 / 1 | 0 / 0 | |
| 60 / 10 / 1 | 19 / 35 | 13 / 0 | 0 / 0 | 94 / 0 |
| 55 / 12 / 1 | 45 / 35 | 33 / 0 | 0 / 0 | 100 / 4 |
| 70 / 14 / 1 | 0 / 35 | 4 / 1 | 0 / 0 | |

What it says: price dominates at equal Production (55 makes it a strictly better Horseman that beats a mixed army 70% of the time; 65 and up loses the Horseman's own mirror), range 1 is worse than range 2 at every price (it must stand beside its target, where the target hits back next turn), and the bow's strength matters less than either. Nothing shoots its way past a spear line in the open: a 2-move Spearman reaches a rider two hexes away and strikes it the turn it fires, because a shot is the rider's whole turn (no hit-and-run). These are small equal-Production skirmishes without the AI; full-game effects are in the AI-impact gate and the Mongol win-share run (branch `claude/mongol-keshig`, docs/learnings.md 2026-10-17).
