# Cloud build box runbook

The box (an EC2 instance, c8g.16xlarge, 64 vCPUs, 128 GB, spot, `/data` 600 GB XFS) runs verifies, lane tests,
sim builds and Playwright from `tools/cloud/`. Terraform is `infra/cloud-box/`; the box scripts are `tools/cloud/box/`.
How agents use it is in the `legatum-workflow` skill. This page is for when it misbehaves: symptom, cause, fix.

First look, always: `tools/cloud/box.sh status` (state, load, disk, jobs, slots busy, CPU utilisation per hour, filler).
**Load far below the core count with a long queue means a jam (locks, slots, disk), not a lack of CPU.**

## Hung verifies (a verify "runs" for hours, load is low)

* **Cause:** `verify-branch.sh` waited on a lock directory (`.progress/verify/.locks/<branch>` in the slot, or
  `~/LegatumRuns/gate-cache/locks/<step>.lock`) whose holder had been killed (box-job kill, a dropped client, a reboot).
  The old wait was a blind four-hour `mkdir` loop.
* **Fixed (2026-10-18):** `tools/astra/lockdir.sh` records `pid + process start time` in every lock and takes over a lock
  whose holder is dead, so a stale lock now costs about two seconds. `verify-job.sh` also clears the slot's own locks at
  start and sweeps dead gate-cache locks every 20 s. `box-job kill` waits 15 s for the traps to run.
* **If it still happens:** the log says `lockdir: <dir> held by a dead process ...: taking it over` when the fix fires. A
  verify that waits with that line absent is waiting on a live holder: find it with
  `tools/cloud/box.sh shell 'cat <lock>/pid; ps -p <pid>'` and look at what it is doing. Do not kill another agent's job.
* **Never** fix a stale lock by hand without checking the pid is dead first.

## Stale locks and the slot queue

* **Symptom:** many jobs "waiting for a slot", slots idle or load low.
* **Slot locks** are `flock` files (`/data/slots/sN.lock`): the kernel releases them when the holder dies, they cannot go
  stale. The **queue tickets** (`/data/slots/ticket.N`) hold the waiter's pid; a dead waiter's ticket is skipped and removed.
* **Strict FIFO:** the head waiter blocks everyone behind it. Waiters that were queued before the slot count went from 5 to
  12 run older code that only tries s1..s5; new waiters behind such a head may still take s6..s12 (`v2.N` markers).
* **Check:** `cat /data/slots/serving /data/slots/next-ticket` (the difference is the queue length), `box-job list`.

## A full disk

* **Symptom:** `/data` above 90%, jobs failing with ENOSPC or waiting "for disk".
* **Cause:** every slot keeps its own cargo `target/`. Debug-profile test builds (lane `cargo test`) dominated: 57 GB of
  `target/debug` in one slot, 18 GB of it incremental caches and 26 GB of debuginfo.
* **Fixes in place:** jobs run with `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=line-tables-only`; after each job the
  slot drops `incremental/` dirs and, if the slot is over `SLOT_BUDGET_GB` (34), its `target/debug`; above 75% disk the
  coldest idle slots lose `target/` (re-seeded from the golden slot by reflink on next use); a job waits up to 30 min for
  `MIN_FREE_GB` (40) instead of dying mid-build. Tunables live in `/data/bin/config` (`DISK_HIGH_PCT`, `DISK_LOW_PCT`,
  `SLOT_BUDGET_GB`, `MIN_FREE_GB`).
* **Now:** `tools/cloud/box.sh sweep` trims every idle slot (busy ones are skipped under `flock -n`) and old job dirs;
  `box.sh gc` also drops the coldest idle slot caches when above 75%. Only idle slots, stale locks and old job dirs may be deleted.
* **Growing the volume** (the 2026-10-18 grow, 300 to 600 GB): change `data_volume_gb` in `infra/cloud-box/variables.tf`;
  `infra/cloud-box/tf.sh plan -input=false -var aws_profile=<admin-profile> -var "ssh_public_key=$(cat ~/.legatum-box/id_ed25519.pub)" -out plan.bin`
  must show **one in-place change** to `aws_ebs_volume.data` and nothing else; the user approves, the lead applies
  (`tf.sh apply plan.bin`). Then on the box (XFS on the raw disk, no partition, so no `growpart`):
  `sudo xfs_growfs /data; df -h /data`. EBS allows one modification per volume per 6 hours.
