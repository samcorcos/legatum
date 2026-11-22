# Decision diff: Signalwrights (playtest-8.9.5)

## Top five disagreements

Paired results below are his category orders minus Hard's, with both seats continued by Hard. Positive favors his choice. The intervals are approximate 95% intervals over 48 paired reseeded dice runs. When a gate rejects an option before scoring, the trace has no comparable valuation gap; the listed threshold or preliminary site score is the available evidence.

1. **T32 Settler and escort movement**: +0.0 [+0.0, +0.0] win percentage points; -1.9 [-2.5, -1.4] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
2. **T137 founding site and timing**: +4.2 [-4.0, +12.3] win percentage points; +1.2 [+0.7, +1.8] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
3. **T118 founding site and timing**: +2.1 [-2.0, +6.2] win percentage points; +1.1 [+0.5, +1.7] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
4. **T8 Gold purchase**: -2.1 [-6.2, +2.0] win percentage points; -1.0 [-1.8, -0.3] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.
5. **T12 city production (Settler)**: +0.0 [+0.0, +0.0] win percentage points; -1.1 [-1.8, -0.4] VP. Attribution unresolved: the available diagnostic has no comparable score for both orders.

## Ranked decisions

| Turn | Category | His choice | Hard's choice | Win Δ pp [95% interval] | VP Δ [95% interval] | Why |
|---:|---|---|---|---:|---:|---|
| 32 | Settler and escort movement | MoveUnitPath { unit: UnitId(54), to: Hex { q: 14, r: 25 } | MoveUnit { unit: UnitId(54), to: Hex { q: 16, r: 23 } } | +0.0 [+0.0, +0.0] | -1.9 [-2.5, -1.4] | Founding or Settler movement changes the colony path. |
| 137 | founding site and timing | FoundCity { unit: UnitId(107) } | MoveUnit { unit: UnitId(107), to: Hex { q: 3, r: 29 } } | +4.2 [-4.0, +12.3] | +1.2 [+0.7, +1.8] | Founding or Settler movement changes the colony path. |
| 118 | founding site and timing | FoundCity { unit: UnitId(156) } | MoveUnitPath { unit: UnitId(156), to: Hex { q: 4, r: 29 } | +2.1 [-2.0, +6.2] | +1.1 [+0.5, +1.7] | Founding or Settler movement changes the colony path. |
| 8 | Gold purchase | none | c1:Building { building: Granary }:108g | -2.1 [-6.2, +2.0] | -1.0 [-1.8, -0.3] | Different Gold spending or purchase timing on this turn. |
| 12 | city production (Settler) | Eleusis: Some(Unit { unit: Settler }) | Eleusis: None | +0.0 [+0.0, +0.0] | -1.1 [-1.8, -0.4] | Different city production remains in effect until Hard changes it. |
| 138 | city production (building) | Eleusis: Some(Building { building: Workshop }) | Eleusis: Some(Building { building: Hydroponics }) | +6.2 [-0.7, +13.2] | +0.4 [-0.1, +0.8] | Different city production remains in effect until Hard changes it. |
| 58 | founding site and timing | FoundCity { unit: UnitId(91) } | MoveUnit { unit: UnitId(91), to: Hex { q: 12, r: 23 } } | +2.1 [-2.0, +6.2] | +0.8 [+0.3, +1.3] | Founding or Settler movement changes the colony path. |
| 33 | army movement | MoveUnit { unit: UnitId(3), to: Hex { q: 9, r: 24 } }; SetAutomation { unit: UnitId(2), automation: None | MoveUnit { unit: UnitId(2), to: Hex { q: 3, r: 27 } }; MoveUnit { unit: UnitId(3), to: Hex { q: 11, r: 23 } } | +0.0 [+0.0, +0.0] | +1.0 [+0.4, +1.6] | Different attack or movement changes immediate combat and position. |
| 108 | Gold purchase | c26:Unit { unit: Marksman }:428g | c26:Unit { unit: Soldier }:160g; c33:Unit { unit: Soldier }:160g | +4.2 [-1.5, +9.9] | +0.3 [-0.2, +0.8] | Different Gold spending or purchase timing on this turn. |
| 112 | war and peace declarations | RespondToProposal { proposal: ProposalId(25), accept: false | RespondToProposal { proposal: ProposalId(25), accept: true | +2.1 [-2.0, +6.2] | +0.5 [-0.0, +1.1] | War or peace orders change the immediate diplomatic state. |
| 37 | city production (Settler) | Eleusis: Some(Unit { unit: Settler }) | Eleusis: Some(Unit { unit: Soldier }) | +0.0 [+0.0, +0.0] | +0.5 [-0.2, +1.2] | Different city production remains in effect until Hard changes it. |
| 136 | escort and garrison movement | none | MoveUnit { unit: UnitId(3), to: Hex { q: 11, r: 24 } } | +4.2 [-1.5, +9.9] | +0.0 [-0.4, +0.5] | Different garrison or escort movement changes the exposed unit position. |
| 43 | city production (Settler) | Marathon: Some(Unit { unit: Settler }) | Marathon: Some(Unit { unit: CompositeArcher }) | +0.0 [+0.0, +0.0] | -0.5 [-0.9, -0.0] | Different city production remains in effect until Hard changes it. |
| 136 | army movement | MoveUnitPath { unit: UnitId(124), to: Hex { q: -1, r: 28 } | MoveUnit { unit: UnitId(124), to: Hex { q: 3, r: 29 } } | +2.1 [-5.0, +9.2] | +0.2 [-0.2, +0.6] | Different attack or movement changes immediate combat and position. |
| 9 | city production (building) | Eleusis: Some(Building { building: Granary }) | Eleusis: Some(Unit { unit: Settler }) | +0.0 [+0.0, +0.0] | -0.4 [-1.0, +0.2] | Different city production remains in effect until Hard changes it. |
| 114 | founding site and timing | FoundCity { unit: UnitId(161) }; MoveUnitPath { unit: UnitId(156), to: Hex { q: 3, r: 29 } | MoveUnit { unit: UnitId(161), to: Hex { q: 10, r: 29 } } | -2.1 [-6.2, +2.0] | -0.1 [-0.6, +0.3] | Founding or Settler movement changes the colony path. |
| 35 | army movement | MoveUnit { unit: UnitId(3), to: Hex { q: 11, r: 23 } } | MoveUnit { unit: UnitId(2), to: Hex { q: 2, r: 29 } }; MoveUnit { unit: UnitId(3), to: Hex { q: 10, r: 23 } } | +0.0 [+0.0, +0.0] | -0.3 [-0.9, +0.3] | Different attack or movement changes immediate combat and position. |
| 115 | war and peace declarations | ProposeTreaty { to: TeamId(7), kind: Peace } | none | +0.0 [+0.0, +0.0] | +0.3 [-0.2, +0.8] | War or peace orders change the immediate diplomatic state. |
| 123 | escort and garrison movement | MoveUnit { unit: UnitId(124), to: Hex { q: 3, r: 29 } } | MoveUnit { unit: UnitId(3), to: Hex { q: 11, r: 24 } }; RangedAttack { unit: UnitId(124), target: Hex { q: 2, r: 28 } } | +2.1 [-5.0, +9.2] | -0.1 [-0.7, +0.6] | Different garrison or escort movement changes the exposed unit position. |
| 42 | army movement | MoveUnit { unit: UnitId(3), to: Hex { q: 13, r: 22 } } | MoveUnit { unit: UnitId(3), to: Hex { q: 15, r: 20 } } | +0.0 [+0.0, +0.0] | +0.3 [-0.2, +0.7] | Different attack or movement changes immediate combat and position. |
| 7 | Gold purchase | none | c1:Building { building: Granary }:106g | +0.0 [+0.0, +0.0] | +0.2 [-0.4, +0.9] | Different Gold spending or purchase timing on this turn. |
| 10 | Gold purchase | none | c1:Improvement { improvement: Pasture, tile: Hex { q: 15, r: 22 } }:66g | +0.0 [+0.0, +0.0] | +0.2 [-0.1, +0.5] | Different Gold spending or purchase timing on this turn. |
| 49 | army movement | MoveUnit { unit: UnitId(3), to: Hex { q: 9, r: 26 } } | MoveUnit { unit: UnitId(3), to: Hex { q: 11, r: 24 } } | +0.0 [+0.0, +0.0] | +0.1 [-0.4, +0.7] | Different attack or movement changes immediate combat and position. |
| 111 | Gold purchase | none | c1:Building { building: Laboratory }:280g | +0.0 [-8.3, +8.3] | +0.1 [-0.6, +0.9] | Different Gold spending or purchase timing on this turn. |
| 109 | Gold purchase | none | c26:Unit { unit: Soldier }:160g | +0.0 [-5.8, +5.8] | +0.1 [-0.5, +0.7] | Different Gold spending or purchase timing on this turn. |
| 110 | Gold purchase | none | c1:Building { building: Laboratory }:280g | +0.0 [+0.0, +0.0] | -0.1 [-0.7, +0.6] | Different Gold spending or purchase timing on this turn. |
| 111 | city production (building) | Eleusis: Some(Building { building: Laboratory }) | Eleusis: None | +0.0 [-8.3, +8.3] | -0.1 [-0.8, +0.7] | Different city production remains in effect until Hard changes it. |
| 115 | attack target | MoveUnitPath { unit: UnitId(159), to: Hex { q: 11, r: 24 }; MoveUnitPath { unit: UnitId(3), to: Hex { q: 15, r: 20 } | MoveUnit { unit: UnitId(3), to: Hex { q: 13, r: 19 } }; RangedAttack { unit: UnitId(159), target: Hex { q: 12, r: 20 } } | +0.0 [+0.0, +0.0] | +0.1 [-0.5, +0.6] | Different attack or movement changes immediate combat and position. |
| 9 | Gold purchase | none | c1:Improvement { improvement: Pasture, tile: Hex { q: 15, r: 22 } }:64g | +0.0 [+0.0, +0.0] | -0.0 [-0.7, +0.6] | Different Gold spending or purchase timing on this turn. |
| 172 | war and peace declarations | ProposeTreaty { to: TeamId(7), kind: Peace } | none | +0.0 [+0.0, +0.0] | +0.0 [+0.0, +0.0] | War or peace orders change the immediate diplomatic state. |

## Method and limits

Engine: `playtest-8.9.5` (64ad962f98c6). Replay: 229 turns, 0 hash mismatches; 0 AI strategic order mismatches in 1585 seat turns.
Candidate heuristic: category weights, resolved effects, early turn and active war priority, with category and phase caps plus two per turn; 30 valued candidates. The table ranks by absolute win change (percentage points) plus ten times absolute VP change; this is a display order, not an AI value function. Branches: 2880 runs, 48 salts, dice reseeded; each single category is swapped for one turn, then every seat plays Hard through T230 or an earlier finish.
The swap measures an order bundle in a category, not the independent value of every order inside it. When a branch finishes early, its final VP is used. Confidence intervals describe dice sampling only; they do not cover model bias, category interactions, or the fact that the recorded opponents also respond as Hard after the fork. A 0 [0, 0] interval means no difference appeared in these 48 runs, not that the choices are equivalent.

Raw paired runs and diagnostics: `signalwrights-2026-10-18-8.9.5-decision-diff.json` (not kept in the repository; regenerate it with `tools/decision-audit/designer-diff.sh`).
