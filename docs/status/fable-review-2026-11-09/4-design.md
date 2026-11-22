# Review 4: game design

Scope: the rules on master `a9eb46864`, read as a 4X designer. Read-only; no sims run. `GR:n` means `.claude/skills/legatum-rules/references/game-rules.md` line n. "Inferred" marks my own reasoning. Wall-clock estimates are my guesses, not timed.

## The short version

The AI is bad at war, and that is the main reason he wins at T107 to T126. But the rules also make conquest the cheapest, fastest and safest source of points, and the two rubber bands the pillars name are not in the rules. A competent AI will narrow his margin; it will not change which strategy is best. The AI being trained now is learning the value of conquest under these rules, so the conquest-pricing changes below are worth deciding at the 11-13 to 11-18 checkpoint, before the next round of evaluator games, not after.

## 1. Where the rules fall short of the pillars

**Pillar 11 (competitive to the end).**
- Decadence is off by default (`crates/core/src/experiments.rs:318`, GR:1051). Coalitions are "a derived warning, not a mechanic": no order, no price change (GR:971-977). A call to arms can be declined at no cost and the Alliance stays (GR:892-895). The only live rubber band is tech diffusion, 10% per empire that knows the tech, capped at 50% (`data.rs:181-183`, `economy.rs:969`). It helps a science laggard and does nothing against a conqueror.
- The hold is 10 turn ends (`data.rs:212`). In the 8.9.29 Aztecs game three AI strike operations expired with one attack between them during it (`docs/playtests/human-reasoning.md:444-450`). Ten turns is shorter than the time an army takes to stage and cross the map (inferred from 2-move infantry and 6 turns of march in his own sieges).
- Measured before decadence existed: the T100 leader won 65 to 73% (`docs/learnings.md:2772`). With decadence it fell to 43 to 51% (`learnings.md:3374,3404`). It is off again, so expect the older number.

**Pillar 10 (no dominant strategy).**
- Conquest pays four times for one act. Capturing a rival's original capital gives +1 permanent VP (GR:1046, GR:832, `scoring.rs:202-204`); the city's yields raise Largest Economy; captured held wonders and goal buildings bring their 2 VP (GR:303); and captures count for the era Fight objective (GR:1031, 1054-1066). His own words: "they had a city with a wonder that would get me points faster" (`human-reasoning.md:416-420`).
- The price of a capture is 1 population and 6 turns of no output (`data.rs:80`, GR:815-818). There is no occupation, loyalty, war weariness or unrest (GR:1384-1423, §9).
- The peaceful ceiling is 6 era points + 2 goal + 2 wonder = 10 (GR:1041-1043), which needs era 7. The median winner enters the Future era at T204 (GR:186). Conquest wins have ended at T107 to T126. So the non-war paths cannot be in the race at all when a war path works (inferred from those two numbers).
- "Diplomatic" is named in pillar 10, but the classifier hook is inactive and no rule supports it (`docs/strategy-diversity.md:32,161`).

**Pillar 3 (turns stay the same size).** Delegation exists for cities (GR:340-366), ranged garrisons, exploring, sleep/heal and goto (GR:89-116). Nothing delegates a war. The group-move order was rejected and Generals wait on the AI (`legatum-design/SKILL.md:182`). Today this is hidden because he wins with 5 to 8 units (24 to 29 active minutes, 405 to 460 orders a game, `human-reasoning.md:258-269`). When the AI defends properly, armies triple and so do war orders (inferred).

**Pillar 8 (asymmetry with drawbacks).** Per-seat spread on 8.9.29 is Mongols 16.5% to Switzerland 9.0%, ratio 1.83 against the 1.5 target (`learnings.md:4452`). Three drawbacks do not bite a good player: India's corruption ("barely even noticed"), Germany's −2 Food, and Byzantium's levy cap (`legatum-factions/SKILL.md`, 11-05 and 11-07 entries).

## 2. The victory system

- Sources (`scoring.rs:177-225`): three live crowns at 1 VP; own wonder 2 held; era objectives 1 each, permanent; capitals 1 each, permanent; faction goal 2; captured wonders 2 held.
- Two of three crowns follow size: Largest Economy is city Gold + Production (`scoring.rs:136-142`), Strongest Army is unit value (`scoring.rs:144-154`). A conqueror takes both by default.
- A plausible conquest line to 10 by the Medieval era: 3 capitals + 2 crowns + 2 or 3 era points (Fight objectives fall out of the same wars) + 2 goal. Every one of those except the crowns is permanent, so nothing a coalition does afterwards takes them back. Only the crowns and held buildings can break a hold (GR:1082).
- So yes: a win can be reached before a contest forms, and the rubber bands have no teeth in the rules. They are AI behaviour, which is exactly the part that has not worked in any of his last five wins.
- Era objectives are met by accident 20 to 55% of the time (figure from the rules reference §6, read earlier). The objective deck (`objectives=deck`, GR:1018-1039) already fixes that and is built, opt-in.

