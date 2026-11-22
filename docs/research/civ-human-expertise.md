# Mining human Civ expertise for the Legatum AI

Researched 2026-10-07. Phone summary: `civ-human-expertise.png`.

**Sources.** The human-strategy evidence comes from the CPL founder's guide, a popular Steam multiplayer
guide, Comrade Kaine and Gaming StackExchange. The AI-code evidence was read from public GitHub
repositories. CivFanatics and Reddit were not sampled in this round.

Our rules limits (from `.claude/skills/legatum-rules/references/game-rules.md` §9) are respected below: no
builders, districts, religion, spies, Great People, pillaging, naval units, trade routes, promotion
choices or tech trading. Any Civ 6 tactic that depends on them is listed under "Does not transfer".

Much of this is already in the AI (§10: opponent assessment, `ThreatTracker`, escort and snipe-settlers
behaviours, siege forecasts, bought Walls). Each item below says what is new or worth re-checking.

## The 10 most transferable behaviours for our AI

1. **Scout wide and early, one scout per bordering rival.**
   Source: CreaM Steam guide (two scouts by about turn 30, then one per neighbour; scouting is the basis
   of offence and defence); CPL guide (most players open scout, scout); Comrade Kaine (scout first on a
   decent landmass).
   Legatum: Scout costs 30, 3 moves, sight 2 (§2, §7); `plan_scouts` exists. Re-check the trigger for the
   second scout and for a standing scout per neighbour, and keep scouts alive (retreat logic exists).

2. **Read the rush from public numbers before the border is crossed.**
   Source: CreaM (a public military figure above about 150 in the first 30 turns means be careful; under
   about 60 marks a weak target); CPL (watch what a rival is teching and which strategic resources it
   produces; a cavalry-class unit near your border without your own counter means you are dead).
   Legatum: the public Strongest Army title and sighting deltas feed `OpponentAssessment`. Today walls and
   shooters wait for observed aggression (2+ Soldiers at the border). Add a pre-border trigger on a rival
   within 10 hexes whose public army share grows faster than ours before turn ~40, as a tunable gene.

3. **Hold a fortified line with ranged units behind it; sally only when it kills.**
   Source: CPL (fortify, shoot the same tile with several ranged units, finish kills with ranged to keep
   formation; do not attack out unless the kill is safe; a classical unit in the city beats Walls alone,
   both is best); CreaM (the same: fortified units are very strong, rivers and hills help).
   Legatum: ranged has no retaliation, range 1-2; fortify exists; city strength is 10 + 2 x pop + Walls,
   and the garrison is a shielded second layer (§4). Audit that defenders stay fortified and that ranged
   units stand where two or more cover the likely attack hex, with attacks gated by `forecast_attack_mode`.

4. **Count trades in production, not just hit points.**
   Source: CPL (a lost unit is wasted production, make the opponent waste theirs); Freeciv README.AI
   (military want = battle profit x amortisation, where profit = enemy shields lost x P(win) minus own
   shields lost x P(lose) minus upkeep x duration; `ai/default/daimilitary.c`, GPL-2.0).
   Legatum: units have production costs (Soldier 40, Ranger 50). Check that attack and sally forecasts
   weigh expected exchange in production plus upkeep, not only capture probability; also Freeciv's trick of
   building a defender so a stuck attacker can go back on offence.

