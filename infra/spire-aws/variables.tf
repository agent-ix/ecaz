variable "region" {
  description = "AWS region for the Phase 13 verification topology."
  type        = string
}

variable "availability_zone" {
  description = "AZ for the baseline single-AZ topology decided in Phase 13a.1."
  type        = string
}

variable "vpc_cidr" {
  description = "CIDR for the Phase 13 VPC."
  type        = string
  default     = "10.42.0.0/16"
}

variable "subnet_cidr" {
  description = "CIDR for the private data-plane subnet."
  type        = string
  default     = "10.42.1.0/24"
}

variable "ami_id" {
  description = "Amazon Linux 2023 arm64 AMI id. SPIRE AWS verification follows the repo AWS Graviton/aarch64 lane."
  type        = string
}

variable "coordinator_instance_type" {
  description = "Graviton EC2 instance type for the SPIRE coordinator. Phase 13e correctness default is m7g.large; representative/stress runs override to r7g.* after quota proof."
  type        = string
  default     = "m7g.large"

  validation {
    condition     = can(regex("^(m7g|m8g|r7g|c7g|c8g)\\.", var.coordinator_instance_type))
    error_message = "SPIRE AWS verification must use the established Graviton/aarch64 lane: m7g, m8g, r7g, c7g, or c8g."
  }
}

variable "remote_instance_type" {
  description = "Graviton EC2 instance type for each SPIRE remote. Phase 13e correctness default is m7g.large; representative/stress runs override to r7g.* after quota proof."
  type        = string
  default     = "m7g.large"

  validation {
    condition     = can(regex("^(m7g|m8g|r7g|c7g|c8g)\\.", var.remote_instance_type))
    error_message = "SPIRE AWS verification must use the established Graviton/aarch64 lane: m7g, m8g, r7g, c7g, or c8g."
  }
}

variable "remote_count" {
  description = "Number of SPIRE remote nodes. Phase 13a.1 default is 3."
  type        = number
  default     = 3
}

variable "coordinator_storage_gb" {
  description = "gp3 root volume size for the coordinator."
  type        = number
  default     = 200
}

variable "remote_storage_gb" {
  description = "gp3 root volume size for each remote."
  type        = number
  default     = 100
}

variable "coordinator_extra_store_volume_count" {
  description = "Additional gp3 EBS volumes attached to the coordinator for SPIRE local-store tablespace benchmarks."
  type        = number
  default     = 0

  validation {
    condition     = var.coordinator_extra_store_volume_count >= 0 && var.coordinator_extra_store_volume_count <= 8
    error_message = "coordinator_extra_store_volume_count must be between 0 and 8."
  }
}

variable "coordinator_extra_store_volume_gb" {
  description = "Size in GiB for each additional coordinator local-store benchmark volume."
  type        = number
  default     = 200
}

variable "owner" {
  description = "Owner handle for the cost-tag set defined in Phase 13a.8."
  type        = string
}

variable "auto_stop_at" {
  description = "ISO-8601 deadline for the AutoStop cost tag."
  type        = string
}

variable "phase_label" {
  description = "Phase tag value applied to every resource."
  type        = string
  default     = "13-spire-aws-verification"
}

variable "key_name" {
  description = "Optional EC2 key pair name. Session Manager is the primary access path; SSH is not required."
  type        = string
  default     = null
}
