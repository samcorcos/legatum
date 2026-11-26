/**
 * The faction and difficulty names a seat may carry. A record naming anything else did not come from the game.
 * Keep in step with `Faction::ALL` and `AiDifficulty` in crates/core: the core test
 * `upload_worker_knows_every_faction` reads this file and fails when a faction is added or renamed.
 */
export const FACTIONS: readonly string[] = [
  "Switzerland",
  "India",
  "Byzantium",
  "Germany",
  "Celts",
  "Mongols",
  "Sparta",
  "Athens",
  "Aztecs",
  "Egypt",
  "Rome",
  "Kyivan Rus'",
];

export const DIFFICULTIES: readonly string[] = ["Easy", "Normal", "Hard"];
