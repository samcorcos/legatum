# Decision diff: Tollkeepers (playtest-8.9.4)

## Top five disagreements

Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over 48 paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.

1. **T31 founding site and timing**: +0.0 [+0.0, +0.0] win percentage points; +1.3 [+0.9, +1.8] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
2. **T196 army movement**: -2.1 [-15.8, +11.6] win percentage points; -0.9 [-1.2, -0.7] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
3. **T47 army movement**: +0.0 [+0.0, +0.0] win percentage points; -1.1 [-2.0, -0.1] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
4. **T155 escort and garrison movement**: -2.1 [-6.2, +2.0] win percentage points; +0.6 [+0.1, +1.2] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
5. **T48 army movement**: +0.0 [+0.0, +0.0] win percentage points; +0.7 [-0.1, +1.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.

## Ranked decisions

| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |
|---:|---|---|---|---:|---:|---|
| 31 | founding site and timing | FoundCity { unit: UnitId(50) } | MoveUnit { unit: UnitId(50), to: Hex { q: 34, r: 6 } } | +0.0 [+0.0, +0.0] | +1.3 [+0.9, +1.8] | Founding or Settler movement changes the colony path. |
| 196 | army movement | none | MoveUnit { unit: UnitId(2), to: Hex { q: 28, r: 11 } }; MoveUnit { unit: UnitId(25), to: Hex { q: 15, r: 20 } } | -2.1 [-15.8, +11.6] | -0.9 [-1.2, -0.7] | Different attack or movement changes immediate combat and position. |
| 47 | army movement | none | MoveUnit { unit: UnitId(2), to: Hex { q: 29, r: 9 } }; MoveUnit { unit: UnitId(25), to: Hex { q: 16, r: 20 } } | +0.0 [+0.0, +0.0] | -1.1 [-2.0, -0.1] | Different attack or movement changes immediate combat and position. |
| 155 | escort and garrison movement | MoveUnitPath { unit: UnitId(3), to: Hex { q: 32, r: 11 } | none | -2.1 [-6.2, +2.0] | +0.6 [+0.1, +1.2] | Different garrison or escort movement changes the exposed unit position. |
| 48 | army movement | MoveUnitPath { unit: UnitId(79), to: Hex { q: 22, r: 15 } | MoveUnit { unit: UnitId(2), to: Hex { q: 28, r: 9 } }; MoveUnit { unit: UnitId(25), to: Hex { q: 18, r: 18 } } | +0.0 [+0.0, +0.0] | +0.7 [-0.1, +1.6] | Different attack or movement changes immediate combat and position. |
| 57 | founding site and timing | FoundCity { unit: UnitId(92) } | MoveUnitPath { unit: UnitId(92), to: Hex { q: 34, r: 20 } | +0.0 [+0.0, +0.0] | -0.6 [-1.4, +0.3] | Founding or Settler movement changes the colony path. |
| 155 | army movement | MoveUnitPath { unit: UnitId(167), to: Hex { q: 26, r: 15 }; MoveUnitPath { unit: UnitId(168), to: Hex { q: 27, r: 16 } | MoveUnit { unit: UnitId(167), to: Hex { q: 23, r: 16 } }; MoveUnit { unit: UnitId(168), to: Hex { q: 26, r: 17 } } | +0.0 [+0.0, +0.0] | +0.5 [-0.2, +1.1] | Different attack or movement changes immediate combat and position. |
| 51 | city production (Settler) | Bellinzona: Some(Unit { unit: Settler }) | Bellinzona: Some(Building { building: Walls }) | +0.0 [+0.0, +0.0] | -0.4 [-1.1, +0.4] | Different city production remains in effect until Hard changes it. |
| 111 | Gold purchase | none | c21:Building { building: Walls }:280g | +0.0 [+0.0, +0.0] | -0.4 [-0.9, +0.2] | Different Gold spending or purchase timing on this turn. |
| 9 | Gold purchase | none | c1:Improvement { improvement: Pasture, tile: Hex { q: 25, r: 16 } }:56g | +0.0 [+0.0, +0.0] | +0.4 [-0.2, +0.9] | Different Gold spending or purchase timing on this turn. |
| 10 | Gold purchase | none | c1:Improvement { improvement: Pasture, tile: Hex { q: 25, r: 16 } }:57g | +0.0 [+0.0, +0.0] | +0.4 [-0.3, +1.0] | Different Gold spending or purchase timing on this turn. |
| 46 | army movement | none | MoveUnit { unit: UnitId(2), to: Hex { q: 29, r: 9 } }; MoveUnit { unit: UnitId(25), to: Hex { q: 16, r: 20 } } | +0.0 [+0.0, +0.0] | +0.4 [-0.5, +1.2] | Different attack or movement changes immediate combat and position. |
| 102 | city production (building) | Bellinzona: Some(Building { building: Granary }) | Bellinzona: Some(Unit { unit: Settler }) | +0.0 [+0.0, +0.0] | -0.4 [-1.1, +0.4] | Different city production remains in effect until Hard changes it. |
| 106 | escort and garrison movement | MoveUnitPath { unit: UnitId(79), to: Hex { q: 20, r: 17 } | none | -2.1 [-6.2, +2.0] | -0.1 [-0.8, +0.5] | Different garrison or escort movement changes the exposed unit position. |
| 81 | Settler and escort movement | MoveUnitPath { unit: UnitId(117), to: Hex { q: 22, r: 20 } | none | +0.0 [+0.0, +0.0] | +0.3 [-0.4, +1.0] | Founding or Settler movement changes the colony path. |
| 44 | city production (building) | Lugano: Some(Building { building: Walls }) | Lugano: Some(Building { building: Market }) | +0.0 [+0.0, +0.0] | -0.3 [-1.0, +0.5] | Different city production remains in effect until Hard changes it. |
| 128 | war and peace declarations | none | DeclareWar { against: TeamId(6), breach: false } | -2.1 [-6.2, +2.0] | -0.1 [-0.7, +0.6] | War or peace orders change the immediate diplomatic state. |
| 131 | founding site and timing | FoundCity { unit: UnitId(155) } | MoveUnit { unit: UnitId(155), to: Hex { q: 17, r: 23 } } | +0.0 [+0.0, +0.0] | +0.3 [-0.4, +1.0] | Founding or Settler movement changes the colony path. |
| 110 | Gold purchase | none | c21:Building { building: Walls }:280g | +0.0 [+0.0, +0.0] | +0.2 [-0.5, +1.0] | Different Gold spending or purchase timing on this turn. |
| 51 | war and peace declarations | RespondToProposal { proposal: ProposalId(9), accept: true | RespondToProposal { proposal: ProposalId(9), accept: false | +0.0 [+0.0, +0.0] | -0.2 [-1.0, +0.5] | War or peace orders change the immediate diplomatic state. |
| 8 | Gold purchase | none | c1:Improvement { improvement: Farm, tile: Hex { q: 24, r: 18 } }:56g | +0.0 [+0.0, +0.0] | +0.2 [-0.2, +0.6] | Different Gold spending or purchase timing on this turn. |
| 46 | Gold purchase | c14:Unit { unit: CompositeArcher }:213g; c1:Unit { unit: Horseman }:140g | c14:Building { building: Walls }:203g | +0.0 [+0.0, +0.0] | +0.2 [-0.6, +1.0] | Different Gold spending or purchase timing on this turn. |
| 109 | Gold purchase | none | c21:Building { building: Walls }:280g | +0.0 [+0.0, +0.0] | -0.2 [-0.9, +0.5] | Different Gold spending or purchase timing on this turn. |
| 112 | Gold purchase | none | c21:Building { building: Walls }:280g | +0.0 [+0.0, +0.0] | -0.2 [-0.8, +0.4] | Different Gold spending or purchase timing on this turn. |
| 32 | army movement | MoveUnitPath { unit: UnitId(30), to: Hex { q: 14, r: 16 }; SetAutomation { unit: UnitId(25), automation: Some(Explore) | MoveUnit { unit: UnitId(25), to: Hex { q: 15, r: 25 } }; MoveUnit { unit: UnitId(28), to: Hex { q: 28, r: 14 } } | +0.0 [+0.0, +0.0] | +0.2 [-0.4, +0.7] | Different attack or movement changes immediate combat and position. |
| 98 | city production (building) | Bellinzona: Some(Building { building: Granary }) | Bellinzona: Some(Unit { unit: Settler }) | +0.0 [+0.0, +0.0] | -0.1 [-0.7, +0.6] | Different city production remains in effect until Hard changes it. |
| 105 | escort and garrison movement | MoveUnitPath { unit: UnitId(78), to: Hex { q: 20, r: 20 }; MoveUnitPath { unit: UnitId(79), to: Hex { q: 23, r: 16 } | none | +0.0 [+0.0, +0.0] | -0.1 [-0.4, +0.3] | Different garrison or escort movement changes the exposed unit position. |
| 88 | city production (building) | Bellinzona: Some(Building { building: Granary }) | Bellinzona: Some(Unit { unit: Settler }) | +0.0 [+0.0, +0.0] | -0.0 [-0.8, +0.7] | Different city production remains in effect until Hard changes it. |
| 2 | city production (unit) | Bellinzona: Some(Unit { unit: Scout }) | Bellinzona: Some(Building { building: Granary }) | +0.0 [+0.0, +0.0] | +0.0 [-0.5, +0.5] | Different city production remains in effect until Hard changes it. |
| 196 | escort and garrison movement | none | MoveUnit { unit: UnitId(209), to: Hex { q: 20, r: 15 } }; MoveUnit { unit: UnitId(78), to: Hex { q: 13, r: 15 } } | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different garrison or escort movement changes the exposed unit position. |

## Method and limits

Engine: `playtest-8.9.4` (eeccf4800d39). Replay: 246 turns, 0 hash mismatches; 0 AI strategic order mismatches in 1641 seat turns.
Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; 30 valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: 2880 runs, 48 salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T247 or an earlier finish.
The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these 48 runs, not that the choices are equivalent.

Raw paired runs and diagnostics: `tollkeepers-2026-10-18-8.9.4-decision-diff.json` (not kept in the repository; regenerate it with `tools/decision-audit/designer-diff.sh`).
