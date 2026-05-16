output "region" {
  value = var.region
}

output "availability_zone" {
  value = var.availability_zone
}

output "vpc_id" {
  value = aws_vpc.spire_aws.id
}

output "subnet_id" {
  value = aws_subnet.spire_aws_data.id
}

output "coordinator_sg_id" {
  value = aws_security_group.coordinator.id
}

output "remote_sg_id" {
  value = aws_security_group.remote.id
}

output "artifact_bucket" {
  value = aws_s3_bucket.artifacts.bucket
}

output "coordinator_instance_id" {
  value = aws_instance.coordinator.id
}

output "coordinator_private_ip" {
  value = aws_instance.coordinator.private_ip
}

output "remote_instance_ids" {
  value = aws_instance.remote[*].id
}

output "remote_private_ips" {
  value = aws_instance.remote[*].private_ip
}

output "remote_secret_arns" {
  value = aws_secretsmanager_secret.remote[*].arn
}

output "topology" {
  description = "Phase 13 topology object consumed by scripts/spire-aws/*.sh and (future) ecaz aws ..."
  value = {
    region            = var.region
    availability_zone = var.availability_zone
    coordinator = {
      instance_id = aws_instance.coordinator.id
      private_ip  = aws_instance.coordinator.private_ip
    }
    remotes = [
      for i in range(var.remote_count) : {
        node_id     = i + 2
        instance_id = aws_instance.remote[i].id
        private_ip  = aws_instance.remote[i].private_ip
        secret_arn  = aws_secretsmanager_secret.remote[i].arn
        secret_name = aws_secretsmanager_secret.remote[i].name
      }
    ]
    artifact_bucket = aws_s3_bucket.artifacts.bucket
  }
}
