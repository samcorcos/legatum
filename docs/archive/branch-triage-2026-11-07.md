> Archived 2026-11-22: kept for history; it no longer describes the project.

# Stale `astra/*` branch triage, 2026-11-07

Read-only triage. Nothing deleted, no tags created. The lead decides.

**Scope:** `git branch --list 'astra/*' --no-merged master` lists 195. 20 are on the in-flight or being-landed list and are skipped (listed at the bottom), leaving **175**: **(a) 57, (b) 102, (c) 16**.

## How each branch was judged

- `git cherry master BRANCH` (patch equivalence), `git merge-tree` (does merging change master at all), reverse-applying each of the branch's own commits to master, and a file-by-file check of the code and assets the branch added.
- Branch name and subject against `plans/ai-war-progress.txt`, `docs/learnings.md`, `docs/sim-results/ai-quality/ideas.md`, the master merge log (`git log master --merges`), and the Codex task report `.reviews/astra-NAME.md`.
- Only 12 of the 175 have no patch left to cherry-pick. Almost nothing reverse-applies, because master moved under every branch (RULES_VERSION 5 to 14, faction renames). So class (a) rests on 'the same feature or files are on master through a land/ or claude/ branch', which is stated per row. Class (b) rows with a 'report says' note rest on the task report. Unlisted risk: a branch in (b) might hold a small idea the reports do not mention.
- Class (c) is a short list to look at, not a recommendation to land: each needs a rebase onto RULES_VERSION 14 and, for AI genes, the human-seat gate.

**Surprise:** 56 of the 175 have a 'Merge ...' or 'WIP left in lane' commit as their tip, and 31 have no task report in `.reviews/`. Nothing in the repo records which `astra/` tips the lead already treated as dead, so the (b) bucket is a judgement from age, name and report text.

## (a) Already on master by another route (57)

