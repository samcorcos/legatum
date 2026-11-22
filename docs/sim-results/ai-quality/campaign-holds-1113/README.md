# Why Hard drops the campaign After the Riflemen (2026-11-13, `claude/campaign-holds`, rules 30)

**Question.** Hard wins `scenarios/lab/athens-after-the-riflemen.json` 21 of 50. In 24 of its 29 losses it never makes a melee attack on Tenochtitlan. Its plan reports `funded forecast unavailable` from about T142 and returns to Peace. Why?

**Answer.** The war is lost on T135, when Hard spends its Gold on the wrong thing. Plan routing and the funded forecast are only how the loss shows up. In 26 of 50 runs the plan routes a Campaign on T135. `plan_siege_buys` then buys Siege for Horseman #116 (80 Gold) and on T137 for Soldier #126. Cannonry completes on T135, so from T136 the Arquebusiers can refit to Field Cannons: 74 Gold each, with Siege built in and more firepower. The Siege purchases leave the treasury too short for those refits. Without Field Cannons nothing can shoot the walled city down. `siege_stance` never fires, Athens has `war_siege` 0 so no siege pass runs, and the late `campaign` pass parks the Arquebusiers. Tenochtitlan heals from 118 to 200 HP between T137 and T142 and gains a third defender. The plan's last forecast then fails and the plan goes back to Peace. Of those 26 runs Hard wins 2, and its 24 losses are exactly the 24 losses with no melee attack. In the other 24, no Campaign is routed before T139, the Gold goes to the refits on T136 and T137, the Field Cannons shoot the city to 0, and Hard wins 19.

Gene `refit_before_siege` (default 0) keeps the Gold for those refits before buying Siege. At 1 the card goes from 21/50 to 35/50 (median 801 to 1360). The 26 T135 runs go from 2 wins to 17, and the other 24 from 19 to 18.

## The trace (runs 0, 1, 3 lost; 7, 10 won)

The first 50 runs were rebuilt with every seat order traced (temporary local change, not committed) and the Tenochtitlan forecasts printed each turn.

