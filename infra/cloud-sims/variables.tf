variable "region" {
  description = <<-EOT
    Region for the servers, bucket and guardrails. Default us-east-1: the deepest Graviton spot
    capacity (six AZs, so run-batch.sh can pick the cheapest one), and spot prices for c7g/c8g
    differ by only a few percent between the US regions. Switching to us-east-2 or us-west-2 is
    safe (run-batch.sh reads the region from the Terraform outputs); compare
    `aws ec2 describe-spot-price-history` first if the bill matters.
  EOT
  type        = string
  default     = "us-east-1"
}

variable "aws_profile" {
  description = "AWS CLI profile of the dedicated sims account (null: use the default credential chain)."
  type        = string
  default     = null
}

variable "alert_email" {
  description = "Email that receives the budget alerts (50%, 80%, 100% of the monthly cap, plus a 100% forecast). Required: no default, so the address is never committed."
  type        = string
}

variable "monthly_budget_usd" {
  description = "Monthly cost cap for the whole account, in USD. At 100% actual spend, a Budget action denies new server launches. Required: no default, so your cap is your own choice and is never committed."
  type        = number
}

variable "launcher_user_name" {
  description = "IAM user that tools/cloud/*.sh run as from the Mac. Terraform creates it with least-privilege launch rights; at 100% of the budget the deny policy is attached to it. Create its access key by hand (see README), so no secret lands in Terraform state. Required."
  type        = string
  # No default: the name is yours to choose and is not written in the repository. Set it in terraform.tfvars.
}

variable "allowed_instance_types" {
  description = "Instance-type patterns the launcher may start (a typo or a stray GPU type is refused by IAM)."
  type        = list(string)
  default     = ["c7g.*", "c8g.*", "c6g.*", "m7g.*", "m8g.*"]
}

variable "results_expire_days" {
  description = "Objects in the sims bucket (binaries and results) are deleted after this many days."
  type        = number
  default     = 30
}

variable "reaper_grace_minutes" {
  description = "The AWS-side reaper terminates a legatum-sims server this many minutes after its expiry tag (the in-server shutdown fires first; this is the net under it)."
  type        = number
  default     = 15
}

variable "spot_vcpu_quota" {
  description = <<-EOT
    Request this Service Quotas limit for "All Standard Spot Instance Requests" (vCPUs); 0 skips
    the request. The quota doubles as a cost ceiling: the account can never run more vCPUs than
    this at once (64 vCPUs = one c7g.16xlarge, about $0.7 an hour at spot prices). The sims account
    started at 384, so the default skips the request; a lower value than the current quota is
    refused by AWS, so set it only to raise the quota (or lower it by hand in the console).
  EOT
  type        = number
  default     = 0
}

variable "create_spot_service_linked_role" {
  description = "Create the EC2 Spot service-linked role. Needed once per account; set false if the account already has it."
  type        = bool
  default     = true
}
