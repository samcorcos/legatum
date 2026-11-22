# Army modernization save fixtures

Original-engine `PlayerView`s before T142, seats 2 Mongols, 3 Byzantium, 4 Sparta,
5 Japan, plus Japan's T117 view with Hiroshima's active Archer queue, extracted from the copied `staging-ring-chur-20261026.legatum`.
All 142 replayed state hashes match engine `0b2ff473c291f0c53bb82c4e82a8ec4f4c262c85`.
The committed views use the current goal enum so the current AI can read them.
Source SHA-256: `ad4faaa0fd3ef9aff48697c7d3ac056665aa9eeca9d4e2242efc2a15dbe24d9d`.

Generate using `tools/scenarios/extract-state.sh SAVE --out DIR --turns 117,142`,
then gzip `T142.json` `T117-p5.view.json` and the four `T142-pN.view.json` files with deterministic gzip (`mtime=0`).
Do not hand-edit the saved views. Tests explicitly label mutations as counterfactuals.
See `docs/sim-results/ai-quality/army-modernization-1026.md` for full provenance,
resource reservations, Gold, upgrade quotes and production history.
