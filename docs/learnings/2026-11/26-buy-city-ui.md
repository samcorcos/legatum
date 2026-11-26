# Buy a city in the app: core text that reaches the player (2026-11-26)

- `buy_city` refusals and `CityOfferEnded.reason` name players by their seat name ("You are at War with AI 2",
  "War between AI 1 and AI 2"). The app says empires, so every surface runs them through `buyCity.ts::renameSeats`
  (the report through `ReportContext.renameSeats`). Any new core reason string that names a player needs the same.
- The price breakdown labels are core strings, so the copy lint (`ui/copySource.test.ts`) reads them in
  `crates/core`: "Base price" and "armed unit" failed it. They are now "Starting price", "military unit" and
  "Population and buildings when the offer was made".
- "Gold held" is banned by the copy lint ("held" for Gold is bookkeeping): an escrow is "your 600 Gold offer" or
  "Byzantium offered 600 Gold".
- `city_buys` carries, for a Byzantine viewer, every known rival city with the buyer's own refusal ("does not touch
  your land", "is a capital"), but for any other viewer only refusal-free entries plus its own bordering cities with a
  refusal written from the buyer's side ("You are at War with ..."). Show a non-buyer only the refusal-free ones,
  or the line says "you" to the wrong empire.
- `rules.generated.json` is written by `serde_json::to_string_pretty` from a sorted map: a hand-added nested object
  must have sorted keys too, or `UPDATE_UI_RULES` reorders it in the next refresh.
- A running offer's `turns_left` counts the current turn: an offer made at the end of turn 11 reads "3 turns" on
  turn 12 and changes hands at the end of turn 14.
