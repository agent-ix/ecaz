# Artifact Manifest — SPIRE AWS pass-correctness (Graviton 4)

Packet: `reviews/task-30/958-spire-aws-pass-correctness/`
Owner: coder B (SPIRE AWS lane)
Branch: `task-30-phase13-spire-aws-prep`
Follow-on to: `reviews/task-30/957-spire-aws-prep-local-verification/`

## AWS Topology Identity (run 2 — downsized after quota hit)

| Field            | Value                                                 |
| ---------------- | ----------------------------------------------------- |
| AWS account      | `932658697181`                                        |
| IAM principal    | `arn:aws:iam::932658697181:user/ecaz-operator`        |
| Region           | `us-west-2`                                           |
| AZ               | `us-west-2a`                                          |
| AMI              | `ami-04e0d7d889f694536` (AL2023 arm64, 2026-05-15)    |
| Coordinator type | `r8g.xlarge` (Graviton 4 / Neoverse V2, 4 vCPU/32 GB) |
| Remote type      | `r8g.xlarge` × 2 (Graviton 4 / Neoverse V2)           |
| Total vCPU       | 12 (under the 16-vCPU r-family account limit)         |
| Head SHA         | `9cfe0bcd` (task-30-phase13-spire-aws-prep)           |
| Cost tag owner   | `kreneskyp`                                           |
| Cost tag deadline| `2026-05-24T00:00:00Z`                                |
| VPC ID           | TBD (filled after `make provision`)                   |
| Subnet ID        | TBD                                                   |
| S3 bucket        | TBD                                                   |
| Coordinator id   | TBD                                                   |
| Remote ids       | TBD                                                   |
| Snapshot ids     | TBD (filled after `make snapshot` pre-teardown)       |

## Artifacts

| # | Artifact | Stage | Command | Timestamp | Key result |
|---|----------|-------|---------|-----------|-----------|
|   |          |       |         |           |           |

## Findings

### F7 — r-family vCPU account quota = 16, original plan needed 40

Status: **resolved by topology downsize**. Run 1 (head `0b0793bb`,
2026-05-23 ~16:32 UTC) provisioned VPC + S3 + secrets + IAM + 3 of 4
EC2 instances, then aborted at `aws_instance.remote[1]` with:

    Error: creating EC2 Instance: VcpuLimitExceeded
    You have requested more vCPU capacity than your current vCPU
    limit of 16 allows for the instance bucket that the specified
    instance type belongs to.

Original sizing (1 × r8g.4xlarge + 3 × r8g.2xlarge = 16 + 24 = 40 vCPU)
required a quota increase request. Operator chose to downsize for this
initial smoke run: 1 × r8g.xlarge + 2 × r8g.xlarge = 12 vCPU total
(each node still has 32 GB RAM to handle the release-profile fat-LTO
peak observed in the prior Graviton baseline cycle).

Emergency teardown completed at ~16:38 UTC after ~6 min of partial
provisioning. 3 instances destroyed, no orphaned resources
(verified via `aws ec2 describe-instances --filters
'Name=tag:Phase,Values=13-spire-aws-verification'
'Name=instance-state-name,Values=pending,running'` returning empty).
Spend impact: ~$0.21.

Follow-up: request `Running On-Demand Standard instances` quota
increase from 16 → 64 vCPU for r-family before opening packet 959
(pass-representative on 1M dbpedia, which wants the larger nodes).

(F3–F6 from request.md are resolved by the pre-AWS plumbing commits;
F7 is the first runtime finding of this packet.)
