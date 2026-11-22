# Why Hard keeps missing strategies that are obvious to you: overnight report (2026-11-06)

Sources: two independent reviews of every coach review, decision diff, your recorded reasoning and the AI code (Fable: `fable.md`; Astra: `.reviews/astra-strategy-gap-review.md`), and overnight simulations (`docs/sim-results/strategy-gap-1105/README.md`, `docs/sim-results/faction-ablation/README.md`).

## The short answer

Hard doesn't miss your strategies because it values them wrongly by a little. It misses them because **they never reach a comparison**. Your plans are trades: Walls instead of troops, cash held for the decisive moment, a lost border town for time to mobilise, a slower first VP that sets up the next two. Hard decides these in separate steps that run in a fixed order:
- a troop quota per city is bought before anything is valued (`production.rs::uncapped_army`, the garrison floor);
- earlier passes reserve units and Gold before later ones see them;
- the evaluator has no terms for defensive depth, time bought or tempo.

So the trade you make is not a choice Hard has.

Our tuning loop then hides this:
- **Gene tuning only adjusts the menu Hard already has.** It can't add a new choice, so it improves play inside the wrong constraints.
- **Every census is AI against AI.** You are the only player who exploits these gaps, so nothing in the test pool punishes them.
- **The human-seat benchmark isn't a human opponent.** Hard takes over your seat and immediately plays like Hard.
- **Porting your playbook per faction left the shared quota in place.** That quota was the real cause for every faction.

Every gain this week (Swiss, Celts, Aztecs, India) came from deleting a quota or fixing a bug, never from copying your moves.

## What the overnight simulations found (4,096 AI games per arm)

- **Walls count as a city's garrison (your India and Swiss play, applied to everyone): every faction gains on its own,** by +1.7 to +5.0 points:

  | Faction | Gain (points) |
  |---|---:|
  | Athens | +5.0 |
  | Mongols | +4.5 |
  | India | +4.4 |
  | Aztecs | +4.2 |
  | Byzantium | +4.0 |
  | Sparta | +3.1 |
  | Germany | +3.0 |
  | Celts | +1.7 |

  Your hunch that aggressive factions prefer troops didn't show: even Sparta gains. Fewer cities fall (1.83 → 1.61 per empire) and fewer empires are eliminated (11.9% → 9.7%).
- **A lean war army** (one unit above the garrison instead of five) helps Byzantium (+3.2), the Celts (+2.5) and the Mongols (+1.6).
- **Both together make Hard harder to beat** in your seat's benchmark, by 4.2 points. **Both are now on master** as Hard defaults. Easy and Normal are unchanged, since they weren't measured.
- **Not every prediction held:**
  - A smaller Hard bonus army, lower defence margins and looser settling rules did nothing.
  - One gene Fable predicted would matter is never read outside the Swiss path.
  - The "peace-cycler" exploit (make Peace, redeclare after the truce) doesn't work: Hard keeps its army through truces, and the cycler lost badly.

## What was wrong in how we measure (fixes queued)

- **Benchmark arms meant to switch a faction playbook off could leave it on.** A shared 0 is overwritten by the faction's built-in value. Codex task `gate-off-switch-fix` makes arms explicit per faction and re-checks past ledger rows.
- **A test claimed to cover Hard while running on Normal.** Same task.

## Predicted blind spots still hidden, and what is being tested

| Blind spot | Where it hides | Status |
|---|---|---|
| Captures are compared by population, so a 6-population town beats a 1-population original capital or Great Work city | `army_commit.rs::capture_hold_now` | Codex `capture-trade-vp`, census + benchmark |
| A strike that needs two purchases (Bomber plus a capturer) is never considered, because buys are judged one unit at a time | `decisive_buy.rs::decisive_candidates` | Codex `complementary-buys` |
| Byzantium hires mercenaries only after war starts, two a turn, ignoring arrival time; Companies are measured as costing Byzantium 8.8 points | `company.rs::hire_purpose`, `best_offer` | Codex `byzantium-companies-misplay` |
| The Swiss chase the 1,000 Gold hoard at a loss (the goal is measured backwards, +1.3 when switched off) | `decisive_buy.rs::decisive_floor`, gold reserve | Codex `swiss-hoard-misplay` |
| A Swiss limited war still sizes its army as cities + extra | `production.rs::uncapped_army` | covered in part by the new defaults; re-measure |
| Hard takes the cheapest first VP even when a slower one sets up the next two | `crowns.rs::chosen_objective` | next batch |
| Athens broadcasts Share the Learning without weighing the techs it hands to the army that can break its hold | `goal_actions.rs::broadcast_now` | next batch, with the Athens redesign |

## The process change

1. **Trace which options never reach a comparison** (Codex `decision-trace`, queued). For every strategic choice, record whether it was:
   - gated out before valuation,
   - considered and rejected,
   - selected but taken away by a later pass,
   - or executed.

   Today all four end up as "the gene didn't improve wins". Run on 256 games and on your India game, it should list the blind spots directly, before you find them by playing.
2. **An exploiter league.** Scripted opponents that play each strategy you use: thin-army builder, pre-Walls rusher, timed tech attacker, Settler stalker, hold denier. Hard must beat them, not just other Hard AIs. The peace-cycler was the first one.
3. **Test sequences, not single turns.** Swapping one of your decisions without the moves around it measures a broken hybrid. Compare 5- and 10-turn bundles from your recorded positions instead.
4. **Use LLM-proposed plans to widen the menu,** written as executable conditional plans and scored by simulation. Use them to add choices, not to micromanage units.

## Also overnight

- **The rule-off test for all nine factions:** 13 of 51 rules do nothing, and two are backwards (Byzantine mercenaries, the Swiss hoard). Sparta's +50% civilian cost is the heaviest rule in the game: +17.5 points when removed. See `docs/sim-results/faction-ablation/README.md`.
- **Germany rework, measured by Codex:** Germany 6.4% → 17.1%. It's being merged with master and re-measured, with a −3 Food arm in case it overshoots, then a UI review before landing.
- **Landed on master:**
  - the aircraft card rebuilt on the unit card;
  - the Swiss sheet copy;
  - the original-capital star seal;
  - Walls-as-garrison and the lean war army;
  - the faction rule-off switches.