## 3. Dominant choices, dead options, non-decisions

- **Star Fort** (320 Production, Military Engineering, GR:207, 289). Industrial units ignore every wall tier (GR:210). It is obsolete one era after it unlocks. Castle (240 for +5, GR:288) is weak for the same reason. Fold the line to two tiers or make tier 3 resist built-in Siege until Modern.
- **Pyramids** cost 240 for 2 held VP; the Great Library costs 600 with no yield (GR:293-294). Each empire completes one wonder, so the choice is which era, and the Ancient one is far the cheapest. Hagia Sophia (400) is cheaper than the era before it (GR:295). Prices should rise by era.
- **Promotions** are automatic, +10% a level, no choice (`data.rs:82-86`). Outside Sparta a player would not notice them gone.
- **Research order.** No prerequisites inside an era and all five techs are required to advance (GR:179). The only decision is order within five. His complaint "research without choices" (`human-reasoning.md:127`) still applies structurally.
- **Science after the last tech** converts at half rate and scores nothing (GR:183); Most Advanced is 1 VP. Science is a support stat, not a path.
- **Calls to arms**: Decline is free (GR:895), so Join is rarely right for a self-interested ally. This is why alliances do not open fronts.
- **Alliance upkeep** of 1 Gold per city per turn (GR:399) taxes the coalition, not the leader.
- **Supply users that fail the story test**: the University uses Iron and the Bank uses Niter (GR:499-500). They exist to make resources bind, but he prefers mechanics that follow the story.
- **Roadworks** +1 Gold for 140 (GR:283): bought for roads only; fine, but the Gold line is noise.
- **Archers** at 132 against Spearmen at 80 (GR:240, 247): the coach found Archers did nothing against a Hoplite rush (`human-reasoning.md:182`). His reflex ("walls and archers") is taught wrong by the price list.

## 4. What good 4X games have that this lacks, and what to cut

Worth the cost:
- **A cost of occupation** (Civ 6 occupied cities, EU4 overextension). The game has the hook already: capture resistance.
- **Commitment in alliances** (already agreed 10-17 as "real coalitions items 2 to 4", not started: `agreed-vs-built.md` G37).
- **A way to turn a lead back** (TI4: points can be denied late). Permanent capital points remove this.
- **A real tech choice per era.**

Not worth it now: trade routes, religion, espionage, great people, navy. Each is a new system; he prefers bending existing ones.

Could be cut: Star Fort, promotion levels for non-Sparta factions (keep XP only where a faction rule reads it), the Roadworks Gold line, the science conversion's Food and Production options (Gold alone does the job).

## 5. Mid-to-late-game tedium

What grows with empire size: one order per unit per turn; upgrades unit by unit (he batch-upgraded by hand at T72 and T94-T102, `human-reasoning.md:448`); Siege bought per unit (GR:656); aircraft each need an order every turn (GR:272); production choices (40% were repeats before automation, `legatum-design/SKILL.md:372`).

Cheapest fixes, none of them a rule change, so none touches the freeze:
1. **Upgrade all** and **buy Siege for all here**: one button that issues the existing orders for every eligible unit in view, with the total price.
2. **Standing assault**: an attack order on a city repeats each turn until the city falls, the unit drops below a chosen HP, or the forecast turns bad. Inferred from GR:116 that only goto persists today; verify before building.
3. **Bomber standing strike** on a chosen city, the same idea.
4. Generals stay the real answer once the AI can run a siege.

## 6. Factions

Strong identity with a real drawback: Celts (cannot keep cities), Mongols (no terrain defence, no early Walls), Sparta (+50% on everything civil), Aztecs (kills become population, no horses), Athens (Greece Captive). Byzantium's decided kit (capital falls, empire falls) will join them when merged.

Number tweaks or drawbacks that do not bite: Germany (Foundry +6, double supply "rarely bound", −2 Food "cost him nothing"), India (corruption dodged by keeping the economy in the core), Switzerland (strengths are three Gold numbers; its goal makes it worse: the ablation found the hoard goal lowers Swiss wins because the Gold sits unspent, GR:1305-1308).

