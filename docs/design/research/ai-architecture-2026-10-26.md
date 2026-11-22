
**AI architecture proposal — 2026-10-26**

**Recommendation**

Build an operational planner that searches executable campaigns: which objective, which units, what funding, when to attack, and when to change course. Keep today’s tactical routines as its initial executor. Start with one front and a handful of alternatives.

Then train a small evaluator from the resulting experiments. Use the designer’s tapes and successful strategy-lab policies to supply candidate strategies and difficult positions throughout.

The evidence supports changing the AI’s decision structure. It does not establish that hand-written tactics have reached their ceiling, or that a neural policy would automatically solve the problem. Today, different layers repeatedly disagree about which army is available and what it can accomplish.

The north star remains **Hard matching the designer’s performance from comparable starting positions, then providing a credible challenge in his playtests**. Two weeks should establish a measurable gain and a viable architecture, not promise human parity.

**What the AI actually is today**

Audited checkout: `master`, `acc7ddfc0`.

- **Plain Hard is a scripted, value-scored policy.** It has persistent opponent observations, campaigns, tactical forecasts, production priorities, and a goal-driven planner. Ordered planning passes can reserve units or choose production before later scorers run.
- **The goal planner already makes strategic choices.** It compares Economy, Peace, Campaign, Contain, Settler snipes, and field strikes. Plans have commitment, review dates, Gold reserves, and recorded termination reasons. Campaigns already store committed unit IDs.
- **Hard + Search already performs real-engine rollouts.** It generally compares the best analytically nominated plans, rather than searching a tree of operational decisions. Defaults are three candidates, twenty turns, four samples; the app reduces this for larger lobbies. Experimental options include alternative fronts, postures, opponent policies, and four tactical order variants.
- **Learning already exists.** `winchance.rs` uses fitted, fixed-point weights. `learned_macro.rs` is a separate 32-feature integer model, disabled by default. Its earlier plan replacement underperformed the existing search. Opening selection helped the eight-player holdout, but did not establish a global improvement.
- **Strategy bundles already exist experimentally.** Hard v2 draws league-tuned openings, but remains opt-in because its validation found it easier for a strong player.

The architectural opportunity is therefore more specific than “add search” or “add machine learning”: **make useful alternatives representable, executable, and accurately evaluated**.

Sources: [scripted planner](../../../crates/ai/src/scripted.rs), [goal planner](../../../crates/ai/src/plan.rs), [search](../../../crates/ai/src/search/mod.rs), [distillation results](../../sim-results/distilled-macro/README.md), [Hard v2](../../../crates/ai/src/hard_v2.rs).

**What the recent games establish**

| Observed failure | Architectural implication |
|---|---|
| Sparta game: Japan lost 53 combat units to Sparta’s three despite substantial military production. Nearby troops were credited as support before they could arrive together. | Evaluate arrival schedules and combined orders, not just nearby strength. |
| Athens game: an old Mongol war reserved Switzerland’s modern gun while obsolete troops staged indefinitely. | Allocate units between fronts according to current threats and opportunity costs. |
| Swiss Bombers flattened a city, but ground forces failed to capture it. | Plan reconnaissance, bombing, ground capture, and follow-through together. |
| Japan air game: all four AIs reached Flight; none built Fighters. Aircraft builds were interrupted, and no AI rebased aircraft. | Air needs production commitments, coverage, financing, and basing decisions—not only target selection. |
| Germany attacked a city expecting to break Athens’s hold, but Athens retained ten VP and Germany lost its own live goal. | Calculate actual scoring consequences before assigning strategic value. |
| Human purchases exploited saved Gold; AI reserves lacked comparable preparation for an air offensive. | Saving needs a named purchase, deadline, and competing investment value. |

These are overlapping failures, not percentages of blame. The audits demonstrate mechanisms and bounded interventions, not counterfactual whole-game wins. For example, releasing the Swiss gun produced a capture in a six-turn fixture with recorded replies; that does not prove it defeats an adapting human.

The headroom study also needs careful interpretation: historical control won **27.7%**, hindsight best-of-eight RNG variants **56.1%**, and hindsight best-of-eight policy variants **66.0%**. The last figure is an oracle selecting after outcomes are known, not a deployable policy’s win rate. It establishes selection headroom, not a promised gain.

Sources: the reports `sparta-game-1025.md`, `athens-game-1025.md`, and `japan-air-war-1026.md` on `astra/analyze-sparta-game-1025`, `astra/analyze-athens-game-1025`, and `astra/analyze-japan-air-war-1026`; [headroom decomposition](../../sim-results/ai-quality/headroom-decomposition.md).

