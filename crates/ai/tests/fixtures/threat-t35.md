# Runaway-conqueror regression (playtest 6)

`threat-t35.json.gz` is the complete, unmodified state before turn 35 of the user's
playtest-6 game: a human Mongols on Hard who rushed and eliminated the Byzantium (turn 40)
and the Athens (turn 50) while every other AI stayed at Peace.
`threat-t35-orders.json` holds the archive's recorded orders for turns 35 through 52
(`[{turn, orders}]`, all eight seats), trimmed from the full history.

Source: a copy of the user's autosave taken at turn 53 (`autosave-t52.legatum`, format
`legatum-save` v1, client `wasm`, saved 2026-10-03T19:40Z). Its `game` field is a JSON
string holding an archive with complete history (`metadata.complete_history: true`):
game id `web-1791054439310-56e37f48728f8000-d5047d6a3768c000`, seed `191528788803819`,
Standard speed, 48x32, barbarians on, engine commit
`a05bf36de6ce15c5bfa72bc6231900b6d3af84eb` (playtest-6), rules fingerprint
`724fdc05f876eb0b525809757c2c5a6aa37b2149628ff6a0c836cabaa67bb09a`. Seats: P0 human
Mongols; P1 Celts, P2 Mongols, P3 Sparta, P4 Athens, P5 Switzerland,
P6 Japan, P7 Byzantium, all Hard. The original saves in `~/Library` were never touched.

Extraction: the archive's `tape` (version 4) was written to JSON, and
`cargo run --release -p legatum-sim --example extract_state -- tape.json 35 threat-t35.json threat-t35-orders.json 18`
replayed the recorded orders from the setup with the current engine, then
`gzip -9n threat-t35.json`.

Hash convention: the playtest-6 engine serialized `Player.byzantium_salvage_science` even at
zero; the current engine skips zero (6a5f3e5f), so plain `GameState::hash` values differ
from the archive. The tests re-insert the zero field for each player that lacks it
(`archived_hash`) and then match the archive's recorded hash after every turn from 34 to
52, including the initial state of this fixture (`447510247064851724`). The same
re-insertion also matches the archive's initial hash and turn-1 hash when replaying from
the setup, so the replay is the user's game, not an approximation.

The current scripted brain (master `1cb70f05`, AI unchanged since playtest-6 apart from
serialization) reproduces the archived AI orders on every turn except three (turn 22 P2,
turns 42 and 48 P1), checked with `examples/playtest_ai_trace.rs`; the tests use recorded
orders, not regenerated ones, for everything before the turn they examine.

The archived outcome is historical. Under the 2026-10-09 Mongols siege penalty, replaying
the same orders from turn 35 leaves every empire alive through turn 52 and the human with one
city. The current tests check that direct victims recognize the raider, distant empires do not
mistake the failed assault for a runaway conquest, exposed cities prepare defenders, nobody
signs an Alliance with the raider during the resumed turn-45 play, and moving a fogged human
unit leaves Japan's view, orders and threat model unchanged. Later states are
counterfactuals.

Since master 91977180 (cities shield their garrisons) the recorded orders resolve differently
from turn 35, so only the fixture's own hash (turn 34) is checked against the archive; later
archived hashes are history. The threat tests drive brains with the recorded orders as before.
With the current rules and stable unit identities, Byzantium falls, Athens survives, and the
human holds four cities after turn 52. The watcher brains still name the human as their top
threat on the recorded turns.

`threat-t35-unit-refs.json` records the first observed owner, kind and position for each
unit named in those orders. It was extracted by replaying this save and the unmodified
order tape under archive engine `a05bf36`. Current-rule replays bind each archived unit
to a live unit through that identity before translating later orders; an order for a
unit that has not been produced under current rules cannot be issued.