| Branch | Last commit | Ahead | Evidence |
|---|---|---:|---|
| `astra/diplomacy-slice-1` | 2026-09-26 | 4 | Diplomacy is on master (`app/src/diplomacy.ts`, `relations.ts` and the alliance e2e specs exist; 11 to 22 of the branch's added code files are on master). Tip is a pre-merge snapshot. |
| `astra/diplomacy-competition-experiment` | 2026-09-27 | 6 | Diplomacy is on master (`app/src/diplomacy.ts`, `relations.ts` and the alliance e2e specs exist; 11 to 22 of the branch's added code files are on master). Tip is a pre-merge snapshot. |
| `astra/diplomacy-slice-2` | 2026-09-27 | 7 | Diplomacy is on master (`app/src/diplomacy.ts`, `relations.ts` and the alliance e2e specs exist; 11 to 22 of the branch's added code files are on master). Tip is a pre-merge snapshot. |
| `astra/merge-diplomacy` | 2026-09-27 | 7 | Diplomacy is on master (`app/src/diplomacy.ts`, `relations.ts` and the alliance e2e specs exist; 11 to 22 of the branch's added code files are on master). Tip is a pre-merge snapshot. |
| `astra/adopt-balance-v9` | 2026-09-28 | 1 | V9 faction balance adopted 2026-09-28 (`docs/sim-results/2026-09-28-adopt-faction-balance-v9.md`); later rules rewrote the same files. |
| `astra/diplomacy-alliances-coalitions` | 2026-09-28 | 14 | Diplomacy is on master (`app/src/diplomacy.ts`, `relations.ts` and the alliance e2e specs exist; 11 to 22 of the branch's added code files are on master). Tip is a pre-merge snapshot. |
| `astra/merge-diplomacy-2` | 2026-09-28 | 28 | Diplomacy is on master (`app/src/diplomacy.ts`, `relations.ts` and the alliance e2e specs exist; 11 to 22 of the branch's added code files are on master). Tip is a pre-merge snapshot. |
| `astra/purchase-tape-decoder` | 2026-09-28 | 1 | Same commit subject is already on master. |
| `astra/behaviour-primitives` | 2026-09-30 | 2 | Every code file the branch added is on master (`crates/ai/src/openings*`, `victory/goals.rs`, `city_fire_tests.rs`); landed by another route. |
| `astra/opening-levers` | 2026-09-30 | 1 | Every code file the branch added is on master (`crates/ai/src/openings*`, `victory/goals.rs`, `city_fire_tests.rs`); landed by another route. |
| `astra/ai-city-strike-awareness` | 2026-10-01 | 1 | Every code file the branch added is on master (`crates/ai/src/openings*`, `victory/goals.rs`, `city_fire_tests.rs`); landed by another route. |
| `astra/ai-faction-goal-planning` | 2026-10-01 | 1 | Every code file the branch added is on master (`crates/ai/src/openings*`, `victory/goals.rs`, `city_fire_tests.rs`); landed by another route. |
| `astra/ai-faction-goal-planning-v2` | 2026-10-01 | 2 | Every code file the branch added is on master (`crates/ai/src/openings*`, `victory/goals.rs`, `city_fire_tests.rs`); landed by another route. |
| `astra/hard-openings-confirmation` | 2026-10-01 | 1 | `git cherry` shows the commit patches already on master. |
| `astra/hard-openings-handoff` | 2026-10-01 | 2 | `git cherry` shows the commit patches already on master. |
| `astra/hard-vs-human-playbook` | 2026-10-01 | 1 | `git cherry` shows the commit patches already on master. |
| `astra/scout-deadlock` | 2026-10-07 | 3 | Task report says the fix was merged into master (merge f9161e91 for the scout fix); tip is a leftover. |
| `astra/scout-deadlock-fix2` | 2026-10-07 | 3 | Task report says the fix was merged into master (merge f9161e91 for the scout fix); tip is a leftover. |
| `astra/combat-scenarios` | 2026-10-10 | 1 | All added code/asset files exist on master (checked file by file). |
| `astra/historical-faction-rename-r2` | 2026-10-10 | 1 | Factions are renamed on master (Switzerland, Mongols, Sparta, ...). |
| `astra/remove-quick-speed` | 2026-10-10 | 1 | Quick speed is gone on master (CLAUDE.md, decided 2026-10-10). |
| `astra/resource-medallions-integrate` | 2026-10-10 | 0 | `git merge-tree` of the branch into master changes nothing: no unique content. |
| `astra/ui-batch-1010` | 2026-10-10 | 1 | Merge-only branch (no commits of its own); its parents are on master. |
| `astra/ui-batch-1010b` | 2026-10-10 | 0 | `git merge-tree` of the branch into master changes nothing: no unique content. |
| `astra/search-snipe-payback` | 2026-10-11 | 3 | Its report `docs/sim-results/hard-vs-human/search-snipe-payback.md` is on master. |
| `astra/search-vs-hard-divergence` | 2026-10-11 | 1 | `git cherry` shows the commit patches already on master. |
| `astra/unit-era-samples` | 2026-10-11 | 2 | `git merge-tree` of the branch into master changes nothing: no unique content. |
| `astra/unit-repaint-infantry` | 2026-10-11 | 1 | Its one patch reverse-applies cleanly on master: already there. |
| `astra/diplomacy-overview` | 2026-10-18 | 1 | Diplomacy is on master (`app/src/diplomacy.ts`, `relations.ts` and the alliance e2e specs exist; 11 to 22 of the branch's added code files are on master). Tip is a pre-merge snapshot. |
| `astra/germany-abroad` | 2026-10-18 | 2 | Germany's movement rule is on master (learnings 2026-11-05: movement drawback carries the faction); the Germany rework replaced the rest. |
| `astra/germany-abroad-ai` | 2026-10-18 | 8 | Germany's movement rule is on master (learnings 2026-11-05: movement drawback carries the faction); the Germany rework replaced the rest. |
| `astra/review-evolve-gene-parity` | 2026-10-18 | 0 | `git merge-tree` of the branch into master changes nothing: no unique content. |
| `astra/review-perf-baseline-refresh` | 2026-10-18 | 1 | `git cherry` shows the commit patches already on master. |
| `astra/review-verify-lock-recovery` | 2026-10-18 | 2 | Task report says the fix was merged into master (merge f9161e91 for the scout fix); tip is a leftover. |
| `astra/remove-dead-rule-switches` | 2026-10-21 | 1 | Same commit subject is already on master. |
| `astra/combat-fx-aircraft-art` | 2026-10-24 | 1 | Aircraft art is on master (`claude/painted-aircraft` and the aircraft card landed 11-02 and 11-05). |
| `astra/gate-rules-2-copper-build` | 2026-10-24 | 0 | `git merge-tree` of the branch into master changes nothing: no unique content. |
| `astra/gate-rules-2-wonder-build` | 2026-10-24 | 0 | `git merge-tree` of the branch into master changes nothing: no unique content. |
| `astra/review-split-lobby` | 2026-10-24 | 1 | All added code/asset files exist on master (checked file by file). |
| `astra/aircraft-use-oil` | 2026-10-25 | 1 | Aircraft use Oil is on master via `astra/revive-aircraft-oil` and `air-oil-ai` (train-1031, train-1101d); these are the failed-gate first attempts. |
| `astra/era-relax-artillery` | 2026-10-25 | 1 | Artillery and Industrial-era siege landed through `astra/revive-artillery-siege` (10-31) and `claude/siege-era-gear`; these are the pre-gate attempts. |
| `astra/gate-aircraft-probe` | 2026-10-25 | 1 | Aircraft use Oil is on master via `astra/revive-aircraft-oil` and `air-oil-ai` (train-1031, train-1101d); these are the failed-gate first attempts. |
| `astra/review-art-celebration-vignettes` | 2026-10-25 | 1 | All added code/asset files exist on master (checked file by file). |
| `astra/review-art-future-era` | 2026-10-25 | 1 | `git merge-tree` of the branch into master changes nothing: no unique content. |
| `astra/unit-panel-onto-land6` | 2026-10-25 | 0 | Merge-only branch (no commits of its own); its parents are on master. |
| `astra/fix-aircraft-oil-gate` | 2026-10-26 | 8 | Aircraft use Oil is on master via `astra/revive-aircraft-oil` and `air-oil-ai` (train-1031, train-1101d); these are the failed-gate first attempts. |
| `astra/modern-auto-siege` | 2026-10-26 | 1 | Artillery and Industrial-era siege landed through `astra/revive-artillery-siege` (10-31) and `claude/siege-era-gear`; these are the pre-gate attempts. |
| `astra/modern-siege-adapt-ai` | 2026-10-26 | 7 | Artillery and Industrial-era siege landed through `astra/revive-artillery-siege` (10-31) and `claude/siege-era-gear`; these are the pre-gate attempts. |
| `astra/noise-1026-army` | 2026-10-26 | 1 | All added code/asset files exist on master (checked file by file). |
| `astra/noise-1026-modern` | 2026-10-26 | 1 | Artillery and Industrial-era siege landed through `astra/revive-artillery-siege` (10-31) and `claude/siege-era-gear`; these are the pre-gate attempts. |
| `astra/noise-1026-oil` | 2026-10-26 | 1 | Aircraft use Oil is on master via `astra/revive-aircraft-oil` and `air-oil-ai` (train-1031, train-1101d); these are the failed-gate first attempts. |
| `astra/cap-games-mounted` | 2026-10-31 | 1 | Mounted rebalance is on master (merge of `claude/mounted-on`, 10-31). |
| `astra/heraldry-redraw` | 2026-10-31 | 2 | India heraldry is on master (46 of 47 touched files exist; the only added file is a screenshot spec). |
| `astra/mounted-ui-fixes` | 2026-10-31 | 3 | Mounted rebalance is on master (merge of `claude/mounted-on`, 10-31). |
| `astra/germany-foundry-gate` | 2026-11-01 | 0 | Merge-only branch (no commits of its own); its parents are on master. |
| `astra/swiss-audit` | 2026-11-03 | 1 | `git cherry` shows the commit patches already on master. |
| `astra/siege-card-fit` | 2026-11-04 | 1 | Siege card above Turn details landed in `claude/land-1104b` (progress log 11-04). |

## (b) Superseded or abandoned (102)

| Branch | Last commit | Ahead | Evidence |
|---|---|---:|---|
| `astra/faction-balance-1p5x-v2` | 2026-09-27 | 7 | Balance-study record for V9 (adopted 09-28); superseded by V9 and the later faction reworks. |
| `astra/faction-balance-rerun` | 2026-09-28 | 18 | Balance-study record for V9 (adopted 09-28); superseded by V9 and the later faction reworks. |
| `astra/league-runner` | 2026-09-29 | 2 | Early league/evolution tooling. The league was dropped for the climb program; `tools/league/` on master has the kept parts. |
| `astra/2026-09-30-overnight-coach-9fcdbef8-00028-unescorted-settler-turns` | 2026-09-30 | 7 | Default-off opening primitives from the 09-30 overnight coach loop; report says strength and diversity gains unverified. Settler escort and rush locks were redone later as `settler_escort` and `campaign_commit`. |
| `astra/2026-09-30-overnight-coach-9fcdbef8-00076-rush-failures` | 2026-09-30 | 7 | Default-off opening primitives from the 09-30 overnight coach loop; report says strength and diversity gains unverified. Settler escort and rush locks were redone later as `settler_escort` and `campaign_commit`. |
| `astra/2026-09-30-overnight-coach-9fcdbef8-00101-rush-failures` | 2026-09-30 | 7 | Default-off opening primitives from the 09-30 overnight coach loop; report says strength and diversity gains unverified. Settler escort and rush locks were redone later as `settler_escort` and `campaign_commit`. |
| `astra/coach-loop` | 2026-09-30 | 5 | Default-off opening primitives from the 09-30 overnight coach loop; report says strength and diversity gains unverified. Settler escort and rush locks were redone later as `settler_escort` and `campaign_commit`. |
| `astra/vp-planner` | 2026-09-30 | 11 | Default-off opening primitives from the 09-30 overnight coach loop; report says strength and diversity gains unverified. Settler escort and rush locks were redone later as `settler_escort` and `campaign_commit`. |
| `astra/hard-recapture-r2` | 2026-10-01 | 8 | Hard recapture did not move the human benchmark (learnings 2026-10-01 'not merged'). |
| `astra/pacing-validation-r2` | 2026-10-01 | 2 | Pacing evidence for the removed Quick speed. |
| `astra/city-art-eras` | 2026-10-03 | 1 | Old city-art capture script; the art pipeline moved on. |
| `astra/oathbound-home-healing` | 2026-10-04 | 1 | Pre-rename Oathbound experiments (before the historical rename 10-10). |
| `astra/public-ownership-under-fog` | 2026-10-04 | 1 | WIP snapshots left 10-04 'before switching tasks'; no report; fog and forecast work was redone on master. |
| `astra/split-counterattack` | 2026-10-04 | 1 | WIP snapshots left 10-04 'before switching tasks'; no report; fog and forecast work was redone on master. |
| `astra/worked-tiles-forecast` | 2026-10-04 | 1 | WIP snapshots left 10-04 'before switching tasks'; no report; fog and forecast work was redone on master. |
| `astra/oathbound-blood-healing` | 2026-10-07 | 1 | Pre-rename Oathbound experiments (before the historical rename 10-10). |
| `astra/phone-specs-out-of-release` | 2026-10-07 | 1 | Phone-only Playwright exclusion; the phone suite was restructured since. |
| `astra/tollkeepers-goblin-mocks` | 2026-10-07 | 1 | Portrait mock variants for factions that no longer exist. |
| `astra/tollkeepers-vault-goblin-variants` | 2026-10-07 | 1 | Portrait mock variants for factions that no longer exist. |
| `astra/goal-first-ai` | 2026-10-08 | 1 | Early goal-first and committed-strategy AI (10-08..11); replaced by the goal-driven AI and the climb program. |
| `astra/goal-rites-expansion-gated` | 2026-10-08 | 1 | Early goal-first and committed-strategy AI (10-08..11); replaced by the goal-driven AI and the climb program. |
| `astra/search-samples-terminal-value` | 2026-10-08 | 1 | Search-era experiments; report says gains unproven or default OFF; search levels not shipped. |
| `astra/tactical-assault-sync` | 2026-10-08 | 2 | Search-era experiments; report says gains unproven or default OFF; search levels not shipped. |
| `astra/tactical-assault-sync-r2` | 2026-10-08 | 3 | Search-era experiments; report says gains unproven or default OFF; search levels not shipped. |
| `astra/tollkeepers-benchmark-zero` | 2026-10-08 | 2 | Pre-rename Tollkeepers AI genes; measured off by default, faction no longer exists. |
| `astra/committed-strategies` | 2026-10-09 | 7 | Early goal-first and committed-strategy AI (10-08..11); replaced by the goal-driven AI and the climb program. |
| `astra/committed-strategies-r2` | 2026-10-09 | 7 | Early goal-first and committed-strategy AI (10-08..11); replaced by the goal-driven AI and the climb program. |
| `astra/goal-first-glassroad-gate-r2` | 2026-10-09 | 1 | Pre-rename Glassroad experiments; Glassroad regression 'unisolated' in its own report; faction gone. |
| `astra/human-moves-vocabulary` | 2026-10-09 | 4 | Early league/evolution tooling. The league was dropped for the climb program; `tools/league/` on master has the kept parts. |
| `astra/league-training-aws` | 2026-10-09 | 2 | Early league/evolution tooling. The league was dropped for the climb program; `tools/league/` on master has the kept parts. |
| `astra/cavalry-experiment` | 2026-10-10 | 1 | Cavalry experiment; replaced by the mounted rebalance (130%/+40%) that landed 10-31. |
| `astra/cavalry-experiment-v2` | 2026-10-10 | 5 | Cavalry experiment; replaced by the mounted rebalance (130%/+40%) that landed 10-31. |
| `astra/choir-waxen-synod` | 2026-10-10 | 1 | Pre-rename Choir and Veyr work; Veyr gene rejected by its own CI. |
| `astra/glassroad-85-hall-phase` | 2026-10-10 | 1 | Pre-rename Glassroad experiments; Glassroad regression 'unisolated' in its own report; faction gone. |
| `astra/glassroad-elves-variants` | 2026-10-10 | 1 | Portrait mock variants for factions that no longer exist. |
| `astra/permanent-block-tag` | 2026-10-10 | 1 | Small availability-tag fix from 10-10; master's tag code has since changed. |
| `astra/search-peace-when-behind` | 2026-10-10 | 6 | Search-era experiments; report says gains unproven or default OFF; search levels not shipped. |
| `astra/settler-safety-human-level` | 2026-10-10 | 1 | 10-10/11 AI-quality study branches; findings recorded in `ideas.md` #11-13 and learnings; capital guard came back as `capital_guard` (off). |
| `astra/tollkeepers-83-contain-leader` | 2026-10-10 | 1 | Pre-rename Tollkeepers AI genes; measured off by default, faction no longer exists. |
| `astra/tollkeepers-83-goal-funding` | 2026-10-10 | 1 | Pre-rename Tollkeepers AI genes; measured off by default, faction no longer exists. |
| `astra/tollkeepers-merchant-line` | 2026-10-10 | 11 | Pre-rename Tollkeepers AI genes; measured off by default, faction no longer exists. |
| `astra/veyr-84-wild-window` | 2026-10-10 | 1 | Pre-rename Choir and Veyr work; Veyr gene rejected by its own CI. |
| `astra/choir-84-a2-regression` | 2026-10-11 | 1 | Pre-rename Choir and Veyr work; Veyr gene rejected by its own CI. |
| `astra/choir-third-city-timing` | 2026-10-11 | 1 | Pre-rename Choir and Veyr work; Veyr gene rejected by its own CI. |
| `astra/committed-army-last-seen` | 2026-10-11 | 1 | Early goal-first and committed-strategy AI (10-08..11); replaced by the goal-driven AI and the climb program. |
| `astra/gene-context-capguard` | 2026-10-11 | 2 | 10-10/11 AI-quality study branches; findings recorded in `ideas.md` #11-13 and learnings; capital guard came back as `capital_guard` (off). |
| `astra/glassroad-capital-defense` | 2026-10-11 | 1 | Pre-rename Glassroad experiments; Glassroad regression 'unisolated' in its own report; faction gone. |
| `astra/hvh-until-turn-r2` | 2026-10-11 | 1 | 10-10/11 AI-quality study branches; findings recorded in `ideas.md` #11-13 and learnings; capital guard came back as `capital_guard` (off). |
| `astra/map-art-legibility-polish` | 2026-10-11 | 1 | UI polish from 10-11 that master reworked; the Sakoku intruder rule was removed 10-22. |
| `astra/oathbound-campaign-payback-gate` | 2026-10-11 | 1 | Pre-rename Oathbound experiments (before the historical rename 10-10). |
| `astra/oathbound-crown-aware-plan` | 2026-10-11 | 1 | Pre-rename Oathbound experiments (before the historical rename 10-10). |
| `astra/oathbound-genes-eight-map` | 2026-10-11 | 1 | Pre-rename Oathbound experiments (before the historical rename 10-10). |
| `astra/overreach-v3-ui-leftovers` | 2026-10-11 | 1 | UI polish from 10-11 that master reworked; the Sakoku intruder rule was removed 10-22. |
| `astra/remine-failed-genes` | 2026-10-11 | 1 | 10-10/11 AI-quality study branches; findings recorded in `ideas.md` #11-13 and learnings; capital guard came back as `capital_guard` (off). |
| `astra/rival-vp-hold-response` | 2026-10-11 | 1 | 10-10/11 AI-quality study branches; findings recorded in `ideas.md` #11-13 and learnings; capital guard came back as `capital_guard` (off). |
| `astra/sakoku-show-intruder` | 2026-10-11 | 2 | UI polish from 10-11 that master reworked; the Sakoku intruder rule was removed 10-22. |
| `astra/tactical-close-to-contact` | 2026-10-11 | 1 | Search-era experiments; report says gains unproven or default OFF; search levels not shipped. |
| `astra/terrain-massif-peaks` | 2026-10-11 | 1 | Hand-cut S2 terrain sprites (81+ asset files) that are not on master; painted organic terrain replaced them. Check before deleting only if the art is wanted. |
| `astra/terrain-single-trees` | 2026-10-11 | 1 | Hand-cut S2 terrain sprites (81+ asset files) that are not on master; painted organic terrain replaced them. Check before deleting only if the art is wanted. |
| `astra/terrain-wood-edge` | 2026-10-11 | 1 | Hand-cut S2 terrain sprites (81+ asset files) that are not on master; painted organic terrain replaced them. Check before deleting only if the art is wanted. |
| `astra/tollkeeper-low-odds-attacks` | 2026-10-11 | 1 | Pre-rename Tollkeepers AI genes; measured off by default, faction no longer exists. |
| `astra/ui-batch-1011a` | 2026-10-11 | 2 | UI polish from 10-11 that master reworked; the Sakoku intruder rule was removed 10-22. |
| `astra/era-scored-grayout` | 2026-10-18 | 1 | UI/rule tweaks from 10-18 reviews; each file is touched on master since, content not present. |
| `astra/levies-count-aircraft` | 2026-10-18 | 1 | UI/rule tweaks from 10-18 reviews; each file is touched on master since, content not present. |
| `astra/nest-upgrade-rows` | 2026-10-18 | 1 | UI/rule tweaks from 10-18 reviews; each file is touched on master since, content not present. |
| `astra/overlays-followups-2` | 2026-10-18 | 1 | UI/rule tweaks from 10-18 reviews; each file is touched on master since, content not present. |
| `astra/city-names-more` | 2026-10-22 | 1 | Old review/hygiene branch (10-18..10-26); its content is a report or a fix for a release that shipped long ago. |
| `astra/oss-docs-generalise` | 2026-10-23 | 1 | Old review/hygiene branch (10-18..10-26); its content is a report or a fix for a release that shipped long ago. |
| `astra/ai-gate-audit2-rule` | 2026-10-24 | 4 | The 10-24..25 era-rule bundle: its own all-seats AI gate failed ('not ready to land'); era objectives came back as `revive-era-objectives` and the objective deck on 10-31. |
| `astra/calibrate-era-objectives` | 2026-10-24 | 4 | The 10-24..25 era-rule bundle: its own all-seats AI gate failed ('not ready to land'); era objectives came back as `revive-era-objectives` and the objective deck on 10-31. |
| `astra/climb-round2-start` | 2026-10-24 | 1 | Climb-program run logs (10-24, 10-26); climb results live in `docs/sim-results/ai-quality/climb-program/`. |
| `astra/ws-rules-eras-cavalry` | 2026-10-24 | 3 | The 10-24..25 era-rule bundle: its own all-seats AI gate failed ('not ready to land'); era objectives came back as `revive-era-objectives` and the objective deck on 10-31. |
| `astra/ai-gate-rules-2` | 2026-10-25 | 1 | The 10-24..25 era-rule bundle: its own all-seats AI gate failed ('not ready to land'); era objectives came back as `revive-era-objectives` and the objective deck on 10-31. |
| `astra/bisect-era-rules-gate` | 2026-10-25 | 6 | The 10-24..25 era-rule bundle: its own all-seats AI gate failed ('not ready to land'); era objectives came back as `revive-era-objectives` and the objective deck on 10-31. |
| `astra/fix-era-relax-war` | 2026-10-25 | 8 | The 10-24..25 era-rule bundle: its own all-seats AI gate failed ('not ready to land'); era objectives came back as `revive-era-objectives` and the objective deck on 10-31. |
| `astra/gate-rules-1025` | 2026-10-25 | 2 | The 10-24..25 era-rule bundle: its own all-seats AI gate failed ('not ready to land'); era objectives came back as `revive-era-objectives` and the objective deck on 10-31. |
| `astra/gate-rules-1025b` | 2026-10-25 | 4 | The 10-24..25 era-rule bundle: its own all-seats AI gate failed ('not ready to land'); era objectives came back as `revive-era-objectives` and the objective deck on 10-31. |
| `astra/night-lander-1025` | 2026-10-25 | 7 | Old review/hygiene branch (10-18..10-26); its content is a report or a fix for a release that shipped long ago. |
| `astra/release-8910-failures` | 2026-10-25 | 1 | Old review/hygiene branch (10-18..10-26); its content is a report or a fix for a release that shipped long ago. |
| `astra/review-art-terrain-increment` | 2026-10-25 | 1 | Hand-cut S2 terrain sprites (81+ asset files) that are not on master; painted organic terrain replaced them. Check before deleting only if the art is wanted. |
| `astra/tech-card-alternatives` | 2026-10-25 | 1 | UI study; `claude/techcard-fold` landed the tech-card change (train-1103b). |
| `astra/climb-director-1026` | 2026-10-26 | 3 | Climb-program run logs (10-24, 10-26); climb results live in `docs/sim-results/ai-quality/climb-program/`. |
| `astra/fix-staging-ring` | 2026-10-26 | 3 | Failed war gate (report); staging ring was redone as `fix-staging-ring-r2`. |
| `astra/luna-blunder-review-1026` | 2026-10-26 | 1 | Old review/hygiene branch (10-18..10-26); its content is a report or a fix for a release that shipped long ago. |
| `astra/noise-1026-peace` | 2026-10-26 | 1 | Benchmark-only copies for the 10-26 noise study (`docs/sim-results/ai-quality/gate-noise-1026.md`). |
| `astra/noise-1026-strike` | 2026-10-26 | 1 | Benchmark-only copies for the 10-26 noise study (`docs/sim-results/ai-quality/gate-noise-1026.md`). |
| `astra/search-air-execution` | 2026-10-26 | 70 | Air execution for the search AI (70 commits, 145k lines); search AI is not the shipped Hard, and air landed through `air_stage_read` and air-oil-ai. |
| `astra/strategy-lab-r1` | 2026-10-26 | 3 | Strategy Lab rounds 1-6: 'No strategy promoted' (r5 report); r6 'does not validate a decisive-army implementation'. 57 commits, 268k lines of results. |
| `astra/strategy-lab-r2` | 2026-10-26 | 16 | Strategy Lab rounds 1-6: 'No strategy promoted' (r5 report); r6 'does not validate a decisive-army implementation'. 57 commits, 268k lines of results. |
| `astra/strategy-lab-r3` | 2026-10-26 | 25 | Strategy Lab rounds 1-6: 'No strategy promoted' (r5 report); r6 'does not validate a decisive-army implementation'. 57 commits, 268k lines of results. |
| `astra/strategy-lab-r4` | 2026-10-26 | 44 | Strategy Lab rounds 1-6: 'No strategy promoted' (r5 report); r6 'does not validate a decisive-army implementation'. 57 commits, 268k lines of results. |
| `astra/strategy-lab-r5` | 2026-10-26 | 49 | Strategy Lab rounds 1-6: 'No strategy promoted' (r5 report); r6 'does not validate a decisive-army implementation'. 57 commits, 268k lines of results. |
| `astra/strategy-lab-r6` | 2026-10-27 | 57 | Strategy Lab rounds 1-6: 'No strategy promoted' (r5 report); r6 'does not validate a decisive-army implementation'. 57 commits, 268k lines of results. |
| `astra/art-ship-terrain` | 2026-10-31 | 3 | Hand-cut S2 terrain sprites (81+ asset files) that are not on master; painted organic terrain replaced them. Check before deleting only if the art is wanted. |
| `astra/japan-tokugawa` | 2026-10-31 | 1 | Tokugawa Japan replaced by India (train-1031: 'Japan returns'); faction not in the nine on master. |
| `astra/contain-builds-army` | 2026-11-01 | 3 | Rejected on evidence 11-02 (see-the-leader -1.17 pp, contain-builds-army cut Athens's wars on the leader 40 to 29, combo weaker than coalition alone); `coalition_push` landed instead. |
| `astra/leader-pressure-combo` | 2026-11-01 | 16 | Rejected on evidence 11-02 (see-the-leader -1.17 pp, contain-builds-army cut Athens's wars on the leader 40 to 29, combo weaker than coalition alone); `coalition_push` landed instead. |
| `astra/see-the-leader` | 2026-11-01 | 6 | Rejected on evidence 11-02 (see-the-leader -1.17 pp, contain-builds-army cut Athens's wars on the leader 40 to 29, combo weaker than coalition alone); `coalition_push` landed instead. |
| `astra/see-the-leader-gated` | 2026-11-01 | 11 | Rejected on evidence 11-02 (see-the-leader -1.17 pp, contain-builds-army cut Athens's wars on the leader 40 to 29, combo weaker than coalition alone); `coalition_push` landed instead. |
| `astra/stall-1101` | 2026-11-01 | 1 | Skill and progress-note merge only; notes are on master. |
| `astra/neighbour-lane-watch` | 2026-11-02 | 4 | Gene left 0: inconclusive (progress log 11-02). |
| `astra/ai-peace-settler-gates` | 2026-11-04 | 1 | `peace_settler_capture` re-gated clearly harmful 10-24..11-03 (rivals +3.12 pp softer); default off, unbenchmarked. |

## (c) Unique unlanded work worth a look (16)

| Branch | Last commit | Ahead | Evidence |
|---|---|---:|---|
| `astra/filler-fifo-tiebreak` | 2026-10-26 | 1 | Filler picks the oldest queued batch among equal priorities (10-26); master has slot FIFO but check this tiebreak. |
| `astra/ai-raze-air-value` | 2026-10-27 | 1 | Genes off by default, never benchmarked (10-27). Worth a gate run: truce-horizon forecasts and raze/aircraft evacuation. |
| `astra/ai-truce-horizon` | 2026-10-27 | 1 | Genes off by default, never benchmarked (10-27). Worth a gate run: truce-horizon forecasts and raze/aircraft evacuation. |
| `astra/improved-supply` | 2026-10-31 | 5 | Supply extraction timing and per-deposit AI values (10-31, 5 commits after a WIP); no report. Look before deleting. |
| `astra/defend-goal-objects` | 2026-11-01 | 1 | Gene that scores held VP cities and leader targets (11-01); needs reconciling with `capital-vp` before a gate (its own report). |
| `astra/launch-copy` | 2026-11-01 | 2 | Launch copy and README draft awaiting the designer's review (progress log 11-02). |
| `astra/war-era-resources` | 2026-11-01 | 4 | Era-resource valuation for wartime (idea #41), per-faction era table in `ai-quality/war-era-resources-1101/`; 'broader obsolete-army problem not solved'. Unlanded. |
| `astra/readme-launch-draft` | 2026-11-02 | 1 | Launch copy and README draft awaiting the designer's review (progress log 11-02). |
| `astra/absent-civ6-reasons` | 2026-11-04 | 1 | Docs: why Civ systems are absent (11-04), docs only, clean. |
| `astra/ai-wonder-air-orders` | 2026-11-04 | 2 | Opt-in aircraft action policy (11-04), correctness tests only, not benchmarked; fits open problem 'air stage'. |
| `astra/checked-arith-cl2` | 2026-11-04 | 2 | Clippy fix in `crates/core/src/validation.rs` (11-04); small, check whether master already carries it. |
| `astra/combat-lab-gate` | 2026-11-04 | 2 | Fast dev build for the combat lab check (11-04); small, relevant to the stale combat-lab tool. |
| `astra/siege-target-reach` | 2026-11-04 | 2 | Joint siege targets by reachable force (11-04), bears on 'Swiss attack fails to finish'; unlanded, 193k lines mostly data. |
| `astra/war-attack-fixture` | 2026-11-04 | 1 | New attack fixture (war-india-nicaea-t68) from 11-04: useful for the war suite's attack side. |
| `astra/worktree-owner-token-cl5` | 2026-11-04 | 2 | Ops fix: orphan cleanup deleted the pool acquisition guard (race). Test included; small. |
| `astra/byzantium-companies-misplay` | 2026-11-06 | 8 | Codex task of 11-06 (8 commits, repinned tape); not in the in-flight skip list, so listed for the lead. |

## Skipped (in flight or being landed by another agent)

`astra/rules-batch-1107`, `astra/germany-projects-admission`, `astra/byzantium-theodosian-walls`, `astra/peace-names-every-party`, `astra/gate-off-switch-fix`, `astra/decision-trace`, `astra/swiss-hoard-misplay`, `astra/core-forecast-truth`, `astra/core-rule-bugs`, `astra/athens-greece-captive`, `astra/objectives-capture-accounting`, `astra/pending-spend-forecast`, `astra/automation-cannot-choose`, `astra/text-guide-rules`, `astra/text-city-terms`, `astra/release-docs-hygiene`, `astra/map-art-fixes`, `astra/payoff-art-consistency`, `astra/complementary-buys`, `astra/capture-trade-vp`. Of these 24 names, 20 still show as unmerged.

## Done (lead, 2026-11-07)
The 159 branches in (a) and (b) were archived: each tip is kept as tag `archive/astra/<name>` and the branch was deleted. Restore one with `git branch astra/<name> archive/astra/<name>`. Class (c) and the in-flight branches remain as branches.
