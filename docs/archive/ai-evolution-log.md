> Archived 2026-11-22: kept for history; it no longer describes the project.

# AI evolution run log (contract)

The evolution run (`tools/evolve/`, Astra task `ai-evolution`) writes this log. The dashboard's Evolution panel, the morning report and the film pipeline (`tools/film/`) only read it. Change it here first, then both sides.

A run lives in `docs/sim-results/evolve-<run-id>/` (the run id is `YYYY-MM-DD-HHMM`). Large files are gzip-compressed (`.json.gz`); everything else is plain JSON or JSONL so a half-finished run can be read while it runs.

| File | Written | Contents |
| --- | --- | --- |
| `run.json` | at start, updated at the end | `{run_id, started_at, finished_at?, status: "running"\|"finished"\|"stopped", config: {population, games_per_candidate, speed, seeds, coach_every, genes: [{name, min, max, default, description}]}, baseline: {proxy_win_rate, ai_cities_t150, archetype_entropy}}` |
| `generations.jsonl` | one line per generation | `{gen, t, wall_s, best: {id, fitness, proxy_win_rate, league_win_rate, faction_ratio, archetype}, mean_fitness, fitness_p10, fitness_p90, sigma, archetype_share: {Rush: 0.2, …}, winner_entropy, elites: {<archetype>: {id, fitness}}, genome_best: {<gene>: value}, gene_mean: {<gene>: value}, gene_spread: {<gene>: value}}` |
| `checkpoints/<gen>.json` | every coach cycle and at the end | `{gen, genome, benchmark: {proxy_win_rate, ai_survival_t150, faction_win_rates, archetype_win_rates, matchup_matrix}, showcase: [{game_id, archive_path, seed, why, highlight_turns: [{turn, player, note}]}]}`. The showcase games are archived tapes (`~/LegatumData/archive`), chosen to illustrate the checkpoint: a typical game of the best candidate against the Proxy, plus any game that shows a new behavior. |
| `coach/<gen>.json` | every coach cycle | `{gen, observations: [{kind: "emerged"\|"disappeared"\|"dominant"\|"countered"\|"weakness", behavior, archetype?, evidence: [{game_id, turn, player, detail}], metric_change: {name, before, after}, confidence}], dominant_strategy: {archetype, share, counter_proposal}?, proposals: [...], summary}`. `summary` is two or three plain sentences a person can read: what the AI does differently than 10 generations ago, and why it wins or loses more. |
| `events.jsonl` | whenever it happens | milestones for the story: `{gen, t, kind: "new_best"\|"archetype_takeover"\|"archetype_extinct"\|"league_add"\|"proposal_adopted"\|"proposal_rejected"\|"stall"\|"guardrail", detail, game_id?}` |
| `digest-<date>.md` | nightly and at the end | the written report of the run (see the film and report pipeline) |

Rules:
- Append-only while running, with each line written atomically, so readers can tail it.
- Behavior names come from the archetype classifier (`strategy-diversity-metrics`) and the incident detector (`crates/analysis`), so the coach, the dashboard and the film use the same words.
- Every number shown to the user must be traceable to this log (pillar 5): the film and report cite the generation and game id they come from.

## Readers

All three read only the files above (`tools/film/lib/runlog.mjs`), tolerate a run that is still being written, and show whatever archetype and behavior names the log uses.

- **Dashboard Evolution panel** (`tools/dashboard/evolution.{mjs,js,css}`): the newest `evolve-*` run (others in a picker), refreshed in the server's 15-second background pass. "Watch" links open showcase and evidence games in the Replays section at the cited turn; the server exports those tapes in the background with `target/release/legatum-sim --replay-export` (a few per cycle).
- **Morning report**: `node tools/film/report.mjs <run-dir>` writes `<run-dir>/report.html` (self-contained), linked from the panel.
- **Film**: `tools/film/make-film.sh <run-dir>` writes the report, then `~/LegatumData/films/<run-id>.mp4` with `.srt`/`.vtt` captions, a `.jpg` poster and `.story.md` (the story angles Astra considered, the one it chose and why, and the fact check). Resumable: voice clips, replays and rendered scenes are cached in `~/LegatumData/films/cache/<run-id>/`. `--no-llm` uses the template story.
- **Hook for the producer**: at the end of a run and nightly, run this one line (it takes minutes; background it):
  `tools/film/make-film.sh docs/sim-results/evolve-<run-id> > ~/LegatumData/films/<run-id>.log 2>&1 &`
- **Stand-in**: `node tools/film/fake-run.mjs` writes a small run in this format from real `legatum-sim` games (with `--sim` pointing at a Proxy-capable build it benchmarks against the Proxy), marked `standin` in `run.json`.
- Reader tolerances: `checkpoints/0.json` is optional (the stand-in writes one as a baseline); `benchmark.matchup_matrix` may be `{archetypes, ahead_rate, pairs}` or nested `{A: {B: rate}}`; unknown fields are ignored.