* **Raising `SLOTS`:** edit `SLOTS=` in `/data/bin/config` (and `bootstrap.sh`'s default); new slots seed from the golden
  slot by reflink and only grow as they diverge, but budget `SLOTS x SLOT_BUDGET_GB + about 25 GB shared` against the volume.

## The queue is long but the box is not busy

* Check `box.sh status`: slots busy of 12, queued for a slot, CPU utilisation. With free slots and a queue, look for a stuck
  head waiter (dead pid with a recycled live pid is the rare case: remove its `/data/slots/ticket.N`).
* Lane jobs run `cargo test`; on the box the cargo shim turns plain `cargo test <simple flags>` into `cargo nextest run` plus
  `cargo test --doc`, so tests use many cores instead of sitting at 130% CPU for an hour. `LEGATUM_BOX_NO_NEXTEST=1`
  forces plain `cargo test`. Each job's parallelism (cargo jobs, test threads) is scaled to the cores per held slot at start.

## Load far above the core count (load 365 on 64 cores)

* **Symptom:** `box.sh status` load average several times `nproc`, every job crawling, Playwright timeouts and flakes.
* **Cause:** twelve slots, and every job sized itself for the whole box: `verify-branch.sh` (cargo jobs ncpu/4 in each of
  three or four concurrent lanes, 32 test threads, up to 21 Playwright workers, vitest per core), WebKit runs at Playwright's
  default of 32 workers, `wasm-opt` with one thread per core, nextest and rayon one thread per core.
* **Fix (2026-10-18):** `tools/cloud/box/budget.sh` gives each job 1.25 x ncpu / (slots held + jobs queued, 4 to SLOTS) cores:
  about 6 at full occupancy, 20 for a lone job. `box-job` exports it as `LEGATUM_CORE_BUDGET` with `BOX_CARGO_JOBS`,
  `NEXTEST_TEST_THREADS`, `RAYON_NUM_THREADS`, `BINARYEN_CORES`, `VITEST_MAX_WORKERS` and `LEGATUM_PW_WORKERS`;
  `verify-branch.sh` sizes its lanes from it. `verify-job.sh`, `lane-run.sh`, `build-sim.sh` and `webkit.sh` apply it
  themselves (they are read from `/data/bin` when the command starts), so jobs that queued under an older `box-job` are
  sized too, through `verify-branch`'s `LEGATUM_VERIFY_*` overrides, whatever age the shipped master is.
* **Not covered:** jobs that were already running when the fix deployed keep their old sizes until they end; a budget is
  fixed when a job starts, so a long job that started on a quiet box keeps a big share. Raw `remote.sh -- <cmd>` commands
  only get the env defaults (cargo jobs, rayon), not their own tools' flags.
* **Check:** a started job logs `core budget N of 64 (slots held H, queued Q)`; `uptime` should settle near 64 to 80 with all
  slots held. If it does not, find the process family: `ps -eLo stat,comm | awk '$1 ~ /^R/ {c[$2]++} END {for (k in c) print c[k], k}' | sort -rn`.
* The filler never adds load while slots are full: it starts a batch only with no queue and two free slots, and SIGSTOPs
  a running one when a job queues or all slots are taken.

## Script versions (`box_sync`)

* Every `remote.sh` / `verify-remote.sh` call syncs `tools/cloud/box/*` to `/data/bin`. It used to overwrite from the
  caller's checkout, so an agent on an old checkout downgraded the box scripts.
* Now `/data/bin/VERSION` holds the commit of the last change to `tools/cloud/box` and a sync happens only when the caller's
  box scripts are clean, on master, and a descendant of the box's version. Otherwise it says
  `box: keeping the box scripts at <sha>, not syncing: ...`. To test unmerged box scripts, `LEGATUM_BOX_SYNC_FORCE=1`; the
  next master sync replaces them.

## Idle filler and utilisation

* `tools/cloud/filler.sh submit SPEC.json [--commit REF]` queues pure-CPU batches (AI optimisation arms, combat-lab sweeps) that
  run at CPUWeight=1 and nice 19 in `legatum-filler.slice`, so real jobs get the cores. The box stays up while there is
  filler work and stops 20 minutes after the last job. Format and rules: header of `tools/cloud/filler.sh`.
* `tools/cloud/box.sh util [hours]` prints mean CPU busy % per hour and how much of it was filler (sampled each minute by
  the `legatum-util` timer into `/data/util/cpu.tsv`). `verify-branch.sh` subtracts the filler's cores from the load average it
  sizes Playwright workers by.
* The box role cannot delete from S3, so filler batches need unique ids; `filler.sh fetch ID` clears an id's markers.
