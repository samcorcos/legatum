---
name: legatum-design
description: This skill should be used whenever proposing or evaluating Legatum game design (factions, openings, rules, victory, balance, AI strategy) or responding to the user's design ideas. It records the user's design philosophy, what was decided, and which ideas were rejected and why, so settled questions are not reopened.
---

# Legatum design: how we think

For anything about factions, also load `legatum-factions` (the faction method, current designs and lessons).

The source of truth for decisions is the design doc's decisions log (see CLAUDE.md). This skill holds the *reasoning style* and the rejected ideas. Load `legatum-rules` too: every idea must work in this game's actual rules.

Current principles and decisions, one line each with date and who. The user's full words, superseded decisions and the dated decision log are in `references/` (index at the bottom). Search them before proposing anything that sounds familiar.

## Principles the user holds

- **Factions are TI4, not Eclipse or Civ:** each breaks one core rule and pays for it with a constraint that follows from its story; no arbitrary number tweaks (user, 2026-10). Full method in `legatum-factions`.
- **Every mechanic must matter in this game** and serve explore, expand, build, compete; if a rule needs explaining, simplify it (user, 2026-10-17). Every rule fits one plain sentence.
- **Bonuses scale across eras:** percentages or existing progression, never flat numbers that fade.
- **Expansion is the core loop;** staying small is not a strategy (user, 2026-10-01).
- **Civ 6 familiarity is the default** (city strike only with Walls, Palace +2 Production, +1 sight on hills, own land plus one ring always visible) unless we deliberately fix a Civ problem (pillar 2; user, 2026-10-17).
- **Every tech a faction researches gives it something** (user, 2026-11-22). Researching is never a trade-off against yourself (user, 2026-11-01).
- **Rock-paper-scissors:** rush beats greedy expansion, Walls and archers beat rush, greedy expansion out-scales turtles. Early rushes may be final; the counter is scouting, then Walls and archers (user, 2026-10-17).
- **Walls are strong but beatable** and cost 100 Production (user, 2026-10-07, 2026-11-06). Bombers are meant to be very strong; Fighters and air superiority are the counter; never nerf Bombers or move Flight as a pacing fix (user, 2026-11-10, 2026-11-13).
- **The unit triangle ends at the Modern era** (user, 2026-10-20). One unit per class per era with small jumps (user, 2026-10-10); power jumps grow in later eras (user, 2026-10-18).
- **Each era is defined by its resource,** revealed by a tech of the era it is used in; deposits are deliberately not placed fairly; resources are a reason for war (user, 2026-10-22, 2026-11-07).
- **Conquest is the fastest economy and winning is zero-sum** (user, 2026-10-07/08). A leader may run away if the others can respond; rubber bands keep it close (pillar 11).
- **No exceptions for edge cases** (user, 2026-10-20); a faction exception the player cannot see is a bug.
- **No arbitrary caps, no tech boosts, no hard era cutovers** (user, 2026-10-17, 2026-10-09).
- **Every VP is a trade-off,** a VP is never bought with Gold or Production alone, and an Age card makes the player do something he would not otherwise do (user, 2026-10-20, 2026-11-13/15).
- **Game length:** the target is about 250 turns, with a calendar year beside the turn reaching about 2026 at turn 250; a win near turn 120 is a pacing failure; before adding a VP source show what it does to the end turn (user, 2026-11-11, 2026-11-14).
- **Mid-late game must not be boring;** research is its core (user, 2026-10-09).

## AI principles

- AI gets harder by deciding better, never by cheating (pillar 6); public treasuries are public to everyone (user, 2026-11-13).
- AI behaviour emerges from value scoring and search, not hard-coded rules the search cannot reach (user, 2026-10-22). AI strategy is search, not imitation (user, 2026-10-06).
- Fix weak AI play in the AI, not by bending rules (user, 2026-10-07; again for Rome, 2026-11-21).
- Information has strategic value: the AI is paranoid and values map visibility (user, 2026-10-24).
- Coalitions must be real (commit, strike together) and come from hidden wariness, not a visible meter; stop-the-leader is instinct (user, 2026-10-17, 2026-11-08, 2026-10-24).
- Army size follows intent, no middle ground (user, 2026-10-30).
- His games always train the AI (user, 2026-11-07, correcting an invented rule). No expensive cloud training batches (user, 2026-11-07).
- Measure how Hard feels against a human (the designer-seat score), not only AI against AI.

