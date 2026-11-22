# Decision diff: Signalwrights (playtest-8.9.6)

## Top five disagreements

Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over 48 paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.

1. **T141 city production (building)**: +6.2 [-4.5, +17.0] win percentage points; +0.6 [-0.1, +1.3] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
2. **T95 Gold purchase**: +6.2 [-4.5, +17.0] win percentage points; +0.5 [-0.4, +1.4] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
3. **T41 founding site and timing**: +0.0 [-8.3, +8.3] win percentage points; -1.1 [-1.9, -0.2] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
4. **T96 Gold purchase**: -6.2 [-19.8, +7.3] win percentage points; -0.4 [-1.3, +0.6] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
5. **T88 founding site and timing**: +2.1 [-11.6, +15.8] win percentage points; -0.8 [-1.7, +0.2] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.

## Ranked decisions

| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |
|---:|---|---|---|---:|---:|---|
| 141 | city production (building) | Marathon: Some(Building { building: Workshop }) | Marathon: Some(Building { building: Market }) | +6.2 [-4.5, +17.0] | +0.6 [-0.1, +1.3] | Different city production remains in effect until Hard changes it. |
| 95 | Gold purchase | none | c21:Improvement { improvement: Farm, tile: Hex { q: 30, r: 20 } }:116g | +6.2 [-4.5, +17.0] | +0.5 [-0.4, +1.4] | Different Gold spending or purchase timing on this turn. |
| 41 | founding site and timing | FoundCity { unit: UnitId(59) } | MoveUnit { unit: UnitId(59), to: Hex { q: 19, r: 25 } } | +0.0 [-8.3, +8.3] | -1.1 [-1.9, -0.2] | Founding or Settler movement changes the colony path. |
| 96 | Gold purchase | none | c21:Improvement { improvement: Farm, tile: Hex { q: 30, r: 20 } }:116g | -6.2 [-19.8, +7.3] | -0.4 [-1.3, +0.6] | Different Gold spending or purchase timing on this turn. |
| 88 | founding site and timing | FoundCity { unit: UnitId(120) } | MoveUnit { unit: UnitId(120), to: Hex { q: 15, r: 28 } } | +2.1 [-11.6, +15.8] | -0.8 [-1.7, +0.2] | Founding or Settler movement changes the colony path. |
| 98 | Gold purchase | none | c21:Improvement { improvement: Farm, tile: Hex { q: 30, r: 20 } }:118g | +8.3 [-1.5, +18.2] | -0.1 [-0.9, +0.6] | Different Gold spending or purchase timing on this turn. |
| 19 | Gold purchase | none | c1:Improvement { improvement: Pasture, tile: Hex { q: 21, r: 24 } }:70g | +0.0 [+0.0, +0.0] | +0.8 [+0.3, +1.4] | Different Gold spending or purchase timing on this turn. |
| 48 | army movement | SetAutomation { unit: UnitId(2), automation: Some(Explore) | MoveUnit { unit: UnitId(2), to: Hex { q: 20, r: 19 } } | -4.2 [-14.2, +5.9] | -0.3 [-1.2, +0.5] | Different attack or movement changes immediate combat and position. |
| 97 | Gold purchase | none | c21:Improvement { improvement: Farm, tile: Hex { q: 30, r: 20 } }:118g | +0.0 [-10.1, +10.1] | +0.7 [-0.1, +1.5] | Different Gold spending or purchase timing on this turn. |
| 126 | escort and garrison movement | none | MoveUnit { unit: UnitId(102), to: Hex { q: 32, r: 17 } }; MoveUnit { unit: UnitId(2), to: Hex { q: 16, r: 25 } } | -4.2 [-12.3, +4.0] | -0.3 [-0.9, +0.3] | Different garrison or escort movement changes the exposed unit position. |
| 21 | Gold purchase | none | c1:Improvement { improvement: Pasture, tile: Hex { q: 21, r: 24 } }:72g | -2.1 [-6.2, +2.0] | -0.5 [-0.9, -0.1] | Different Gold spending or purchase timing on this turn. |
| 67 | attack target | MoveUnit { unit: UnitId(2), to: Hex { q: 20, r: 25 } }; MoveUnit { unit: UnitId(62), to: Hex { q: 19, r: 23 } } | MoveUnit { unit: UnitId(102), to: Hex { q: 19, r: 25 } }; MoveUnit { unit: UnitId(2), to: Hex { q: 22, r: 24 } } | +4.2 [-7.4, +15.8] | -0.3 [-1.1, +0.6] | Different attack or movement changes immediate combat and position. |
| 110 | founding site and timing | FoundCity { unit: UnitId(151) } | MoveUnit { unit: UnitId(151), to: Hex { q: 32, r: 19 } } | +2.1 [-5.0, +9.2] | -0.4 [-1.0, +0.2] | Founding or Settler movement changes the colony path. |
| 66 | war and peace declarations | RespondToProposal { proposal: ProposalId(36), accept: false | RespondToProposal { proposal: ProposalId(36), accept: true | -4.2 [-14.2, +5.9] | +0.1 [-0.8, +1.1] | War or peace orders change the immediate diplomatic state. |
| 166 | city production (building) | Athens: Some(Building { building: Clearinghouse }) | Athens: Some(Unit { unit: Phalangite }) | +4.2 [-8.8, +17.2] | +0.1 [-0.7, +1.0] | Different city production remains in effect until Hard changes it. |
| 49 | army movement | SetAutomation { unit: UnitId(2), automation: Some(Explore) | MoveUnit { unit: UnitId(2), to: Hex { q: 21, r: 17 } } | +0.0 [-5.8, +5.8] | -0.5 [-1.1, +0.2] | Different attack or movement changes immediate combat and position. |
| 121 | escort and garrison movement | none | MoveUnit { unit: UnitId(102), to: Hex { q: 32, r: 17 } }; MoveUnit { unit: UnitId(2), to: Hex { q: 16, r: 25 } } | +0.0 [-10.1, +10.1] | +0.5 [-0.3, +1.3] | Different garrison or escort movement changes the exposed unit position. |
| 117 | city production (Settler) | Athens: Some(Unit { unit: Settler }) | Athens: Some(Building { building: MachineShop }) | -2.1 [-11.3, +7.1] | -0.2 [-1.0, +0.5] | Different city production remains in effect until Hard changes it. |
| 32 | war and peace declarations | ProposeTreaty { to: TeamId(2), kind: Peace } | none | +2.1 [-2.0, +6.2] | +0.2 [-0.3, +0.7] | War or peace orders change the immediate diplomatic state. |
| 140 | city production (building) | Laurion: Some(Building { building: Clearinghouse }) | Laurion: Some(Unit { unit: Phalangite }) | +0.0 [-8.3, +8.3] | +0.4 [-0.3, +1.2] | Different city production remains in effect until Hard changes it. |
| 25 | founding site and timing | MoveUnit { unit: UnitId(35), to: Hex { q: 25, r: 21 } } | FoundCity { unit: UnitId(35) }; MoveUnitPath { unit: UnitId(41), to: Hex { q: 23, r: 25 } | +2.1 [-2.0, +6.2] | +0.2 [-0.5, +0.9] | Founding or Settler movement changes the colony path. |
| 26 | founding site and timing | MoveUnit { unit: UnitId(35), to: Hex { q: 24, r: 21 } } | FoundCity { unit: UnitId(35) }; MoveUnit { unit: UnitId(41), to: Hex { q: 23, r: 25 } } | +0.0 [+0.0, +0.0] | -0.4 [-1.0, +0.3] | Founding or Settler movement changes the colony path. |
| 20 | Gold purchase | none | c1:Improvement { improvement: Pasture, tile: Hex { q: 21, r: 24 } }:72g | +0.0 [+0.0, +0.0] | +0.3 [-0.1, +0.7] | Different Gold spending or purchase timing on this turn. |
| 45 | army movement | SetAutomation { unit: UnitId(2), automation: Some(Explore) | MoveUnit { unit: UnitId(2), to: Hex { q: 18, r: 17 } } | -2.1 [-6.2, +2.0] | -0.1 [-0.7, +0.5] | Different attack or movement changes immediate combat and position. |
| 27 | founding site and timing | MoveUnit { unit: UnitId(35), to: Hex { q: 23, r: 21 } }; MoveUnit { unit: UnitId(41), to: Hex { q: 24, r: 24 } } | FoundCity { unit: UnitId(35) }; FoundCity { unit: UnitId(41) } | +0.0 [-5.8, +5.8] | +0.2 [-0.5, +1.0] | Founding or Settler movement changes the colony path. |
| 68 | army movement | MoveUnit { unit: UnitId(98), to: Hex { q: 20, r: 22 } } | MoveUnit { unit: UnitId(98), to: Hex { q: 23, r: 21 } } | +0.0 [-10.1, +10.1] | +0.2 [-0.6, +1.0] | Different attack or movement changes immediate combat and position. |
| 47 | army movement | SetAutomation { unit: UnitId(2), automation: Some(Explore) | MoveUnit { unit: UnitId(2), to: Hex { q: 20, r: 19 } } | +0.0 [-5.8, +5.8] | +0.1 [-0.4, +0.7] | Different attack or movement changes immediate combat and position. |
| 18 | Gold purchase | none | c1:Improvement { improvement: Pasture, tile: Hex { q: 21, r: 24 } }:70g | +0.0 [+0.0, +0.0] | +0.1 [-0.4, +0.6] | Different Gold spending or purchase timing on this turn. |
| 44 | war and peace declarations | RespondToProposal { proposal: ProposalId(16), accept: false | RespondToProposal { proposal: ProposalId(16), accept: true | +0.0 [+0.0, +0.0] | +0.1 [-0.1, +0.3] | War or peace orders change the immediate diplomatic state. |
| 230 | city production (Settler) | Phaleron: Some(Unit { unit: Settler }) | Phaleron: Some(Unit { unit: Phalangite }) | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | Different city production remains in effect until Hard changes it. |

## Method and limits

Engine: `playtest-8.9.6` (6328686221ab). Replay: 233 turns, 0 hash mismatches; 1 AI strategic order mismatches in 1374 seat turns.
Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; 30 valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: 2880 runs, 48 salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T234 or an earlier finish.
The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these 48 runs, not that the choices are equivalent.

Raw paired runs and diagnostics: `signalwrights-2026-10-19-8.9.6-decision-diff.json` (not kept in the repository; regenerate it with `tools/decision-audit/designer-diff.sh`).
