# Cleared settler destination regression

Source: `~/LegatumData/archive/2026/09/native-1790590960323252000-54022-13.tape.json.zst`.
Incidents I000049 and I000803 describe the same P4 Settler47 episode.

The snapshots and simultaneous orders in `settler-cleared-goto.json.gz` were
extracted at branch base `ff48dcc`, whose core, AI and WASM source trees match
archive engine `6928b348c6a61fc865539b3df076c31fee78c1e4`. The original rules
fingerprint is `ebd462fb4851552d2f1a9ef10866da426485879f9bccb07a3b632dce1f077ed6`.
The native archive verifier matched the initial hash and all 265 resolved hashes.
An independent Normal AI run also matched every archived seat's generated orders
and every resolved hash, using the standard per-player derived brain seed.

Unmodified snapshots (state immediately before resolving the named turn):

| Window | Seat / settler | Initial state hash |
| --- | --- | --- |
| T33–36 | P6 / 29, produced T33, founded T36 | 17992056441836798299 |
| T116–128 | P4 / 47, produced T115 | 2411881861751387837 |
| T127–130 | P5 / 49, produced T127, founded T130 | 9419487988869790429 |

After merging master `14884f3`, the unfixed current AI and resolver still repeat
P4's MoveUnit to (8,12) on all 13 turns, leaving Settler47 at (16,9). Every turn
emits `MoveBlocked: no route to the destination`, clears goto and leaves full
movement. The old rule fingerprint differs from current master, so current-rule
continuations use these verified snapshots rather than bypassing compatibility.

The tests preserve the archived order-only control. Policy counterfactuals replace
the selected seat's orders with real current planning and keep other seats'
simultaneous archived orders. Their later states are counterfactuals, not archived
hash reproduction claims. The successful windows document the original archive. Their exact-order and
founding-turn assertion was removed when G8 pacing and bundle D became defaults:
re-resolving the old simultaneous orders now changes production and spawning,
so those old turn numbers no longer describe current-rule continuations. The
blocked window still checks legal progress through T128, including save/load
and brain reconstruction on every turn.

The minimal corridor test remembers an open choke that a peaceful neighbour now
owns outside sight. Its rich distant site is blocked; a modest local branch is
legally reachable. The test must actually found on that branch within 16 recovery
turns, including rebuilding the brain after each turn. Changing only hidden
ownership must not change PlayerView or either initial/recovery decision.

Recovery uses the existing `MoveUnitPath` order when full movement and a missing
goto suggest an unsuccessful move through remembered territory. The explicit
route preserves the settlement goal while resolution checks individual steps;
the distant route can no longer prevent safe local reconnaissance. A retained
confirmed path keeps this behavior until fresh route information is available.
No new brain memory, engine observation, scoring rule or settler cap is needed.
Reservations simulate these confirmed steps rather than an invented detour.

Before the fix, the current-rules control, successful windows and hidden-world
invariance passed; both actual recovery regressions failed. After the fix all
five pass, including reconstructing a brain from the saved state every turn.
