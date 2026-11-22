# Legatum rules reference (code-verified): index

Facts below are cited to file (and usually function), read 2026-10-02 and audited against `master` on 2026-11-02 (city strike and city-shields-garrison and Celts concealment updated 2026-10-03; Influence removed and diplomacy cut to Peace, Alliance or War on 2026-10-03; Byzantium salvage updated 2026-10-04). Purpose: stop
recommendations that assume Civ 6 defaults this game doesn't have. If a fact isn't in the cited
file, say so rather than guessing.

**Rules version: 50** (`crates/core/src/lib.rs::RULES_VERSION`, checked 2026-11-22). Every save carries `save_version = RULES_VERSION`; a save or tape from any other version is refused, and there are no migrations. The version log and old compatibility notes are in `history.md`.

The reference was one file, `game-rules.md`, until 2026-11-22. It is now one file per system, each holding its original text verbatim and in its original order. Open only the files your question needs, plus `not-in-game.md` before proposing anything.

| File | Open it when you need to know |
| --- | --- |
| `not-in-game.md` | Which Civ 6 systems do not exist here (always read before proposing a mechanic) |
| `turn-and-orders.md` | What orders exist; one order per unit per turn; Sleep, Skip and Heal; auto-explore; the resolution phases; automatic garrison fire; order UI; co-op Gold gifts to a teammate (`GiveGold`, rules 50) |
| `movement.md` | Movement rules, multi-turn goto, meeting engagements, roads and Roadworks, passing through your own units, route previews |
| `setup.md` | New-game faction choice, AI levels (Easy, Normal, Hard, Training), the calendar, what every empire starts with and what is legal on turn 1 |
| `research-and-eras.md` | The nine eras, the tech tree and its gates, research costs, game length and pacing |
| `units.md` | Unit stats and costs, the land triangle, Outriders, indirect fire and built-in Siege, Future-era units, one unit per class per era |
| `air.md` | Aircraft, strikes, recon, rebase, hangars, Air Defence, aircraft promotion, aircraft sight |
| `wonders.md` | The world wonder race, wonder tiles and held wonder VP, natural wonders |
| `cities-and-production.md` | City and Settler rules (Settler needs population 2), population is territory, borders, city automation, banked production, hidden faction-useless items, tile improvements |
| `economy-and-purchases.md` | Upkeep, buying items (`BuyItem`) and tiles (`BuyTile`), treasury bounds, resources, strategic supply; the `no_purchase_under_threat` experiment |
| `combat.md` | Damage formula, melee, pursuit, split counterattacks, ranged, Walls and Siege, city shield, city strike, capture, the Celtic sack, Greece's captives, original-capital VP (held, rules 50), Raze, the occupation lock, fortify and heal, promotions |
| `diplomacy.md` | Peace, Alliance or War; Alliance cost, the upkeep waiver (rules 50) and calls to arms; offers; leaving; first contact; surprise wars; coalitions; diplomacy AI |
| `victory-and-scoring.md` | VP and the VP target, winning by holding the target, winning by capitals, World Ages (the only era scoring since rules 50), the near-victory warning, decadence (off), the game history record |
| `map-and-exploration.md` | Balanced founding sites, ruins, vision and sight sharing, revealing attackers |
| `barbarians.md` | Barbarian camps and raids, Wild camps, barbarians and the Celts, yield denial |
| `factions.md` | Every faction's rules and goals, special units, Mercenary Companies |
| `experiments.md` | Simulator-only rule experiments (research cost, faction-rule ablation); World Ages and decadence experiments are in `victory-and-scoring.md`, `walls-cost` is in `units.md` |
| `ai-hard-genes.md` | AI-only genes and decision rules with their defaults (Hard defence, purchases, campaigns, Settlers, Peace, coalitions, faction playbooks) |
| `ai-architecture.md` | How the AI is built: views, ScriptedBrain, difficulty, openings, threat response, conquest, proxies, victory goals |
| `coach-questions.md` | The app-side Coach questions option (no rules change) |
| `history.md` | Save compatibility history and the rules-version log |

When a rule changes, edit the system file that holds it (search the folder with grep if unsure), keep citations as `file::function`, and add a line to `history.md` only for version bumps or compatibility notes.