**Compute and live constraints**

The recent climb program reports approximately **7.6 core-seconds per complete four-player Small game**:

| Workload | Derived capacity |
|---|---:|
| Complete games, one core-hour | About 474 |
| Complete games, fully available 64-core box-hour | About 30,300 |
| 100,000 complete games | About 3.3 box-hours |
| One million complete games | About 33 box-hours |

These are arithmetic projections from the recorded workload, not a fresh benchmark. Eight-player games, search, data export, and long stalemates can cost substantially more. At half the box’s available capacity, elapsed time doubles.

The older **2,700 games in fifteen seconds** measurement used short milestone runs. It must not price complete self-play games. Similarly, combat-lab throughput excludes much of the full game.

All estimates below include experimentation and evaluation allowances. Tuesday’s first batch should measure current complete games, operational continuations, and data collection separately.

Every architecture must preserve:

- PlayerView-only decisions and observation-derived memory. Hidden hangars and armies remain uncertain.
- Deterministic inference, seeded sampling, stable ordering, and serialized planning memory.
- Fixed work limits rather than machine-speed-dependent search cutoffs.
- The existing rules, prices, resources, and simultaneous order model.
- No LLM calls during gameplay or per simulated game.

For live integration, propose an initial **whole-lobby planning target of p95 below one second**, checked on the designer’s actual client, with maximum latency reported separately. This is a proposed acceptance target. The existing search comments describe roughly second-scale search turns; native box speed cannot establish client performance.

Sources: [climb program](../../sim-results/ai-quality/climb-program/README.md), [optimization program](../../sim-results/ai-quality/optimization-program.md), [app search configuration](../../../crates/ai/src/search/mod.rs).

**Architecture A: search executable operations**

Represent an operation as a target, assigned ground and air units, staging locations, arrival times, production and purchase commitments, home-defense allocation, and deadline. Extend the existing campaign record rather than create a competing planner.

Search alternatives such as:

- Attack with the force ready now.
- Wait for a specific reinforcement or upgrade.
- Release a distant garrison and replace its defensive role.
- Save for an aircraft purchase, then attack.
- Bomb and capture together.
- Withdraw, defend a countdown, or pursue another reachable objective.

Each alternative must produce legal orders through the same executor used live. Tactical refusals return a reason—blocked route, lethal arrival, missing capture unit, insufficient supply—so the operation can change. Later passes cannot silently spend its Gold or reserve its gun without invalidating and reconsidering the operation.

Begin with a small portfolio of rollouts. Add a shallow beam or Monte Carlo tree over successive operation choices only after that works. Branch at meaningful events such as reinforcement arrival, new sightings, failed staging, or a threatened victory hold.

Use the real resolver and shared sampled worlds across alternatives. Opponents submit simultaneous orders from their own observations; search never sees their actual submitted orders. Include aggressive proxy opponents alongside Hard.

