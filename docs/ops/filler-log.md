# Box filler log

Batches the ops check queues on the cloud box's idle filler (`tools/cloud/filler.sh`), with their purpose and, once
finished, a one-line result. The climb program's own batches are logged by its daemon, not here.

| Queued (UTC) | Batch id | Purpose | Result |
| --- | --- | --- | --- |
| 2026-10-24 01:3x | `ops-hvh-master-44d04058b-1024` | Human-seat benchmark on master after the faction rename (256 salts from 900001, dice reseeded per salt): a fresh baseline for the climb's banking gate | **failed** rc=101 after 74 min: all games ran, then `hard_vs_human` panicked writing `out/hard-vs-human.json` because `out/` didn't exist (it doesn't create `--out-dir`; `milestone_suite` does). Re-queued below |
| 2026-10-24 04:2x | `ops-hvh-master-0a297a5e3-1024` | Same benchmark on master 0a297a5e3 (256 salts from 900001), with `"inputs": {"out/.keep": ""}` so the filler creates `out/` first; 2 h timeout | done rc=0 in 53 min (05:20Z): overall score 0.441 (95% CI 0.425–0.456), milestone ratio 0.50; results via `filler.sh fetch` |
| 2026-10-24 10:3x | `ops-census8-master-5f786a22a-1024` | Balanced eight-empire Hard census on master 5f786a22a (2,048 games, Medium, seed 8241001): fresh reference while the climb waits on its director; the human-seat benchmark was skipped because master's engine is unchanged since 0a297a5e3 | done rc=0 in 7.5 min: 0 panics, mean 300 turns, all VP-hold wins (hold break rate 48%). Win share: Mongols 22.4%, Athens 18.7%, Byzantium 18.0%, Sparta 12.6%, Japan 9.1%, Switzerland 8.4%, Germany 6.7%, Celts 4.1% |
| 2026-10-24 12:3x | `ops-census8-bank-77305485e-1024` | Same census (2,048 games, seed 8241001) on the round-1 bank branch `astra/climb-bank-r1-land` 77305485e, ready for lead review: a direct before/after against the master census above | done rc=0 in 7 min: 0 panics, mean 298 turns, all VP-hold wins, hold break rate 47% (master 48%). The spec uploaded no outputs, so per-faction win shares aren't in results; rerun with `outputs` if the director needs them |
| 2026-10-24 12:3x | `ops-hvh-bank-77305485e-1024` | Human-seat benchmark (256 salts from 900001, dice reseeded) on the same bank branch, to compare with master's 0.441 | done rc=0 in 35 min: overall score 0.308 (95% CI 0.292–0.327) vs master's 0.441 (0.425–0.456), milestone ratio 0.37 vs 0.50. The CIs don't overlap: a clear drop on the bank branch, for the director to weigh before banking |
| 2026-10-24 16:2x | `ops-census8-master-42ec5dac4-1024b` | Eight-empire Hard census (2,048 games, seed 8241001) on master 42ec5dac4: the core engine changed since the 5f786a22a census (resources, validation, mapgen, view). Uploads only the .md report (the full JSON is 800 MB) | done rc=0 in 8 min (155 s of games): 2,048/2,048 finished, 0 panics, avg 300.1 turns, all VP-hold wins. Per-seat win rates (functional record only, no balance reading): Mongols 22.4%, Athens 18.7%, Byzantium 18.0%, Sparta 12.6%, Japan 9.1%, Switzerland 8.4%, Germany 6.7%, Celts 4.1% |
| 2026-10-25 16:2x | `ops-census8-master-86c627fb2-1025b` | Eight-empire Hard census (2,048 games, Medium, seed 8241001) on master 86c627fb2: 104 crate commits since the 42ec5dac4 census, and the climb is idle until the director returns. Uploads only the .md report. (A first submit, `-1025`, was cancelled: it passed `--out`, the replay directory, not `--out-dir`) | done rc=0 in 2 min: 2,048/2,048 finished, 0 panics, avg 298.3 turns, all VP-hold wins (hold break rate 47.7%). Win share (functional record only): Mongols 24.5%, Athens 19.6%, Byzantium 17.6%, Sparta 10.7%, Switzerland 9.8%, Japan 8.9%, Germany 5.8%, Celts 3.2% |
| 2026-10-25 16:2x | `ops-hvh-master-86c627fb2-1025` | Human-seat benchmark (256 salts from 900001, `LEGATUM_HVH_RESEED_DICE=1`) on master 86c627fb2, to compare with 0a297a5e3's 0.441 | done rc=0 in 42 min: overall score 0.344 (95% CI 0.327–0.363), milestone ratio 0.37, vs 0a297a5e3's 0.441 (0.425–0.456) and 0.50. The CIs don't overlap: master dropped about as far as the r1 bank branch did (0.308), so banking the r1 winners looks like the cause |
| 2026-10-25 22:3x | `ops-census8-land8911-4791d2598-1025` | Eight-empire Hard census (2,048 games, Medium, seed 8241001) on the 8.9.11 release branch `claude/land-8911` 4791d2598 (15 crate commits ahead of master, incl. the Cavalry Outrider rule): a panic and hold-telemetry check before it lands. Master's engine is unchanged since 86c627fb2, so no master rerun | done rc=0 in 8 min (159 s of games): 2,048/2,048 finished, 0 panics, avg 279.3 turns (master 86c627fb2: 298.3), all VP-hold wins, hold break rate 43.3% (47.7%). Win share (functional record only): Mongols 24.0%, Athens 17.7%, Byzantium 17.7%, Sparta 16.3%, Switzerland 8.8%, Japan 6.8%, Germany 6.1%, Celts 2.6%. The branch has since landed on master |
| 2026-10-26 04:3x | `ops-hvh-master-a76aa7621-1026` | Human-seat benchmark (256 salts from 900001, `LEGATUM_HVH_RESEED_DICE=1`) on master a76aa7621: 17 crate commits since 86c627fb2's 0.344, including the 8.9.11 landing (Outrider). The climb is idle until the director returns | done rc=0 in 49 min: overall score 0.377 (95% CI 0.359–0.395), milestone ratio 0.37 (0.33–0.40). Up from 86c627fb2's 0.344 (0.327–0.363), the CIs barely overlapping, and still well below 0a297a5e3's 0.441 (0.425–0.456): the 8.9.11 landing recovered part of the drop, not all of it |

## Incidents

- 2026-10-27 10:30Z: easing. r6 Byzantium finished and r6 Germany/Switzerland are now running (Germany at gen 24). The seven r3 league gates are still queued, about 28 h. `astra/filler-fifo-tiebreak` is still unlanded.

- 2026-10-27 04:30Z: still starving. r6 Athens finished, r6 Byzantium now wins every tie, so r6 Germany/Switzerland gen0 and the eight r3 league gates have waited about 22 h. `astra/filler-fifo-tiebreak` is still unlanded; the priority bump was not retried.

- 2026-10-26 16:30Z: the tie-break starvation recurred. With every climb batch at priority 40, r5 Athens and Byzantium gens keep winning, so r5 Germany/Switzerland gen1 (queued 06:41 EDT), all four r6 gen0 (02:04 EDT) and the eight r3 league gates (02:41 EDT) had waited 6 to 10 h. The fix, `astra/filler-fifo-tiebreak` 0ba304e0f (oldest upload first among equal priorities, with a test), finished 06:35 EDT but isn't on master or the box. The ops check's priority bump on the 14 starved specs was refused by the auto-mode permission check, so the queue was left as is.

- 2026-10-26: r5 Germany and Switzerland gen0 (queued 02:00 EDT) and all of r6 had not started after 4.5 h. Two causes: the filler runs one batch at a time and Codex strategy-lab batches at priority 30 held it from about 04:30 EDT; and among the priority-40 climb batches, `pick_pending` breaks ties alphabetically, so the re-queued r5 Athens/Byzantium gens always won (same bug as r4 Sparta on 10-25). The ops check bumped `climb-r5-germany-gen0-1026015852` and `climb-r5-switzerland-gen0-1026020116` to priority 39 and queued Codex task `filler-fifo-tiebreak` for the real fix.

- 2026-10-24: the climb's `climb-r2-celts-gen3` batch reached S3 `pending/` at 05:38Z, about a minute before the idle
  filler daemon exited "queue empty" (05:39Z). Submit's `ensure` saw the daemon still alive, so nothing restarted it
  and r2/Celts sat for 45 min with the box idle. The ops check restarted it with `filler.sh ensure` at 06:23Z.
  Cause: `cmd_run` in `tools/cloud/box/box-filler` exits on idle without a last look at `pending/` after releasing
  its lock. Fix to queue: after unlocking, re-check `pending/` once and re-exec if anything arrived.

## 2026-10-27 16:20Z
- No batches queued: the box is stopped (14:40Z) and can't start. AWS month to date hit the monthly cap, so the budget deny-launch policy blocks `ec2:StartInstances`. The climb daemon had already exited at 12:27Z with nothing left to run (r6 Switzerland finished). Filler resumes when the user raises the cap or on Nov 1.

## 2026-10-27 22:25Z
- No batches queued. The box is back (cap raised, launched 21:22Z) but month-to-date spend is close to the cap, and the AI-war W0.4/W0.5 gate needs the remaining headroom (~$30 batch). Idle filler would spend it, so the box is left to idle-stop. The climb daemon has nothing queued and the director is still disabled.

## 2026-10-28 04:35Z
- No batches queued. Month-to-date spend is close to the monthly cap and the box is busy with the lead's AI-war step-2 gates (verify of `claude/fighters-on`); the remaining headroom is reserved for that work. The climb has nothing queued and its director is still disabled.

## 2026-10-28 10:25Z
- No batches queued. AWS month to date is close to the monthly cap; the remaining headroom is the lead's (AI-war step 2: the door_defense AWS gate awaits the designer). The climb has nothing queued and its director is still disabled.
- Gotcha: `box.sh util` calls `box_up`, so this check's first step started the idle-stopped box (launched 10:21:39Z, nothing to run; idle-watch stops it). `box.sh status` doesn't start it. Ops checks should run `status` first and call `util` only when the box is already running.

## 2026-10-28 16:25Z
- No batches queued. AWS month to date is close to the monthly cap; the remaining headroom is the lead's (door_defense AWS gate awaits the designer). The climb has nothing queued and its director is still disabled.
- The ops check again started the stopped box with `box.sh util` (16:21:24Z). The scheduled task's prompt lists `util` before `status`, so this happens whenever the box is stopped; the prompt's order needs changing.

## 2026-10-28 22:24Z
- No batches queued. Month-to-date spend was last read close to the monthly cap (16:25Z); this run couldn't read the budget (the launcher profile has no budgets:ViewBudget). The remaining headroom is the lead's door_defense gate. The climb has nothing queued and its director is still disabled.
- `box.sh util` started the stopped box a third time today (22:21:28Z). The scheduled prompt still lists `util` before `status`.

## 2026-10-29 04:24Z
- No batches queued. The box is running the lead's 8.9.16 release check and a sim build. Month-to-date spend couldn't be read (the default AWS profile's token is invalid); last known close to the monthly cap. The remaining headroom is the lead's. The climb has nothing queued and its director is still disabled.

## 2026-10-29 10:25Z
- No batches queued. The box is idle; month-to-date spend couldn't be read (the default AWS profile's token is still invalid); last known close to the monthly cap. The climb has nothing queued and its director hasn't run since 10-24. `box.sh util` started the stopped box again (10:21:28Z).

## 2026-10-29 16:24Z
- No batches queued. The box is idle; month-to-date spend couldn't be read (the default AWS profile's token is still invalid); last known close to the monthly cap. The climb has nothing queued and its director hasn't run since 10-24. `box.sh util` started the stopped box again (16:21:28Z).