## Current decisions worth knowing before proposing

- Victory: hold the VP target for 10 turns, or take every capital (user, 2026-10-06, 2026-11-15). Every faction goal is worth exactly 2 VP. Crowns and decadence are deleted (user, 2026-11-11).
- Era scoring is World Ages only (owner, 2026-11-22, rules 50): the three-objective era arm and its switch are deleted; iterate on World Ages itself (cards, counts, pacing). Each Age's pool has three peaceful cards every faction can reach; one card is drawn per Age.
- At Peace, strength settles a contested tile (Sam, 2026-11-22, rules 51): units of empires not at War stepping onto one tile never both bounce; the team with the most combined strength among its claimants moves in (as force decides a fallen city), nobody fights, ties are a seeded draw; an exact force tie at a fallen city is a seeded draw too. Nobody is pushed off a tile they already stand on.
- Fable review batch (Sam, 2026-11-22, rules 50): a rival's original capital is 1 VP only while held (the Celtic sack keeps its point for good); Alliance upkeep is waived while at War with a near-winner; same-turn ties between empires are drawn by seed; co-op Gold gifts to a teammate exist; `no_purchase_under_threat` is an off-by-default experiment. First pass the owner may veto: wonder prices on one era curve, Star Fort at Machinery. Details: `references/decided-2026-10-31-onward.md`.
- Humble the mighty works for the leader (owner, 2026-11-22, rules 53, "Leader's card counts top rival"): when no rival team is ahead of yours, a capture from the rival with the most VP counts (ties included). Rejected with it: a default 300-turn limit, a per-empire redraw, leaving it.
- Diplomacy: Peace, Alliance or War; Alliances cost Gold per turn, no signing fee; defenders join at once and an ally's offensive war raises a call to arms (user, 2026-10-03, 2026-10-23, 2026-11-01). Folding calls into plain war was decided on 2026-11-11, but `WarCall` is still in `crates/core` on 2026-11-22: check the code before relying on either. Peace starts a 10-turn truce (user, 2026-10-25).
- Captures: a captured city loses half its population; the occupation lock is the rule in every game; any empire can raze a captured city (user, 2026-11-13, 2026-10-25/31).
- Start: every empire starts with its capital founded on a balanced site of 8 Food and 8 Production (user, 2026-11-07, 2026-11-17).
- Eras: nine (Machine Age added 2026-11-14; Modern between World War and Future). Standard is the only pace; Quick is removed (user, 2026-10-10).
- Buying tiles with Gold is back as a trial (user, 2026-11-20). City automation is the first delegation feature (user, 2026-10-31).
- Resolution order when a defender slips into a besieged city: slip in, then ranged, then melee (user, 2026-11-21).
- Saves from other rules versions are refused; no migrations (user, 2026-10-31).
- Going back a turn is allowed in local games (Sam, 2026-11-22): save-scumming is acceptable, because training should learn only from moves the player meant. Menu → Load an earlier turn opens any of the last 10 autosaved turns; the uploaded record keeps only the line played on and lists each rewind (`rewinds`). Online games cannot go back (the host would rewind everyone).
- Rules freeze: lifted 2026-11-11 (user). Talk before building tech-tree changes (user, 2026-11-01).

## How to propose

Use a short template: broken rule, flavour-linked constraint, path to 10 VP, visible counter. State the feasibility check result from `legatum-rules`. Offer a recommendation, not a menu. When the user pushes back, update this skill (one line here, detail in a reference) in the same turn.

