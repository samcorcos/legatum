# Why Hard's cities alternate production (diagnosis, 2026-11-15)

Diagnosis only: no AI behaviour changed on this branch. Follows the scenario study
(`claude/fable-scenarios` c317908c3, `fable-scenarios-1115/README.md`, G3 T31 and G4 T89), whose
suspected mechanism was read, not traced. Unit names are the code's (Soldier = Spearman,
Ranger = Archer, Marksman = Crossbowman).

## Answer

Three separate causes, all deterministic. None is Gold-buying or score jitter (±15 against gaps
of hundreds), and no city was in danger of falling when it switched.

1. **The "finish what was started" bonus is paid to the wrong item** (`production.rs::choose_item_inner`).
   It reads `opt.production_banked`, which the view sets to 0 for the active build
   (`core/view.rs`, `ProductionOption`). So the item just switched away from gets up to +50% and
   the item under way gets nothing. The routine pass re-scores every combat unit under way each
   turn (`reconsider` in `plan_cities`), so two near-equal units trade places every turn until
   one is a turn from done. This is the Soldier and Ranger flip in G3.
2. **Two passes disagree and neither yields** (`plan_emergency_production` against the routine pass).
   The emergency pass fires on `needs_ranged` alone (an enemy unit within 3 hexes, a garrison, no
   shooter within 1 hex) with threat at or below local defence, and replaces whatever is under
   way with a shooter. Next turn the routine pass re-scores that shooter and puts the old item
   back. This is G4 (Composite Archer and Machine Shop in Phaleron, T95 to T99).
3. **Item values fall off a cliff at threat > 0** (`item_base_value`, `threatened = threat > 0`).
   Threat on a city moves between 0 and 20 to 44 from turn to turn as one unit crosses a line.
   A Settler is worth 726 when threatened and 4,228 when not; a great work is worth 0 when
   threatened and 900 + Science x 4 when not. Each crossing evicts the build under way.

The study's reading that the emergency pass "protects only Walls and Settlers" is right as far
as it goes, but it is the smaller part: the routine pass made 64% (G3) and 75% (G4) of the
switches.

