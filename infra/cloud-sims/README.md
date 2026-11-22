# Cloud simulation runner (infra/cloud-sims)

On-demand spot Graviton servers for big simulation batches. The fixed pieces are Terraform
(this directory); the servers themselves are started and stopped per batch by `tools/cloud/`.
Nothing here runs between batches except an empty bucket, a tiny reaper function and the budget.

## What Terraform creates (tagged `project=legatum-sims`)

| Piece | Purpose |
| --- | --- |
| S3 bucket `legatum-sims-<account id>` | binaries in, results out. All public access blocked, TLS only, SSE-S3, **objects expire after 30 days**. |
| Instance role + profile `legatum-sims-server` | S3 on that bucket only, plus the SSM agent policy (Session Manager). Nothing else. |
| Security group `legatum-sims-server` | **no inbound rules**; outbound open. Debug through SSM Session Manager. |
| IAM user (`launcher_user_name`, yours to choose) + access key | what `tools/cloud/*` and the watchdog run as (no admin session needed). Least privilege: see below. |
| Budget `legatum-sims-monthly`, the amount you set in `monthly_budget_usd` | emails at 50%, 80%, 100% of actual spend and at a 100% forecast. At 100% a Budget action attaches `legatum-sims-deny-launch` (denies RunInstances, RequestSpotInstances, fleets, StartInstances) to the launcher. |
| Lambda `legatum-sims-reaper` (every 15 min) | terminates any `project=legatum-sims` server past its `expiry` tag plus 15 minutes, with the laptop off or crashed. |
| Spot service-linked role | needed once per account for spot requests. |

The launcher can only: start **spot** servers of the allowed Graviton types, tagged `project=legatum-sims`,
with the sims instance profile; terminate or stop servers tagged `project=legatum-sims`; describe EC2,
read spot and on-demand prices; read and write the sims bucket; pass the server role to EC2; and open
Session Manager sessions to sims servers.

## Guardrails: nothing runs unneeded or forgotten

Layered, each independent of the others:

1. **Self-destruct in user-data.** The first line of every server's user-data is `shutdown -h +<max-hours*60>`. `--max-hours` is required (default 2, at most 12) and instance-initiated shutdown is set to *terminate*.
2. **Work, upload, power off.** The server runs its slice, uploads results and a `DONE` marker, then shuts down (also on failure: an EXIT trap does it).
3. **The launcher script cleans up.** `run-batch.sh` terminates its batch's servers on Ctrl-C, on any error, and at the end.
4. **The AWS-side reaper** (above) terminates anything past its expiry tag, even if the Mac is off.
5. **The watchdog** (`tools/ops/watchdog.sh`, when `LEGATUM_AWS_PROFILE` is set) alerts on a server past expiry, on any server while no `cloud-*` job is registered, and when AWS cannot be reached.
6. **Spot vCPU quota**: the account can never run more vCPUs than its Service Quota (384 at creation, six 64-vCPU servers).
7. **Budget**: your `monthly_budget_usd`, alerts, and the launch ban at 100%. Billing data lags by hours, so this is the last net, not the first.
8. **Everything expires**: bucket objects after 30 days, reaper logs after 14.

Check `tools/cloud/status.sh` after every batch. "no legatum-sims servers running" is the answer you want.

## One-time setup (the account already exists)

1. Pick the AWS CLI profile you administer the account with (called `<admin-profile>` below) and sign in with it; with SSO that is `aws sso login --profile <admin-profile>` when the session expires.
2. Choose the alert email and write `infra/cloud-sims/terraform.tfvars` (gitignored; `example.tfvars` shows the shape):
   ```
   alert_email        = "you@example.com"
   aws_profile        = "<admin-profile>"
   launcher_user_name = "<launcher-user>"   # any name; no default, so it is never committed
   ```
3. Plan, read it, apply:
   ```
   cd infra/cloud-sims
   terraform init
   terraform plan -out plan.bin      # read-only
   terraform apply plan.bin          # the only step that creates anything
   ```
   `apply` on the empty account creates the whole setup with no manual steps. Confirm the budget subscription email AWS sends (the notification needs it).
   If `aws_iam_service_linked_role.spot` fails with "already exists", set `create_spot_service_linked_role = false`.
