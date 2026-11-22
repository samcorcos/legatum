# Decision-audit continuation challengers

These are fixed sensitivity policies, not AI defaults or proposed game changes. Both use only
existing `AiParams` decision genes in `crates/ai/src/params.rs`; unspecified genes and faction
personalities keep their built-in values. The audit validates the schema and bounds on load.

- `economic.json` weights economic research and public economy gaps highly, cuts military
  production and research to their legal minima, keeps the full purchase reserve, and turns off
  campaign commitment, persistence, siege and focus-fire premiums. It still defends itself through
  the unmodified Hard rules.
- `conquest.json` maximizes campaign commitment, military production and research, nearby city
  value and early-strike eagerness. It lowers the local-force threshold for siege and campaign
  persistence, raises focus-fire value, and removes the routine purchase hold. It still uses the
  unmodified Hard legality, threat and battle planners.

Use `--continue hard,params:audit-in/economic.json,params:audit-in/conquest.json` with both files
shipped through `run.sh --ship`. Compare the same branch arms and salts across all three. The
`--memory-reset` option adds a fourth, cold Hard row.

Validation on 2026-10-31: the repository's smallest tape,
`crates/wasm/tests/fixtures/original-three-tape.json`, ran with two salts and horizons 2 and 5
under Hard, both challengers and cold Hard. All 48 branch runs completed without a panic, reached
turn 6, and had zero invalid continuation orders in each policy's 12 rows. That old one-turn tape
has one replay hash mismatch on today's engine, so this check establishes legality and progress,
not decision-value fidelity. Use a matching engine and zero-mismatch tape for substantive rankings.
