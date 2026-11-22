# Japan 8.9.11 air-war positions

Extracted on original engine `d1233d48678a2e89630fa456f710db100084a7de`, seed
`3811224394289556`, from a copy of `japan-win-bombers-20261026.legatum`.
All 234 tape hashes and all 221 German planning order lists match. The source
SHA-256 is `f9f81664243336424ddc8c519b1e76fb367b930ef42308b8e42c42cda50188e7`.

`Tn` is the state before resolving that turn. Views are player-fogged. Memory is
from before the original brain planned; `recorded` is its tape order list.
The committed views and states use the current building and goal enums. The source save
and recorded orders remain unchanged.
`T214.orders` contains every player's tape orders. State JSON stores the seed as
a decimal string (the WASM save convention); Rust fixtures parse it as u64.
`default.json` is the zero-new-gene control; `defense.json` is the experimental arm.

Tests use unchanged Germany T185 for Fighter production and T186 for retaining a
Bomber build. T214 Bomber evacuation is unchanged state and recorded opponents.
The interception test explicitly changes Germany's single Bomber to a Fighter;
it does not claim Germany historically had a Fighter or that one reverses the
four-Bomber attack. Athens T197 proves actual offensive use. T178 Germany and
T198 Athens are diagnostic control positions. No human identity or live save is
included.

Rebuild with `tools/scenarios/extract-state.sh --audit-plans`, as documented in
`docs/sim-results/ai-quality/japan-air-war-1026.md`. Gzip files use mtime 0.