4. Write the launcher's key into `~/.aws/credentials` under a profile name of your choice, without echoing it, and
   tell the tools which profile that is:
   ```
   cd infra/cloud-sims
   P=<launcher-profile>
   aws configure set aws_access_key_id "$(terraform output -raw launcher_access_key_id)" --profile "$P"
   aws configure set aws_secret_access_key "$(terraform output -raw launcher_secret_access_key)" --profile "$P"
   aws configure set region us-east-1 --profile "$P"
   mkdir -p ../../.private && echo "LEGATUM_AWS_PROFILE=$P" >> ../../.private/cloud.env   # git-ignored
   ```
   `tools/cloud/lib.sh` reads `LEGATUM_AWS_PROFILE` from the environment, else from `.private/cloud.env` in this
   checkout or the main checkout (`.private.example/cloud.env` shows the shape). Unset, the AWS CLI's default
   credential chain is used.
5. `tools/cloud/build-linux.sh`, then a first small batch (`run-batch.sh --max-hours 0.5 -- --games 200 ...`) as a check (see the determinism step in `docs/learnings.md`).

**State** is local (`terraform.tfstate` in this directory, gitignored). It holds the launcher's secret key, so keep it out of git and backups you share. Losing it is survivable: `terraform import`, or destroy by hand (everything is tagged `project=legatum-sims`).

**Rotate the key:** `terraform apply -replace=aws_iam_access_key.launcher`, then redo step 4.

**Re-enable launching after the cap tripped:** at the start of next month spend resets, but the deny policy stays attached until detached: `aws iam detach-user-policy --user-name "$(terraform output -raw launcher_user)" --policy-arn $(terraform output -raw deny_launch_policy_arn)` (with the admin profile).

**Tear down:** `terraform destroy` (the bucket has `force_destroy`).

## Region and instance choice

Default `us-east-1`: the most Graviton spot capacity (six AZs; `run-batch.sh` tries the cheapest AZ first and falls through on capacity errors), and Budgets and billing live there. Spot prices checked 2026-10-06 for a 16xlarge (64 vCPUs): c7g $0.66-0.80/h, c8g $0.71-0.85/h, c6g $0.60-0.83/h; on-demand is $2.32 (c7g) and $2.55 (c8g). Graviton4 (c8g) is claimed about 30% faster per core for about 7% more at spot, so **c8g.16xlarge is the default**; `--type c7g.16xlarge` is the fallback. The claim is unmeasured for this workload: the first real batch prints server-hours and games per second, so compare then.

## Costs

Assumptions: c8g.16xlarge spot about $0.71/h (+$0.005/h public IPv4 for each server), 20 to 30 GB gp3 root volume (pennies), S3 and Lambda inside free-tier-sized usage. Games per second per Graviton4 core is **an estimate** (4 to 6 a second on a duel, against 8.6 on one laptop core from `docs/sim-results/perf/ledger.csv`, 85% scaling over 64 cores), to be replaced by the first real run.

| | Estimate |
| --- | --- |
| One server, per hour | about $0.72 |
| 1v1 duel (Hard vs Hard), per million games | 0.8 to 1.2 server-hours, **about $0.60 to $0.90** |
| Small-map Standard duel, per million games | about 1.5x the duel: $0.90 to $1.40 |
| 8-player Standard, per million games | about 10x the duel: $6 to $9 |
| A $100 cap, as an example | about 140 server-hours: roughly 110 to 165 million duels at the estimated cost per million |

`run-batch.sh` prints the current spot price, the cost if every server runs the full max-hours, and the on-demand ceiling before launching, and the server-hours used afterwards.
The AWS bill is authoritative.

## Using it

See `tools/cloud/run-batch.sh --help`. Results land in `~/LegatumRuns/cloud/<batch>/` (`merged.json.gz`, which the analysis scripts read through `load_report`, and each server's own report and log under `slice-*/`). A spot interruption loses that server's slice (no partial results); the script says which games and seeds are missing.
Index-scheduled flags (`--strategy-matchups`, `--duel-matrix`) cannot be split and are refused; `--mirror` and the rotations need `--align` to match their period.
