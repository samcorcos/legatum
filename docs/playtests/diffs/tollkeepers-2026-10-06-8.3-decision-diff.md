# Decision diff: Tollkeepers (playtest-8.3)

## Top five disagreements

Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over 48 paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.

1. **T44 founding site and timing**: +81.2 [+70.1, +92.4] win percentage points; +2.2 [+1.9, +2.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
2. **T36 founding site and timing**: +25.0 [+11.3, +38.7] win percentage points; +2.0 [+1.5, +2.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
3. **T23 city production (building)**: -20.8 [-37.3, -4.4] win percentage points; -1.0 [-1.7, -0.3] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
4. **T41 city production (building)**: +8.3 [-6.9, +23.6] win percentage points; +0.3 [-0.2, +0.8] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
5. **T63 Gold purchase**: -4.2 [-9.9, +1.5] win percentage points; -0.0 [-0.1, +0.0] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.

## Ranked decisions

| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |
|---:|---|---|---|---:|---:|---|
| 44 | founding site and timing | FoundCity { unit: UnitId(19) } | MoveUnit { unit: UnitId(19), to: Hex { q: 5, r: 8 } } | +81.2 [+70.1, +92.4] | +2.2 [+1.9, +2.6] | Founding or Settler movement changes the colony path. |
| 36 | founding site and timing | FoundCity { unit: UnitId(15) } | MoveUnit { unit: UnitId(15), to: Hex { q: 9, r: 5 } } | +25.0 [+11.3, +38.7] | +2.0 [+1.5, +2.6] | Founding or Settler movement changes the colony path. |
| 23 | city production (building) | Halden Crossing: Some(Building { building: Granary }) | Halden Crossing: Some(Unit { unit: Settler }) | -20.8 [-37.3, -4.4] | -1.0 [-1.7, -0.3] | Different city production remains in effect until Hard changes it. |
| 41 | city production (building) | Marrow Quay: Some(Building { building: Market }) | Marrow Quay: Some(Unit { unit: Soldier }) | +8.3 [-6.9, +23.6] | +0.3 [-0.2, +0.8] | Different city production remains in effect until Hard changes it. |
| 63 | Gold purchase | none | c5:Unit { unit: Ranger }:141g | -4.2 [-9.9, +1.5] | -0.0 [-0.1, +0.0] | Different Gold spending or purchase timing on this turn. |
| 63 | army movement | MoveUnitPath { unit: UnitId(2), to: Hex { q: 21, r: 11 } | MoveUnit { unit: UnitId(2), to: Hex { q: 18, r: 8 } }; MoveUnit { unit: UnitId(3), to: Hex { q: 3, r: 13 } } | -4.2 [-9.9, +1.5] | -0.0 [-0.1, +0.0] | Different attack or movement changes immediate combat and position. |
| 51 | city production (Settler) | Marrow Quay: Some(Unit { unit: Settler }) | Marrow Quay: Some(Building { building: Walls }) | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different city production remains in effect until Hard changes it. |
| 58 | army movement | SetAutomation { unit: UnitId(2), automation: None | MoveUnit { unit: UnitId(2), to: Hex { q: 17, r: 3 } }; MoveUnit { unit: UnitId(3), to: Hex { q: 3, r: 13 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different attack or movement changes immediate combat and position. |
| 61 | Gold purchase | none | c1:Unit { unit: Ranger }:140g; c3:Unit { unit: Ranger }:140g | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different Gold spending or purchase timing on this turn. |
| 61 | army movement | MoveUnitPath { unit: UnitId(2), to: Hex { q: 19, r: 10 } | MoveUnit { unit: UnitId(2), to: Hex { q: 16, r: 10 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different attack or movement changes immediate combat and position. |
| 62 | Gold purchase | c1:Building { building: Walls }:225g | c1:Unit { unit: Ranger }:141g; c3:Building { building: Walls }:225g | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different Gold spending or purchase timing on this turn. |
| 62 | city production (building) | Aster Vault: Some(Building { building: Walls }) | Aster Vault: None | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different city production remains in effect until Hard changes it. |
| 65 | Gold purchase | none | c7:Improvement { improvement: Mine, tile: Hex { q: 4, r: 11 } }:115g | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different Gold spending or purchase timing on this turn. |
| 65 | army movement | none | MoveUnit { unit: UnitId(2), to: Hex { q: 19, r: 9 } }; MoveUnit { unit: UnitId(3), to: Hex { q: 8, r: 16 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different attack or movement changes immediate combat and position. |
| 66 | Gold purchase | none | c7:Building { building: Granary }:145g | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different Gold spending or purchase timing on this turn. |
| 66 | army movement | MoveUnitPath { unit: UnitId(3), to: Hex { q: 3, r: 13 } | MoveUnit { unit: UnitId(2), to: Hex { q: 17, r: 8 } }; MoveUnit { unit: UnitId(24), to: Hex { q: 5, r: 8 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different attack or movement changes immediate combat and position. |
| 67 | Gold purchase | none | c3:Building { building: Workshop }:175g | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different Gold spending or purchase timing on this turn. |
| 67 | city production (building) | Aster Vault: Some(Building { building: Workshop }) | Aster Vault: Some(Unit { unit: Soldier }) | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different city production remains in effect until Hard changes it. |
| 68 | Gold purchase | none | c3:Building { building: Workshop }:175g | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different Gold spending or purchase timing on this turn. |
| 69 | Gold purchase | none | c3:Building { building: Workshop }:176g | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different Gold spending or purchase timing on this turn. |
| 69 | city production (building) | Aster Vault: Some(Building { building: Workshop }) | Aster Vault: Some(Building { building: Laboratory }) | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different city production remains in effect until Hard changes it. |
| 71 | army movement | MoveUnitPath { unit: UnitId(22), to: Hex { q: 7, r: 11 } | MoveUnit { unit: UnitId(2), to: Hex { q: 7, r: 10 } }; MoveUnit { unit: UnitId(22), to: Hex { q: 6, r: 11 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different attack or movement changes immediate combat and position. |
| 71 | founding site and timing | FoundCity { unit: UnitId(30) } | MoveUnit { unit: UnitId(30), to: Hex { q: 12, r: 6 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Founding or Settler movement changes the colony path. |
| 72 | escort and garrison movement | none | MoveUnit { unit: UnitId(24), to: Hex { q: 7, r: 15 } }; MoveUnit { unit: UnitId(8), to: Hex { q: 6, r: 16 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different garrison or escort movement changes the exposed unit position. |

## Method and limits

Engine: `playtest-8.3` (6e3780bf03ee). Replay: 91 turns, 0 hash mismatches; 0 AI strategic order mismatches in 91 seat turns.
Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; 24 valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: 2304 runs, 48 salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T92 or an earlier finish.
The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these 48 runs, not that the choices are equivalent.

Raw paired runs and diagnostics: `tollkeepers-2026-10-06-8.3-decision-diff.json` (not kept in the repository; regenerate it with `tools/decision-audit/designer-diff.sh`).