5. **Gate war on advantage, brevity and number of fronts.**
   Source: CreaM (ask how many turns the war will take; if more than about 20 turns or no clear edge,
   pick another target or none; quick brutal first strike; win an early war, snowball, then repeat);
   Unciv `MotivationToAttackAutomation.kt` (MPL-2.0: -20 per concurrent war, -10/-8/-3 for target cities
   beyond 20/14/10 hexes, +10 for an isolated target city, relative score and force ratios, "no other
   threats" +10); Unciv threat bands (force ratio above 2, 1.5, below 0.667, below 0.5).
   Legatum: has Peace, Alliance and War only, 8 empires, and a conquest nomination gate
   (`assessed_conquest_allowed`). Add or verify an explicit concurrent-war penalty and an estimated
   turns-to-first-capture gate (a human rule of thumb of 20 turns, to be rescaled to our game length).

6. **Pick the target by distance to our capital, cluster, and stage a simultaneous strike.**
   Source: Unciv `HeadTowardsEnemyCityAutomation.kt` (target the enemy city nearest our capital, then
   cities within 10 hexes of it; ranged units skip a city at 1 HP because capture is for melee);
   CreaM (gather units two tiles outside the border, then attack with most at once; surround with about
   three units; bring siege against walls, escort it, and buy siege in a captured city; do not let
   siege units stand alone).
   Legatum: turns resolve simultaneously, so an all-at-once strike is the native model. City sight is 3
   (+1 on Hills), unit sight 2: stage just outside city sight so the target's sighting deltas show no
   buildup. Siege Engines (+12 vs cities) arrive late, so before them the AI should cluster 3+ units
   and use ranged first. `city_assault.rs` and `opportunity.rs` cover much of this; re-check staging.

7. **Estimate arrival time for danger, and garrison edge and damaged cities first.**
   Source: Freeciv `assess_danger` (every enemy unit is path-found to the city; danger is its attack
   rating, and urgency depends on arrival turns); Unciv `isCityThatNeedsDefendingInWartime`
   (a city needs a defender if damaged or if an enemy city is within 5 hexes); Gaming StackExchange
   (barbarians send scouts, and camps only spawn raiders after the scout reports back, so kill or follow
   the scout).
   Legatum: `ThreatTracker` scores rivals, but buying or building a defender could key on enemy ETA versus
   our build and buy time (Freeciv's urgency), with a cheap Scout or Ranger picket on a Hills hex 3-4 hexes
   out to buy a turn of warning. Verify whether barbarian camps in `barbarians.rs` behave like Civ 6's.

8. **Score settler sites with ring distance, terrain, and a "no death march" radius; settle on turn 1-2.**
   Source: Unciv `CityLocationTileRanker.kt` (a new city 4-7 tiles from others scores up to +8 and 3 or
   closer is -25 to -30, with the capital's ring counted double; Hills +14, river +20, coast +3, unexplored
   tiles valued as the average of known ones; search radius shrinks to 1-5 as the settler gets 8 or more
   tiles from home); CPL (settle turn 1 or 2, every turn waited is a turn behind); CreaM (early
   development matters more than perfect late tiles); CPL/CreaM (escort the settler with a scout).
   Legatum: Settler needs population 2, costs 75 + 20 per trained, no starting city (§2); `civilians.rs`
   has site scoring and escorts. Cross-check the weights (hill and river bonus, ring spacing 4-6, edge
   penalty) and the unexplored-tile prior against our own sim winners.

9. **Choose the opening from start geography and neighbour distance, not only from faction.**
   Source: CreaM (two Horse tiles within reach of the capital on open ground: go straight for a rush; a
   barbarian camp or city-state in sight: second warrior instead of a builder; faction swaps and strategy
   follow where you spawn); Comrade Kaine (Slinger-first only if a threat is nearby; the scout, slinger
   combo lures and clears a barbarian camp); CPL (swap the war plan for a faith plan on a faith civ).
   Legatum: Pillar 10 wants several viable openings and per-faction asymmetry. Opening brains exist
   (`openings.rs`): feed them start-time facts the scripted AI already sees (rival distance within 10
   hexes, Iron or Coal visible after Fabrication, a barbarian camp in sight), not just the faction prior.

10. **Close out a lead by defending the hold, scouting everyone, and trusting nobody late.**
    Source: CreaM (the leader should refresh intel on the last rival, and the player ahead around turn 70
    usually wins; when behind, scouts are your defence and you bleed weak neighbours); CPL (a 20-turn
    friendship is a cover for an attack force, so watch friends' stats near its end).
    Legatum: a team must stay at the VP target for 10 turns at Standard (5 at Quick); every AI already
    ranks holders as the top threat (§6). Add leader-side behaviour: keep garrison and title margins
    (titles are strict-highest, incumbent keeps ties) and scout every rival while holding, and treat a
    treaty partner's army growth near the end as a threat. The trailing side should strike the cheapest
    title (Strongest Army) or a recently captured city, as the existing title planner already hints.

### Does not transfer (needs systems we do not have)

Great General and Great Person races, encampment projects, Defender of the Faith and other religion
tools, pillage-to-fund (one CreaM guide values a single pillage at about 300 gold), builder-based tile
improvement timing, trade routes, city-state envoys and annexation, spy and air-bridge tactics, last-move
or first-move timing (our turns resolve together, with no peer-to-peer ordering), and Casus Belli and
war-weariness numbers.

## Data sources

| Source | What it contains | Format and licence | Size | Usable for learning or validating? |
|---|---|---|---|---|
| CPL (cpl.gg, Discord league) | Ranked Civ 6 multiplayer games and rules. No public game-log or save corpus found. | Binary `.Civ6Save` files circulate privately in the league | Unknown | No public data. Could ask organisers for consented results. Civ 6 differs from Legatum, so high effort, low payoff. |
| Civ 6 save parsers: `github.com/pydt/civ6-save-parser` (Play Your Damn Turn), `github.com/selimbucher/civ6.ch` (Swiss league uses it for Glicko-2 ratings) | Tools to parse Civ 6 saves, not data | MIT | Small | Only if we obtain saves. Gives per-turn state to mine build orders. Not a dataset. |
| HF `yinhuankuang/rl-game-traces-civilization-6` (huggingface.co/datasets/yinhuankuang/rl-game-traces-civilization-6) | One player's Civ 6 sessions: mkv screen video, mouse and keyboard event logs (jsonl, parquet), screenshots | Licence "other"; Chinese labels | 413 files, 363.6 GB | No. Pixels plus inputs, no game state, unknown skill, unclear rights. |
| CivRealm (arXiv 2401.10568; `github.com/bigai-ai/civrealm`, GPL-3.0) | A Freeciv-web gym environment, not human data: 10 mini-game types, 10,000 generated instances each, tensor and LLM baselines | GPL-3.0 code, docker server | Environment only | Not for human behaviour. Useful as evidence: the PPO baseline trained 2M full-game steps and chased unit production, while the paper notes growth needs more cities. LLM agents (Mastaba) expanded better but failed to defend against invasions. |
| Freeciv and Longturn (`github.com/longturn/games`, freeciv GPL-2.0) | Longturn runs long human games. Their repo holds settings, rulesets and scripts. Freeciv servers can write scorelogs and savegames. No public archive of finished human game logs was reachable (longturn.net is bot-walled). | Text savegames, scorelog | Unknown | Ask Longturn admins. Rules differ (ships, ocean, slow turns), so use only for macro curves (cities by turn, score over time). |
| Unciv (`github.com/yairm210/Unciv`, MPL-2.0, about 11k stars) | An open Civ 5 style game with a readable AI; multiplayer saves are JSON files. No public corpus of human games found. | Kotlin code, JSON saves | Code only | Study the AI (below). Human data only if we recruit players. |
| Freeciv AI (`github.com/freeciv/freeciv`, `ai/default/*.c`, `doc/README.AI`) | Mature military, settler, danger and diplomacy heuristics; `daimilitary.c` is 2,072 lines, `daisettler.c` 1,402 | GPL-2.0 | n/a | Algorithms to port. Copy ideas, not code, to stay clear of GPL. |
| FreeCol (`github.com/FreeCol/freecol`, GPL-2.0, Java) | AI split into `AIPlayer`, `ColonyPlan`, `mission/`, `Wish` | GPL-2.0 | n/a | Mission and wish architecture (a goal queue that cities and units fill). Low priority; colonial game. |
| C-evo, OpenCiv | No repository found at the guessed GitHub paths (404). Not evaluated. | Unknown | n/a | Skipped. |
| StarCraft II replays (AlphaStar) | Blizzard's anonymised human replays, the supervised stage's input (Nature data-availability note points to a public GitHub) | Blizzard replay licence | Large | Not our game. Shows what a replay corpus looks like. |
| **Our own archive** (`docs/game-archive.md`: `~/LegatumData/archive/*.tape.json.zst`, plus playtest tapes with the two humans) | Full deterministic tapes of sims and real playtests | Ours | 605 sim-result files in `docs/sim-results/` | Yes. The only source with the right rules. Human playtest tapes are rare and valuable: treat them as the validation set for any behaviour above. |

## Open-source AI heuristics that transfer

| Heuristic | Source file | Maps to |
|---|---|---|
| War motivation as a sum of modifiers (relative strength, concurrent wars, distance, isolation, score ratio, allies) | Unciv `MotivationToAttackAutomation.kt`, `DeclareWarPlanEvaluator.kt` | `conquest.rs`, `diplomacy.rs` scoring; add concurrent-war and distance terms |
| Gang up only if combined force is needed (penalty when we would win alone or lose even together) | Unciv `DeclareWarPlanEvaluator.kt` | Coalition scoring (`coalition::select`) |
| Threat level from force ratio bands | Unciv `Automation.kt::threatAssessment` | `ThreatTracker` thresholds (30 watched, 60 top threat) |
| Nearest-to-capital target, then cluster within 10 | Unciv `HeadTowardsEnemyCityAutomation.kt` | `campaign.rs`, `city_assault.rs` |
| Garrison damaged or edge cities first in wartime | Unciv `UnitAutomation.kt::tryGarrisoningLandUnit` | `hard_defense.rs`, `military.rs` |
| Settler site ranking: ring distance, hills, river, coast, unexplored prior, shrinking search radius | Unciv `CityLocationTileRanker.kt` | `civilians.rs` site scoring |
| Arrival-time danger and urgency; expected-loss defender choice | Freeciv `daimilitary.c::assess_danger`, `dai_choose_defender_versus` | `purchase_defense.rs`, `production.rs::choose_item` |
| Military want = battle profit x time discount | Freeciv `README.AI` | Attack and buy forecasts |
| Wish and mission queues | FreeCol `server/ai/` | Optional refactor, not needed now |

## AlphaStar at our scale: verdict

**No, not as such.** What was verified:

- AlphaStar began from supervised learning on Blizzard's anonymised human replays, then ran a league. The
  January 2019 version's league ran 14 days on 16 TPUs per agent and each agent saw up to 200 years of
  play (DeepMind blog). The Nature version trained longer (recalled, not re-checked: about 1M replays and
  tens of days on a large TPU pool).
- Gym-uRTS (arXiv 2105.13807, MIT) got a state-of-the-art bot on a tiny RTS in about 60 hours on one GPU,
  but only single-map and against scripted bots; its own motivation is that full-game systems cost
  thousands of GPUs and CPUs for weeks.
- CivRealm (ICLR 2024) is the closest Civ-like baseline, and its PPO agent and LLM agents both struggle
  on the full game; only mini-games were solved well.
- Lux AI Season 3 had an imitation-learning entry in third place (Kaggle write-up title only), so cloning
  strong bots works at small scale when a good teacher exists.

For Legatum: there is no human replay corpus (a handful of playtests), the horizon is about 300
simultaneous turns with hidden information, and our best "teacher" is our own scripted AI, so cloning is
circular. The cheap analogue is already in place: the evolve and hill-climb ledger is a population of
parametrised scripted agents with exploiters. Spend effort on (a) the behaviours above as new genes,
(b) narrow learned components with checkable labels (attack-forecast calibration, site-value regression
from sim outcomes, win-probability heads), and (c) a held-out human-playtest set for validation. Revisit
neural league training only if the scripted league plateaus and the budget changes.

## Suggested next steps (skateboard first)

1. Add 2 or 3 genes: pre-border public-army trigger (#2), concurrent-war penalty (#5), arrival-time
   urgency for defender buys (#7). Run functionality sims only (no balance sims until the AI is
   competent, per the user's standing rule), and log each on the hill-climb ledger.
2. Audit staging outside city sight (#6) and settler-site weights (#8) against the sim archive.
3. Ask CPL and Longturn organisers whether consented game results or saves exist; do not scrape.