This follows the general action-abstraction approach demonstrated by [Puppet Search](https://skatgame.net/mburo/ps/aiide15-puppet.pdf), adapted to Legatum’s simultaneous, partially observed game.

**What it fixes.** It directly addresses piecemeal attacks, stale reservations, endless staging, and air-ground handoffs. Explicit purchase commitments support saving. Actual scoring and arrival deadlines make leader-stopping concrete. Peace remains an offer requiring acceptance; withdrawal must work even when peace is refused.

**Risks.** The executor may still reject every promising plan. Sampled opponents may defend too poorly. Short horizons may discourage costly wars or reward delaying defeat. A growing candidate set can exceed the live budget.

**Smallest useful increment.** One known city, one front, and three executable choices: commit together, gather/refit, or withdraw/reassign. Demonstrate better deadline capture and casualty outcomes on unseen positions, beyond what the existing local fixes achieve. Aircraft production and multi-front trees can follow.

**Measurement.** Capture or retain the objective by its deadline; committed units actually arriving; losses; idle staging turns; home losses after releasing guards; actual hold breaks. Require transfer to full games.

**Architecture B: learn the value of positions and operations**

Train a small model on **two to ten million sampled positions from roughly 100,000–250,000 diverse games**. Use a mixture of current Hard, proxy attackers, validated strategy bundles, historical champions, and exploratory candidates. Weak Hard self-play alone would reproduce its blind spots.

Inputs should describe what matters to the choice: usable force and arrival time, promotions, current Gold and planned purchases, supply bottlenecks, own aircraft and coverage, stale sightings, and public victory deadlines. The current macro model clips Gold at 500; reusing that representation would erase distinctions relevant to expensive purchases and large reserves.

Predict eventual victory and auxiliary outcomes such as capture by a deadline, losses, and hold survival. Train the principal value against actual outcomes, not solely the old evaluator’s predictions. Use matched alternative continuations to teach action ranking.

A small network—approximately 96 inputs and two hidden layers of 64 and 32 units—is a reasonable starting design. Quantize it into bounded integer inference for Rust and WASM. Its latency must be measured; small parameter count alone is not proof.

Use it to rank planner candidates and score search leaves. Keep exact known scoring consequences in the rules-based calculation. Learning should estimate uncertain futures, not relearn whether capturing a city removes a technology crown.

Combining learned strategic selection with tactical search has research precedent, including [Barriga, Stanescu, and Buro’s microRTS work](https://arxiv.org/abs/1709.03480); that is evidence for the approach, not a Legatum performance forecast.

**What it fixes.** Better trade-offs between saving and spending, veterans and replacement units, research and immediate force, own growth and rival denial. It can improve campaign selection and search’s horizon estimate.

**What it cannot fix alone.** A model cannot choose a Fighter excluded from consideration or move artillery already reserved by an earlier pass. Minimal candidate and execution interfaces remain necessary.

**Risks.** Correlated positions masquerading as millions of examples; learning weak-policy outcomes; outdated rules; exploiting evaluator errors; good prediction without better decisions.

**Smallest useful increment.** Train a linear baseline and one small network on 20,000 games, then compare their ranking of campaign-versus-refit-versus-economy choices. Hold tactics and candidate generation fixed. Scale only if this improves unseen continuations and full-game results.

**Measurement.** Calibration and alternative-choice regret, split by era and faction; then the same war, human-seat, and milestone gates as A. A lower prediction error is insufficient for promotion.

**Architecture C: learn strategy selection from tapes and successful bundles**

Use the designer’s dozens of games to learn **which coherent plan to pursue and when to abandon it**, rather than attempting to clone every unit order.

Extract decision episodes with the designer’s observation at that moment: preparing a rush, saving for aircraft, modernizing a force, exploiting a bombing window, or interrupting a victory hold. Preserve what he could see; later revelations are not training inputs.

Combine these episodes with current-rules strategy-lab winners. A bundle contains research, production, treasury targets, force requirements, deployment, and conditions for reconsideration. A small learned selector chooses a bundle from faction, map, neighbors, and observed threats.

Ask for preferences only where they add information: two plausible continuations and a short explanation of the difference. Collect corrections on situations the learned policy itself reaches, because imitation errors change subsequent situations—the problem addressed by [DAgger](https://arxiv.org/abs/1011.0686).

Keep several successful bundles and test counters against them. Human initialization followed by diverse league opponents is also a component of [AlphaStar’s training approach](https://deepmind.google/blog/alphastar-grandmaster-level-in-starcraft-ii-using-multi-agent-reinforcement-learning/), although its scale is not a useful budget comparison here.

**What it fixes.** It offers the fastest route to recognizable human strategy: banking toward a purchase, committing to a force composition, preparing an air transition, and changing plans before a countdown expires.

**Risks.** Copying behavior that exploits today’s weak opponents; overfitting a few maps; missing losing examples; reproducing intentions without execution. Existing league winners are candidates, not automatically validated strategies. Hard v2’s failed transfer is the warning.

**Smallest useful increment.** One faction, two contrasting bundles, and a simple observation-based selector. Compare it against each fixed bundle and the existing opening draw on unseen maps. Use existing written reasoning first; reserve roughly one to two hours of designer review for ambiguous examples.

**Measurement.** Human agreement is diagnostic. Success means the selected bundle beats fixed selection, achieves its intended milestones, survives counter-strategies, and improves the common full-game gates.

**Build and compute estimates**

These are planning estimates for one implementation lane with review, not measured Codex-versus-Claude speed claims. Codex owns implementation and experiments; Claude owns architecture review, failure interpretation, and design checks. One work-week means five focused working days.

| Architecture | First increment: Codex / Claude | Broader candidate: Codex / Claude | Box-hours: first / broader |
|---|---:|---:|---:|
| A: executable operations | 1–2 / 0.2–0.4 weeks | 3–5 / 0.5–1 weeks | 8–24 / 40–120 |
| B: learned evaluation | 1–2 / 0.2–0.4 weeks | 3–5 / 0.5–1 weeks | 4–12 / 20–80 |
| C: learned bundle selection | 1–2 / 0.2–0.4 weeks | 2–4 / 0.5–1 weeks | 2–8 / 10–40 |

Broader estimates include the first increment. They assume reuse of existing extraction, simulation, training, and benchmark infrastructure.

For B, 100,000–250,000 plain games cost approximately 3.3–8.2 box-hours at the recorded rate. Allowing two to five times that simulation cost for richer policies, plus training, export, and repeated gates, motivates the wider budget. A spends more on alternative continuations; C spends more on extraction and representation work than model fitting.

**How we decide whether any architecture works**

Freeze the code, rules, opponents, and dataset splits before comparisons. Reserve whole games and maps; adjacent turns from the same tape cannot serve as independent holdouts. The three diagnosed games are development cases.

Use four comparisons: current Hard; independently gated local fixes; the new controller without search or learning; and the complete candidate. This separates architectural gains from repairs already available on the analysis branches.

The evaluation has four layers:

1. **War suite.** Preserve the existing attack regression check, especially Golden Horde. Add held-out cases for coordinated arrivals, stale guards, bombing-to-capture, air defense/rebasing, funded purchases, and real countdown interruption. Use recorded replies for short mechanism checks and adaptive opponents for performance claims.
2. **Human-seat benchmark.** Candidate in the designer’s seat should win more; candidate only in rival seats should make that fixed seat win less. Report both separately. Explicitly reseed combat dice, preserve pairing, and add new maps beyond the historical eight.
3. **His milestones.** Track captures, eliminations, cities, VP, Gold available for intended purchases, and finish time. Sparta’s T40 sweep and Switzerland’s T100 treasury remain useful historical references, with rule-version qualifications. Do not improve city counts by sacrificing full-game wins.
4. **Fresh playtests.** Record whether the designer still wins comfortably, which failures remain exploitable, and whether the wars force meaningful responses. A few games reveal failure modes; they do not establish a precise win rate.

The October 23 archive had 13 wins in 17 finished games, including eight of ten four-player Small games. That is an older, mixed-version sample, not a contradiction of his October 26 report. Refresh the target from recent completed games.

For the two-week experiment, propose **a 15-point improvement in held-out operational success and a five-point human-seat win-rate improvement**, with paired intervals reported. These are go/no-go ambitions, not forecasts. Preserve the existing non-regression gates; a noisy apparent gain is not grounds to declare parity.

**Two weeks, starting Tuesday October 27**

| Dates | Work and reviewable result |
|---|---|
| **Oct 27–28** | Freeze baseline and rules. Restore view-plus-memory fixtures. Separate existing branch fixes into comparison arms. Measure current throughput and client latency. Specify the operation record and executor contract. |
| **Oct 29–30** | Implement one-front allocation and commit/gather/withdraw alternatives. Surface tactical blockers. Run bounded resolver comparisons. Deliver the first inspectable campaign trace by Friday. |
| **Nov 2** | Test unseen arrival and stale-guard positions against adaptive opponents. Compare against local fixes alone. If gain is absent, diagnose candidate or execution failure before increasing search. |
| **Nov 3–4** | Search operation alternatives with shared samples. Add one air-ground capture sequence using existing aircraft, plus correct victory-deadline scoring. Keep new aircraft production strategy outside the minimum scope. |
| **Nov 5** | Calibrate deterministic live work limits on the actual client. Verify save/resume and native/WASM agreement. Keep expensive search offline if live latency fails. |
| **Nov 6–8** | Freeze the candidate; run full war, human-seat, fresh-map, and milestone gates. Test counter-strategies. Prepare a short failure report and playtest build if gates pass. |
| **Nov 9** | Review designer playtests and measured outcomes. Retain only demonstrated improvements. Decide whether the next bottleneck calls for broader operational search, learned evaluation, or better strategy candidates. |

Deliverables are an executable operation interface, reproducible scenarios, an ablation report, latency measurements, and a candidate with an explicit acceptance result. The interface and test corpus remain useful even if the first search policy fails.

The recommendation is **A first, B next if evaluation becomes the limiting factor, with C supplying candidates and teaching examples throughout**.

**Source limitations**

This proposal uses the audited checkout, the three analysis branches, and the repository’s design/rules skills. The external Claude design artifact referenced by `CLAUDE.md` was inaccessible. Historical benchmark figures above are dated evidence, not newly executed measurements.

**Questions**

1. Should the first playtest emphasize coordinated early conquest or the late air-and-ground offensive? Recommendation: early coordination first, with one air-ground regression case in week two.
2. Is the proposed one-second p95 whole-lobby planning target appropriate on your laptop, or should plain Hard have a tighter limit?
3. Can we reserve several recent complete games as untouched evaluation tapes and use one to two hours of your time for strategy comparisons?
4. After two weeks, should we continue only on demonstrated benchmark gains, or also fund a promising architecture whose gains are still confined to unseen war scenarios? Recommendation: require transfer before expanding compute substantially.