## Version 2 producer additions

`config.harness_version = 2` identifies the new producer. Existing top-level generation and
checkpoint fields remain available to old readers; additions may be ignored. Version 1 state
cannot resume under version 2. The executable is frozen in `data/bin/legatum-sim` before any
game, and `config.simulator_sha256` must match on resume.

- Configuration records `proxy_start: "war"`, `league_start: "peace"`, `proxy_version: 2`,
  `simulator_source`, `evaluations_per_generation` (population + incumbent), `selection`,
  `bootstrap`, `balance_games`, and `validation_speeds: ["standard"]`, and
  `nice: {requested, effective, error?, thread_cap?}`. Nice defaults on; failure to lower OS
  priority is explicitly recorded and limits simulator threads to half the logical cores.
- The 69 genes are 40 faction personality knobs and 29 shared `decision.<name>` genes. Each
  gene has its own min/max/default/description. `gene_spread` is in that gene's native units,
  no longer universally in 0–100 knob points.
- `generations.jsonl` adds `cpu_s` (child process CPU, including overlapping coach work),
  `candidates: [{id, rank, fitness, fitness_se?, fitness_ci95?, bootstrap_repeats?, method?}]`,
  and `incumbent: {id_before, id_after, fresh_fitness, fitness_se, promoted,
  comparisons: [{id, delta, se, ci, alpha}]}`. Failed candidates lack uncertainty. Candidate
  `rank` is the ordinal CMA-ES minimization rank after the boundary penalty.
- `fitness_se` is the standard deviation of 256 opponent-stratified, whole-game bootstrap
  scores, recomputing the entire nonlinear fitness. The candidate interval uses a normal
  approximation at 95%. Paired comparisons reuse identical draws/seeds/opponents and use
  alpha `0.05 / population`; the lower bound must exceed zero to replace the incumbent.
  Bootstrap samples live only in the compressed cache, not in the small log.
- `best` / `genome_best` still mean the raw generation leader, **not** the incumbent. Its
  score and historical peaks are not comparable across seed blocks. `best.fitness_se` is
  included. `new_best` events now mean a successful paired incumbent replacement.
- Checkpoints benchmark the selected incumbent (baseline remains stock), record both start
  modes, and add `benchmark.uncertainty: {fitness_se, fitness_ci95, bootstrap_repeats, method}`.
- Niche elites retain their old shape but are the latest generation's best representative of
  each observed niche. Only candidates in the same generation are compared by scalar fitness.
- `best-params.json` contains the selected incumbent; an unevaluated final CMA mean is never
  labelled best. No export changes the default AI.

`run.json.final_validation` now has this shape:

```json
{
  "speeds": {
    "standard": {
      "proxy": {
        "games": 512,
        "builtin": {"rate": 0.5, "ci95": [0.45, 0.55]},
        "evolved": {"rate": 0.4, "ci95": [0.35, 0.45]},
        "improvement": 0.1, "paired_se": 0.02, "ci95": [0.04, 0.16],
        "method": "paired discordant outcomes; simultaneous Wilson bounds (conservative 95%)"
      },
      "league": {"games": 512, "wins": 280, "rate": 0.546875, "ci95": [0.50, 0.59]},
      "balance": {"builtin": {}, "evolved": {}},
      "stalls": 0
    }
  },
  "seeds": {"held_out_proxy": "950001", "held_out_league": "970001", "held_out_balance": "990001"},
  "proxy_start": "war", "league_start": "peace",
  "verdict": {"decision": "don't-adopt", "reasons": ["example only"], "scope": "parameter candidate only; no automatic change to stock AI"}
}
```

The numbers above illustrate the shape, not measured evidence. Each `balance` entry contains
`games`, `factions: {name: {wins, seats, rate, ci95_simultaneous}}`, raw `faction_ratio` (null if
undefined), `faction_ratio_upper95`, `archetype_counts`, `winner_counts`, `archetype_entropy`,
`winner_entropy`, `winner_entropy_ci95`, `dominant`, `matchup_matrix`, and `stalls`. Zero wins
are not smoothed away. All self-play seats use the same candidate at Peace starts.

The Standard validation must pass the criteria in [ai-evolution.md](ai-evolution.md#final-validation-and-adoption).
A skipped or failed validation has empty `speeds`, a don't-adopt verdict, and reasons (plus
`error` on failure). The final digest always has an explicit adopt / don't-adopt verdict,
confidence intervals, guardrail results, and measured SE/throughput. This is advisory only.
