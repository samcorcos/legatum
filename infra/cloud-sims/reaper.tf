# AWS-side reaper: every 15 minutes, terminate legatum-sims servers past their expiry tag (plus a
# grace period). Cost: a few thousand Lambda invocations a month, inside the free tier.
data "archive_file" "reaper" {
  type        = "zip"
  source_file = "${path.module}/reaper/handler.py"
  output_path = "${path.module}/.build/reaper.zip"
}

data "aws_iam_policy_document" "lambda_trust" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "reaper" {
  name               = "legatum-sims-reaper"
  assume_role_policy = data.aws_iam_policy_document.lambda_trust.json
}

data "aws_iam_policy_document" "reaper" {
  statement {
    actions   = ["ec2:DescribeInstances"]
    resources = ["*"]
  }
  statement {
    actions   = ["ec2:TerminateInstances"]
    resources = ["arn:${data.aws_partition.current.partition}:ec2:*:${local.account_id}:instance/*"]
    condition {
      test     = "StringEquals"
      variable = "ec2:ResourceTag/project"
      values   = ["legatum-sims"]
    }
  }
  statement {
    actions   = ["logs:CreateLogStream", "logs:PutLogEvents"]
    resources = ["${aws_cloudwatch_log_group.reaper.arn}:*"]
  }
}

resource "aws_iam_role_policy" "reaper" {
  name   = "terminate-expired-sims"
  role   = aws_iam_role.reaper.id
  policy = data.aws_iam_policy_document.reaper.json
}

resource "aws_cloudwatch_log_group" "reaper" {
  name              = "/aws/lambda/legatum-sims-reaper"
  retention_in_days = 14
}

resource "aws_lambda_function" "reaper" {
  function_name    = "legatum-sims-reaper"
  role             = aws_iam_role.reaper.arn
  runtime          = "python3.12"
  architectures    = ["arm64"]
  handler          = "handler.handler"
  filename         = data.archive_file.reaper.output_path
  source_code_hash = data.archive_file.reaper.output_base64sha256
  timeout          = 60
  memory_size      = 128

  environment {
    variables = {
      GRACE_MINUTES  = tostring(var.reaper_grace_minutes)
      FALLBACK_HOURS = "6"
    }
  }

  depends_on = [aws_cloudwatch_log_group.reaper]
}

resource "aws_cloudwatch_event_rule" "reaper" {
  name                = "legatum-sims-reaper"
  schedule_expression = "rate(15 minutes)"
}

resource "aws_cloudwatch_event_target" "reaper" {
  rule = aws_cloudwatch_event_rule.reaper.name
  arn  = aws_lambda_function.reaper.arn
}

resource "aws_lambda_permission" "reaper" {
  statement_id  = "AllowEventBridge"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.reaper.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.reaper.arn
}