## 2026-10-29 22:24Z
- No batches queued: AWS month to date is over the cap in the ops prompt (read with the admin profile; the budget is readable again). The climb has nothing queued and its director hasn't run since 10-24. `box.sh util` started the stopped box again (22:21:30Z).

## 2026-10-30 04:24Z
- No batches queued: AWS month to date is over the cap in the ops prompt. The climb has nothing queued and its director hasn't run since 10-24. `box.sh util` started the stopped box again (04:21:30Z).

## 2026-10-30 16:24Z
- No batches queued: AWS month to date is over the cap in the ops prompt. The climb has nothing queued and its director hasn't run since 10-24. `box.sh util` started the stopped box again (16:21:32Z).

## 2026-10-30 22:24Z
- No batches queued: AWS month to date is over the cap in the ops prompt. The climb has nothing queued and its director hasn't run since 10-24. `box.sh util` started the stopped box again (22:21:31Z).

## 2026-11-01 04:2xZ
| 2026-11-01 04:2x | `ops-census8-master-454abdb1a-1101` | Eight-empire Hard census (2,048 games, Medium, seed 8241001) on master 454abdb1a after the train-1101 merge (faction trims, rules cleanup, no-grandfathering, India AI): panic and turn-length check. Nothing from the AI-war step 3 is queued for the filler (the training games are the lead's). November's budget has reset | queued |

## 2026-11-01 11:2xZ
- Finished `ops-census8-master-454abdb1a-1101`: rc=0, 2,048 games in 123 s (plus a 6.5 min build), 0 panics, average 206.8 turns; VP holds 3,866 started, 40.8% broken, 511 comebacks, 0 stalled.
| 2026-11-01 11:2x | `ops-census8-master-391c5eadd-1101b` | Eight-empire Hard census (2,048 games, seed 8241001) on master 391c5eadd after the train-1103 merge (world wonders, objective deck, rules simplifications, Germany and Sparta trims): panic and turn-length check. Nothing from AI-war step 3 is queued for the filler | queued |
- `filler.sh status` starts a stopped box (11:21Z), like `box.sh util` did; the ops prompt allows it only for `box.sh status`.

## 2026-11-01 17:3xZ
- Finished `ops-census8-master-391c5eadd-1101b`: rc=0, 2,048 games in 100 s (465 s wall with build), 0 panics, average 216.0 turns; VP holds 3,577 started, 36.9% broken, 476 comebacks, 0 stalled.
| 2026-11-01 17:3x | `ops-hvh-master-606f08471-1101` | Human-seat benchmark (256 salts from 900001, `LEGATUM_HVH_RESEED_DICE=1`) on master 606f08471: first since a76aa7621's 0.377 on 10-26, after the 10-31/11-01 trains. A census was skipped: the engine is unchanged since 391c5eadd apart from archive tests. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-01 23:3xZ
- Finished `ops-hvh-master-606f08471-1101`: rc=0, 407 s wall on 64 cores; overall_score 0.247 (112 wins), against a76aa7621's 0.377 on 10-26. The engine is unchanged between 606f08471 and master 1ac5c8ef8, so a census was skipped.
| 2026-11-01 23:3x | `ops-hvh-a76aa7621-recheck-1101` | Same human-seat benchmark (256 salts from 900001, reseeded dice) rebuilt at a76aa7621, to tell an AI regression since 10-26 from a change in the benchmark's scenarios or scoring. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-02 05:3xZ
- Finished `ops-hvh-a76aa7621-recheck-1101`: rc=0, 2,496 s wall; overall_score 0.377, the same as 10-26. The benchmark is stable, so master's 0.247 (606f08471) is a real drop in Hard since a76aa7621 (351 first-parent commits).
| 2026-11-02 05:3x | `ops-hvh-bisect-bd2797068-1102` | Human-seat benchmark bisect, quarter 1 (first-parent commit 88 of 351, 10-27) between a76aa7621 (0.377) and 606f08471 (0.247) | queued |
| 2026-11-02 05:3x | `ops-hvh-bisect-b7e99860a-1102` | Same bisect, midpoint (commit 176, 10-30) | queued |
| 2026-11-02 05:3x | `ops-hvh-bisect-3630a3075-1102` | Same bisect, quarter 3 (commit 264, 10-31, after India) | queued |

## 2026-11-02 11:2xZ
- Finished the three bisect probes (all rc=0, ~450 s each): bd2797068 (commit 88, 10-27) 0.169, b7e99860a (176) 0.174, 3630a3075 (264) 0.212, against a76aa7621's 0.377 and 606f08471's 0.247. The drop is already complete by commit 88; later commits partly recover.
| 2026-11-02 11:2x | `ops-hvh-bisect-0b2ff473c-1102b` | Human-seat benchmark bisect inside a76aa7621..bd2797068: first-parent commit 22 of 88 | queued |
| 2026-11-02 11:2x | `ops-hvh-bisect-77e1a44f0-1102b` | Same, commit 44 of 88 | queued |
| 2026-11-02 11:2x | `ops-hvh-bisect-d4bc63868-1102b` | Same, commit 66 of 88 | queued |

## 2026-11-02 17:3xZ
- Finished the three probes inside a76aa7621..bd2797068 (all rc=0, 2,177-2,933 s): 0b2ff473c (commit 22) 0.388, 77e1a44f0 (44) 0.408, d4bc63868 (66) 0.390. The drop is between commits 66 and 88 (10-27). Only two of those merges touch play: 83 `1e481f8c3` (8.9.14: modernization upgrades on) and 87 `faabbc701` (gate-rivals-bomb: the benchmark's rivals now bomb like plain Hard). If 87 is the cause, the drop is a harder benchmark, not a weaker Hard seat.
| 2026-11-02 17:3x | `ops-hvh-bisect-15cf9f6e9-1102c` | Human-seat benchmark bisect: commit 82, just before the 8.9.14 merge | queued |
| 2026-11-02 17:3x | `ops-hvh-bisect-56f4848b7-1102c` | Same, commit 86: after 8.9.14, before gate-rivals-bomb | queued |

## 2026-11-02 23:3xZ
- Finished the last two bisect probes (rc=0): 15cf9f6e9 (commit 82, before 8.9.14) 0.390, 56f4848b7 (commit 86, after 8.9.14, before gate-rivals-bomb) 0.369. Commit 88 was 0.169, and 87 `faabbc701` is the only play change in between, so the drop from 0.377 to 0.247 is the benchmark getting harder (its rivals now bomb, gate re-frozen at 16.2% on purpose), not a weaker Hard seat. 8.9.14 costs at most ~0.02, within noise. Bisect closed.
| 2026-11-02 23:3x | `ops-census8-master-90c37a513-1102` | Eight-empire Hard census (2,048 games, seed 8241001) on master 90c37a513 after train-1102 and the land race merge (engine changed in 126 files since 391c5eadd): panic and turn-length check. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-03 05:3xZ
- Finished `ops-census8-master-90c37a513-1102` (rc=0, 458 s): 2,048 games, 0 panics, mean 192.5 turns, all VP-hold wins, holds broken 37.9%, 490 comebacks, 0 stalled.
| 2026-11-03 05:3x | `ops-census8-heldout-357358c96-1103` | Eight-empire Hard census on held-out seed 9311003, 4,096 games, master 357358c96 (engine unchanged since 90c37a513): per-faction win shares on a second seed for the pillar 8 ratio. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-03 11:2xZ
- Finished `ops-census8-heldout-357358c96-1103` (rc=0, 415 s): 4,096 games on held-out seed 9311003, 0 panics, mean 191.4 turns, all VP-hold wins, holds broken 38.6%, 1,036 comebacks, 0 stalled. Win rate per seat: Athens 20.6%, Sparta 16.8%, Mongols 15.8%, Aztecs 15.2%, Byzantium 12.9%, India 10.4%, Germany 8.0%, Switzerland 6.9%, Celts 5.9% (best/worst 3.5x; functionality check only, no balance conclusions while the AI is still being built).
| 2026-11-03 11:2x | `ops-hvh-master-42ffebc09-1103` | Human-seat benchmark (256 salts from 900001, `LEGATUM_HVH_RESEED_DICE=1`) on master 42ffebc09: first on master since 606f08471 (0.247), after train-1102, the land race merge and train-1103b's declare/raid work. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-03 17:2xZ
- Finished `ops-hvh-master-42ffebc09-1103` (rc=0, 365 s): overall_score 0.213, down from 0.247 at 606f08471 and well under a76aa7621's 0.377. Hard is still below its 10-26 level on master.
| 2026-11-03 17:2x | `ops-hvh-master-4d7792303-1103` | Same human-seat benchmark (256 salts from 900001, reseeded dice) on master 4d7792303, after decisive_buy 1: does master move from 0.213. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-03 23:2xZ
- Finished `ops-hvh-master-4d7792303-1103` (rc=0, 525 s): overall_score 0.239, up from 0.213 at 42ffebc09 (decisive_buy 1), still under 0.247 at 606f08471.
| 2026-11-03 23:2x | `ops-hvh-master-87ee825b8-1103b` | Same human-seat benchmark (256 salts from 900001, reseeded dice) on master 87ee825b8 (Playtest 8.9.25: train-1103b + decisive_buy + found_now). A first `-1103` spec was cancelled before it ran (bad argv). Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-04 05:2xZ
- Finished `ops-hvh-master-87ee825b8-1103b` (rc=0, 462 s): overall_score 0.227 on Playtest 8.9.25 master, down from 0.239 at 4d7792303, still under 0.247 at 606f08471.
| 2026-11-04 05:2x | `ops-hvh-master-47c106278-1104` | Same human-seat benchmark (256 salts from 900001, reseeded dice) on master 47c106278, after contest_hold 100 landed. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-04 11:2xZ
- Finished `ops-hvh-master-47c106278-1104` (rc=0, 506 s): overall_score 0.228 at 47c106278 (contest_hold 100), flat against 0.227 at 87ee825b8 and still under 0.247 at 606f08471.
| 2026-11-04 11:2x | `ops-hvh-master-2256d47eb-1104` | Same human-seat benchmark (256 salts from 900001, reseeded dice) on master 2256d47eb, after land-1104b and the Golden Horde/siege work (59 commits). Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-04 17:3xZ
- Finished `ops-hvh-master-2256d47eb-1104` (rc=0, 527 s): overall_score 0.233 at 2256d47eb (land-1104b, Golden Horde/siege work), up from 0.228 at 47c106278 and still under 0.247 at 606f08471.
| 2026-11-04 17:3x | `ops-census8-master-eeb150f74-1104` | Eight-empire Hard census (2,048 games, seed 8241001) on master eeb150f74: panic and turn-length check, first census since 90c37a513 (11-02) after train-1103b, decisive_buy, contest_hold and land-1104b. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-04 23:2xZ
- Finished `ops-census8-master-eeb150f74-1104` (rc=0, 343 s): 2,048 games, 0 panics, mean 186.9 turns (192.5 at 90c37a513), all VP-hold wins, holds broken 44.0% (37.9%), 571 comebacks, 0 stalled.
| 2026-11-04 23:2x | `ops-census8-heldout-3b984418a-1104` | Eight-empire Hard census on held-out seed 9311003, 4,096 games, master 3b984418a (engine unchanged since 2256d47eb, which already has its benchmark and census): second-seed panic and turn-length check, comparable with 11-03's held-out run. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-05 05:2xZ
- Finished `ops-census8-heldout-3b984418a-1104` (rc=0, 443 s): 4,096 games on held-out seed 9311003, 0 panics, mean 187.1 turns, all VP-hold wins, holds broken 41.8%, 1,107 comebacks, 0 stalled. Matches the main-seed census at eeb150f74 (186.9 turns, 44.0% broken).
- Nothing new queued: the filler already has 4 batches queued or running and the box was 42% filler-busy last hour.

## 2026-11-05 11:2xZ
- The four batches queued at the last check have all finished (filler now 0 queued, 131 finished); the daemon had stopped.
| 2026-11-05 11:2x | `ops-hvh-master-562226396-1105` | Human-seat benchmark (256 salts from 900001, reseeded dice) on master 562226396: first benchmark with all three faction playbooks (Swiss, Celts, Aztecs) on by default, comparable with 0.233 at 2256d47eb. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-05 17:2xZ
- Finished `ops-hvh-master-562226396-1105` (rc=0, 505 s): overall_score 0.236 at 562226396 (Swiss, Celts and Aztec playbooks on), up from 0.233 at 2256d47eb and still under 0.247 at 606f08471.
| 2026-11-05 17:2x | `ops-census8-heldout-562226396-1105` | Eight-empire Hard census on held-out seed 9311003, 4,096 games, master 562226396: panic and turn-length check with the three faction playbooks on, comparable with 187.1 turns / 41.8% holds broken at 3b984418a. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-05 23:2xZ
- Finished `ops-census8-heldout-562226396-1105` (rc=0, 359 s): 4,096 games on held-out seed 9311003 at 562226396, 0 panics, mean 184.6 turns (median 183, 104-311), holds broken 45.4%, 1,149 comebacks, 0 stalled. Close to 187.1 turns / 41.8% broken at 3b984418a.
- Nothing new queued: master is still 562226396 (only ops commits since), the human-seat benchmark and held-out census have both run on it, and AI-war step 3 has nothing queued for the box.

## 2026-11-06 05:3xZ
- No filler batch since the 11-05 held-out census; the 01-02Z filler share (30-40%) came from batches others queued (finished count 132 → 134).
| 2026-11-06 05:3x | `ops-hvh-master-59a49a481-1106` | Human-seat benchmark (256 salts from 900001) on master 59a49a481, after 36 commits including the faction-rule ablation switches and the aircraft/Swiss UI merge; comparable with 0.236 at 562226396. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-06 11:2xZ
- Finished `ops-hvh-master-59a49a481-1106` (rc=0, 571 s): overall_score 0.197 at 59a49a481, down from 0.236 at 562226396 (the 36 commits between include the faction-rule ablation switches and the aircraft/Swiss UI merge). Rerun on current master below to confirm.
| 2026-11-06 11:2x | `ops-hvh-master-dcc3838a2-1106` | Human-seat benchmark (256 salts from 900001) on master dcc3838a2: confirms or clears the 0.236 → 0.197 drop seen at 59a49a481. Nothing from AI-war step 3 is queued for the filler | queued |
| 2026-11-06 11:2x | `ops-census8-master-dcc3838a2-1106` | Eight-empire Hard census (2,048 games, seed 8241001) on master dcc3838a2: panic and turn-length check after 100 commits since 562226396, including the Germany rework and Walls rule | queued |

## 2026-11-06 17:2xZ
- Finished `ops-hvh-master-dcc3838a2-1106` (rc=0, 533 s): overall_score 0.118 at dcc3838a2, down again from 0.197 at 59a49a481 and 0.236 at 562226396. The drop is confirmed and growing.
- Finished `ops-census8-master-dcc3838a2-1106` (rc=0, 272 s): 2,048 games, 0 panics, mean 180.2 turns (186.9 at eeb150f74), holds broken 45.0%, 557 comebacks, 0 stalled.
| 2026-11-06 17:2x | `ops-hvh-master-37d5e3c4d-1106` | Human-seat benchmark (256 salts from 900001) on master 37d5e3c4d, after the Germany rework merge (23 commits, 45 engine files past dcc3838a2): tracks the 0.236 → 0.118 regression. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-06 23:2xZ
- `ops-hvh-master-37d5e3c4d-1106` FAILED (rc=101, 469 s): panicked at hard_vs_human.rs:1689 writing the report, because its spec passed `--out-dir out` without creating `out/`. No score.
- Spec drift found: the 0.233 (2256d47eb) and 0.236 (562226396) runs set `LEGATUM_HVH_RESEED_DICE=1`; the 0.197 (59a49a481) and 0.118 (dcc3838a2) runs did not. So 0.236 → 0.197 compares different dice settings; only 0.197 → 0.118 is like for like. Both variants requeued below on the same commit to separate the two.
| 2026-11-06 23:2x | `ops-hvh-reseed-37d5e3c4d-1106` | Human-seat benchmark, 256 salts from 900001, reseeded dice (the 0.236 baseline's setting), master 37d5e3c4d (no engine change to 4da7de6cf) | queued |
| 2026-11-06 23:2x | `ops-hvh-noreseed-37d5e3c4d-1106` | Same without reseeded dice (the 0.197 / 0.118 runs' setting), so both series continue on one commit | queued |

## 2026-11-07 05:2xZ
- Finished `ops-hvh-reseed-37d5e3c4d-1106` (rc=0, 378 s): overall_score 0.188 (CI 0.172-0.207) with reseeded dice.
- Finished `ops-hvh-noreseed-37d5e3c4d-1106` (rc=0, 166 s): overall_score 0.225 (CI 0.214-0.236) without reseeded dice.
- On one commit the two settings differ by ~0.04, so dice setting matters. Like-for-like (no reseed): 0.197 at 59a49a481, 0.118 at dcc3838a2, 0.225 at 37d5e3c4d. The dcc3838a2 dip recovered after the Germany merge; the reported regression is no longer live.
| 2026-11-07 05:2x | `ops-hvh-noreseed-e6a4cbcb1-1107` | Human-seat benchmark, 256 salts from 900001, no reseed, master e6a4cbcb1 (after the 11-07 Codex backlog landing): continues the like-for-like series. Nothing from AI-war step 3 is queued for the filler | queued |

## 2026-11-07 11:2xZ
- Finished `ops-hvh-noreseed-e6a4cbcb1-1107` (rc=0, 600 s): overall_score 0.326 (CI 0.314-0.336), no reseed. Like-for-like series: 0.197 (59a49a481) → 0.118 (dcc3838a2) → 0.225 (37d5e3c4d) → 0.326 (e6a4cbcb1), a jump of ~0.10 across the 11-07 Codex backlog landing.
| 2026-11-07 11:2x | `ops-hvh-noreseed-92d41edae-1107` | Same benchmark on master 92d41edae (rules-batch-1107, RULES_VERSION 19): checks whether the e6a4cbcb1 jump holds after the rule cuts. Nothing from AI-war step 3 is queued for the filler | queued |
