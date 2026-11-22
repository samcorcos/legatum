terraform {
  required_version = ">= 1.5"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.60, < 7.0"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.4"
    }
  }

  # State lives outside the checkout (worktrees get deleted). tf.sh passes the path:
  #   -backend-config=path=$HOME/.legatum-box/terraform.tfstate
  backend "local" {}
}

provider "aws" {
  region  = var.region
  profile = var.aws_profile

  default_tags {
    tags = {
      # Not "legatum-sims": the sims reaper terminates any legatum-sims server that has no valid
      # expiry tag 6 hours after launch, and this box is persistent. The account-wide budget
      # (infra/cloud-sims) still covers it, and the launcher's budget deny policy blocks StartInstances.
      project = "legatum-box"
    }
  }
}
