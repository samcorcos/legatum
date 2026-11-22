data "aws_caller_identity" "current" {}
data "aws_partition" "current" {}

locals {
  account_id  = data.aws_caller_identity.current.account_id
  bucket_name = "legatum-sims-${local.account_id}" # created by infra/cloud-sims; lifecycle expires everything after 30 days
  partition   = data.aws_partition.current.partition
}

data "aws_vpc" "default" {
  default = true
}

data "aws_subnet" "box" {
  vpc_id            = data.aws_vpc.default.id
  availability_zone = var.availability_zone
  default_for_az    = true
}

data "aws_s3_bucket" "sims" {
  bucket = local.bucket_name
}

data "aws_iam_user" "launcher" {
  user_name = var.launcher_user_name
}

# Ubuntu 24.04 arm64 (Canonical): Playwright WebKit and Chromium dependencies are supported on Ubuntu
# only (the sims servers run Amazon Linux). SSM agent is preinstalled.
data "aws_ssm_parameter" "ubuntu" {
  name = "/aws/service/canonical/ubuntu/server/24.04/stable/current/arm64/hvm/ebs-gp3/ami-id"
}

# ---------------------------------------------------------------------------------------------
# Instance role: this bucket's box/ prefix and the SSM agent. No EC2, no IAM, no GitHub, nothing else.
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

resource "aws_iam_role" "box" {
  name               = "legatum-box"
  assume_role_policy = data.aws_iam_policy_document.ec2_trust.json
}

data "aws_iam_policy_document" "box_bucket" {
  statement {
    sid       = "ListBoxPrefix"
    actions   = ["s3:ListBucket"]
    resources = [data.aws_s3_bucket.sims.arn]
    condition {
      test     = "StringLike"
      variable = "s3:prefix"
      values   = ["box/*"]
    }
  }
  statement {
    sid       = "ReadWriteBoxObjects"
    actions   = ["s3:GetObject", "s3:PutObject", "s3:AbortMultipartUpload"]
    resources = ["${data.aws_s3_bucket.sims.arn}/box/*"]
  }
}

resource "aws_iam_role_policy" "box_bucket" {
  name   = "box-prefix-only"
  role   = aws_iam_role.box.id
  policy = data.aws_iam_policy_document.box_bucket.json
}

resource "aws_iam_role_policy_attachment" "box_ssm" {
  role       = aws_iam_role.box.name
  policy_arn = "arn:${local.partition}:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_instance_profile" "box" {
  name = "legatum-box"
  role = aws_iam_role.box.name
}

resource "aws_security_group" "box" {
  name        = "legatum-box"
  description = "legatum build box: no inbound at all (SSM Session Manager only); outbound open for package mirrors, crates.io, npm, S3 and SSM"
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
    Name = "legatum-box"
  }
}

# ---------------------------------------------------------------------------------------------
# Persistent warm-cache volume (survives instance replacement and stops).
# ---------------------------------------------------------------------------------------------
resource "aws_ebs_volume" "data" {
  availability_zone = var.availability_zone
  size              = var.data_volume_gb
  type              = "gp3"
  iops              = var.data_volume_iops
  throughput        = var.data_volume_throughput_mbps
  encrypted         = true

  tags = {
    Name = "legatum-box-data"
    role = "build-box-data"
  }
}

resource "aws_volume_attachment" "data" {
  device_name = "/dev/sdf"
  volume_id   = aws_ebs_volume.data.id
  instance_id = aws_instance.box.id
}

# ---------------------------------------------------------------------------------------------
# The box.
# ---------------------------------------------------------------------------------------------
locals {
  user_data = templatefile("${path.module}/user-data.sh.tftpl", {
    data_volume_id = replace(aws_ebs_volume.data.id, "-", "")
    ssh_public_key = trimspace(var.ssh_public_key)
    idle_minutes   = var.idle_minutes
    idle_watch     = file("${path.module}/../../tools/cloud/box/idle-watch.sh")
  })
}

resource "aws_instance" "box" {
  ami                    = data.aws_ssm_parameter.ubuntu.value
  instance_type          = var.instance_type
  availability_zone      = var.availability_zone
  subnet_id              = data.aws_subnet.box.id
  vpc_security_group_ids = [aws_security_group.box.id]
  iam_instance_profile   = aws_iam_instance_profile.box.name

  # A shutdown from inside (the idle watcher, `shutdown -h`) STOPS the box, keeping the disks.
  # Spot rejects this attribute (persistent spot already stops on shutdown); on-demand sets it.
  instance_initiated_shutdown_behavior = var.use_spot ? null : "stop"

  # Needs a public address to reach SSM, S3 and the package mirrors without a NAT gateway ($0.005/h while running).
  associate_public_ip_address = true

  dynamic "instance_market_options" {
    for_each = var.use_spot ? [1] : []
    content {
      market_type = "spot"
      spot_options {
        spot_instance_type             = "persistent"
        instance_interruption_behavior = "stop"
      }
    }
  }

  metadata_options {
    http_tokens                 = "required" # IMDSv2 only
    http_endpoint               = "enabled"
    http_put_response_hop_limit = 2 # containers (docker builds) reach IMDSv2 for S3 credentials
  }

  root_block_device {
    volume_size           = var.root_volume_gb
    volume_type           = "gp3"
    encrypted             = true
    delete_on_termination = true
  }

  user_data = local.user_data

  volume_tags = {
    Name    = "legatum-box-root"
    project = "legatum-box"
  }

  tags = {
    Name = "legatum-box"
    role = "build-box"
  }

  lifecycle {
    # A newer Ubuntu AMI must never replace a box with a warm disk.
    # The cap Lambda keeps its counters in these tags.
    ignore_changes = [ami, associate_public_ip_address, volume_tags, tags["cap-day"], tags["cap-last"], tags["cap-used-min"], tags_all["cap-day"], tags_all["cap-last"], tags_all["cap-used-min"]]
  }
}
