data "aws_caller_identity" "current" {}
data "aws_partition" "current" {}
data "aws_vpc" "default" {
  default = true
}

locals {
  account_id  = data.aws_caller_identity.current.account_id
  bucket_name = "legatum-sims-${local.account_id}"
}

# ---------------------------------------------------------------------------------------------
# Bucket: simulator binaries in, results out. Everything expires; nothing is public.
# ---------------------------------------------------------------------------------------------
resource "aws_s3_bucket" "sims" {
  bucket        = local.bucket_name
  force_destroy = true # transient data only: `terraform destroy` must not get stuck on objects
}

resource "aws_s3_bucket_public_access_block" "sims" {
  bucket                  = aws_s3_bucket.sims.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_ownership_controls" "sims" {
  bucket = aws_s3_bucket.sims.id
  rule {
    object_ownership = "BucketOwnerEnforced"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "sims" {
  bucket = aws_s3_bucket.sims.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "sims" {
  bucket = aws_s3_bucket.sims.id

  rule {
    id     = "expire-everything"
    status = "Enabled"
    filter {}

    expiration {
      days = var.results_expire_days
    }
    abort_incomplete_multipart_upload {
      days_after_initiation = 2
    }
  }
}

data "aws_iam_policy_document" "bucket_tls_only" {
  statement {
    sid       = "DenyInsecureTransport"
    effect    = "Deny"
    actions   = ["s3:*"]
    resources = [aws_s3_bucket.sims.arn, "${aws_s3_bucket.sims.arn}/*"]
    principals {
      type        = "*"
      identifiers = ["*"]
    }
    condition {
      test     = "Bool"
      variable = "aws:SecureTransport"
      values   = ["false"]
    }
  }
}

resource "aws_s3_bucket_policy" "sims" {
  bucket     = aws_s3_bucket.sims.id
  policy     = data.aws_iam_policy_document.bucket_tls_only.json
  depends_on = [aws_s3_bucket_public_access_block.sims]
}

# ---------------------------------------------------------------------------------------------
# Servers: an instance role that can read and write the bucket and nothing else (plus the SSM
# agent's own permissions, so Session Manager works without any inbound port), and a security
# group with no inbound rules at all.
# ---------------------------------------------------------------------------------------------
data "aws_iam_policy_document" "ec2_trust" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "server" {
  name               = "legatum-sims-server"
  assume_role_policy = data.aws_iam_policy_document.ec2_trust.json
}

data "aws_iam_policy_document" "server_bucket" {
  statement {
    sid       = "ListBucket"
    actions   = ["s3:ListBucket", "s3:GetBucketLocation"]
    resources = [aws_s3_bucket.sims.arn]
  }
  statement {
    sid       = "ReadWriteObjects"
    actions   = ["s3:GetObject", "s3:PutObject", "s3:AbortMultipartUpload"]
    resources = ["${aws_s3_bucket.sims.arn}/*"]
  }
}

resource "aws_iam_role_policy" "server_bucket" {
  name   = "bucket-only"
  role   = aws_iam_role.server.id
  policy = data.aws_iam_policy_document.server_bucket.json
}

resource "aws_iam_role_policy_attachment" "server_ssm" {
  role       = aws_iam_role.server.name
  policy_arn = "arn:${data.aws_partition.current.partition}:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_instance_profile" "server" {
  name = "legatum-sims-server"
  role = aws_iam_role.server.name
}

resource "aws_security_group" "server" {
  name        = "legatum-sims-server"
  description = "legatum-sims servers: no inbound at all (debug through SSM Session Manager); outbound open for S3, SSM and package mirrors"
  vpc_id      = data.aws_vpc.default.id

  # Deliberately no ingress blocks.
  egress {
    description = "all outbound"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "legatum-sims-server"
  }
}

# EC2 Spot needs its service-linked role once per account; a first spot request would otherwise
# fail for a launcher that may not create roles.
resource "aws_iam_service_linked_role" "spot" {
  count            = var.create_spot_service_linked_role ? 1 : 0
  aws_service_name = "spot.amazonaws.com"
}

# New accounts start with a handful of spot vCPUs. The quota is also the hardest possible cost
# ceiling: the account can never run more than this many vCPUs at once.
resource "aws_servicequotas_service_quota" "spot_vcpus" {
  count        = var.spot_vcpu_quota > 0 ? 1 : 0
  service_code = "ec2"
  quota_code   = "L-34B43A08" # All Standard (A, C, D, H, I, M, R, T, Z) Spot Instance Requests
  value        = var.spot_vcpu_quota
}
