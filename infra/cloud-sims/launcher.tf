# The identity the Mac's tools/cloud scripts and the watchdog run as, so batches never depend on an
# SSO session. Terraform creates its access key; the secret lives in the (gitignored, local) state
# file and is shown only by `terraform output -raw launcher_secret_access_key` (README). Rotate with
# `terraform apply -replace=aws_iam_access_key.launcher`.
resource "aws_iam_user" "launcher" {
  name = var.launcher_user_name
}

resource "aws_iam_access_key" "launcher" {
  user = aws_iam_user.launcher.name
}

data "aws_iam_policy_document" "launcher" {
  # Servers: spot only, tagged, an allowed type, and only with the sims instance profile.
  statement {
    sid       = "RunTaggedSpotServers"
    actions   = ["ec2:RunInstances"]
    resources = ["arn:${data.aws_partition.current.partition}:ec2:*:${local.account_id}:instance/*"]
    condition {
      test     = "StringEquals"
      variable = "aws:RequestTag/project"
      values   = ["legatum-sims"]
    }
    condition {
      test     = "StringEquals"
      variable = "ec2:InstanceMarketType"
      values   = ["spot"]
    }
    condition {
      test     = "StringLike"
      variable = "ec2:InstanceType"
      values   = var.allowed_instance_types
    }
    condition {
      test     = "ArnEquals"
      variable = "ec2:InstanceProfile"
      values   = [aws_iam_instance_profile.server.arn]
    }
  }

  statement {
    sid     = "RunInstancesOtherResources"
    actions = ["ec2:RunInstances"]
    resources = [
      "arn:${data.aws_partition.current.partition}:ec2:*::image/*",
      "arn:${data.aws_partition.current.partition}:ec2:*:${local.account_id}:subnet/*",
      "arn:${data.aws_partition.current.partition}:ec2:*:${local.account_id}:network-interface/*",
      "arn:${data.aws_partition.current.partition}:ec2:*:${local.account_id}:volume/*",
      "arn:${data.aws_partition.current.partition}:ec2:*:${local.account_id}:spot-instances-request/*",
      aws_security_group.server.arn,
    ]
  }

  statement {
    sid       = "PriceLookups"
    actions   = ["pricing:GetProducts", "pricing:DescribeServices", "pricing:GetAttributeValues"]
    resources = ["*"]
  }

  statement {
    sid       = "TagOnLaunch"
    actions   = ["ec2:CreateTags"]
    resources = ["arn:${data.aws_partition.current.partition}:ec2:*:${local.account_id}:*/*"]
    condition {
      test     = "StringEquals"
      variable = "ec2:CreateAction"
      values   = ["RunInstances"]
    }
  }

  statement {
    sid       = "StopOwnServersOnly"
    actions   = ["ec2:TerminateInstances", "ec2:StopInstances"]
    resources = ["arn:${data.aws_partition.current.partition}:ec2:*:${local.account_id}:instance/*"]
    condition {
      test     = "StringEquals"
      variable = "ec2:ResourceTag/project"
      values   = ["legatum-sims"]
    }
  }

  statement {
    sid       = "ReadOnlyEc2"
    actions   = ["ec2:DescribeInstances", "ec2:DescribeInstanceStatus", "ec2:DescribeSubnets", "ec2:DescribeSpotPriceHistory", "ec2:DescribeImages", "ec2:DescribeSecurityGroups", "ec2:DescribeAvailabilityZones", "ec2:DescribeInstanceTypes", "ec2:DescribeInstanceTypeOfferings"]
    resources = ["*"]
  }

  statement {
    sid       = "LatestAmazonLinuxAmi"
    actions   = ["ssm:GetParameter", "ssm:GetParameters"]
    resources = ["arn:${data.aws_partition.current.partition}:ssm:*::parameter/aws/service/ami-amazon-linux-latest/*"]
  }

  statement {
    sid       = "PassServerRoleToEc2"
    actions   = ["iam:PassRole"]
    resources = [aws_iam_role.server.arn]
    condition {
      test     = "StringEquals"
      variable = "iam:PassedToService"
      values   = ["ec2.amazonaws.com"]
    }
  }

  statement {
    sid       = "BucketList"
    actions   = ["s3:ListBucket", "s3:GetBucketLocation"]
    resources = [aws_s3_bucket.sims.arn]
  }
  statement {
    sid       = "BucketObjects"
    actions   = ["s3:GetObject", "s3:PutObject", "s3:DeleteObject"]
    resources = ["${aws_s3_bucket.sims.arn}/*"]
  }

  # Session Manager into a sims server for debugging.
  statement {
    sid       = "SessionManagerToSimsServers"
    actions   = ["ssm:StartSession"]
    resources = ["arn:${data.aws_partition.current.partition}:ec2:*:${local.account_id}:instance/*"]
    condition {
      test     = "StringEquals"
      variable = "ssm:resourceTag/project"
      values   = ["legatum-sims"]
    }
  }
  statement {
    sid     = "SessionManagerDocuments"
    actions = ["ssm:StartSession"]
    resources = [
      "arn:${data.aws_partition.current.partition}:ssm:*:*:document/SSM-SessionManagerRunShell",
      "arn:${data.aws_partition.current.partition}:ssm:*:*:document/AWS-StartInteractiveCommand",
    ]
  }
  statement {
    sid       = "SessionManagerHousekeeping"
    actions   = ["ssm:TerminateSession", "ssm:ResumeSession", "ssm:DescribeSessions", "ssm:DescribeInstanceInformation", "ssm:GetConnectionStatus"]
    resources = ["*"]
  }
}

# A managed policy, not an inline user policy: inline user policies are capped at 2,048 bytes,
# which this policy exceeds (managed policies allow 6,144).
resource "aws_iam_policy" "launcher" {
  name   = "legatum-sims-launch"
  policy = data.aws_iam_policy_document.launcher.json
}
resource "aws_iam_user_policy_attachment" "launcher" {
  user       = aws_iam_user.launcher.name
  policy_arn = aws_iam_policy.launcher.arn
}

