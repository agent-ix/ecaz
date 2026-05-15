locals {
  tags = {
    Project  = "ecaz"
    Phase    = var.phase_label
    Owner    = var.owner
    AutoStop = var.auto_stop_at
  }
}

resource "aws_vpc" "spire_aws" {
  cidr_block           = var.vpc_cidr
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags = { Name = "ecaz-spire-aws" }
}

resource "aws_subnet" "spire_aws_data" {
  vpc_id            = aws_vpc.spire_aws.id
  cidr_block        = var.subnet_cidr
  availability_zone = var.availability_zone

  tags = { Name = "ecaz-spire-aws-data" }
}

resource "aws_route_table" "spire_aws_data" {
  vpc_id = aws_vpc.spire_aws.id
  tags   = { Name = "ecaz-spire-aws-data" }
}

resource "aws_route_table_association" "spire_aws_data" {
  subnet_id      = aws_subnet.spire_aws_data.id
  route_table_id = aws_route_table.spire_aws_data.id
}

resource "aws_vpc_endpoint" "s3" {
  vpc_id            = aws_vpc.spire_aws.id
  service_name      = "com.amazonaws.${var.region}.s3"
  vpc_endpoint_type = "Gateway"
  route_table_ids   = [aws_route_table.spire_aws_data.id]
}

resource "aws_vpc_endpoint" "ssm" {
  vpc_id              = aws_vpc.spire_aws.id
  service_name        = "com.amazonaws.${var.region}.ssm"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.spire_aws_data.id]
  security_group_ids  = [aws_security_group.endpoint.id]
  private_dns_enabled = true
}

resource "aws_vpc_endpoint" "ssmmessages" {
  vpc_id              = aws_vpc.spire_aws.id
  service_name        = "com.amazonaws.${var.region}.ssmmessages"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.spire_aws_data.id]
  security_group_ids  = [aws_security_group.endpoint.id]
  private_dns_enabled = true
}

resource "aws_vpc_endpoint" "ec2messages" {
  vpc_id              = aws_vpc.spire_aws.id
  service_name        = "com.amazonaws.${var.region}.ec2messages"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.spire_aws_data.id]
  security_group_ids  = [aws_security_group.endpoint.id]
  private_dns_enabled = true
}

resource "aws_vpc_endpoint" "secretsmanager" {
  vpc_id              = aws_vpc.spire_aws.id
  service_name        = "com.amazonaws.${var.region}.secretsmanager"
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.spire_aws_data.id]
  security_group_ids  = [aws_security_group.endpoint.id]
  private_dns_enabled = true
}

resource "aws_security_group" "endpoint" {
  name        = "ecaz-spire-aws-endpoint"
  description = "Allow data-plane SGs to reach interface VPC endpoints."
  vpc_id      = aws_vpc.spire_aws.id

  ingress {
    description = "HTTPS from the data plane."
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = [var.subnet_cidr]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "coordinator" {
  name        = "ecaz-spire-aws-coord"
  description = "SPIRE coordinator: no public ingress; egress to remote SG on 5432."
  vpc_id      = aws_vpc.spire_aws.id
  tags        = { Name = "ecaz-spire-aws-coord" }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "remote" {
  name        = "ecaz-spire-aws-remote"
  description = "SPIRE remote: ingress only from the coordinator SG on 5432."
  vpc_id      = aws_vpc.spire_aws.id
  tags        = { Name = "ecaz-spire-aws-remote" }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group_rule" "coord_to_remote_pg" {
  type                     = "ingress"
  from_port                = 5432
  to_port                  = 5432
  protocol                 = "tcp"
  security_group_id        = aws_security_group.remote.id
  source_security_group_id = aws_security_group.coordinator.id
  description              = "PG from coordinator to remote."
}

resource "aws_s3_bucket" "artifacts" {
  bucket_prefix = "ecaz-spire-aws-"
}

resource "aws_s3_bucket_versioning" "artifacts" {
  bucket = aws_s3_bucket.artifacts.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "artifacts" {
  bucket = aws_s3_bucket.artifacts.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_iam_role" "node" {
  name = "ecaz-spire-aws-node"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "ec2.amazonaws.com" }
      Action    = "sts:AssumeRole"
    }]
  })
}

resource "aws_iam_role_policy_attachment" "node_ssm" {
  role       = aws_iam_role.node.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_role_policy" "node_artifacts_secrets" {
  name = "ecaz-spire-aws-node-artifacts-secrets"
  role = aws_iam_role.node.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect   = "Allow"
        Action   = ["s3:GetObject", "s3:PutObject", "s3:ListBucket"]
        Resource = [aws_s3_bucket.artifacts.arn, "${aws_s3_bucket.artifacts.arn}/*"]
      },
      {
        Effect   = "Allow"
        Action   = ["secretsmanager:GetSecretValue", "secretsmanager:DescribeSecret"]
        Resource = aws_secretsmanager_secret.remote[*].arn
      },
    ]
  })
}

resource "aws_iam_instance_profile" "node" {
  name = "ecaz-spire-aws-node"
  role = aws_iam_role.node.name
}

resource "random_password" "remote" {
  count   = var.remote_count
  length  = 32
  special = false
}

resource "aws_secretsmanager_secret" "remote" {
  count = var.remote_count
  name  = "ecaz-spire-aws-remote-${count.index + 1}"
}

resource "aws_secretsmanager_secret_version" "remote" {
  count     = var.remote_count
  secret_id = aws_secretsmanager_secret.remote[count.index].id

  secret_string = jsonencode({
    host        = aws_instance.remote[count.index].private_ip
    port        = 5432
    dbname      = "postgres"
    user        = "ecaz_coord"
    password    = random_password.remote[count.index].result
    sslmode     = "verify-full"
    sslrootcert = "/etc/ssl/certs/ecaz-spire-aws-ca.pem"
  })
}

resource "aws_instance" "coordinator" {
  ami                    = var.ami_id
  instance_type          = var.coordinator_instance_type
  subnet_id              = aws_subnet.spire_aws_data.id
  vpc_security_group_ids = [aws_security_group.coordinator.id]
  iam_instance_profile   = aws_iam_instance_profile.node.name
  key_name               = var.key_name

  root_block_device {
    volume_type           = "gp3"
    volume_size           = var.coordinator_storage_gb
    iops                  = 3000
    throughput            = 125
    delete_on_termination = true
    encrypted             = true
  }

  metadata_options {
    http_tokens = "required"
  }

  tags = { Name = "ecaz-spire-aws-coord", Role = "coordinator" }
}

resource "aws_instance" "remote" {
  count                  = var.remote_count
  ami                    = var.ami_id
  instance_type          = var.remote_instance_type
  subnet_id              = aws_subnet.spire_aws_data.id
  vpc_security_group_ids = [aws_security_group.remote.id]
  iam_instance_profile   = aws_iam_instance_profile.node.name
  key_name               = var.key_name

  root_block_device {
    volume_type           = "gp3"
    volume_size           = var.remote_storage_gb
    iops                  = 3000
    throughput            = 125
    delete_on_termination = true
    encrypted             = true
  }

  metadata_options {
    http_tokens = "required"
  }

  tags = { Name = "ecaz-spire-aws-remote-${count.index + 1}", Role = "remote" }
}