**Switzerland (measured weakest, 9.0%).** Idea: *Swiss Gold has no purchase limit: Switzerland may buy any number of items in a city each turn.* Everyone else is held to one per city per turn (GR:447). Story: Swiss money raised pike armies overnight. It turns the hoard from a dead number into a visible threat, makes "hold 5,000 for 2 VP or spend it now" a live decision, and triple upkeep is already the price. One check in `BuyItem` validation.

**Germany (weakest identity; inferred second, I did not find the per-seat table).** Idea: *a city with a Ruhr Foundry stops growing.* Population is territory (GR:385), so this bites where −2 Food never did. It creates a timing decision in every city (grow first, then industrialise) and pushes Germany to get land by settling and conquest, which its Production pays for. Story: the Ruhr towns were factory towns, not farmland.

## 7. Top 8 proposals, ranked by fun per unit of cost and risk

"Checkpoint" means: build as an off-by-default rule experiment now (the freeze allows it; defaults do not change), sim it, and flip the default with one rules version at the 11-13 to 11-18 checkpoint, before the evaluator games are regenerated.

**1. Occupation: a captured city yields half until its founder makes peace with you or is eliminated.** *Checkpoint; the one I would break the freeze for.*
- Problem: conquest costs 6 turns and 1 population, then compounds.
- Fit: bends capture resistance; Civ 6 players know occupied cities (pillar 2); one sentence; "conquest is the fastest economy" stays true but now has a digestion step, and peace starts the 10-turn truce, so conquest chains get natural pauses in which rivals can act (pillar 11). It gives the victim a real lever: refuse peace.
- Risk: rewards finishing an empire off. Elimination is already allowed and final, so this sharpens an existing incentive. The AI must learn to price peace.
- Test: replay the three 8.9.29 Aztec scenario cards and a 512-game paired census with the flag; read T100-leader win rate, median game length, zero-capture winners. About 1 to 2 hours local on 16 cores.

**2. Capital points are held, not permanent: 1 VP for each rival original capital you hold.** *Checkpoint.*
- Problem: permanent conquest points cannot be answered.
- Fit: he liked "a point held in a city and taken with it"; holder becomes target; a coalition that retakes one capital breaks the hold. Celts keep theirs on the sack (their rule break).
- Risk: longer games; liberation ping-pong. Read no-winner games.
- Test: re-score his five recent winning tapes offline (script on saves, about 1 hour), then the same paired census.

**3. Declining a call to arms ends the Alliance; an Alliance at War with the team on a victory hold pays no upkeep.** *Checkpoint.*
- Problem: coalitions sign and do nothing.
- Fit: already agreed as commitment (G37); uses the public hold as the trigger, so no Alarm meter. The first half is one line.
- Risk: AIs drop alliances to dodge wars. Measure fronts opened per coalition.
- Test: existing coalition scenarios plus the census counter "allies at War with the hold target that attacked within 10 turns".

**4. War shortcuts: Upgrade all, Siege all, standing assault.** *Ship now; UI only.*
- Problem: war orders scale with army size (pillar 3).
- Risk: a standing assault that suicides; stop on a bad forecast.
- Test: count orders per turn on his tapes before and after; target a flat line after T100. Half a day of agent time.

**5. Swiss unlimited purchases per city per turn.** *Checkpoint (faction rule; he asked for a freeze on factions).*
- Test: `tools/sim/ablate.sh` Swiss census, plus "Hold Biel" scenario with a 2,000-Gold treasury.
- Risk: an unbreakable human Swiss defence. Walls still cannot be bought.

**6. Make the objective deck the default.** *Checkpoint; already built.*
- Problem: era points scored by accident; Fight objectives stack on conquest.
- Fit: "every VP a trade-off"; Spend cards give Gold factions a path.
- Test: `tools/sim/objective-deck-report.py` exists; one census.

**7. Germany: a Foundry city stops growing.** *Wait until he has played reworked Germany once more; then ablate.*
- Risk: the AI builds Foundries at population 4 everywhere. Needs an AI rule first.

**8. Enter the next era with four of the five techs.** *Wait; after the AI checkpoint.*
- Problem: no tech decision.
- Fit: bends the strict gate; skipped techs stay researchable and get cheaper by diffusion, so skipping is a tempo choice, not a self-inflicted penalty.
- Risk: every empire skips the same tech; then that tech is the problem. The sim will show it in one census.

Also cheap, data only, at the checkpoint: wonder prices rising by era; Star Fort folded into Castle.

Not proposed because rejected: turn limits, a visible alarm meter, research VP, VP for peace, capital protection, group move, forced peace, tolls, earlier Walls. Bringing decadence back is his stated plan once the AI competes; proposals 1 and 2 are the conquest-specific version of it and may make it unnecessary.
