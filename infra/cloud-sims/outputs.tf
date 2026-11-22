# tools/cloud/*.sh read these with `terraform output` (local state only, no AWS call).
output "region" {
  value = var.region
}

output "bucket" {
  value = aws_s3_bucket.sims.id
}

output "instance_profile" {
  value = aws_iam_instance_profile.server.name
}

output "security_group_id" {
  value = aws_security_group.server.id
}

output "launcher_user" {
  value = aws_iam_user.launcher.name
}

output "deny_launch_policy_arn" {
  description = "Detach this from the launcher user to re-enable launching after the budget cap tripped."
  value       = aws_iam_policy.deny_launch.arn
}

output "launcher_access_key_id" {
  value = aws_iam_access_key.launcher.id
}

output "launcher_secret_access_key" {
  description = "Secret of the launcher's access key. Never print or commit; README shows how to write it to ~/.aws/credentials without echoing it."
  value       = aws_iam_access_key.launcher.secret
  sensitive   = true
}
