# Army commitment: two genes from the decision audit's army category (2026-10-31)

Branch `worktree-agent-aad9a85942375c33d` (on `claude/raze-hold-on`, so `raze_value` 1 and `buy_hold` 3 are the
control). Follow-up to `decision-audit.md`, whose largest category was the army: in the designer's seat Hard pulls
back or skips units where he attacks. Genes `capture_hold` and `strike_kill`, both AI decisions only, both default
off (`crates/ai/src/army_commit.rs`). Runner `army-commit/run.sh`, arms in `army-commit/params/`, outputs in
`army-commit/out/`. Everything heavy ran on the cloud box.

**Verdict.** `capture_hold` 1 is the fix: the pass behind the largest army disagreement drops out of the audit, the
army share falls by a quarter, the whole-policy gap shrinks, and the quarter gate's seat-only arm gains +3.91 pp
[+0.98, +6.84]. Rivals-only reads +1.56 [-2.15, +5.27] (stronger rivals, inside noise). The war suite and census
don't move. `strike_kill` adds nothing in the audit; its quarter-gate arm (with `capture_hold`) is +5.47 seat-only,
+0.59 rivals-only. Ready for the lead's full gate: `capture_hold` 1 alone, and optionally with `strike_kill` 1.

## 1. The patterns

**Tool.** `decision_audit --army-detail` (diff only, today's code, `--engine here`) writes one row per military unit
of his seat and turn: both resolutions (where the unit ended, its attacks and kills), the nearest enemy city and unit
at war before and after each, and the planning passes that gave Hard's order. The passes come from a new staged
trace field, `OrderTrace::passes` (`Planner::pass` marks each pass of `planner_with` and `plan_military`; recorded
only by `order_diagnosis`, which `crates/ai/tests/order_trace.rs` checks emits the live orders). Classes and pass
counts: `tools/decision-audit/army.py`.

Four games replay well enough on today's rules: Mongols (0 hash mismatches), Sparta (6), Athens (8), Swiss (32,
from about T150). Japan drifts from the start (234) and is left out. Over the four games, 1,148 unit-turns at war
where his unit and Hard's ended differently or attacked differently:

| Pattern | Unit-turns | What happens | Hard's pass |
|---|---:|---|---|
| **Pulled back into a captured or damaged city** | 102 (`hold_captures`), of them 21 of the 60 kills he made and Hard didn't | After a capture he keeps his 3-6 units attacking; Hard brings the nearest unit within 3 back into every city in resistance (or damaged with enemies about), or holds the one inside it, before any attack pass runs | `hold_captures` |
| Strike force stages past a kill | 17 skipped attacks, 14 skipped kills | The opportunity strike sends its force to staging tiles, or holds it there, beside a wounded enemy he kills | `press_strike` |
| Upgrade instead of attack | 11 skipped attacks | `hold_or_upgrade` in the city pass | `cities` |
| Early anti-Mongol garrison | 5 kills (Swiss) | every city garrisoned first while at war with Mongols | `garrison_cities` |

The `hold_captures` disagreements are exactly the largest army windows of the audit: Athens T158-T177 (the T160
window, +2.5 cities), Swiss T159-T173 (T160), Mongols T159-T172 (T160), Sparta T34-T40 (T40) and T90-T94 (T90). Most
of them have no enemy that could retake the city that turn (no visible enemy unit at all in Mongols T159 and T168,
Athens T158, Sparta T93).

## 2. The fixes

