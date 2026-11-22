---
name: legatum-rules
description: This skill should be used before proposing, designing, explaining or reviewing any Legatum game mechanic, faction, opening, build order, AI behaviour, balance lever or rule change, and whenever a rule or mechanic changes. It holds a code-verified reference of how the game actually works (orders, turn resolution, turn-1 availability, costs, systems that do NOT exist) so recommendations are possible in this game rather than borrowed from Civ 6.
---

# Legatum rules

Legatum looks like Civ 6 but differs in ways that matter. Recommendations reasoned from "Civ in general" have repeatedly been impossible here:
- A Workshop-first opening (Workshop needs Bronze Working; at turn 1 only the Granary, the turn-1 units (Scout, Spearman, Archer, Horseman) and a Settler at population 2 are buildable).
- Hit-and-run (one order per unit per turn, simultaneous resolution).
- Traffic tolls (no trade routes).
- Moving cities (not a thing).

## Before recommending anything

1. Open `references/index.md`, then read only the system files for the mechanics involved (for example `combat.md`, `economy-and-purchases.md`), and always `references/not-in-game.md` ("Systems that do NOT exist"). Do not load the whole folder: the AI files alone are about 250 KB.
2. Check feasibility against it:
   - **Order model.** Can a player express this with one order per unit or city per turn, under simultaneous resolution? Does it need a new order type or new UI?
   - **Availability.** Is every building, unit or tech it relies on available when the plan uses it? Check unlock techs and costs at the relevant speed.
   - **Existing systems.** Does it depend on a system that doesn't exist? If so, it's a new mechanic: say so explicitly and weigh the cost (the user dislikes adding whole mechanics for one faction).
   - **Story fit.** Does it follow from the faction's or system's story? (See the faction-design memory: no arbitrary number tweaks.)
3. If the reference is silent or unclear, verify in code (crates/core first) before stating it as fact. Never present an assumption as a rule.
4. When describing an AI opening or build order, describe what actually happens turn by turn (e.g. from a recorded tape), not the configured priority list.

## Keeping it true (mandatory)

Any change to rules, mechanics, factions, costs, orders, defaults or AI architecture must update the matching system file under `references/` (see `references/index.md`) in the same change, and `references/history.md` for a rules-version bump. That covers changes by Claude, Astra (Codex) or any agent. Correct wrong entries immediately, and keep citations current as `file::function`, never line numbers. Astra write tasks that change rules must include "update the matching file under .claude/skills/legatum-rules/references/" in their prompt, and the review checks it.