**Switching costs delay, not Production** (banking is per item), and the traces confirm it: G3
delivered its first unit about 4 turns late; G4 had 566 Production (about 5 turns of the whole
empire's output, 25% of the window) parked in unfinished items at T108 where the model had 0.

**The Pyramids were not lost to alternation.** Hard values the great work at 0 on every
threatened turn, and at about 1,200 when unthreatened, below a Settler (4,228) and any unit
(2,100 to 2,900) with 181 of 240 already paid. No change tried here finishes it in more than 3
of 12 runs. That is a valuation gap of its own (see Follow-ups).

## Traces (rules 30, 12 dice salts each, all traced)

Method: `tools/prod_trace.rs` is the study's `fable_fork` Hard arm plus a twin `ScriptedBrain`
running `order_diagnosis` on the same view each turn. The twin's orders equalled the shadow
brain's on every turn of every run. Temporary `eprintln!` lines in `plan_emergency_production`
gave the trigger terms; they were never committed.

### G3 T31, Germany, Essen (17 to 19 Production a turn), salt 0

| Turn | Under way | Pass that set it | Why | Threat / defence |
| --- | --- | --- | --- | --- |
| 31 | Pyramids 181/240 | emergency → Ranger | `needs_ranged` only: one Soldier at 3 hexes, garrison, no shooter | 0 / 26 |
| 32 | Ranger 17/132 | routine reconsider → Walls | early gate "anticipatory defense: shooter or Walls" | 0 / 20 |
| 33–37 | Walls | kept, done T37 | | 41, 31, 31, 31, 8 / 0–26 |
| 38 | idle | emergency → Ranger | threat > defence | 44 / 16 |
| 39 | Ranger | routine → Soldier | 2,271 against 1,983; Soldier's banked bonus | |
| 40 | Soldier | emergency → Ranger | `needs_ranged` only | 22 / 26 |
| 41 | Ranger | routine → Soldier | 2,810 against 2,233 | |
| 42 | Soldier | emergency → Ranger | `needs_ranged` only | |
| 43–45 | Ranger, Soldier, Ranger | routine, each turn | banked bonus to the other item | |
| 46 | Soldier 76/80 | routine → Ranger | Soldier one turn from done | |
| 47 | Ranger 126/132 | routine → Soldier | | |
| 47–48 | | Soldier done T47, Ranger T48 | | |

Essen was at 200 of 200 HP and never forecast to fall. Berlin fell at T32 (before any of this
could matter). Pyramids: gated "nonpositive item value before time discount" on every turn T39
to T50. In the three salts where it finished (1, 10, 11) the threat was exactly 0 on a turn the
city was idle and the `victory` pass set it.

Caveat on T31: the Pyramids were the model's build, not Hard's `work_plan`, so the routine pass
would have dropped them that turn anyway (`abandon_wonder`). That first switch is partly an
artefact of handing a model's position to Hard.

### G4 T89, Athens, 5 cities (about 113 Production a turn), salt 0

| City | What happened |
| --- | --- |
| Athens | Arquebusier 171/272 dropped T95 by routine reconsider for a Machine Shop; T98 emergency → Composite Archer; T99 routine → Marksman. Arquebusier still at 171 at T108 |
| Phaleron | Composite Archer and Machine Shop traded every turn T95 to T99 (emergency `needs_ranged` against routine reconsider), then Marksman |
| Eleusis | Marksman 131/214 dropped T95 for a Star Fort |
| Marathon | Soldier 50/72 (one turn left) → Ranger → Marksman, T106 to T107 |
| Laurion | 6 Production a turn spread over five items |

No city took damage in any run and none was forecast to fall.

### Counts, mean of 12 salts

| | G3 T31–T50 | G4 T89–T108 |
| --- | --- | --- |
| Switches away from an unfinished item | 11.0 | 13.67 |
| (study's "switches", which also count picks after a completion) | 12.92 | 23.25 |
| by routine reconsider | 7.08 | 10.25 |
| by emergency, `needs_ranged` only (threat ≤ defence) | 2.33 | 3.17 |
| by emergency, threat > defence | 1.33 | 0.25 |
| by emergency, `exposed_capture` only | 0.25 | 0 |
| away from an item 1 or 2 turns from done | 2.75 | 1.83 |
| back to an item with banked progress | 6.92 | n/c |
| City-turns with threat > defence | 7.75 of 22 | 3.67 of 100 |
| City-turns forecast to fall | 0.5 (Berlin) | 0 |
| New Production parked in unfinished items at the end | 219 | 566 [392–735] |

## Rules 35

- **Read:** the rules-35 `production.rs` differs from rules 30 in 32 lines, none in these
  functions; `view.rs` still sets `production_banked` to 0 for the active build.
- **Run** (`tools/prod_switch_census.rs`, 4 games of 8 Hard seats to T150, seeds 9311000–3):
  2,505 switches away from an unfinished item in 17,368 city-turns (14%); 28% of city-turns with
  an enemy unit within 3 hexes, 12% elsewhere. 678 were an A-B-A flip. About 10,000 Production
  per game sat in unfinished items at T150 (about 260 per city). So it is present on rules 35
  and is not confined to threatened cities.

## Already recorded?

- The coach playbook has two earlier instances, each fixed with a lock on one item: "Walls
  swapped for a shooter mid-build" (8.9.2 Sparta; `keep_walls`, then `walls_under_way`) and the
  8.4 Settler swapped at 80 of 163 (`keep_settler`). Nobody had recorded the general pattern or
  the routine pass's part in it.
- The opposite risk is also in the playbook: "Defence doesn't scale with the army at the door"
  (victims building a Settler or the Pyramids with 3 to 5 of his units within 3 tiles).
- `human-reasoning.md` has him switching once on purpose (Archers to Spearmen against mounted
  units). His tapes were not counted for switch rates here.

## Proposal

Smallest fix, both parts by value, no lock:

**A. Price the incumbent's own progress.** In `choose_item_inner`, compute the "finish what was
started" term from `opt.production_stored` (the item's progress whether active or banked)
instead of `opt.production_banked`. One token. The active build then carries its sunk share,
and with the existing time-to-complete discount a city converges on whichever item is nearest
done.

**B. Stop the emergency pass overriding on a standing preference.** In
`plan_emergency_production`, do not fire when the only trigger is `needs_ranged` without
bombardment (threat ≤ local defence, no `exposed_capture`). The want for a shooter stays in the
routine scorer, which already has it (early gate "anticipatory defense: shooter or Walls", the
`rangers` tally), where it competes on value with what is under way.

Scratch prototype (`tools/prototype.diff`, rules 30, Hard seat only, from the cut turn; never
committed), paired by salt against the baseline, 95% CI:

| | G3: switches | G3: completions | G4: switches | G4: completions | G4 T153 guard: wins |
| --- | --- | --- | --- | --- | --- |
| Baseline | 11.0 | 2.33 | 13.67 | 10.0 | 11 of 12 |
| A | 5.5 (−5.5 [−6.8, −4.2]) | 3.0 (+0.67 [+0.1, +1.2]) | 13.58 | 9.83 | 12 of 12 |
| B | 10.75 | 2.83 | 4.75 (−8.9 [−11.2, −6.6]) | 11.08 | not run |
| A + B | 6.83 (−4.2 [−5.7, −2.6]) | 2.25 (−0.08 [−0.6, +0.4]) | 4.75 (−8.9 [−11.2, −6.6]) | 11.58 (+1.58 [+0.7, +2.5]) | 12 of 12 |

Other measures with A + B: G4 parked Production 566 → 219, Gold at T108 165 → 204, units at
T108 8.3 → 9.5, military units completed 0.67 → 1.83; G3 military units completed 0.67 → 1.08.
Pyramids finished: baseline 3 of 12, A 3, B 0, A + B 0, A + B + C 1 (C: great works allowed
when threat ≤ local defence). VP in G3 follows the Pyramids exactly.

So the expected effect of A + B: switches about halved in G3 and cut by two thirds in G4, one
to two more completions in 20 turns in G4, **no gain on the Pyramids**, and G3 completions flat.

**Risk.** B removes the one path that interrupts a building for a shooter when a garrisoned
city sees an enemy unit at 3 hexes. A city that should switch and now does not is the playbook's
"army at the door" case. In the three scenarios here no city took more damage (damaged
city-turns unchanged: 2.0, 0, 1.9 → 1.8). A's risk is small: it can only make a city stay with
the item it has.

**Guards before landing** (each with and without the change, same salts):
- Scenario 6 (G4 T153, defending Marathon): stay at 11 of 12 wins or better.
- The 8.4 Walls and Settler cases (`keep_walls`, `keep_settler` tests) and the hard-vs-human
  war scenarios in `docs/sim-results/hard-vs-human/war-scenarios/`: no more cities lost.
- The coach's 8.9.26 Sparta opening (his five Hoplites arriving at T19 to T38): victims' cities
  alive at T40 no lower.
- A new pin: a city with a unit under way and a second unit of near-equal value banked keeps
  building the first (fails today).

**Follow-ups, not in the smallest fix** (inferred from the traces, not prototyped beyond C):
- Cause 3: replace the binary `threatened` in `item_base_value` with a graded term (threat
  against local defence) for Settlers and great works. In the G4 T153 guard, where threat really
  exceeds defence, A + B moved switches only 31.7 → 28.0, so this is where the rest is.
- Great works: a work 75% paid and worth 2 VP scores about 1,200 against a Settler's 4,228.
  That, not alternation, is why the Pyramids are not finished.
- Then the emergency pass can enter the incumbent in its own pick with its real turns left, and
  `keep_walls` and `keep_settler` become redundant.

## Traced or inferred

- **Traced:** every table above; which pass made each switch and its trigger terms; the banked
  bonus going to the non-active item (option scores per turn); the Pyramids' value gate; the
  prototype numbers (rules 30); the rules-35 census.
- **Inferred:** that A + B behaves the same on rules 35 (code identical in these functions, not
  run there); the effect of grading `threatened`; anything about full-game win rate. The study's
  horizon is 20 turns and 12 dice salts of one position each.

## Reproduce

Engine: a checkout of 51670e4a9 (rules 30) with `tools/prod_trace.rs` copied to
`crates/sim/examples/`, `tools/prototype.diff` applied for the prototype arms, built with
`CARGO_BUILD_JOBS=4 cargo build --release -p legatum-sim --example prod_trace`.

```sh
# one2.sh GAME TURN HORIZON TAG SALT ; arms: PROTO=a|b|ab|abc PROTO_FROM=<cut turn>
PROTO=ab PROTO_FROM=89 tools/one2.sh game4 89 20 g4ab 0
python3 tools/tab.py g4ab-s0.jsonl Athens     # per-turn table for one city
python3 tools/pstats.py g4 g4ab               # per-salt counts
```

`tools/stats.py` attributes each switch to its pass; it needs the `EMG` trace lines, which came
from the uncommitted logging. `tools/prod_switch_census.rs` runs on rules 35 as
`prod_switch_census <seed> <games> <last turn>`. About 1 minute per build and under 1 minute per
12-salt arm on 4 threads.
