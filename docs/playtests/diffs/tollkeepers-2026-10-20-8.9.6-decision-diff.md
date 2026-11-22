# Decision diff: Tollkeepers (playtest-8.9.6)

## Top five disagreements

Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over 48 paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.

1. **T42 Gold purchase**: +0.0 [+0.0, +0.0] win percentage points; +1.0 [+0.3, +1.8] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
2. **T125 city production (building)**: -2.1 [-6.2, +2.0] win percentage points; +0.8 [+0.3, +1.4] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
3. **T109 Gold purchase**: +0.0 [+0.0, +0.0] win percentage points; +0.9 [+0.2, +1.7] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
4. **T42 army movement**: +0.0 [+0.0, +0.0] win percentage points; +0.9 [+0.2, +1.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
5. **T14 city production (building)**: +0.0 [+0.0, +0.0] win percentage points; -0.8 [-1.7, -0.0] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.

## Ranked decisions

| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |
|---:|---|---|---|---:|---:|---|
| 42 | Gold purchase | c16:Unit { unit: Phalangite }:136g | none | +0.0 [+0.0, +0.0] | +1.0 [+0.3, +1.8] | Different Gold spending or purchase timing on this turn. |
| 125 | city production (building) | Thun: Some(Building { building: Clearinghouse }) | Thun: Some(Unit { unit: Marksman }) | -2.1 [-6.2, +2.0] | +0.8 [+0.3, +1.4] | Different city production remains in effect until Hard changes it. |
| 109 | Gold purchase | none | c1:Unit { unit: Soldier }:140g; c31:Unit { unit: Soldier }:140g | +0.0 [+0.0, +0.0] | +0.9 [+0.2, +1.7] | Different Gold spending or purchase timing on this turn. |
| 42 | army movement | SetAutomation { unit: UnitId(2), automation: None; SetAutomation { unit: UnitId(32), automation: Some(Explore) | MoveUnit { unit: UnitId(2), to: Hex { q: 14, r: 7 } }; MoveUnit { unit: UnitId(32), to: Hex { q: 9, r: 22 } } | +0.0 [+0.0, +0.0] | +0.9 [+0.2, +1.6] | Different attack or movement changes immediate combat and position. |
| 14 | city production (building) | Bellinzona: Some(Building { building: Walls }) | Bellinzona: Some(Unit { unit: Settler }) | +0.0 [+0.0, +0.0] | -0.8 [-1.7, -0.0] | Different city production remains in effect until Hard changes it. |
| 27 | Gold purchase | none | c1:Unit { unit: Soldier }:87g | +0.0 [+0.0, +0.0] | +0.8 [-0.0, +1.7] | Different Gold spending or purchase timing on this turn. |
| 29 | Gold purchase | none | c1:Unit { unit: Soldier }:89g | +0.0 [+0.0, +0.0] | +0.7 [+0.0, +1.3] | Different Gold spending or purchase timing on this turn. |
| 147 | founding site and timing | FoundCity { unit: UnitId(240) } | none | +0.0 [+0.0, +0.0] | +0.6 [-0.1, +1.4] | Founding or Settler movement changes the colony path. |
| 47 | war and peace declarations | RespondToProposal { proposal: ProposalId(21), accept: false | RespondToProposal { proposal: ProposalId(21), accept: true | +0.0 [+0.0, +0.0] | +0.5 [-0.1, +1.2] | War or peace orders change the immediate diplomatic state. |
| 27 | war and peace declarations | RespondToProposal { proposal: ProposalId(7), accept: false | RespondToProposal { proposal: ProposalId(7), accept: true | +0.0 [+0.0, +0.0] | +0.4 [-0.3, +1.1] | War or peace orders change the immediate diplomatic state. |
| 48 | city production (Settler) | Bellinzona: Some(Unit { unit: Settler }) | Bellinzona: Some(Unit { unit: Phalangite }) | +0.0 [+0.0, +0.0] | -0.4 [-1.1, +0.2] | Different city production remains in effect until Hard changes it. |
| 75 | city production (Settler) | Bellinzona: Some(Unit { unit: Settler }) | Bellinzona: Some(Unit { unit: Soldier }) | +0.0 [+0.0, +0.0] | +0.4 [-0.3, +1.2] | Different city production remains in effect until Hard changes it. |
| 111 | Gold purchase | none | c31:Building { building: Granary }:175g | +2.1 [-2.0, +6.2] | -0.2 [-0.9, +0.5] | Different Gold spending or purchase timing on this turn. |
| 43 | army movement | SetAutomation { unit: UnitId(2), automation: Some(Explore); SetAutomation { unit: UnitId(32), automation: Some(Explore) | MoveUnit { unit: UnitId(2), to: Hex { q: 11, r: 16 } }; MoveUnit { unit: UnitId(32), to: Hex { q: 27, r: 19 } } | +0.0 [+0.0, +0.0] | +0.4 [-0.3, +1.1] | Different attack or movement changes immediate combat and position. |
| 103 | founding site and timing | standing movement to [-2, 21] | FoundCity { unit: UnitId(160) } | +0.0 [+0.0, +0.0] | +0.4 [-0.3, +1.0] | Founding or Settler movement changes the colony path. |
| 46 | war and peace declarations | none | DeclareWar { against: TeamId(5), breach: false } | +0.0 [+0.0, +0.0] | -0.3 [-0.9, +0.2] | War or peace orders change the immediate diplomatic state. |
| 83 | founding site and timing | FoundCity { unit: UnitId(127) } | MoveUnitPath { unit: UnitId(127), to: Hex { q: 3, r: 24 } | +0.0 [+0.0, +0.0] | +0.3 [-0.4, +1.0] | Founding or Settler movement changes the colony path. |
| 104 | founding site and timing | FoundCity { unit: UnitId(160) } | MoveUnit { unit: UnitId(160), to: Hex { q: -1, r: 20 } } | +0.0 [+0.0, +0.0] | +0.3 [-0.3, +1.0] | Founding or Settler movement changes the colony path. |
| 57 | army movement | MoveUnit { unit: UnitId(80), to: Hex { q: 2, r: 27 } }; SetAutomation { unit: UnitId(32), automation: Some(Explore) | MoveUnit { unit: UnitId(32), to: Hex { q: 38, r: 11 } }; MoveUnit { unit: UnitId(80), to: Hex { q: 3, r: 24 } } | +0.0 [+0.0, +0.0] | -0.3 [-1.0, +0.4] | Different attack or movement changes immediate combat and position. |
| 110 | Gold purchase | none | c31:Unit { unit: Soldier }:140g | +0.0 [+0.0, +0.0] | +0.3 [-0.3, +0.8] | Different Gold spending or purchase timing on this turn. |
| 108 | Gold purchase | none | c1:Unit { unit: Soldier }:140g; c31:Unit { unit: Soldier }:140g | -2.1 [-6.2, +2.0] | +0.1 [-0.7, +0.8] | Different Gold spending or purchase timing on this turn. |
| 44 | army movement | MoveUnitPath { unit: UnitId(43), to: Hex { q: 14, r: 8 }; SetAutomation { unit: UnitId(2), automation: None | MoveUnit { unit: UnitId(2), to: Hex { q: 9, r: 22 } }; MoveUnit { unit: UnitId(32), to: Hex { q: 29, r: 19 } } | +0.0 [+0.0, +0.0] | +0.2 [-0.3, +0.8] | Different attack or movement changes immediate combat and position. |
| 161 | escort and garrison movement | MoveUnit { unit: UnitId(80), to: Hex { q: 7, r: 24 } } | MoveUnit { unit: UnitId(275), to: Hex { q: 5, r: 25 } } | +0.0 [+0.0, +0.0] | -0.2 [-0.8, +0.5] | Different garrison or escort movement changes the exposed unit position. |
| 36 | founding site and timing | FoundCity { unit: UnitId(57) } | MoveUnitPath { unit: UnitId(57), to: Hex { q: 3, r: 19 } | +0.0 [+0.0, +0.0] | -0.1 [-0.6, +0.4] | Founding or Settler movement changes the colony path. |
| 67 | city production (Settler) | Bellinzona: Some(Unit { unit: Settler }) | Bellinzona: Some(Unit { unit: CompositeArcher }) | +0.0 [+0.0, +0.0] | -0.1 [-0.9, +0.7] | Different city production remains in effect until Hard changes it. |
| 122 | city production (building) | Biel: Some(Building { building: Market }) | Biel: Some(Unit { unit: Marksman }) | +0.0 [+0.0, +0.0] | +0.1 [-0.6, +0.8] | Different city production remains in effect until Hard changes it. |
| 28 | Gold purchase | none | c1:Unit { unit: Soldier }:89g | +0.0 [+0.0, +0.0] | -0.1 [-0.7, +0.5] | Different Gold spending or purchase timing on this turn. |
| 72 | army movement | SetAutomation { unit: UnitId(32), automation: Some(Explore) | MoveUnit { unit: UnitId(32), to: Hex { q: 8, r: 18 } }; MoveUnit { unit: UnitId(80), to: Hex { q: 3, r: 24 } } | +0.0 [+0.0, +0.0] | +0.1 [-0.7, +0.9] | Different attack or movement changes immediate combat and position. |
| 161 | attack target | MoveUnit { unit: UnitId(102), to: Hex { q: 4, r: 23 } }; MoveUnitPath { unit: UnitId(276), to: Hex { q: 2, r: 20 } | MoveUnit { unit: UnitId(276), to: Hex { q: 3, r: 24 } }; RangedAttack { unit: UnitId(102), target: Hex { q: 4, r: 25 } } | +0.0 [+0.0, +0.0] | -0.1 [-0.7, +0.5] | Different attack or movement changes immediate combat and position. |
| 155 | founding site and timing | FoundCity { unit: UnitId(254) } | MoveUnitPath { unit: UnitId(254), to: Hex { q: 7, r: 25 } | +0.0 [+0.0, +0.0] | +0.1 [-0.7, +0.8] | Founding or Settler movement changes the colony path. |

## Method and limits

Engine: `playtest-8.9.6` (6328686221ab). Replay: 244 turns, 0 hash mismatches; 0 AI strategic order mismatches in 1708 seat turns.
Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; 30 valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: 2880 runs, 48 salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T245 or an earlier finish.
The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these 48 runs, not that the choices are equivalent.

Raw paired runs and diagnostics: `tollkeepers-2026-10-20-8.9.6-decision-diff.json` (not kept in the repository; regenerate it with `tools/decision-audit/designer-diff.sh`).
