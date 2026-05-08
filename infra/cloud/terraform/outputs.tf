output "profile" {
  value = var.profile
}

output "region" {
  value = var.region
}

output "vpc_id" {
  value = aws_vpc.this.id
}

output "subnet_id" {
  value = aws_subnet.private.id
}

output "db_instance_id" {
  value = aws_instance.db.id
}

output "db_private_ip" {
  value = aws_instance.db.private_ip
}

output "db_volume_id" {
  value = aws_ebs_volume.db.id
}

output "loader_instance_id" {
  value = aws_instance.loader.id
}

output "s3_bucket" {
  value = aws_s3_bucket.this.bucket
}
