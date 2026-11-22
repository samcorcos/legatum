# T58 scout reconnaissance regression

Source: `~/LegatumData/archive/2026/09/native-1790566309101610000-51980-10.tape.json.zst`.
Incidents: I000064, I000066, I000068, I000420, I000063, I000065.

`scout-t58.json.gz` contains the exact start-of-T58 state and T58–64 simultaneous
orders. It was extracted after native replay with the archive's engine revision
`78d81d1911c31581c12d1ffca7b67d6c07377501` reproduced all 258 post-turn hashes
and the first 59 complete generated order maps. The archive's rules fingerprint
is `22e888da3d4e307ee9dcad30305453fb4a4a34949422e1303e5bc116e2510c19`;
the start-of-T58 state hash is `4965912538210507113`.

Extraction used `git archive` of that revision's Rust workspace into a temporary
directory, `new_game` with the tape setup, and `resolve_turn` with each tape order
map. For T1–59, `ScriptedBrain::with_difficulty` was recreated with Normal
difficulty and `Rng::derive(setup.seed, &[purpose::AI, player_id]).next_u64()`;
each generated map matched the tape. The snapshot precedes T58 resolution.

Before this fix, revision `0c7c032ca1a90e1775bcff264be7d435fc79a50b` reproduced
scout 23's retreat from (22,18) to (21,18), return to (22,18) at T59, and repeated
reversals through T63. Soldier 21 at (24,18) was hidden on each return decision,
but its previous sighting remained in PlayerView. Scout 33 likewise retreated
from (5,25) to (6,25), then returned toward soldier 12 at (3,26).

The regression runs the current AI and resolver, recreates brains each turn,
round-trips saves, and replans each scout independently while retaining all
other archived orders. It asserts safety for every resolved movement step.
New city save fields take their normal serde defaults; the initial provenance
hash excludes those empty defaults. Later states are counterfactuals, not claims
of current-engine compatibility with the archive's historical hashes. The AI
receives only `player_view`, never the full snapshot.
