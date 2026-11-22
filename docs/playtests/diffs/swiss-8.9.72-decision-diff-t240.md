# Decision diff: Switzerland (playtest-8.9.72)

## Top five disagreements

Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over 48 paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.

1. **T130 Gold purchase**: +31.2 [+11.8, +50.7] win percentage points; +1.2 [+0.2, +2.1] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
2. **T124 city production (building)**: -22.9 [-39.7, -6.2] win percentage points; -0.4 [-1.4, +0.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
3. **T129 army movement**: +16.7 [-0.2, +33.5] win percentage points; +0.8 [+0.0, +1.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
4. **T129 Gold purchase**: -8.3 [-24.7, +8.0] win percentage points; -0.9 [-2.0, +0.1] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
5. **T103 Gold purchase**: +8.3 [-8.0, +24.7] win percentage points; +0.7 [-0.7, +2.1] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.

## Ranked decisions

| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |
|---:|---|---|---|---:|---:|---|
| 130 | Gold purchase | c11:Unit { unit: Cavalry }:595g; c26:Unit { unit: Cavalry }:595g | c35:Unit { unit: Rifleman }:456g | +31.2 [+11.8, +50.7] | +1.2 [+0.2, +2.1] | Different Gold spending or purchase timing on this turn. |
| 124 | city production (building) | Thun: Some(Building { building: Clearinghouse }) | Thun: Some(Unit { unit: Soldier }) | -22.9 [-39.7, -6.2] | -0.4 [-1.4, +0.6] | Different city production remains in effect until Hard changes it. |
| 129 | army movement | MoveUnit { unit: UnitId(77), to: Hex { q: 30, r: 16 } } | MoveUnit { unit: UnitId(137), to: Hex { q: 31, r: 15 } }; MoveUnit { unit: UnitId(138), to: Hex { q: 31, r: 15 } } | +16.7 [-0.2, +33.5] | +0.8 [+0.0, +1.6] | Different attack or movement changes immediate combat and position. |
| 129 | Gold purchase | none | c35:Unit { unit: Cavalry }:595g | -8.3 [-24.7, +8.0] | -0.9 [-2.0, +0.1] | Different Gold spending or purchase timing on this turn. |
| 103 | Gold purchase | none | c26:Unit { unit: AssaultTrooper }:367g | +8.3 [-8.0, +24.7] | +0.7 [-0.7, +2.1] | Different Gold spending or purchase timing on this turn. |
| 124 | Gold purchase | none | c35:Improvement { improvement: Mine, tile: Hex { q: 26, r: 19 } }:70g | +2.1 [-17.7, +21.9] | -1.2 [-2.1, -0.3] | Different Gold spending or purchase timing on this turn. |
| 115 | Gold purchase | c11:Unit { unit: Cuirassier }:486g; c20:Unit { unit: Cuirassier }:486g | c1:Unit { unit: Cuirassier }:486g | +4.2 [-15.2, +23.5] | -0.6 [-2.1, +0.8] | Different Gold spending or purchase timing on this turn. |
| 120 | war and peace declarations | ProposeTreaty { to: TeamId(4), kind: Peace } | none | -6.2 [-15.3, +2.8] | -0.2 [-0.9, +0.5] | War or peace orders change the immediate diplomatic state. |
| 120 | Gold purchase | none | c31:Unit { unit: Knight }:357g | -6.2 [-26.8, +14.3] | +0.2 [-0.9, +1.3] | Different Gold spending or purchase timing on this turn. |
| 130 | army movement | MoveUnit { unit: UnitId(77), to: Hex { q: 30, r: 16 } } | MoveUnit { unit: UnitId(77), to: Hex { q: 30, r: 17 } } | -4.2 [-20.6, +12.3] | +0.3 [-0.9, +1.5] | Different attack or movement changes immediate combat and position. |
| 79 | Gold purchase | none | c26:Unit { unit: Ranger }:231g | +0.0 [+0.0, +0.0] | -0.7 [-1.3, -0.1] | Different Gold spending or purchase timing on this turn. |
| 77 | army movement | MoveUnit { unit: UnitId(15), to: Hex { q: 26, r: 15 } }; MoveUnit { unit: UnitId(2), to: Hex { q: 26, r: 15 } } | MoveUnit { unit: UnitId(1), to: Hex { q: 22, r: 9 } }; MoveUnit { unit: UnitId(15), to: Hex { q: 23, r: 13 } } | +0.0 [+0.0, +0.0] | +0.5 [-0.1, +1.2] | Different attack or movement changes immediate combat and position. |
| 103 | war and peace declarations | ProposeTreaty { to: TeamId(4), kind: Peace } | none | +2.1 [-7.1, +11.3] | +0.2 [-0.3, +0.8] | War or peace orders change the immediate diplomatic state. |
| 91 | city production (building) | Bellinzona: Some(Building { building: Laboratory }) | Bellinzona: Some(Unit { unit: AssaultTrooper }) | -4.2 [-19.6, +11.2] | -0.0 [-1.3, +1.3] | Different city production remains in effect until Hard changes it. |
| 79 | army movement | SetAutomation { unit: UnitId(1), automation: Some(Explore) | MoveUnit { unit: UnitId(1), to: Hex { q: 22, r: 9 } }; MoveUnit { unit: UnitId(15), to: Hex { q: 26, r: 15 } } | +0.0 [+0.0, +0.0] | -0.4 [-1.0, +0.2] | Different attack or movement changes immediate combat and position. |
| 91 | war and peace declarations | ProposeTreaty { to: TeamId(4), kind: Peace } | none | -2.1 [-13.0, +8.8] | -0.2 [-0.8, +0.4] | War or peace orders change the immediate diplomatic state. |

## Method and limits

Engine: `playtest-8.9.72` (ef96a4702cfa). Replay: 150 turns, 0 hash mismatches; 0 AI strategic order mismatches in 750 seat turns.
Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; 16 valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: 1536 runs, 48 salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T240 or an earlier finish.
The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these 48 runs, not that the choices are equivalent.

Raw paired runs and diagnostics: `diff240.json` (not kept in the repository; regenerate it with `tools/decision-audit/designer-diff.sh`).
