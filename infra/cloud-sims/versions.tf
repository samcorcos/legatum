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
}

provider "aws" {
  region = var.region

  # Optional: AWS_PROFILE normally selects the sims account; this pins it when set.
  profile = var.aws_profile

  default_tags {
    tags = {
      project = "legatum-sims"
    }
  }
}
