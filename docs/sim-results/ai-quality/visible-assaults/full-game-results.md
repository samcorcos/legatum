# Full-game paired invasion-defense measurement

Work in progress for issue #366. Predeclared comparison: 300 paired public seeds 7101–7400, six-player duel mode, only the measured seat's `invasion_defense` differs (0 versus 1), shipped Hard opponents. Seeds 7101–7102 previously appeared in the T180 pipeline smoke; that reuse is disclosed, not treated as held-out validation. The runner safety cap is explicitly 700 turns; the game rules normally end at turn 500. Unfinished games and right-censored assaults will be reported separately.

Frozen source: `1f7db8eed96d513a7b1acc70b8ad72c27458a398`. The existing release runner built at `b3dfd226e49149b31700b9926fa23feb49954c6f` has identical `crates/`, Cargo manifests/lockfile and toolchain inputs at this source; reuse avoids an unnecessary rebuild. The executable is copied into a private target directory for this task.

Report paired differences and bootstrap uncertainty by seed, not individual assault. A confidence interval overlapping zero does not establish nonregression. If effectiveness is unclear, keep #366 open with an explicit next step; do not tune the policy to fit this seed block. Preserve compact per-game outcomes and cohort records. This is measurement only; no behavior or acceptance claim yet.
