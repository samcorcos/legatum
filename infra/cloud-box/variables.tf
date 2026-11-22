variable "region" {
  description = "Must match infra/cloud-sims (the bucket and the launcher live there)."
  type        = string
  default     = "us-east-1"
}

variable "aws_profile" {
  description = "AWS CLI profile of the admin (SSO) session for the sims account."
  type        = string
  default     = null
}

variable "availability_zone" {
  description = "One AZ for the box and its data volume. us-east-1c had the lowest c8g.16xlarge spot price on 2026-10-10 ($0.70/h); the range across the six AZs was $0.70 to $0.81."
  type        = string
  default     = "us-east-1c"
}

variable "instance_type" {
  description = <<-EOT
    c8g.16xlarge: 64 Graviton4 vCPUs, 128 GB RAM. Cost scales with hours used, not size, and a stopped box costs
    nothing, so a big box finishes the same work sooner for the same money. 4 to 6 concurrent gates (each 4 to 16
    compile jobs, fat-LTO links up to 8 GB) fit with room. Same family as the sim servers, so one spot price list.
  EOT
  type        = string
  default     = "c8g.16xlarge"
}

variable "use_spot" {
  description = <<-EOT
    true: a persistent spot request that STOPS (never terminates) on interruption, about 70% cheaper. An interruption
    kills the running jobs (scripts report it and the job is simply re-run); the disk and the warm caches survive.
    false: on-demand, about $2.55/h for c8g.16xlarge. Changing it replaces the instance (the data volume is kept).
    Spot shares the account's spot vCPU quota with the sims servers (448 on 2026-10-10).
  EOT
  type        = bool
  default     = true
}

variable "root_volume_gb" {
  description = "OS disk. Disposable: bootstrap.sh rebuilds everything on it; caches live on the data volume."
  type        = number
  default     = 40
}

variable "data_volume_gb" {
  description = "Persistent encrypted volume mounted at /data: rustup, cargo registry, node, Playwright browsers, docker, the repo mirror, per-slot checkouts with their cargo target dirs (66 to 73 GB each) and the warm master target. 600 GB for 10 slots (2026-10-18; it was 300 GB for 5)."
  type        = number
  default     = 600
}

variable "data_volume_iops" {
  description = "gp3 IOPS (3000 is included in the base price; each extra IOPS is $0.005 a month). Adjustable in place later."
  type        = number
  default     = 3000
}

variable "data_volume_throughput_mbps" {
  description = "gp3 throughput in MB/s (125 is included in the base price; each extra MB/s is $0.04 a month). 250 keeps four parallel link steps from queueing on disk."
  type        = number
  default     = 250
}

variable "ssh_public_key" {
  description = "Public half of the dedicated key tools/cloud/box.sh creates at ~/.legatum-box/id_ed25519. sshd on the box listens on 127.0.0.1 only and is reached through SSM Session Manager, so the key alone opens nothing."
  type        = string
}

variable "launcher_user_name" {
  description = "The IAM user tools/cloud/*.sh run as (created by infra/cloud-sims; the same value as there). Required."
  type        = string
  # No default: the name is yours to choose and is not written in the repository. Set it in terraform.tfvars.
}

variable "idle_minutes" {
  description = "The box powers itself off after this many minutes with no job, no login session and low load."
  type        = number
  default     = 20
}

variable "daily_cap_hours" {
  description = "24 = no daily cap (user, 2026-10-11: use the box whenever there is load; idle stop and the monthly budget remain the guards). Hard cap on running hours per UTC day, enforced from AWS by a Lambda that stops the box (even if the box or the Mac are broken). Worst case per day at spot: cap x $0.73. Starting the box again after the cap is reached gets it stopped within 5 minutes."
  type        = number
  default     = 24
}
