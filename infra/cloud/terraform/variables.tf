variable "profile" {
  description = "Profile name (10k, dev, 1m, 10m, 100m). Must match the tfvars file basename."
  type        = string
}

variable "region" {
  description = "AWS region. Pick a region close to the operator and to dataset mirrors."
  type        = string
  default     = "us-east-1"
}

variable "az" {
  description = "Availability zone for the private subnet."
  type        = string
  default     = "us-east-1a"
}

variable "vpc_cidr" {
  description = "CIDR for the harness VPC."
  type        = string
  default     = "10.42.0.0/16"
}

variable "subnet_cidr" {
  description = "CIDR for the single private subnet."
  type        = string
  default     = "10.42.1.0/24"
}

variable "db_instance_type" {
  description = "Graviton EC2 instance type for the database host."
  type        = string
}

variable "db_volume_gb" {
  description = "gp3 EBS volume size for the database host."
  type        = number
}

variable "db_volume_iops" {
  description = "gp3 IOPS for the database volume."
  type        = number
  default     = 3000
}

variable "db_volume_throughput" {
  description = "gp3 throughput (MiB/s) for the database volume."
  type        = number
  default     = 125
}

variable "loader_instance_type" {
  description = "Graviton EC2 instance type for the corpus loader."
  type        = string
  default     = "c7g.2xlarge"
}

variable "ecaz_git_ref" {
  description = "Git ref of ecaz to install on the DB host (sha, branch, or tag)."
  type        = string
}

variable "ecaz_git_url" {
  description = "Git URL to clone on the DB host."
  type        = string
  default     = "https://github.com/agent-ix/ecaz.git"
}

variable "parquet_retention_days" {
  description = "S3 lifecycle rule: days before raw parquet is expired. Bench artifacts live under a separate prefix and are not expired here."
  type        = number
  default     = 30
}

variable "from_snapshot_id" {
  description = "Optional EBS snapshot id to restore the DB volume from. Empty creates a fresh volume."
  type        = string
  default     = ""
}

variable "tags" {
  description = "Tags applied to every resource. ecaz:profile is always added."
  type        = map(string)
  default     = {}
}

variable "enable_eice_ssh" {
  description = <<-EOT
    Provision an EC2 Instance Connect Endpoint and an SSH ingress rule
    on the DB security group. When true, operators can
    `aws ec2-instance-connect ssh --instance-id <id>` into the
    private-subnet DB host without a NAT gateway or public IP. Acts as
    a fallback when SSM Session Manager is wedged (observed during
    heavy cargo bench compiles on small Graviton hosts during the
    2026-05-16 Graviton baseline cycle). Costs ~$0.10/hr while the
    endpoint exists.
  EOT
  type        = bool
  default     = false
}

variable "ssh_key_name" {
  description = <<-EOT
    Optional AWS key pair name to bake into the DB and loader instances
    for direct SSH (in addition to EC2 Instance Connect's runtime key
    push). Only useful if `enable_eice_ssh = true`. Empty string
    disables.
  EOT
  type        = string
  default     = ""
}
