# Learning-AI pilot: can simulation discover rush defence by itself? (2026-10-07)

**Verdict: yes for defence, no for scouting.** Evolution strategies, with no hand-written strategy,
found a rush defence that beats scripted Hard by a wide margin on held-out seeds. Against the
Mongols conqueror, Hard keeps its capital to turn 60 in 54% of games and the learned policy in
83-87%; at turn 100 it's 33% against 54-62%. The behaviour it found is "prepare defence when the
enemy comes": ranged units while the army is small, Walls and units instead of the default build
once enemies are near, and units called home at war. It did **not** discover sentries: posting
units with sight on the enemy before an attack didn't rise reliably in any run, and when the
learner could only move units it learned to pull them back, not post them forward.

![chart](chart.png)

## Setup

- **Game.** 1v1, Small map, Standard speed, Hard, barbarians on (the `duel_1v1_vs_proxy7` lobby).
  The attacker is Proxy v7 (`HumanPlaybookV7Brain`, the user's conqueror playbook) playing
  **Mongols**, the matchup Hard defends worst (4-6% of full games). The defender is each of the
  other seven factions in turn (seed mod 7), seat alternating. Games stop after turn 60.
- **Learner.** A temporary simulator example (removed after evaluation) let a small policy override
  two kinds of decision each turn:
  - each military unit or scout: keep Hard's order, fortify, go home to its nearest city, watch the
    nearest enemy city from the closest tile it may enter (hills preferred), post at the midpoint
    of the approach route, or picket 3 hexes out from its city toward the enemy;
  - each city: keep Hard's build, or switch to the best melee unit, the best ranged unit, or Walls.
- **Policy.** Linear, argmax, integer fixed point (weights and features in 1/1024), deterministic.
  15 observable features from the defender's own view: bias, turn, at war, enemy military in sight
  (now, near our cities, closest distance, most in the last 5 turns), enemy city in sight, own
  army, plus per unit (scout, in a city, nearest city's garrison, distance home, HP, ranged) or per
  city (capital, garrison, enemies within 4, Walls, damage). 150 parameters.
  The starting point prefers "keep Hard's decision" everywhere, so generation 0 *is* Hard
  (checked: identical results on 32 seeds).
- **Fitness** per game: 100 if the first city (the capital) is still ours, +25 per city owned,
  +5 per enemy military unit killed, -5 per own military unit lost.
- **ES.** OpenAI-ES: 24 antithetic pairs (population 48), sigma 0.5, Adam lr 0.1, centred ranks,
  weight decay 0.005. Every candidate, the centre and plain Hard play the same 128 fresh tune seeds
  each generation (common random numbers), so the learning curve is measured on seeds the centre
  hasn't trained on yet. 150 generations.
- **Runs.** Three independent runs with both heads (A1-A3, tune seeds 8100001+), and three with the
  city head frozen at Hard's choices (U1-U3, seeds 8200001+), to see whether unit moves alone find
  sentries. Each run then played its final centre against plain Hard on 2,048 held-out seeds
  (9100001+); no selection on those seeds.
- **Code.** Measured on master 2451fadd (Hard has changed since; rerun before porting anything).
- **Cost.** Six c8g.16xlarge spot servers for about 13 minutes each (4.7 s per generation): 1.25
  server-hours, about **$0.88** of the $5 budget. About 5.8 million 60-turn games in all (about 1,360 a second per server).

## Results on held-out seeds (2,048 paired games each)

| | capital held T60 | paired diff [95% CI] | cities | enemy killed | own lost | better / worse games |
| --- | --- | --- | --- | --- | --- | --- |
| plain Hard | 54.4% | | 1.34 | 3.6 | 3.5 | |
| A1 | 86.9% | +32.5 [+30.2, +34.8] | 1.61 | 7.0 | 3.2 | 1381 / 571 |
| A2 | 85.4% | +31.0 [+28.7, +33.3] | 1.41 | 6.7 | 2.7 | 1325 / 605 |
| A3 | 83.4% | +29.0 [+26.8, +31.3] | 1.55 | 6.6 | 3.4 | 1284 / 641 |
| U1 (units only) | 64.0% | +5.0 [+3.4, +6.7] | 1.55 | 4.1 | 3.3 | 704 / 508 |
| U2 (units only) | 73.9% | +14.9 [+12.8, +17.0] | 1.67 | 4.4 | 3.6 | 1039 / 795 |
| U3 (units only) | 64.2% | +5.2 [+3.7, +6.7] | 1.53 | 4.2 | 3.4 | 583 / 450 |

- **It holds past the training horizon.** At turn 100 (512 held-out seeds, `ablations/*-t100.json`)
  the capital is held 33.0% for Hard against 61.7% (A1), 53.7% (A2), 55.5% (A3): +21 to +29 points.
- **Every faction gains.** A1 per defender: Japan 55 -> 89%, Germany 47 -> 89%, Sparta 63 -> 95%,
  Byzantium 54 -> 81%, Athens 45 -> 86%, Switzerland 53 -> 89%, Celts 64 -> 81%.
- **Expansion didn't suffer.** Learned policies end with more cities than Hard (1.41-1.67 vs 1.34).
- **Ablation** (512 held-out seeds, T60, `ablations/*-t60.json`): the A policies' city head alone
  gives +21 to +27 points; their unit head alone +3 (n.s.), +13 and +20. The heads co-adapted, so
  the parts don't add up to the whole.

## Behaviour (held-out, before the first attack)

| | sentry turns | turns seeing their city | warning lead | army when attacked |
| --- | --- | --- | --- | --- |
| plain Hard | 3.8 | 2.4 | 4.4 | 3.0 |
| A1 / A2 / A3 | 3.6 / 5.0 / 4.6 | 1.8 / 3.2 / 2.9 | 6.0 / 5.8 / 5.3 | 3.8 / 3.9 / 3.8 |
| U1 / U2 / U3 | 3.4 / 2.6 / 3.2 | 1.4 / 1.1 / 1.2 | 4.6 / 5.3 / 4.6 | 3.2 / 3.3 / 3.3 |

Definitions: *sentry turn* = a turn with an own military unit or scout within 4 of a known enemy
city, outside our land. *Warning lead* = first attack turn minus the first turn, within the 15
before it, that 2+ enemy military units were in sight. *Army when attacked* = own non-scout
military units on that turn.

What the policies actually do (share of decisions, A runs): cities keep Hard's build about 31% of
the time, switch to the best ranged unit about 52% and to Walls about 17%; units keep Hard's
order 83-93%, and otherwise go home (A1, A3) or fortify (A2). The "watch the enemy city",
"midway" and "picket" primitives are used under 2% of the time. The largest weights say the same
in all three runs:
- city: ranged unit while our own army is small (own-army weight about -3.6); move away from the
  default build as enemies get close (closeness weight -3.5 to -4.4); Walls when enemies have been
  seen or the city is the capital;
- unit: go home when enemies were seen recently and the unit is far from home (A1) or at war (A3).

So the "prepare defences when the enemy masses" half of the target emerged: the army is 25% larger
when the attack comes and the attacker loses twice as many units. The warning lead grew by 1-1.7
turns, but sight didn't (turns seeing their city -0.6 to +0.9, sentry turns -0.2 to +1.3), so the
longer lead comes from attacks arriving later against a better-defended city, not from sentries.
With only unit moves to learn (U runs), ES learned to *withdraw*: sentry turns fell and "picket
3 hexes out" or "home" replaced posting forward.

## Why no sentries, and what scaling up would take

- **The cheap lever wins.** Rangers and Walls fix most of the hole. A sentry pays only if the
  policy then acts on what it sees, and that two-step plan (post, see, then prepare) is much
  harder to find by random perturbation than "build Rangers when the army is small".
- **City sight already gives most of the warning.** Proxy v7 stages 2-3 hexes from the city and
  strikes the turn it declares; a city sees 3 hexes. The extra few turns of warning a sentry buys
  matter only if production can turn them into defenders, which a 60-turn horizon barely
  rewards.
- **The policy is memoryless and linear.** One "enemies seen in the last 5 turns" feature is its
  only memory. Recognising "they are massing" from glimpses needs a richer memory (counts over
  time, last-seen positions) or a recurrent policy.
- **Next steps if we want sentries specifically:** opponents that stage out of city sight (a
  Proxy variant that masses 5-6 hexes out); a reward for the information itself only as a
  diagnostic (it would be hand-shaping); a memory of sightings; and per-unit targets chosen by the
  policy rather than three fixed tiles.
- **Before any of this ships** (it is an example only; no rule or shipped AI changed): check the
  policy against non-rushing opponents (plain Hard, other proxies, 8 players) so the Ranger bias
  doesn't cost economy elsewhere, measure full games against Proxy v7 (win rate, not just the
  capital at turn 60/100), and then port the finding as a scripted Hard rule or ship the
  fixed-point policy as a gene. The lead decides.
- **Compute is not the limit.** 150 generations took 12 minutes on one 64-core server; a policy
  10x larger, or opponents drawn from a league of proxies, fits in a few dollars.

The `runs/` and `ablations/` files preserve the historical learning curves and evaluations. The
trained weight files and commands for the removed experiment runner are not retained.
