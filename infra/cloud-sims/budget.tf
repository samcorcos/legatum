# Monthly cost cap: email alerts at 50%, 80% and 100% of actual spend (and a forecast of 100%), and at
# 100% a Budget action attaches a deny policy to the launcher so no new server can start. Billing data
# lags by hours, so this is the backstop; the per-batch max-hours self-destruct, the reaper and the
# spot vCPU quota are what keep a normal month far below the cap.
resource "aws_budgets_budget" "monthly" {
  name         = "legatum-sims-monthly"
  budget_type  = "COST"
  limit_amount = tostring(var.monthly_budget_usd)
  limit_unit   = "USD"
  time_unit    = "MONTHLY"

  dynamic "notification" {
    for_each = [50, 80, 100]
    content {
      comparison_operator        = "GREATER_THAN"
      threshold                  = notification.value
      threshold_type             = "PERCENTAGE"
      notification_type          = "ACTUAL"
      subscriber_email_addresses = [var.alert_email]
    }
  }

  notification {
    comparison_operator        = "GREATER_THAN"
    threshold                  = 100
    threshold_type             = "PERCENTAGE"
    notification_type          = "FORECASTED"
    subscriber_email_addresses = [var.alert_email]
  }
}

data "aws_iam_policy_document" "deny_launch" {
  statement {
    sid    = "BudgetExhaustedNoNewServers"
    effect = "Deny"
    actions = [
      "ec2:RunInstances",
      "ec2:RequestSpotInstances",
      "ec2:RequestSpotFleet",
      "ec2:CreateFleet",
      "ec2:StartInstances",
    ]
    resources = ["*"]
  }
}

resource "aws_iam_policy" "deny_launch" {
  name        = "legatum-sims-deny-launch"
  description = "Attached to the launcher by the budget action when the monthly cap is reached."
  policy      = data.aws_iam_policy_document.deny_launch.json
}

data "aws_iam_policy_document" "budgets_trust" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["budgets.amazonaws.com"]
    }
    condition {
      test     = "StringEquals"
      variable = "aws:SourceAccount"
      values   = [local.account_id]
    }
  }
}

data "aws_iam_policy_document" "budgets_action" {
  statement {
    actions   = ["iam:AttachUserPolicy", "iam:DetachUserPolicy"]
    resources = [aws_iam_user.launcher.arn]
    condition {
      test     = "ArnEquals"
      variable = "iam:PolicyARN"
      values   = [aws_iam_policy.deny_launch.arn]
    }
  }
  statement {
    actions   = ["iam:ListAttachedUserPolicies"]
    resources = [aws_iam_user.launcher.arn]
  }
}

resource "aws_iam_role" "budget_action" {
  name               = "legatum-sims-budget-action"
  assume_role_policy = data.aws_iam_policy_document.budgets_trust.json
}

resource "aws_iam_role_policy" "budget_action" {
  name   = "attach-deny-to-launcher"
  role   = aws_iam_role.budget_action.id
  policy = data.aws_iam_policy_document.budgets_action.json
}

resource "aws_budgets_budget_action" "deny_launch_at_cap" {
  budget_name        = aws_budgets_budget.monthly.name
  action_type        = "APPLY_IAM_POLICY"
  approval_model     = "AUTOMATIC"
  notification_type  = "ACTUAL"
  execution_role_arn = aws_iam_role.budget_action.arn

  action_threshold {
    action_threshold_type  = "PERCENTAGE"
    action_threshold_value = 100
  }

  definition {
    iam_action_definition {
      policy_arn = aws_iam_policy.deny_launch.arn
      users      = [aws_iam_user.launcher.name]
    }
  }

  subscriber {
    subscription_type = "EMAIL"
    address           = var.alert_email
  }

  depends_on = [aws_iam_role_policy.budget_action]
}
