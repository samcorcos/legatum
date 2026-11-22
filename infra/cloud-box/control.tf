# What the Mac's tools/cloud/box.sh, remote.sh and verify-remote.sh may do, added to the existing launcher
# user (infra/cloud-sims) as a separate managed policy. The budget deny policy attached to the same user at
# 100% of the monthly cap also blocks ec2:StartInstances, so a spent budget cannot be bypassed by this box.
data "aws_iam_policy_document" "control" {
  statement {
    sid       = "StartStopTheBoxOnly"
    actions   = ["ec2:StartInstances", "ec2:StopInstances"]
    resources = ["arn:${local.partition}:ec2:${var.region}:${local.account_id}:instance/*"]
    condition {
      test     = "StringEquals"
      variable = "ec2:ResourceTag/project"
      values   = ["legatum-box"]
    }
  }

  # ssh over Session Manager (no inbound ports) and, if sshd is broken, a shell and send-command.
  statement {
    sid       = "SessionToTheBoxOnly"
    actions   = ["ssm:StartSession", "ssm:SendCommand"]
    resources = ["arn:${local.partition}:ec2:${var.region}:${local.account_id}:instance/*"]
    condition {
      test     = "StringEquals"
      variable = "ssm:resourceTag/project"
      values   = ["legatum-box"]
    }
  }
  statement {
    sid     = "SessionDocuments"
    actions = ["ssm:StartSession", "ssm:SendCommand"]
    resources = [
      "arn:${local.partition}:ssm:${var.region}::document/AWS-StartSSHSession",
      "arn:${local.partition}:ssm:${var.region}::document/AWS-RunShellScript",
    ]
  }
  statement {
    sid       = "CommandResults"
    actions   = ["ssm:GetCommandInvocation", "ssm:ListCommandInvocations"]
    resources = ["*"]
  }
}

resource "aws_iam_policy" "control" {
  name   = "legatum-box-control"
  policy = data.aws_iam_policy_document.control.json
}

resource "aws_iam_user_policy_attachment" "control" {
  user       = data.aws_iam_user.launcher.user_name
  policy_arn = aws_iam_policy.control.arn
}
