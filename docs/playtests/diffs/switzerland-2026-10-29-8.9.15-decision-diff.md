# Decision diff: Switzerland (playtest-8.9.15)

## Top five disagreements

Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over 48 paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.

1. **T124 founding site and timing**: +0.0 [+0.0, +0.0] win percentage points; -1.2 [-1.7, -0.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
2. **T109 city production (building)**: +0.0 [+0.0, +0.0] win percentage points; +1.0 [+0.4, +1.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
3. **T106 Gold purchase**: +0.0 [+0.0, +0.0] win percentage points; -1.0 [-1.7, -0.2] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
4. **T136 founding site and timing**: +0.0 [+0.0, +0.0] win percentage points; +0.7 [+0.2, +1.3] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
5. **T44 Gold purchase**: +0.0 [+0.0, +0.0] win percentage points; -0.5 [-1.0, +0.1] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.

## Ranked decisions

| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |
|---:|---|---|---|---:|---:|---|
| 124 | founding site and timing | FoundCity { unit: UnitId(91) } | MoveUnitPath { unit: UnitId(91), to: Hex { q: 13, r: 9 } | +0.0 [+0.0, +0.0] | -1.2 [-1.7, -0.6] | Founding or Settler movement changes the colony path. |
| 109 | city production (building) | Lucerne: Some(Building { building: MachineShop }) | Lucerne: Some(Unit { unit: Soldier }) | +0.0 [+0.0, +0.0] | +1.0 [+0.4, +1.6] | Different city production remains in effect until Hard changes it. |
| 106 | Gold purchase | c12:Unit { unit: Marksman }:374g | c23:Unit { unit: Soldier }:140g | +0.0 [+0.0, +0.0] | -1.0 [-1.7, -0.2] | Different Gold spending or purchase timing on this turn. |
| 136 | founding site and timing | FoundCity { unit: UnitId(114) } | MoveUnitPath { unit: UnitId(114), to: Hex { q: 12, r: 13 } | +0.0 [+0.0, +0.0] | +0.7 [+0.2, +1.3] | Founding or Settler movement changes the colony path. |
| 44 | Gold purchase | none | c7:Unit { unit: Soldier }:140g | +0.0 [+0.0, +0.0] | -0.5 [-1.0, +0.1] | Different Gold spending or purchase timing on this turn. |
| 105 | founding site and timing | FoundCity { unit: UnitId(79) } | MoveUnit { unit: UnitId(79), to: Hex { q: 9, r: 9 } } | +0.0 [+0.0, +0.0] | -0.5 [-1.1, +0.2] | Founding or Settler movement changes the colony path. |
| 43 | war and peace declarations | none | DeclareWar { against: TeamId(3), breach: false } | +0.0 [+0.0, +0.0] | +0.4 [+0.1, +0.8] | War or peace orders change the immediate diplomatic state. |
| 100 | army movement | none | MoveUnit { unit: UnitId(3), to: Hex { q: 9, r: 10 } } | +0.0 [+0.0, +0.0] | -0.4 [-1.1, +0.2] | Different attack or movement changes immediate combat and position. |
| 102 | attack target | RangedAttack { unit: UnitId(32), target: Hex { q: 3, r: 11 } } | MoveUnit { unit: UnitId(32), to: Hex { q: 4, r: 13 } } | +0.0 [+0.0, +0.0] | +0.4 [-0.3, +1.1] | Different attack or movement changes immediate combat and position. |
| 118 | escort and garrison movement | MoveUnit { unit: UnitId(2), to: Hex { q: 12, r: 15 } } | MoveUnit { unit: UnitId(2), to: Hex { q: 10, r: 16 } } | +0.0 [+0.0, +0.0] | +0.4 [-0.3, +1.0] | Different garrison or escort movement changes the exposed unit position. |
| 43 | Gold purchase | none | c7:Unit { unit: Horseman }:192g | +0.0 [+0.0, +0.0] | +0.4 [-0.2, +0.9] | Different Gold spending or purchase timing on this turn. |
| 121 | city production (Settler) | Geneva: Some(Unit { unit: Settler }) | Geneva: Some(Unit { unit: Soldier }) | +0.0 [+0.0, +0.0] | -0.4 [-1.0, +0.3] | Different city production remains in effect until Hard changes it. |
| 136 | city production (Settler) | Geneva: Some(Unit { unit: Settler }) | Geneva: Some(Unit { unit: Ranger }) | +0.0 [+0.0, +0.0] | +0.4 [-0.0, +0.7] | Different city production remains in effect until Hard changes it. |
| 99 | Gold purchase | none | c12:Unit { unit: Soldier }:140g; c7:Unit { unit: Soldier }:140g | +0.0 [+0.0, +0.0] | -0.3 [-1.0, +0.4] | Different Gold spending or purchase timing on this turn. |
| 7 | city production (Settler) | Lucerne: Some(Unit { unit: Settler }) | Lucerne: Some(Unit { unit: Ranger }) | +0.0 [+0.0, +0.0] | -0.3 [-0.6, -0.0] | Different city production remains in effect until Hard changes it. |
| 84 | city production (building) | Basel: Some(Building { building: MachineShop }) | Basel: Some(Unit { unit: Soldier }) | +0.0 [+0.0, +0.0] | -0.3 [-0.9, +0.4] | Different city production remains in effect until Hard changes it. |
| 11 | city production (building) | Lucerne: Some(Building { building: Walls }) | Lucerne: Some(Unit { unit: Settler }) | +0.0 [+0.0, +0.0] | +0.2 [-0.2, +0.7] | Different city production remains in effect until Hard changes it. |
| 32 | founding site and timing | FoundCity { unit: UnitId(23) } | MoveUnit { unit: UnitId(23), to: Hex { q: 4, r: 15 } } | +0.0 [+0.0, +0.0] | -0.2 [-0.6, +0.1] | Founding or Settler movement changes the colony path. |
| 56 | army movement | none | MoveUnit { unit: UnitId(2), to: Hex { q: 12, r: 14 } } | +0.0 [+0.0, +0.0] | +0.2 [-0.2, +0.6] | Different attack or movement changes immediate combat and position. |
| 98 | Gold purchase | none | c12:Unit { unit: Soldier }:140g | +0.0 [+0.0, +0.0] | -0.2 [-0.7, +0.3] | Different Gold spending or purchase timing on this turn. |
| 41 | Gold purchase | none | c7:Unit { unit: Soldier }:140g | +0.0 [+0.0, +0.0] | -0.1 [-0.6, +0.4] | Different Gold spending or purchase timing on this turn. |
| 42 | Gold purchase | none | c7:Unit { unit: Horseman }:192g | +0.0 [+0.0, +0.0] | +0.1 [-0.4, +0.6] | Different Gold spending or purchase timing on this turn. |
| 44 | war and peace declarations | none | DeclareWar { against: TeamId(3), breach: false } | +0.0 [+0.0, +0.0] | +0.1 [-0.1, +0.3] | War or peace orders change the immediate diplomatic state. |
| 45 | war and peace declarations | none | DeclareWar { against: TeamId(3), breach: false } | +0.0 [+0.0, +0.0] | +0.1 [-0.4, +0.5] | War or peace orders change the immediate diplomatic state. |
| 122 | founding site and timing | FoundCity { unit: UnitId(96) }; MoveUnitPath { unit: UnitId(91), to: Hex { q: 15, r: 14 } | MoveUnitPath { unit: UnitId(91), to: Hex { q: 13, r: 10 }; MoveUnitPath { unit: UnitId(96), to: Hex { q: 21, r: 2 } | +0.0 [+0.0, +0.0] | -0.1 [-0.9, +0.7] | Founding or Settler movement changes the colony path. |
| 107 | Gold purchase | none | c23:Unit { unit: Soldier }:140g | +0.0 [+0.0, +0.0] | -0.0 [-0.8, +0.7] | Different Gold spending or purchase timing on this turn. |
| 157 | founding site and timing | FoundCity { unit: UnitId(137) } | MoveUnit { unit: UnitId(137), to: Hex { q: 9, r: 12 } } | +0.0 [+0.0, +0.0] | +0.0 [-0.4, +0.5] | Founding or Settler movement changes the colony path. |
| 46 | army movement | SetAutomation { unit: UnitId(2), automation: Some(Explore) | MoveUnit { unit: UnitId(2), to: Hex { q: 13, r: 16 } } | +0.0 [+0.0, +0.0] | +0.0 [-0.6, +0.6] | Different attack or movement changes immediate combat and position. |
| 51 | army movement | SleepUnit { unit: UnitId(2) } | MoveUnit { unit: UnitId(2), to: Hex { q: 12, r: 14 } } | +0.0 [+0.0, +0.0] | +0.0 [-0.3, +0.3] | Different attack or movement changes immediate combat and position. |
| 64 | army movement | MoveUnitPath { unit: UnitId(2), to: Hex { q: 10, r: 17 } | MoveUnit { unit: UnitId(2), to: Hex { q: 0, r: 16 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different attack or movement changes immediate combat and position. |

## Method and limits

Engine: `playtest-8.9.15` (060f78ef4d22). Replay: 180 turns, 0 hash mismatches; 0 AI strategic order mismatches in 531 seat turns.
Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; 30 valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: 2880 runs, 48 salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T181 or an earlier finish.
The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these 48 runs, not that the choices are equivalent.

Raw paired runs and diagnostics: `switzerland-2026-10-29-8.9.15-decision-diff.json` (not kept in the repository; regenerate it with `tools/decision-audit/designer-diff.sh`).
