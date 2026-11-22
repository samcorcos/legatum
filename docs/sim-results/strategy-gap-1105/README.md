# Strategy gap, experiment 1: the quota ablation census (2026-11-05/06)

The first experiment from `docs/design/strategy-gap/fable.md` (Part 4, "the one experiment to run tonight"): change one shared quota for every faction and see who it helps. Branch `claude/strategy-gap-lab`; nothing here is on master.

**Setup.** Eight-empire Hard, AI only, 4,096 games an arm, seed 9311003, held-out seed 8241001. One `legatum-sim` binary for every arm (commit 928373ab: master fbf1c90fc plus the gene `walls_garrison` and an upkeep counter). Each arm is a params file in `params/`. Every shared change keeps the factions' built-in values through `faction_overrides`, because a shared gene set away from the default turns off `BUILTIN_FACTION_DECISIONS` for that gene. The analysis is `census.py`: win rate per seat, paired by seed, with a bootstrap 95% interval. Cities lost and eliminations are per seat. Upkeep is summed over the game from the new `empire_flow.upkeep_paid` telemetry.

**The control reproduced the published census exactly.** All 4,096 games match `census-playbooks-1105-1105-014755` (same turns, winner, and every seat's VP and cities). A control loaded through an empty params file also matches, game for game, so the params path itself changes nothing. Every arm that reaches a faction differs from control in all 4,096 games. A2 matches in 429 games and A5 in 21, where the gene never bound.

## Which factions each arm reaches

| Arm | Change | Reaches | Does not reach (why) |
|---|---|---|---|
| A1 | `army_war_extra` 5→1 | Byzantium, Celts, Mongols, Sparta, India | Athens (built-in 0, kept), Germany and Switzerland (built-in 1 already), Aztecs (`aztecs_playbook` bit 4 already caps it at 1) |
| A2 | `army_hard_extra` 2→0 | Athens, Aztecs, Celts, Germany, India, Mongols | Sparta 4, Byzantium 8, Switzerland 1 (built-ins kept; the Swiss quota uses `army_banker` anyway) |
| A3 | `walls_garrison` 1 (new gene: the `swiss_playbook` 1 rule without the faction test) | all but Switzerland | Switzerland already plays it (`swiss_playbook` 2) |
| A4 | `defend_margin_percent` 200→100, `door_defense` 100→50 | all but India | India (built-in 250 and 150, kept) |
| A5 | `settler_min_site_score` 60→30, `settler_max_site_distance` 10→16 | Aztecs, Celts, India, Mongols, Sparta | Athens, Byzantium, Switzerland, Germany (built-ins kept) |
| A6 | `conquest_runaway_percent` 45→25 | **none** | the gene is read only under `conquest_bank_goal`, which only Switzerland has, and its built-in 36 was kept. All 4,096 games are identical to control. It is not a general runaway threshold, so the prediction for it in fable.md does not apply |
| A7 | A1 + A3 | union of the two | |

## Results, seed 9311003 (paired change in win rate per seat, pp [95%]; bold = interval excludes 0)

| Arm | Ath | Azt | Byz | Cel | Ger | Ind | Mon | Spa | Swi |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| control win % | 20.0 | 16.9 | 11.3 | 12.1 | 6.3 | 8.7 | 13.8 | 12.6 | 10.8 |
| A1 | -1.4 [-2.6, +0.1] | **-1.6 [-3.0, -0.2]** | **+3.0 [+1.8, +4.2]** | **+2.1 [+0.9, +3.3]** | +0.2 [-0.8, +1.1] | -0.8 [-1.8, +0.2] | **+1.8 [+0.5, +3.1]** | **-1.4 [-2.7, -0.2]** | **-1.9 [-3.1, -0.8]** |
| A2 | -1.1 [-2.3, +0.1] | +0.2 [-1.0, +1.4] | +0.0 [-1.0, +1.1] | +0.2 [-0.8, +1.4] | +0.7 [-0.1, +1.6] | -0.6 [-1.5, +0.2] | +0.4 [-0.7, +1.6] | +0.1 [-1.0, +1.1] | +0.0 [-1.0, +1.1] |
| A3 | +1.1 [-0.5, +2.6] | -0.2 [-1.7, +1.2] | +0.7 [-0.6, +1.9] | -1.2 [-2.5, +0.1] | **+1.2 [+0.2, +2.2]** | **+1.6 [+0.4, +2.7]** | **+1.6 [+0.2, +3.0]** | **-1.9 [-3.2, -0.5]** | **-2.9 [-4.2, -1.7]** |
| A4 | -1.3 [-2.6, +0.1] | +0.8 [-0.6, +2.2] | +0.5 [-0.7, +1.7] | -0.4 [-1.6, +0.8] | +0.4 [-0.6, +1.4] | -0.5 [-1.6, +0.6] | +0.8 [-0.6, +2.0] | +0.8 [-0.5, +2.1] | -1.1 [-2.3, +0.1] |
| A5 | -0.7 [-2.2, +0.7] | +0.9 [-0.5, +2.3] | +0.7 [-0.5, +1.8] | -0.6 [-1.8, +0.5] | +1.0 [-0.1, +1.9] | -0.3 [-1.3, +0.8] | -0.5 [-1.8, +0.7] | +0.7 [-0.6, +1.9] | -1.1 [-2.2, +0.1] |
| A6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| A7 | -0.4 [-2.0, +1.2] | **-2.4 [-3.9, -1.0]** | **+4.1 [+2.7, +5.4]** | +1.1 [-0.1, +2.4] | -0.1 [-1.1, +0.9] | +1.1 [-0.1, +2.3] | **+2.7 [+1.3, +4.1]** | **-3.0 [-4.3, -1.7]** | **-3.2 [-4.4, -2.0]** |

Cities lost per seat-game, all seats: control 1.83, A1 1.78, A2 1.81, **A3 1.66**, A4 1.86, A5 1.87, **A7 1.61**. Seats eliminated: 11.9%, 11.4%, 11.8%, **10.4%**, 12.1%, 11.8%, **9.7%**. Upkeep owed per game falls with A1 for the factions it reaches (Byzantium 901 to 708, Celts 1,076 to 851, Mongols 775 to 654, Sparta 846 to 747). A3 barely moves upkeep: outside Switzerland's `army_banker` path the Hard peacetime quota still counts every city (`garrisoned_city_count`). The gain comes from cities kept: every faction loses fewer, for example Athens 1.44 to 1.22, Germany 1.47 to 1.29 and India 3.59 to 3.34. Full numbers are in `census-9311003.json`.

**How to read a shared arm.** Win share in an AI-only census is zero-sum. When a shared arm makes everyone defend better, the factions that win by taking cities (Sparta) or that already had the rule (Switzerland in A3, the Aztecs in A1) must fall, even if the change is good for each of them. So "+1.5 pp on two factions without hurting others" cannot be read from a shared arm. Two other measures answer it: each faction alone with the gene (its own effect), and the human-seat gate, where the opponent is held fixed.

## Own effect: the gene on one faction only (seed 9311003)

| Gene | Ath | Azt | Byz | Cel | Ger | Ind | Mon | Spa | Swi |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `walls_garrison` 1 | **+5.0 [+3.5, +6.5]** | **+4.2 [+2.8, +5.7]** | **+4.0 [+2.7, +5.3]** | **+1.7 [+0.5, +2.9]** | **+3.0 [+1.9, +4.0]** | **+4.4 [+3.2, +5.6]** | **+4.5 [+3.2, +5.9]** | **+3.1 [+1.8, +4.4]** | (+3.85 when it shipped as `swiss_playbook` 1) |
| `army_war_extra` 1 | (built-in 0) | (already 1) | **+3.2 [+2.1, +4.3]** | **+2.5 [+1.4, +3.7]** | (built-in 1) | +0.3 [-0.7, +1.3] | **+1.6 [+0.3, +2.8]** | -0.3 [-1.5, +0.9] | (built-in 1) |

`walls_garrison` alone helps every faction it reaches, by 1.7 to 5.0 points, and halves eliminations or close to it (Germany 12.3% to 7.9%, Byzantium 14.8% to 10.1%, India 17.2% to 12.0%). India's +4.39 [+3.19, +5.59] matches the India coach screen's env-switch arm to the hundredth (`docs/learnings.md` 2026-11-05), which confirms the gene is the same rule. Sparta's and the Celts' losses in the shared arm are therefore a rivals effect: walled neighbours are harder to take. The lean war army helps the three factions it reaches that fight the most (Byzantium, Celts, Mongols) and is neutral for Sparta and India. Each own-effect arm matched control in 447 to 850 games: the games without that faction, plus those where the gene never bound. Full numbers are in `census-own-9311003.json`.

## Held out, seed 8241001 (shared arms)

| Arm | Ath | Azt | Byz | Cel | Ger | Ind | Mon | Spa | Swi |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| control win % | 19.4 | 16.6 | 10.7 | 11.7 | 7.5 | 9.5 | 13.8 | 13.2 | 10.3 |
| A1 | **-2.6 [-4.0, -1.3]** | -1.0 [-2.4, +0.4] | **+3.3 [+2.1, +4.5]** | **+1.4 [+0.2, +2.6]** | **-1.3 [-2.2, -0.4]** | -0.1 [-1.3, +1.0] | **+2.4 [+1.1, +3.6]** | -1.0 [-2.3, +0.2] | -1.1 [-2.3, +0.0] |
| A3 | +1.0 [-0.6, +2.5] | -1.0 [-2.4, +0.5] | +1.1 [-0.2, +2.4] | -0.8 [-2.1, +0.4] | -0.6 [-1.6, +0.5] | **+1.7 [+0.5, +2.9]** | **+2.8 [+1.3, +4.1]** | **-2.2 [-3.5, -0.8]** | **-2.0 [-3.1, -0.8]** |
| A7 | -1.0 [-2.5, +0.6] | **-1.6 [-3.1, -0.1]** | **+4.2 [+2.9, +5.6]** | +0.1 [-1.2, +1.4] | -0.9 [-1.9, +0.2] | +0.8 [-0.4, +2.0] | **+3.8 [+2.4, +5.2]** | **-3.3 [-4.6, -2.0]** | **-2.2 [-3.4, -1.1]** |

The same pattern holds on the held-out seed. Cities lost per seat-game: control 1.81, A1 1.78, A3 1.64, A7 1.61. Seats eliminated: 12.2%, 11.6%, 10.2%, 9.7% (`census-8241001.json`).

## Human-seat gate (seed 310001, 256 reseeded salts × 8 maps, `gate-arms.json`, `gate-comparison.json`)

The Hard seat on the eight benchmark maps. `riv` gives the candidate to the rivals only; a negative change means the seat wins less, so the rivals got stronger. `seat` gives the candidate to the seat only. Control: 472 of 2,048 seat wins.

| Arm | riv: seat change, pp [95%] | seat: seat change, pp [95%] |
|---|---:|---:|
| A1 | **-2.39 [-4.20, -0.54]** (423) | -0.20 [-1.81, +1.42] (468) |
| A3 | **-3.03 [-5.37, -0.78]** (410) | +1.90 [-0.20, +4.10] (511) |
| A7 | **-4.15 [-6.35, -1.90]** (387) | +1.27 [-0.78, +3.37] (498) |

Against a fixed opponent, the AI with A3 or A7 is harder to beat. The largest single map is 8.4 Sparta: walled India, Byzantium and Germany cut the Spartan seat's wins from 100 to 50 (A3) and to 43 (A7). The seat arms are not worse. What the gate cannot see: no map seats the Aztecs; the Celts appear only as the seat (never as rivals); Athens and Byzantium appear only as rivals (never as the seat).

## Experiment 2: exploiter seat `peace_cycler` (prediction 1)

`crates/sim/examples/peace_cycler.rs` plays each seed twice with eight Hard seats: seat 0 as plain Hard, then as Hard with scripted Peace with every empire it has fought. It accepts the target's Peace, offers Peace after 8 turns of war, and redeclares the turn the truce ends. With no war by T30 it declares on the nearest empire. 2,048 paired games, seed 7700001 (`peace-cycler-summary.json`, run `sg-peace-cycler-1106-000952`).

- **The AI's army does not reset.** Over 24,254 truces, the target's paid military units were 6.81 when Peace was signed and 7.34 at the redeclaration. Hard keeps its army through Peace and grows it. Prediction 1 (the war quota resets) does not hold for the current AI.
- **Cycling loses.** Seat 0 wins 12.4% as plain Hard and 7.8% as the cycler (-4.6 pp [-6.2, -3.1]). Elimination rises from 12.2% to 22.7%, and cities fall from 4.9 to 3.8. The seat redeclared 14 times a game (median). It lost on every faction.
- This is a crude seat: its forced opening war (in 66% of games Hard had fought no one by T30) and constant wars are part of the loss. The answer to the question is the army count. No exploit was found, and the AI is not demobilising.

## Reading

1. **Walls as the garrison is the general rule we kept finding one faction at a time.** It is worth 1.7 to 5.0 points to every faction that adopts it, it cuts cities lost and eliminations for everyone, it holds on the held-out seed, and it makes rivals 3 points harder for a Hard seat to beat. The Swiss and India playbook gains were this rule. Athens (+5.0) and the Mongols (+4.5) gain the most, though nobody had played their playbooks.
2. **The lean war army is a real but narrower gain:** +1.6 to +3.2 for Byzantium, the Celts and the Mongols, neutral for Sparta and India, with less upkeep. It already ships for Germany, Switzerland and (through the playbook) the Aztecs.
3. **The other quotas are not where the gap is.** The peacetime army (A2), the defensive margin and door (A4) and Settler reach (A5) read null on every faction. The runaway threshold (A6) is not wired to anyone but the Swiss.
4. **Fable's "+2 to +4 pp table-wide" cannot show up as win share in an AI-only census,** which is zero-sum. Read a shared gene by own-effect arms, by cities lost and eliminations, and by the gate's rivals arm.

## Recommendation (the lead's call; nothing landed)

- **Ship `walls_garrison` 1 as the shared Hard default** (all factions; it is the Swiss rule, so `swiss_playbook` 1 becomes redundant for that part). Evidence: own effect +1.7 to +5.0 on all eight factions it reaches; shared arm held out; gate rivals arm -3.03 pp [-5.37, -0.78], seat arm +1.90 [-0.20, +4.10].
- **Then `army_war_extra` 1 as the shared default** (A7 together): own effect +3.2 Byzantium, +2.5 Celts, +1.6 Mongols, null for Sparta and India; combined gate rivals arm -4.15 pp [-6.35, -1.90]. Keep Athens at its built-in 0.
- Landing requires the usual default-flip care (`docs/learnings.md`: tests that use a faction as a generic fixture, `hard_vs_human` fixed milestones).
- Follow-up worth one arm: under `walls_garrison` the Hard peacetime quota still counts every city outside `army_banker`. Letting it count only `troop_garrison_cities` (as the Swiss do) is the upkeep half of the Swiss rule, still untested for the others.

## Landing (2026-11-06, branch only)

Recommendations 1 and 2 are now the shared Hard defaults on this branch (commit e7e862a6e, merged with master d7c6d4717): `walls_garrison` 1 and `army_war_extra` 1. Athens keeps its built-in 0. Germany's and Switzerland's built-in `army_war_extra` 1 now equal the shared default and were dropped, with no change in behaviour. The Aztec playbook's cap stays 1. The Swiss `redoubt_garrison` path is unchanged: Swiss seats played it before and still do.

**Census against current master** (4,096 games, seed 9311003, master d7c6d4717 against the flipped branch d582069a7): master matches the earlier control game for game (all 4,096), and the flipped branch matches arm A7 game for game (all 4,096). So the landing is A7 exactly: Byzantium +4.1, Mongols +2.7, Celts +1.1, India +1.1, Aztecs -2.4, Sparta -3.0, Switzerland -3.2; cities lost per seat 1.83 → 1.61, eliminations 11.9% → 9.7% (`census-landing-vs-master.json`; runs `sg-plain-master-9311003-1106-021102`, `sg-plain-flip-9311003-1106-021626`).

**Follow-up arm, not landed: `walls_peace_quota` 1** (new gene, default 0). On top of the flip, the Hard peacetime quota counts only cities without Walls (at least the two free units), which is the Swiss upkeep half of the rule. It is null: Aztecs +1.35 [+0.14, +2.59], every other faction within ±0.9 and not significant. 343 games were identical, and upkeep did not move (for example Germany 856 → 807, Aztecs 1,099 → 1,132). As A2 showed, the peacetime quota rarely binds; the war quota and the garrison floor are where the troops come from (`census-landing-peace-quota.json`, run `sg-walls-peace-quota-9311003-1106-022118`).

## Runs and cost

Run directories (under `~/LegatumRuns/cloud/`): controls `sg-plain-9311003-1105-232602`, `sg-control-9311003-1105-232635`, `sg-plain-8241001-1106-002307`; arms `sg-a1-war-extra-1-9311003-1105-233729`, `sg-a2-hard-extra-0-9311003-1105-235803`, `sg-a3-walls-garrison-9311003-1105-234223`, `sg-a4-defend-door-half-9311003-1106-000327`, `sg-a5-settler-reach-9311003-1106-000924`, `sg-a6-runaway-25-9311003-1106-001610`, `sg-a7-lean-walled-9311003-1105-235311`; held out `sg-a1-war-extra-1-8241001-1106-002830`, `sg-a3-walls-garrison-8241001-1106-003326`, `sg-a7-lean-walled-8241001-1106-003847`; own effects `sg-own-*-9311003-*` (13 runs); gate `sg-gate-1106-1106-003706`; exploiter `sg-peace-cycler-1106-000952`.

28 spot runs, about 2 server-hours, about $1.30 at launch-time spot prices, plus a few launches killed within minutes (see below): under $1.50 in all. Builds and the branch verify ran on the build box.

Each census is one `run-batch.sh` spot server, about 4 minutes. Reproduce one arm:
`tools/cloud/run-batch.sh --count 1 --max-hours 1 --ai-params docs/sim-results/strategy-gap-1105/params/<arm>.json -- --players 8 --difficulty hard --games 4096 --seed 9311003 --label <label>`, then `python3 -I docs/sim-results/strategy-gap-1105/census.py <control run> <name>=<arm run>`.

Tooling notes: three `run-batch.sh` launches in parallel died while polling (exit 254), and each death terminated its server. Two in parallel and sequential chains completed, given a retry on exit 254 and on `MaxSpotInstanceCountExceeded` (other agents' batches filled the spot quota for several minutes at a time).
