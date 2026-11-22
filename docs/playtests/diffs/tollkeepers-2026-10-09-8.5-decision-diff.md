# Decision diff: Tollkeepers (playtest-8.5)

## Top five disagreements

Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over 48 paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.

1. **T85 Gold purchase**: -33.3 [-46.8, -19.9] win percentage points; -0.9 [-1.2, -0.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
2. **T77 escort and garrison movement**: +18.8 [+2.6, +34.9] win percentage points; +1.2 [+0.9, +1.5] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
3. **T69 army movement**: +25.0 [+6.1, +43.9] win percentage points; +0.2 [-0.3, +0.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
4. **T49 founding site and timing**: +0.0 [+0.0, +0.0] win percentage points; -1.3 [-1.9, -0.7] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
5. **T68 Gold purchase**: +4.2 [-1.5, +9.9] win percentage points; +0.8 [+0.5, +1.0] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.

## Ranked decisions

| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |
|---:|---|---|---|---:|---:|---|
| 85 | Gold purchase | none | c19:Unit { unit: Soldier }:129g | -33.3 [-46.8, -19.9] | -0.9 [-1.2, -0.6] | Different Gold spending or purchase timing on this turn. |
| 77 | escort and garrison movement | MoveUnit { unit: UnitId(14), to: Hex { q: 10, r: 3 } } | MoveUnit { unit: UnitId(14), to: Hex { q: 16, r: 4 } }; MoveUnit { unit: UnitId(21), to: Hex { q: 14, r: 18 } } | +18.8 [+2.6, +34.9] | +1.2 [+0.9, +1.5] | Different garrison or escort movement changes the exposed unit position. |
| 69 | army movement | MoveUnit { unit: UnitId(14), to: Hex { q: 6, r: 15 } }; MoveUnit { unit: UnitId(64), to: Hex { q: 11, r: 15 } } | MoveUnit { unit: UnitId(14), to: Hex { q: 5, r: 14 } }; MoveUnit { unit: UnitId(3), to: Hex { q: 17, r: 20 } } | +25.0 [+6.1, +43.9] | +0.2 [-0.3, +0.6] | Different attack or movement changes immediate combat and position. |
| 49 | founding site and timing | FoundCity { unit: UnitId(39) }; MoveUnit { unit: UnitId(44), to: Hex { q: 12, r: 19 } } | MoveUnit { unit: UnitId(39), to: Hex { q: 12, r: 5 } }; MoveUnitPath { unit: UnitId(44), to: Hex { q: 21, r: 13 } | +0.0 [+0.0, +0.0] | -1.3 [-1.9, -0.7] | Founding or Settler movement changes the colony path. |
| 68 | Gold purchase | c6:Building { building: Walls }:234g | none | +4.2 [-1.5, +9.9] | +0.8 [+0.5, +1.0] | Different Gold spending or purchase timing on this turn. |
| 19 | army movement | MoveUnit { unit: UnitId(14), to: Hex { q: 20, r: 15 } }; SetAutomation { unit: UnitId(16), automation: Some(Explore) | MoveUnit { unit: UnitId(14), to: Hex { q: 21, r: 15 } }; MoveUnit { unit: UnitId(16), to: Hex { q: 17, r: 21 } } | +0.0 [+0.0, +0.0] | +1.1 [+0.4, +1.8] | Different attack or movement changes immediate combat and position. |
| 53 | Gold purchase | c12:Building { building: Walls }:213g | none | +0.0 [+0.0, +0.0] | -0.9 [-1.8, -0.1] | Different Gold spending or purchase timing on this turn. |
| 30 | Settler and escort movement | SkipUnit { unit: UnitId(33) } | MoveUnitPath { unit: UnitId(33), to: Hex { q: 5, r: 12 } | +0.0 [+0.0, +0.0] | -0.7 [-1.2, -0.1] | Founding or Settler movement changes the colony path. |
| 73 | Settler and escort movement | standing movement to [3, 14] | MoveUnitPath { unit: UnitId(55), to: Hex { q: 1, r: 16 }; MoveUnitPath { unit: UnitId(62), to: Hex { q: 6, r: 21 } | +4.2 [-10.1, +18.4] | +0.2 [-0.1, +0.4] | Founding or Settler movement changes the colony path. |
| 58 | army movement | MoveUnit { unit: UnitId(14), to: Hex { q: 6, r: 6 } }; MoveUnitPath { unit: UnitId(3), to: Hex { q: 28, r: 13 } | MoveUnit { unit: UnitId(14), to: Hex { q: 7, r: 6 } }; MoveUnit { unit: UnitId(3), to: Hex { q: 14, r: 19 } } | +0.0 [+0.0, +0.0] | +0.5 [+0.1, +1.0] | Different attack or movement changes immediate combat and position. |
| 48 | army movement | MoveUnit { unit: UnitId(21), to: Hex { q: 12, r: 19 } }; MoveUnitPath { unit: UnitId(3), to: Hex { q: 30, r: 4 } | MoveUnit { unit: UnitId(3), to: Hex { q: 27, r: 6 } } | +0.0 [+0.0, +0.0] | -0.5 [-0.9, -0.0] | Different attack or movement changes immediate combat and position. |
| 58 | escort and garrison movement | none | MoveUnit { unit: UnitId(21), to: Hex { q: 13, r: 14 } } | +0.0 [+0.0, +0.0] | -0.4 [-0.8, +0.0] | Different garrison or escort movement changes the exposed unit position. |
| 68 | founding site and timing | FoundCity { unit: UnitId(65) } | none | +4.2 [-1.5, +9.9] | +0.0 [-0.4, +0.4] | Founding or Settler movement changes the colony path. |
| 52 | Gold purchase | c13:Building { building: Walls }:211g | none | +0.0 [+0.0, +0.0] | +0.4 [-0.2, +1.0] | Different Gold spending or purchase timing on this turn. |
| 48 | escort and garrison movement | none | MoveUnit { unit: UnitId(14), to: Hex { q: 27, r: 14 } } | +0.0 [+0.0, +0.0] | -0.3 [-0.8, +0.2] | Different garrison or escort movement changes the exposed unit position. |
| 59 | Gold purchase | c8:Building { building: Walls }:222g | none | +0.0 [+0.0, +0.0] | +0.2 [-0.2, +0.7] | Different Gold spending or purchase timing on this turn. |
| 87 | Gold purchase | none | c15:Unit { unit: Ranger }:162g; c18:Unit { unit: Ranger }:162g | +0.0 [+0.0, +0.0] | -0.2 [-0.4, -0.0] | Different Gold spending or purchase timing on this turn. |
| 46 | army movement | MoveUnit { unit: UnitId(2), to: Hex { q: 19, r: 2 } }; MoveUnitPath { unit: UnitId(21), to: Hex { q: 13, r: 18 } | MoveUnit { unit: UnitId(2), to: Hex { q: 16, r: 4 } }; MoveUnit { unit: UnitId(21), to: Hex { q: 11, r: 18 } } | +0.0 [+0.0, +0.0] | +0.1 [-0.0, +0.2] | Different attack or movement changes immediate combat and position. |
| 86 | Gold purchase | c11:Building { building: Walls }:259g | c11:Unit { unit: Soldier }:129g; c18:Unit { unit: Ranger }:162g | +0.0 [+0.0, +0.0] | +0.1 [-0.0, +0.2] | Different Gold spending or purchase timing on this turn. |
| 60 | Gold purchase | none | c8:Unit { unit: Soldier }:112g | +0.0 [+0.0, +0.0] | -0.0 [-0.1, +0.1] | Different Gold spending or purchase timing on this turn. |
| 67 | city production (improvement) | Coinspire: Some(Improvement { improvement: Farm, tile: Hex { q: 12, r: 7 } }) | Coinspire: Some(Unit { unit: Ranger }) | +0.0 [+0.0, +0.0] | -0.0 [-0.1, +0.1] | Different city production remains in effect until Hard changes it. |
| 46 | escort and garrison movement | none | MoveUnit { unit: UnitId(14), to: Hex { q: 9, r: 16 } } | +0.0 [+0.0, +0.0] | +0.0 [-0.1, +0.1] | Different garrison or escort movement changes the exposed unit position. |
| 47 | Settler and escort movement | MoveUnitPath { unit: UnitId(39), to: Hex { q: 16, r: 3 } | MoveUnitPath { unit: UnitId(39), to: Hex { q: 17, r: 2 } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Founding or Settler movement changes the colony path. |
| 77 | army movement | none | MoveUnit { unit: UnitId(64), to: Hex { q: 10, r: 18 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different attack or movement changes immediate combat and position. |

## Method and limits

Engine: `playtest-8.5` (d52050470eff). Replay: 110 turns, 0 hash mismatches; 11 AI strategic order mismatches in 330 seat turns.
Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; 24 valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: 2304 runs, 48 salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T111 or an earlier finish.
The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these 48 runs, not that the choices are equivalent.

Raw paired runs and diagnostics: `tollkeepers-2026-10-09-8.5-decision-diff.json` (not kept in the repository; regenerate it with `tools/decision-audit/designer-diff.sh`).
