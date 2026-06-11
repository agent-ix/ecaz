---
id: FR-045
title: Cloud Terraform-Managed Infrastructure
type: functional-requirement
artifact_type: FR
status: PROPOSED
object_type: infrastructure
relationships:
  - target: "ix://agent-ix/ecaz/US-021"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-044"
    type: "supports"
    cardinality: "1:1"
---
# FR-045: Cloud Terraform-Managed Infrastructure

## Requirement

The cloud harness SHALL provision all AWS resources via a single
Terraform module rooted at `infra/cloud/terraform/`, with one state
file per profile so multiple sizes can coexist.

## Behavior

1. Profile selection SHALL be driven by `tfvars` files committed at
   `infra/cloud/terraform/profiles/<name>.tfvars`. Defined profiles:
   `10k`, `dev`, `1m`, `10m`, `100m`. A `1b` placeholder is reserved.
2. The module SHALL provision: a VPC with one private subnet (no
   NAT), an S3 VPC endpoint, an S3 bucket for parquet and bench
   artifacts, a DB EC2 instance, a loader EC2 instance, an EBS
   `gp3` volume sized per profile, security groups, and IAM roles
   for SSM + S3 access on both EC2s.
3. EC2 instance families SHALL be Graviton (`m7g`/`c7g`/`r7g`/`r8g`)
   for both DB and loader hosts. The Postgres host AMI SHALL boot a
   `aarch64-unknown-linux-gnu` userspace.
4. SSH SHALL NOT be exposed on any instance. Operator shell access
   SHALL be via SSM Session Manager only.
5. The DB host's `cloud-init` SHALL install Postgres 18, fetch the
   ecaz source at the SHA passed via user-data, and run
   `cargo pgrx install --release` (mirroring the local
   `crates/ecaz-cli/src/dev/install.rs:68` invocation).
6. The S3 bucket SHALL have a lifecycle rule that expires raw
   parquet shards after a configurable retention (default 30 days)
   while retaining bench artifacts indefinitely under a separate
   prefix.
7. Profile defaults (subject to tuning during checkpoint 8):

   | Profile | Rows | Instance | EBS gp3 |
   |---|---|---|---|
   | `10k` | 10k | `m7g.large` | 20 GB |
   | `dev` | 50k–1M | `m7g.large` | 50 GB |
   | `1m` | 1M | `m7g.xlarge` | 100 GB |
   | `10m` | 10M | `m7g.4xlarge` | 500 GB |
   | `100m` | 100M | `r7g.4xlarge` | 2 TB |

8. The Terraform module SHALL emit outputs consumed by `ecaz-cloud`:
   `db_private_ip`, `db_instance_id`, `loader_instance_id`,
   `s3_bucket`, `db_volume_id`, `vpc_id`, `subnet_id`.

## Acceptance Criteria

### FR-045-AC-1

`terraform plan` against `profiles/10k.tfvars` succeeds with no
external state and zero manual variables.

### FR-045-AC-2

`terraform apply` against `profiles/dev.tfvars` produces a stack
whose outputs satisfy the schema above.

### FR-045-AC-3

The DB instance is reachable via SSM `start-session`; SSH (port 22)
is not open in any security group.

### FR-045-AC-4

S3 bucket has the configured lifecycle rule on the parquet prefix.
