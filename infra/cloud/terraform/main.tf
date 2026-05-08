locals {
  name        = "ecaz-cloud-${var.profile}"
  common_tags = merge(var.tags, {
    "ecaz:profile" = var.profile
    "ecaz:harness" = "cloud"
  })
}

# ---------------------------------------------------------------------------
# Network — single VPC, single private subnet, S3 + SSM VPC endpoints, no NAT.
# ---------------------------------------------------------------------------

resource "aws_vpc" "this" {
  cidr_block           = var.vpc_cidr
  enable_dns_support   = true
  enable_dns_hostnames = true
  tags                 = merge(local.common_tags, { Name = local.name })
}

resource "aws_subnet" "private" {
  vpc_id            = aws_vpc.this.id
  cidr_block        = var.subnet_cidr
  availability_zone = var.az
  tags              = merge(local.common_tags, { Name = "${local.name}-private" })
}

resource "aws_route_table" "private" {
  vpc_id = aws_vpc.this.id
  tags   = merge(local.common_tags, { Name = "${local.name}-private" })
}

resource "aws_route_table_association" "private" {
  subnet_id      = aws_subnet.private.id
  route_table_id = aws_route_table.private.id
}

resource "aws_vpc_endpoint" "s3" {
  vpc_id            = aws_vpc.this.id
  service_name      = "com.amazonaws.${var.region}.s3"
  vpc_endpoint_type = "Gateway"
  route_table_ids   = [aws_route_table.private.id]
  tags              = local.common_tags
}

resource "aws_security_group" "endpoints" {
  name   = "${local.name}-endpoints"
  vpc_id = aws_vpc.this.id
  tags   = local.common_tags

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# SSM Session Manager and friends require these three interface endpoints.
locals {
  interface_endpoints = ["ssm", "ssmmessages", "ec2messages"]
}

resource "aws_vpc_endpoint" "interface" {
  for_each            = toset(local.interface_endpoints)
  vpc_id              = aws_vpc.this.id
  service_name        = "com.amazonaws.${var.region}.${each.value}"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private.id]
  security_group_ids  = [aws_security_group.endpoints.id]
  private_dns_enabled = true
  tags                = local.common_tags
}

# ---------------------------------------------------------------------------
# Security groups — DB accepts Postgres only from inside the VPC; no SSH.
# ---------------------------------------------------------------------------