- **`capture_hold` 1** (`hold_captures`, `army_commit.rs`). A front city takes a holder before the attack passes
  only when holding it now is worth more than what the holder would do instead: the visible attack would take it
  empty this turn (`purchase_defense::city_risk_with` with our units removed, the forecast door defense and raze
  use) and a holder would stop that, at a 90% fall chance on the city's population, against the holder's best
  capture this turn (an ungarrisoned enemy city at war it reaches: population x capture chance). Every other front
  city gets its holder from the units the attack passes leave free (`hold_captures_late`, after the campaign pass);
  the early anti-Mongol `garrison_cities` pass skips it too. On the four tapes the 102 `hold_captures`
  disagreements become 3, plus 9 from the late hold (war unit-turns that differ 1,148 -> 1,118; units kept in or
  pulled into a city 133 -> 103; kills he made and Hard didn't 60 -> 44); Hymnal (the 2026-10-04 rush-defense capture) is still held through the Japan
  counterattack (`mongols_still_holds_hymnal_with_capture_hold_and_strike_kill`).
- **`strike_kill` 1** (`press_strike`). A force unit that can kill a visible enemy field unit alone this turn (forecast
  kill chance at least 50%, the target's cost times the kill chance above its own cost times its loss chance) is
  left to the attack passes instead of staging. Mongols kills skipped 13 -> 10; the rest are city assaults.

Integer maths, the fogged view only. Tests: `crates/ai/src/army_commit_tests.rs` (no visible retake: held late;
an adjacent Soldier would retake: held first; the holder's own capture worth more: it goes; the kill test).

## 3. Measured

### Decision audit (his minus Hard, 10-turn windows every 10 turns, 32 salts, his seat on the arm)

`run.sh`-equivalent: `tools/decision-audit/run.sh --engine <commit> --ship params/ARM.json SAVE... -- --window 10
--turns ... --salts 32 --horizons 20,50 --alt params:audit-in/ARM.json --continue params:audit-in/ARM.json`, 61
sampled turns per arm (Mongols 17, Swiss 17, Athens 16, Sparta 11). Share = mean of `fwd` and `rev`; change =
paired by turn and salt against the control. `out/audit-shares-*.txt` (`audit_shares.py`).

| Contrast, cities @20 | Control | `capture_hold` | change | both genes | change |
|---|---:|---:|---:|---:|---:|
| Whole policy | +0.328 | +0.297 | -0.031 | +0.335 | +0.007 |
| Army share | +0.185 | +0.140 | **-0.045** | +0.164 | -0.021 |
| Garrison share | +0.090 | +0.123 | +0.034 | +0.144 | +0.054 |
| Production | -0.010 | -0.005 | +0.004 | +0.011 | +0.021 |
| Settling | +0.055 | +0.064 | +0.009 | +0.062 | +0.006 |
| Diplomacy | +0.004 | +0.023 | +0.019 | +0.036 | +0.032 |
| Research, other | ~0 | ~0 | ~0 | ~0 | ~0 |

At 50 turns: whole policy +0.305 -> +0.260 cities (-0.046), army share +0.117 -> +0.078 (-0.039), garrison +0.107 ->
+0.111; VP @50 whole +0.242 -> +0.207 (-0.035).

- **Army share by game @20** (control -> gene): Sparta +0.32 -> +0.13 (-0.19 ±0.06), Swiss +0.18 -> +0.12 (-0.06
  ±0.03), Athens +0.13 -> +0.14, Mongols +0.11 -> +0.18 (+0.07 ±0.04). The Sparta T90 window goes +0.97 -> -0.31
  cities and Swiss T160 +1.94 -> +0.22; Mongols T160 goes the other way (-0.53 -> +0.34).
- **The garrison share grows because the gene's units are garrison units:** the audit classes a unit within one tile
  of our city at the start of the turn as garrison, and the held capture is our city. Army plus garrison @20 is
  +0.275 -> +0.263; at 50 turns +0.224 -> +0.189. The whole policy is the clean measure.
- With `strike_kill` too, the army gain halves and the whole policy gains nothing (Mongols army +0.11 -> +0.23).
- Today's control already differs from the 10-30 audit: production -0.01 (was +0.13; `buy_hold`), army +0.18 is
  now the largest category alone.

### Quarter gate (`hard_vs_human`, 8 maps x 64 salts = 512 games per arm; strength in pp, positive = stronger AI)

Paired by map and salt, map-stratified bootstrap (`gate_paired.py`, `out/gate-paired.txt`). Control 109/512 seat
wins (21.3%). The runs are deterministic (a rerun gave the same counts).

| Arm | Seat wins | Strength | 95% |
|---|---:|---:|---|
| `capture_hold`, seat only | 129 | **+3.91** | [+0.98, +6.84] |
| `capture_hold`, rivals only | 101 | +1.56 | [-2.15, +5.27] |
| both, seat only | 137 | +5.47 | [+2.34, +8.59] |
| both, rivals only | 106 | +0.59 | [-3.32, +4.49] |

Seat-only gains come mostly from the 8.4 Sparta map (24 -> 43 of 64). Rivals-only moves two maps the opposite ways:
8.6 Germany loss 27 -> 8 (stronger rivals), 8.6 Sparta 41 -> 54 (weaker).

### War suite (64 runs, 33 cases)

38% in every arm (control, `capture_hold`, `strike_kill`, both; seat and all seats). Only
`war-athens-two-pikemen-t112` moves (15 -> 23%, every `capture_hold` arm). The fixtures are 5-20 turns with few
captures.

### All-AI census (512 eight-empire Hard games, seed 7300001, paired, ± 95%)

| Measure | Control | `capture_hold` | both |
|---|---:|---:|---:|
| Captures a game | 14.82 | +0.25 ±0.67 | +0.05 ±0.64 |
| Cities lost a game (captured, razed, sacked) | 16.74 | +0.41 ±0.79 | +0.00 ±0.74 |
| Cities at the end, all empires | 38.43 | -0.32 ±0.30 | -0.06 ±0.29 |
| Game length, turns | 243.8 | -5.8 ±4.2 | -6.5 ±4.2 |

### Decision cost

`war_trace` seat planning ms, runs 0-2 (`out/trace-summary.txt`): Golden Horde thin 4.25 / 4.53 / 3.86 -> 4.30 / 4.49 /
3.83 (both genes), Chur T137 8.91 / 7.83 / 6.50 -> 8.86 / 7.86 / 6.39, hollow Swiss T96 3.41 / 4.32 / 6.00 -> 3.42 /
4.35 / 6.00, Swiss tepid T220 3.55 / 3.69 / 3.72 -> 3.58 / 3.75 / 3.70. The forecast runs only for front cities with a
holder candidate. The census's ms per seat-turn follows the box load and doesn't measure the genes.

## 4. Risks

- **Recaptures.** A city that no visible attack can retake this turn is left to the attack passes; an army out of
  the fog can still walk in. The census shows -0.32 ±0.30 cities at the end across all empires, at the edge of
  noise; Hymnal still holds.
- **Faction spread.** The seat gain is mostly the Sparta map; rivals-only is mixed by map (Germany loss 27 -> 8,
  Sparta 41 -> 54).
- **Per-window noise.** Individual windows swing both ways (Mongols T160 worse, Sparta T40 and T80 worse); the mean
  over 61 windows is the measure.
- **The Swiss tape drifts from T150**, where its T160 window is; Japan was not audited.

## Reproduce

```
tools/cloud/remote.sh <ref> --artifact docs/sim-results/ai-war/army-commit/out -- \
  docs/sim-results/ai-war/army-commit/run.sh suite|census|gate|trace ARM...
tools/decision-audit/run.sh --local --engine here --out DIR SAVE... -- --diff-only --army-detail [--alt params:audit-in/ARM.json]
tools/decision-audit/army.py classes|kills DIR/army.jsonl ...
docs/sim-results/ai-war/army-commit/audit_shares.py AUDIT_DIR 20 cities
docs/sim-results/ai-war/army-commit/gate_paired.py GATE_OUT hold both
```

`--engine COMMIT` (a temporary worktree per launch) avoids the `--engine here` launch race: each launch gets its own
`audit-in/`.