- **`funded forecast unavailable` is the wrong label, and it is the same in wins and losses.** `funded_forecast` (`plan.rs::funded_forecast` → `campaign_forecast_with`) is None on every turn from T134 to T150 in all five runs. Gold is not the reason: income is +34 to +43 a turn, so `funding.rs::gold_runway` is None. The reason is that `opportunity.rs::forecast_siege_rounds` takes a walled 200-HP city held by two or three Riflemen with neither the in-reach force nor the six-turn march force, at 2, 4, 6 or 8 rounds. It succeeds only once the city is at 0 HP (run 7, T141). The trace's gate (`plan.rs`, the `gated_out` loop) reports only the first of the four sources `campaign_candidates` tries. The Campaign candidate actually lived on the third: `grind_forecast` on `campaign_force` (`plan.rs::campaign_candidates`, the fallbacks after `funded_forecast`). `wall_staging_forecast` never applies to Athens (`war_siege` 0). `campaigns.active` is never begun (`campaign.rs::begin_campaign` needs `campaign_can_take`, which uses the same failing siege estimate), so the committed-force grind never applies either.
- **What turns the grind off at T142 (run 0).** `grind_forecast` needs `siege.rs::siege_local_margin` ≥ `campaign_persist` (98 for Athens). The margin falls 185 (T135) → 228 → 198 → 165 → 133 → 129 → 124 → **80** (T142): our force within 3 tiles shrinks (Arquebusier #98 dies, Horseman #116 walks home to guard a Settler) while the Aztecs add a third defender (`military.rs::city_defence_weight` 129 → 142). The grind's own capture clock grows from 7 turns to its cap of 12, because the city heals and nothing shoots it. From T142 no source gives a forecast, so there is no Campaign candidate and the plan takes Peace ("Infeasible").
- **What differs before T142.** Dice decide whether the plan routes the Campaign on T135. In the T135 runs the opening volley left the city at 131 HP, so the grind forecast was 7 turns and the Campaign priced at +453. In the others it was at 151 HP and the grind had no forecast. Once routed, `siege_buy.rs::walled_war_target` sees a walled war target, and `plan_siege_buys` (before `plan_cities` and its `plan_modernization`) buys Siege, melee units first. All 50 runs:

| T135 Campaign routed | Runs | Siege bought by T137 | Arquebusier → Field Cannon refits by T137 | Wins | Losses with no melee attack |
| --- | --- | --- | --- | --- | --- |
| yes | 26 | Horseman #116 (T135), Soldier #126 (T137) in all 26 | 0 (six runs refit one on T139 or T140) | 2 | 24 of 24 |
| no | 24 | none by T137 (3 runs buy one or two from T139) | #98 on T136 and #151 on T137 in all 24 | 19 | 0 of 5 |

- **Bug or valuation error?** A valuation error, not a wrong number. Every figure the plan read was correct: the city really cannot be taken by Arquebusiers whose shots on Walls are halved. The error is in how the Gold was valued. `plan_siege_buys` buys Siege for whatever force units it can, cheapest-to-cover first, without comparing that with a refit that gives a unit Siege built in for the same money. A Siege purchase on a Horseman (Strength 26, which left the front a turn later) beat two refits that turn two shooters into wall-proof Field Cannons. On T140 it even bought Siege for Arquebusier #151 two turns before refitting it, so those 92 Gold were wasted.
- **What a competent human sees on T135.** "Cannonry lands next turn. My three Arquebusiers become Field Cannons for 74 Gold each, with Siege built in and better guns. That is what breaks these Walls. Don't spend 80 on Siege for a Horseman." The Fable seat did the same: it held Gold for the refit that takes the city (capture-tactics lesson 4).

## The gene: `refit_before_siege` (`siege_buy.rs::wall_proof_refits`, used by `plan_siege_buys`)

Turns of research lookahead; 0 is off. The force near the walled war target counts each unit that lacks Siege and whose refit gives a kind with Siege built in (`data::built_in_siege`). The refit must be orderable now (`UnitView::upgrade_to`, priced by `upgrade_breakdown`), or be unlocked by the technology we finish within the gene's turns (`ResearchView::current` and `turns_left`, priced by `rules::units::upgrade_cost_for`, halved after Military Engineering). That refit Gold, cheapest first, is kept in hand as far as Gold plus that many turns of income reaches. Siege is then bought only from Gold beyond it, never for a unit about to refit, and nothing is set aside as `siege_savings` while a refit waits, because those savings raise `gold_reserve` and would block the refit itself. Everything it reads is in the seat's own view. `plan_modernization` still chooses which refit to buy.

## Results, 50 runs per arm (runs 0-49, the runner's salts; `--ai-params params/refit-before-siege.json --params-seat-only`)

| Card | Baseline wins, median | `refit_before_siege` 1 wins, median | Runs changed |
| --- | --- | --- | --- |
| After the Riflemen (`athens-after-the-riflemen`) | 21, 801 | **35, 1360** | 17 gained, 3 lost |
| Stop the Snowball (`athens-stop-the-snowball`) | 4, 110 | 4, 110 | 0 |
| The Courier King (`runaway-conqueror`) | 39, 200 | 39, 200 | 0 |
| Athens against two Pikemen, T112 (`war-athens-two-pikemen-t112`) | 0, 630 | 0, 630 | 0 |
| Walled Nicomedia (`sparta-walled-nicomedia`) | 4, 970 | 4, 970 | 0 |

The baseline reproduces the card's recorded 21/50, median 801. With 50 runs the standard error is about 3.5 wins, so +14 is about four standard errors. The 3 lost runs are 2 and 23 (the two T135 runs that won without the gene) and 48. No regression card changed a single run, which means they never reach the case the gene changes. They show it is harmless there, not that it is good.

## Recommendation

Leave the gene off for now (as asked). It is the first After the Riflemen gene well beyond noise, and the mechanism is clear. Before turning it on by default:

1. Run the full-game seat gate (rivals-only and seat-only), since no Lab card in the regression set exercises it.
2. Run the other walled-city cards that need a Siege decision (Take Gergovia, Ride to Dyrrachium, Before the Walls, Two Cities) to find one where the gene actually fires.

## Not fixed (follow-ups)

- The plan's gate label: when `funded_forecast` is None the trace says `funded forecast unavailable` even when the grind or wall-staging fallback is what decides. It misled the earlier read of this card.
- Athens's `campaign` pass parks shooters (`skip | campaign`) while a routed Campaign has no active campaign and `war_siege` is 0. Even before the refits, the Arquebusiers could have fired from range 2 on several of those turns.
- The 15 losses left with the gene on still mostly end in "Peace (Infeasible)" between T139 and T145. That is the next thing to look at on this card.
