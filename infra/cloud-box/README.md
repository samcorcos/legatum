# Cloud build box (infra/cloud-box)

One persistent Graviton machine for heavy work that overloads the Mac: branch verification (`verify-branch.sh`),
cargo tests and clippy, Playwright (Chromium and native-Linux WebKit) and the Linux sim build. It is **stopped when
idle**: you pay for its EBS disks (about $56 a month since the data volume grew to 600 GB) and for compute only while it runs. Scripts: `tools/cloud/box.sh`,
`remote.sh`, `verify-remote.sh`, `webkit-remote.sh`; how agents use them is in the `legatum-workflow` skill.

State is separate from `infra/cloud-sims` (a sibling root module) and lives outside the checkout:
`~/.legatum-box/terraform.tfstate` (no secrets in it). Always run Terraform through `infra/cloud-box/tf.sh`.

## What Terraform creates (tagged `project=legatum-box`)

| Piece | Purpose |
| --- | --- |
| `aws_instance.box` | c8g.16xlarge (64 vCPUs, 128 GB), Ubuntu 24.04 arm64, **persistent spot request that stops (never terminates) on interruption**, IMDSv2 only, encrypted root (40 GB), instance-initiated shutdown = stop. `use_spot = false` makes it on-demand. |
| `aws_ebs_volume.data` | 600 GB encrypted gp3 (XFS) at `/data`: rustup, cargo, node, Playwright browsers, docker, the repo mirror, 10 job slots with warm cargo `target/` dirs. Survives instance replacement. |
| Role/profile `legatum-box` | SSM agent policy and S3 read/write on `box/*` of the sims bucket. No EC2, IAM, GitHub or user credentials. |
| Security group `legatum-box` | **No inbound rules.** sshd listens on loopback and is reached through SSM Session Manager (ssh over `AWS-StartSSHSession`). |
| Policy `legatum-box-control` on the launcher user | start/stop only instances tagged `project=legatum-box`; Session Manager and send-command to them. The budget's deny policy (RunInstances/StartInstances) on the same user still applies at 100% of the monthly cap. |
| Lambda `legatum-box-cap` (every 5 min) | AWS-side net: `daily_cap_hours` defaults to 24 (no practical daily cap); also stops the box when it has been up 40+ minutes with CPU under 3% for 30 minutes. Idle auto-stop and the monthly budget are the usage guards. |
| On the box: `legatum-idle-watch` (systemd timer, every minute) | powers the box off after 20 minutes with no job, no ssh/SSM session and low load. |

The tag is **not** `legatum-sims` on purpose: the sims reaper terminates any `legatum-sims` server that has no valid
`expiry` tag 6 hours after launch, which would delete a persistent box. The account-wide budget covers both.

## Apply (needs approval; the admin profile is SSO)

```
aws sso login --profile <admin-profile>     # the profile in terraform.tfvars; skip if it is not SSO
tools/cloud/box.sh init                       # creates ~/.legatum-box/id_ed25519 once; prints the ssh_public_key line
cp infra/cloud-box/example.tfvars infra/cloud-box/terraform.tfvars   # paste the key
infra/cloud-box/tf.sh plan -out plan.bin      # read-only
infra/cloud-box/tf.sh apply plan.bin
tools/cloud/box.sh prepare                    # first time: toolchain bootstrap, ~10 min (then `box.sh warm`)
```
The instance starts running after the apply and powers itself off after 20 idle minutes.

## Cost (spot c8g.16xlarge $0.70 to $0.81 an hour on 2026-10-10, us-east-1c $0.70; plus $0.005/h public IPv4)

| | per month |
| --- | --- |
| EBS while stopped: 600 GB data + 40 GB root gp3 at $0.08, +125 MB/s throughput | about $56 |
| Compute at ~3 h a day (90 h) | $66 |
| Compute at ~10 h a day (300 h) | $220 |
| **Total at 3 h a day** | **about $122** |
| **Total at 10 h a day** | **about $276** |
| **Total at 24 h a day (maximum)** | **about $664 in a 31-day month** (gross spot cost at $0.81/h plus IPv4 and storage, before the shared monthly budget stops new launches) |
| On-demand instead of spot ($2.55/h), 3 h a day | about $262 |

The monthly budget (`monthly_budget_usd` in infra/cloud-sims) is for the whole account (sims batches too): at sustained use above ~5 h a day, raise
`monthly_budget_usd` in `infra/cloud-sims` first. Spot and the sims servers share the spot vCPU quota (448 on
2026-10-10); a box start fails with a capacity message when sims batches hold all of it, and every tool then falls back
to local (`remote.sh` exit 3).

## Operations

* `tools/cloud/box.sh status` shows state, runtime against the cap, load, disk and jobs. `tools/cloud/status.sh` also shows it.
* `tools/cloud/box.sh gc` clears old jobs and, above 85% disk, the coldest slot caches (the next job re-seeds from the golden slot).
* Rotate the ssh key: `rm ~/.legatum-box/id_ed25519*`, `box.sh init`, update `terraform.tfvars`, `tf.sh apply` (replaces the instance; the data volume is kept; run `box.sh prepare`).
* Tear down: `tf.sh destroy` (also deletes the data volume, i.e. the caches).
