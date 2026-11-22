# Copy to terraform.tfvars (gitignored) and fill in.
aws_profile    = "<your-admin-profile>"
launcher_user_name = "<your-launcher-user>" # the same value as in infra/cloud-sims/terraform.tfvars
ssh_public_key = "ssh-ed25519 AAAA... legatum-box" # cat ~/.legatum-box/id_ed25519.pub (tools/cloud/box.sh init creates the key)