resource "aws_security_group" "db" {
  name        = "${local.name}-db"
  description = "ecaz DB host: Postgres in-VPC only, egress open"
  vpc_id      = aws_vpc.this.id
  tags        = local.common_tags

  ingress {
    description = "Postgres from inside the VPC"
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "loader" {
  name        = "${local.name}-loader"
  description = "ecaz loader host: egress only"
  vpc_id      = aws_vpc.this.id
  tags        = local.common_tags

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# ---------------------------------------------------------------------------
# IAM — both EC2s get SSM + S3 read/write to the harness bucket.
# ---------------------------------------------------------------------------

data "aws_iam_policy_document" "ec2_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "instance" {
  name               = "${local.name}-instance"
  assume_role_policy = data.aws_iam_policy_document.ec2_assume.json
  tags               = local.common_tags
}

resource "aws_iam_role_policy_attachment" "ssm" {
  role       = aws_iam_role.instance.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

data "aws_iam_policy_document" "bucket_rw" {
  statement {
    actions   = ["s3:ListBucket"]
    resources = [aws_s3_bucket.this.arn]
  }
  statement {
    actions   = ["s3:GetObject", "s3:PutObject", "s3:DeleteObject"]
    resources = ["${aws_s3_bucket.this.arn}/*"]
  }
}

resource "aws_iam_role_policy" "bucket_rw" {
  name   = "${local.name}-bucket-rw"
  role   = aws_iam_role.instance.id
  policy = data.aws_iam_policy_document.bucket_rw.json
}

resource "aws_iam_instance_profile" "instance" {
  name = "${local.name}-instance"
  role = aws_iam_role.instance.name
}

# ---------------------------------------------------------------------------
# S3 bucket for parquet shards and bench artifacts.
# ---------------------------------------------------------------------------

resource "random_id" "bucket" {
  byte_length = 4
}

resource "aws_s3_bucket" "this" {
  bucket        = "${local.name}-${random_id.bucket.hex}"
  force_destroy = false
  tags          = local.common_tags
}

resource "aws_s3_bucket_public_access_block" "this" {
  bucket                  = aws_s3_bucket.this.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_lifecycle_configuration" "this" {
  bucket = aws_s3_bucket.this.id

  rule {
    id     = "expire-raw-parquet"
    status = "Enabled"

    filter {
      prefix = "parquet/"
    }

    expiration {
      days = var.parquet_retention_days
    }
  }
}

# ---------------------------------------------------------------------------
# AMIs — Amazon Linux 2023 on aarch64 (Graviton). Both DB and loader.
# ---------------------------------------------------------------------------

data "aws_ami" "al2023_arm64" {
  most_recent = true
  owners      = ["amazon"]

  filter {
    name   = "name"
    values = ["al2023-ami-2023.*-arm64"]
  }

  filter {
    name   = "architecture"
    values = ["arm64"]
  }
}

# ---------------------------------------------------------------------------
# Database EC2 + EBS volume.
# ---------------------------------------------------------------------------

resource "aws_ebs_volume" "db" {
  availability_zone = var.az
  size              = var.db_volume_gb
  type              = "gp3"
  iops              = var.db_volume_iops
  throughput        = var.db_volume_throughput
  encrypted         = true
  snapshot_id       = var.from_snapshot_id != "" ? var.from_snapshot_id : null
  tags              = merge(local.common_tags, { Name = "${local.name}-db-data" })

  lifecycle {
    # Snapshot id is only meaningful on creation; allow `from_snapshot_id`
    # changes without forcing a replace.
    ignore_changes = [snapshot_id]
  }
}

locals {
  cloud_init_db = templatefile("${path.module}/cloud-init/db.sh.tftpl", {
    ecaz_git_url = var.ecaz_git_url
    ecaz_git_ref = var.ecaz_git_ref
    bucket       = aws_s3_bucket.this.bucket
    profile      = var.profile
  })
}

resource "aws_instance" "db" {
  ami                         = data.aws_ami.al2023_arm64.id
  instance_type               = var.db_instance_type
  subnet_id                   = aws_subnet.private.id
  vpc_security_group_ids      = [aws_security_group.db.id]
  iam_instance_profile        = aws_iam_instance_profile.instance.name
  user_data                   = local.cloud_init_db
  user_data_replace_on_change = false
  associate_public_ip_address = false

  root_block_device {
    volume_size = 20
    volume_type = "gp3"
    encrypted   = true
  }

  tags = merge(local.common_tags, { Name = "${local.name}-db" })
}

resource "aws_volume_attachment" "db_data" {
  device_name = "/dev/sdf"
  volume_id   = aws_ebs_volume.db.id
  instance_id = aws_instance.db.id
}

# ---------------------------------------------------------------------------
# Loader EC2.
# ---------------------------------------------------------------------------

resource "aws_instance" "loader" {
  ami                         = data.aws_ami.al2023_arm64.id
  instance_type               = var.loader_instance_type
  subnet_id                   = aws_subnet.private.id
  vpc_security_group_ids      = [aws_security_group.loader.id]
  iam_instance_profile        = aws_iam_instance_profile.instance.name
  associate_public_ip_address = false

  root_block_device {
    volume_size = 100
    volume_type = "gp3"
    encrypted   = true
  }

  user_data = templatefile("${path.module}/cloud-init/loader.sh.tftpl", {
    ecaz_git_url = var.ecaz_git_url
    ecaz_git_ref = var.ecaz_git_ref
    bucket       = aws_s3_bucket.this.bucket
  })

  tags = merge(local.common_tags, { Name = "${local.name}-loader" })
}
