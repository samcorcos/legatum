# Ops: cloud simulation batches and AWS

How AWS spot batches and the build box are run and paid for (still current for sim cohorts; Mac-specific steps are historical).

Moved verbatim from SKILL.md on 2026-11-22 (original order kept). The current rules are in SKILL.md; where this file disagrees with SKILL.md, SKILL.md wins.

## Cloud simulation batches (user, 2026-10-06; infra/cloud-sims, tools/cloud)
- **What:** on-demand spot Graviton servers (default `c8g.16xlarge`, 64 vCPUs, about $0.72/h) for sim cohorts too big for the Mac. Today the tooling runs only sim binaries, not builds, cargo gates, Codex or Playwright. Dedicated AWS account with a monthly budget cap, nothing unneeded may run.
- **Offload under overload (user, 2026-10-10):** "if the CPU is overloaded by maybe 2x, or really anything that slows down progress meaningfully, we should seek to offload that to AWS if we can." At a load of about 32 or more on 16 cores, or whenever critical-path work is slowed, move what AWS can run, and name what it can't so the tooling gets built. Procedure: ops-supervisor agent, section 5b.
- **Timing ledger (user, 2026-10-10):** every routine step logs wall and CPU time with `tools/ops/timed.py` to `~/LegatumRuns/timings/`, and the supervisor reads `tools/ops/timings-report.py` every run to pick the next bottleneck. Cost is measured in CPU-minutes. Speed-ups must keep quality: no dropped tests or tour coverage.
- **Narrow simulators (user, 2026-10-10):** when sims study one subsystem through full games, build a dedicated Rust harness that drives the real engine for just that slice (combat-scenarios is the model). Never write a second rules implementation.
- **How:** `tools/cloud/build-linux.sh` once per engine change (Docker, linux/arm64 container), then `tools/ops/bgjob.sh cloud-<name> --log .progress/cloud-<name>.log -- tools/cloud/run-batch.sh --count N --max-hours H -- <legatum-sim args>`. Run it under `bgjob.sh` named `cloud-*`: the watchdog alerts on any cloud server while no such job runs. Results merge into `~/LegatumRuns/cloud/<batch>/merged.json.gz`.
- **Always pass `--max-hours`** with real headroom (the default is 2, the cap 12): it is the servers' own `shutdown -h` timer. It is not a target, so a batch that ends early costs less.
- **After every batch run `tools/cloud/status.sh`.** The answer must be "no legatum-sims servers running"; otherwise `tools/cloud/kill-all.sh`. An alert from the watchdog about a cloud server is urgent: it is costing money.
- Balance sims are still off (see above): use it for functional or determinism cohorts and for AI-quality hill-climbing, not balance probes.
- Never run `terraform apply` or any AWS write on your own: the lead asks the user first. Only the launcher scripts may start servers.

- **Where the AWS bill goes (2026-11-01 audit):** almost all of a month's bill is spot compute, about 2/3 batch servers and 1/3 the build box; disk and IPv4 are the only idle costs, and they are small. Cost allocation tags weren't available on the account, so split box vs batch hours from local logs (`~/LegatumRuns/box/*/meta.env` START_EPOCH plus the "exited after Ns" line; `~/LegatumRuns/cloud/*/BATCH_INFO`). Cost tracks benchmark volume, so the lever is gate size, not cleanup. Cloudflare doesn't fit: Workers' memory and CPU caps, and Containers cost more per vCPU than spot.
