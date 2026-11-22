# Decision diff: Switzerland (playtest-8.9.72)

## Top five disagreements

Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over 48 paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.

1. **T124 Gold purchase**: +0.0 [+0.0, +0.0] win percentage points; -2.0 [-2.5, -1.5] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
2. **T17 Settler and escort movement**: +0.0 [+0.0, +0.0] win percentage points; -1.9 [-2.4, -1.4] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
3. **T26 city production (building)**: +0.0 [+0.0, +0.0] win percentage points; -1.4 [-2.1, -0.7] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
4. **T24 escort and garrison movement**: +0.0 [+0.0, +0.0] win percentage points; +0.6 [+0.2, +1.1] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
5. **T23 escort and garrison movement**: +0.0 [+0.0, +0.0] win percentage points; -0.5 [-1.1, +0.0] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.

## Ranked decisions

| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |
|---:|---|---|---|---:|---:|---|
| 124 | Gold purchase | none | c35:Improvement { improvement: Mine, tile: Hex { q: 26, r: 19 } }:70g | +0.0 [+0.0, +0.0] | -2.0 [-2.5, -1.5] | Different Gold spending or purchase timing on this turn. |
| 17 | Settler and escort movement | MoveUnitPath { unit: UnitId(21), to: Hex { q: 34, r: 6 } | MoveUnit { unit: UnitId(21), to: Hex { q: 32, r: 10 } } | +0.0 [+0.0, +0.0] | -1.9 [-2.4, -1.4] | Founding or Settler movement changes the colony path. |
| 26 | city production (building) | Bellinzona: Some(Building { building: Walls }) | Bellinzona: Some(Unit { unit: Settler }) | +0.0 [+0.0, +0.0] | -1.4 [-2.1, -0.7] | Different city production remains in effect until Hard changes it. |
| 24 | escort and garrison movement | MoveUnit { unit: UnitId(2), to: Hex { q: 31, r: 10 } } | MoveUnit { unit: UnitId(2), to: Hex { q: 30, r: 7 } } | +0.0 [+0.0, +0.0] | +0.6 [+0.2, +1.1] | Different garrison or escort movement changes the exposed unit position. |
| 23 | escort and garrison movement | MoveUnit { unit: UnitId(2), to: Hex { q: 30, r: 11 } } | SkipUnit { unit: UnitId(2) } | +0.0 [+0.0, +0.0] | -0.5 [-1.1, +0.0] | Different garrison or escort movement changes the exposed unit position. |
| 11 | escort and garrison movement | SetAutomation { unit: UnitId(15), automation: Some(Explore) | SkipUnit { unit: UnitId(15) } | +0.0 [+0.0, +0.0] | -0.4 [-1.0, +0.2] | Different garrison or escort movement changes the exposed unit position. |
| 131 | war and peace declarations | RespondToProposal { proposal: ProposalId(43), accept: false | RespondToProposal { proposal: ProposalId(43), accept: true | +0.0 [+0.0, +0.0] | -0.4 [-0.8, -0.0] | War or peace orders change the immediate diplomatic state. |
| 121 | Gold purchase | none | c35:Improvement { improvement: Mine, tile: Hex { q: 26, r: 19 } }:70g | +0.0 [+0.0, +0.0] | +0.3 [-0.2, +0.8] | Different Gold spending or purchase timing on this turn. |
| 35 | founding site and timing | standing movement to [34, 7] | FoundCity { unit: UnitId(30) } | +0.0 [+0.0, +0.0] | -0.3 [-1.0, +0.4] | Founding or Settler movement changes the colony path. |
| 45 | army movement | MoveUnitPath { unit: UnitId(1), to: Hex { q: 15, r: 16 }; SetAutomation { unit: UnitId(15), automation: None | MoveUnit { unit: UnitId(1), to: Hex { q: 20, r: 12 } }; MoveUnit { unit: UnitId(15), to: Hex { q: 23, r: 21 } } | +0.0 [+0.0, +0.0] | -0.3 [-0.7, +0.1] | Different attack or movement changes immediate combat and position. |
| 60 | city production (building) | Lugano: Some(Building { building: Granary }) | Lugano: Some(Unit { unit: CompositeArcher }) | +0.0 [+0.0, +0.0] | -0.3 [-0.7, +0.0] | Different city production remains in effect until Hard changes it. |
| 104 | army movement | MoveUnitPath { unit: UnitId(77), to: Hex { q: 22, r: 17 }; SleepUnit { unit: UnitId(1), until_healed: false } | MoveUnit { unit: UnitId(1), to: Hex { q: 27, r: 13 } }; MoveUnit { unit: UnitId(77), to: Hex { q: 22, r: 18 } } | +0.0 [+0.0, +0.0] | -0.2 [-0.7, +0.2] | Different attack or movement changes immediate combat and position. |
| 123 | Gold purchase | none | c35:Improvement { improvement: Mine, tile: Hex { q: 26, r: 19 } }:70g | +0.0 [+0.0, +0.0] | +0.2 [-0.3, +0.8] | Different Gold spending or purchase timing on this turn. |
| 122 | Gold purchase | none | c35:Improvement { improvement: Mine, tile: Hex { q: 26, r: 19 } }:70g | +0.0 [+0.0, +0.0] | +0.2 [-0.3, +0.7] | Different Gold spending or purchase timing on this turn. |
| 79 | Gold purchase | none | c26:Unit { unit: Ranger }:231g | +0.0 [+0.0, +0.0] | -0.2 [-0.5, +0.2] | Different Gold spending or purchase timing on this turn. |
| 113 | founding site and timing | standing movement to [25, 19] | FoundCity { unit: UnitId(110) } | +0.0 [+0.0, +0.0] | -0.2 [-0.6, +0.2] | Founding or Settler movement changes the colony path. |
| 18 | Settler and escort movement | MoveUnitPath { unit: UnitId(21), to: Hex { q: 31, r: 10 } | MoveUnitPath { unit: UnitId(21), to: Hex { q: 34, r: 6 } | +0.0 [+0.0, +0.0] | -0.2 [-0.9, +0.5] | Founding or Settler movement changes the colony path. |
| 97 | army movement | MoveUnit { unit: UnitId(2), to: Hex { q: 35, r: 8 } }; MoveUnit { unit: UnitId(77), to: Hex { q: 22, r: 17 } } | MoveUnit { unit: UnitId(1), to: Hex { q: 21, r: 22 } }; MoveUnit { unit: UnitId(2), to: Hex { q: 30, r: 7 } } | +0.0 [+0.0, +0.0] | -0.2 [-0.7, +0.4] | Different attack or movement changes immediate combat and position. |
| 87 | Gold purchase | none | c20:Improvement { improvement: Mine, tile: Hex { q: 27, r: 9 } }:70g | +0.0 [+0.0, +0.0] | -0.1 [-0.6, +0.3] | Different Gold spending or purchase timing on this turn. |
| 41 | city production (improvement) | Lugano: Some(Improvement { improvement: Quarry, tile: Hex { q: 31, r: 11 } }) | Lugano: Some(Unit { unit: Settler }) | +0.0 [+0.0, +0.0] | -0.1 [-0.5, +0.2] | Different city production remains in effect until Hard changes it. |
| 88 | Gold purchase | none | c20:Improvement { improvement: Mine, tile: Hex { q: 27, r: 9 } }:70g | +0.0 [+0.0, +0.0] | -0.1 [-0.5, +0.2] | Different Gold spending or purchase timing on this turn. |
| 98 | Settler and escort movement | standing movement to [26, 11] | MoveUnit { unit: UnitId(110), to: Hex { q: 30, r: 7 } } | +0.0 [+0.0, +0.0] | -0.1 [-0.6, +0.3] | Founding or Settler movement changes the colony path. |
| 49 | army movement | SetAutomation { unit: UnitId(1), automation: Some(Explore); SetAutomation { unit: UnitId(15), automation: Some(Explore) | MoveUnit { unit: UnitId(1), to: Hex { q: 14, r: 19 } }; MoveUnit { unit: UnitId(15), to: Hex { q: 29, r: 8 } } | +0.0 [+0.0, +0.0] | +0.1 [-0.2, +0.5] | Different attack or movement changes immediate combat and position. |
| 65 | city production (building) | Lugano: Some(Building { building: Roadworks }) | Lugano: Some(Unit { unit: Settler }) | +0.0 [+0.0, +0.0] | +0.1 [-0.4, +0.6] | Different city production remains in effect until Hard changes it. |
| 79 | army movement | SetAutomation { unit: UnitId(1), automation: Some(Explore) | MoveUnit { unit: UnitId(1), to: Hex { q: 22, r: 9 } }; MoveUnit { unit: UnitId(15), to: Hex { q: 26, r: 15 } } | +0.0 [+0.0, +0.0] | +0.1 [-0.3, +0.4] | Different attack or movement changes immediate combat and position. |
| 1 | escort and garrison movement | MoveUnit { unit: UnitId(2), to: Hex { q: 29, r: 9 } }; MoveUnitPath { unit: UnitId(1), to: Hex { q: 28, r: 6 } | MoveUnit { unit: UnitId(1), to: Hex { q: 32, r: 7 } }; MoveUnit { unit: UnitId(2), to: Hex { q: 30, r: 7 } } | +0.0 [+0.0, +0.0] | -0.0 [-0.8, +0.7] | Different garrison or escort movement changes the exposed unit position. |
| 98 | army movement | SetAutomation { unit: UnitId(1), automation: Some(Explore); SkipUnit { unit: UnitId(77) } | MoveUnit { unit: UnitId(1), to: Hex { q: 20, r: 22 } }; MoveUnit { unit: UnitId(2), to: Hex { q: 30, r: 7 } } | +0.0 [+0.0, +0.0] | -0.0 [-0.5, +0.4] | Different attack or movement changes immediate combat and position. |
| 32 | Settler and escort movement | standing movement to [35, 9] | MoveUnitPath { unit: UnitId(30), to: Hex { q: 35, r: 7 } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Founding or Settler movement changes the colony path. |
| 84 | Gold purchase | none | c20:Improvement { improvement: Mine, tile: Hex { q: 27, r: 9 } }:70g | +0.0 [+0.0, +0.0] | +0.0 [-0.4, +0.4] | Different Gold spending or purchase timing on this turn. |
| 141 | war and peace declarations | RespondToProposal { proposal: ProposalId(48), accept: false | RespondToProposal { proposal: ProposalId(48), accept: true | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | War or peace orders change the immediate diplomatic state. |

## Method and limits

Engine: `playtest-8.9.72` (ef96a4702cfa). Replay: 150 turns, 0 hash mismatches; 0 AI strategic order mismatches in 750 seat turns.
Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; 30 valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: 2880 runs, 48 salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T151 or an earlier finish.
The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these 48 runs, not that the choices are equivalent.

Raw paired runs and diagnostics: `swiss-8.9.72-decision-diff.json` (not kept in the repository; regenerate it with `tools/decision-audit/designer-diff.sh`).