- A building's effect must fit its identity; no dead buildings (a building that gives a faction nothing is not offered) (user and lead, 2026-10-05).
- Never propose dropping a resource requirement because the AI misuses it (user, 2026-11-01: "a flaw in ai gameplay").
- Every missing Civ 6 system has a recorded reason in the rules reference (`not-in-game.md`) (user, 2026-10-18).

## Rejected ideas (don't re-propose without new evidence)
- For Humble the mighty's dead card for the leader (owner, 2026-11-22): a default 300-turn limit (no turn limit by default stays the design), a per-empire redraw of a card it cannot score (breaks the one shared card per Age), and leaving it as it was.
- A fungible production pool that carries over to any item (too exploitable; the user rejected it).
- The map-reveal ruin reward ("cartographer's cache"): it felt useless, so it was removed on 2026-10-03. Ruin rewards should be things you feel.
- Removing tech diffusion: it's a rubber band that keeps games competitive (pillar 11). Keep it, and make it visible in research forecasts.
- A fixed number of turns for a hunter's reveal (not flavourful).
- Making Walls available earlier to counter rushes (2026-10-03). The counter to a rush is units: a ranged garrison or two puts up a serious fight. Walls are what end the rush era. Instead: the city absorbs damage before its garrison (Civ 6).
- Traffic tolls; moving cities; Mongols contracts; hit-and-run (needs two-step orders; impossible with simultaneous turns and one order per unit).
- Starting with 3 Settlers (breaks balance; the user chose one).
- The National Academy to rescue Tall (Tall is dropped instead).
- A "+50% unit cost until Steam era" Germany drawback; a flat +30 unit cost.
- Skip-resistance as the Sparta bonus ("weak, flavourless").
- Hard v2 full-game openings as the app's Hard (easier and stalls); the hand-off version is opt-in only.

- Suggesting VP routes to the player ("fastest route", "buy 2 Soldiers to win", near-victory prompts). User, 2026-10-06: "I don't need the suggested way to get more VPs, I should be able to figure that out myself." Show the sources, their rules and their ownership; the player plans.

- Hard era cutovers like Civ 7's Ages (user, 2026-10-09: "I don't want hard era cutovers like civ 7... I do not want that"). No global age transition that resets or converts empires, swaps civs, or ends an era for everyone at once. Eras are per empire and continuous, Civ 6 style: you enter the next era through your own research. Era-staged VP and overreach costs key off each empire's own era or the world's progress, without a reset.

- Fantasy and alien factions (vault goblins, the insectoid Waxen Synod, elves): rejected 2026-10-10 by the designer and a target player. Use real historical peoples.

- A central contested landmark worth VP ("World Seat", Mecatol Rex clone): rejected 2026-10-10. It's too on the nose and favours central starts.


More rejected ideas with dates (Strategic supply needing its improvement, S15 supply priority, the S16 truce-shield change, a bigger Trench Infantry Fortify, the Recursive AI leak, Civ-style deposit guarantees, LLM or Laya imitation) are in the reference files below.

## References (open only when needed)

| File | Open it when |
| --- | --- |
| `references/principles.md` | You need the user's exact words behind a principle, or a principle's history (walls, air, triangle, resources, coalitions, freeze) |
| `references/decided-short-form.md` | Checking a decision from 2026-09-30 to 2026-11-14 (factions' first designs, diplomacy, production, AI direction, review-plan D1-D11, training mode, balanced starts, game length, simplification round 2) |
| `references/how-to-propose.md` | Checking a decision logged from 2026-10-05 to 2026-10-31 (AI as search, goals, overreach and decadence history, tech boosts, Alliances, Outrider and mounted units, wonders, truces, raze, city automation, era objectives) |
| `references/decided-2026-10-31-onward.md` | Checking a decision from 2026-10-31 to 2026-11-21 (world wonders, era objectives model and its replacement by World Ages, the Fable review batch, coalitions, public treasuries, half-population captures, the Machine Age, the calendar, capitals victory) |
