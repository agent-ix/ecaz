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
  description = "Amazon Linux 2023 AMI id. Operator picks the latest patched AMI for the region."
  type        = string
}

variable "coordinator_instance_type" {
  description = "EC2 instance type for the SPIRE coordinator. Phase 13a.1 default is r6i.4xlarge."
  type        = string
  default     = "r6i.4xlarge"
}

variable "remote_instance_type" {
  description = "EC2 instance type for each SPIRE remote. Phase 13a.1 default is r6i.2xlarge."
  type        = string
  default     = "r6i.2xlarge"
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
