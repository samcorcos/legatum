# Save compatibility

`original-three-save.json` was generated with the unmodified parent commit
`d1e0c11007b203c6f6885e1460b62767cca46dad`: seed 3, the original three
factions on a 12x10 grassland map, one founded city and Spearman per faction.
It pins the pre-eight-faction save schema; never regenerate it with new rules.
Tests check its faction names. The matching
tape-loading test lives in `crates/wasm/tests/legacy_factions.rs`.
Historical tape hashes are expected to differ under the new rules.

## Territory regression save

`sparta-colony11-t186.json` is the serialized game state from
`LegatumRuns/bug-saves/unworked-claim-colony11-20261025-0845.legatum` (turn 186).
The outer save wrapper, replay archive, and AI opponent memory were removed;
the setup seed's decimal string became its equivalent JSON number for direct
core deserialization. Cities, tiles, and ownership are unchanged. Removed score fields were dropped.
The source save was read only.
