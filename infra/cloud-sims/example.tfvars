# Copy to terraform.tfvars (gitignored) and fill in.
alert_email = "you@example.com"
aws_profile = "<your-admin-profile>"         # the AWS CLI profile you administer the account with
monthly_budget_usd = 100                     # your monthly cap in USD; launches are denied at 100%
launcher_user_name = "<your-launcher-user>"  # the IAM user Terraform creates for tools/cloud; pick any name
