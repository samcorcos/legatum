# AWS-side guard, independent of the box and of the Mac: every 5 minutes a Lambda (a) adds the elapsed minutes
# to a per-day counter kept in tags on the instance, and stops the box when the day's hard cap is reached;
# (b) stops it when it has been up for 40+ minutes with CPU under 3% for 30 minutes (the in-box idle watcher is
# the first line, this is the net under it). Free-tier sized: about 9,000 invocations a month.
data "archive_file" "cap" {
  type        = "zip"
  source_file = "${path.module}/cap/handler.py"
  output_path = "${path.module}/.build/cap.zip"
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

resource "aws_iam_role" "cap" {
  name               = "legatum-box-cap"
  assume_role_policy = data.aws_iam_policy_document.lambda_trust.json
}

resource "aws_cloudwatch_log_group" "cap" {
  name              = "/aws/lambda/legatum-box-cap"
  retention_in_days = 14
}

data "aws_iam_policy_document" "cap" {
  statement {
    actions   = ["ec2:DescribeInstances", "cloudwatch:GetMetricStatistics"]
    resources = ["*"]
  }
  statement {
    actions   = ["ec2:StopInstances", "ec2:CreateTags"]
    resources = ["arn:${local.partition}:ec2:${var.region}:${local.account_id}:instance/*"]
    condition {
      test     = "StringEquals"
      variable = "ec2:ResourceTag/project"
      values   = ["legatum-box"]
    }
  }
  statement {
    actions   = ["logs:CreateLogStream", "logs:PutLogEvents"]
    resources = ["${aws_cloudwatch_log_group.cap.arn}:*"]
  }
}

resource "aws_iam_role_policy" "cap" {
  name   = "stop-the-box"
  role   = aws_iam_role.cap.id
  policy = data.aws_iam_policy_document.cap.json
}

resource "aws_lambda_function" "cap" {
  function_name    = "legatum-box-cap"
  role             = aws_iam_role.cap.arn
  runtime          = "python3.12"
  architectures    = ["arm64"]
  handler          = "handler.handler"
  filename         = data.archive_file.cap.output_path
  source_code_hash = data.archive_file.cap.output_base64sha256
  timeout          = 60
  memory_size      = 128

  environment {
    variables = {
      DAILY_CAP_MINUTES = tostring(var.daily_cap_hours * 60)
      INTERVAL_MINUTES  = "5"
    }
  }

  depends_on = [aws_cloudwatch_log_group.cap]
}

resource "aws_cloudwatch_event_rule" "cap" {
  name                = "legatum-box-cap"
  schedule_expression = "rate(5 minutes)"
}

resource "aws_cloudwatch_event_target" "cap" {
  rule = aws_cloudwatch_event_rule.cap.name
  arn  = aws_lambda_function.cap.arn
}

resource "aws_lambda_permission" "cap" {
  statement_id  = "AllowEventBridge"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.cap.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.cap.arn
}
