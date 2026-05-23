# Artifact Manifest — SPIRE AWS pass-correctness (Graviton 4)

Packet: `reviews/task-30/958-spire-aws-pass-correctness/`
Owner: coder B (SPIRE AWS lane)
Branch: `task-30-phase13-spire-aws-prep`
Follow-on to: `reviews/task-30/957-spire-aws-prep-local-verification/`

## AWS Topology Identity

| Field            | Value                                                 |
| ---------------- | ----------------------------------------------------- |
| AWS account      | `932658697181`                                        |
| IAM principal    | `arn:aws:iam::932658697181:user/ecaz-operator`        |
| Region           | `us-west-2`                                           |
| AZ               | `us-west-2a`                                          |
| AMI              | `ami-04e0d7d889f694536` (AL2023 arm64, 2026-05-15)    |
| Coordinator type | `r8g.4xlarge` (Graviton 4, Neoverse V2)               |
| Remote type      | `r8g.2xlarge` × 3 (Graviton 4, Neoverse V2)           |
| VPC ID           | TBD (filled after `make provision`)                   |
| Subnet ID        | TBD                                                   |
| S3 bucket        | TBD                                                   |
| Coordinator id   | TBD                                                   |
| Remote ids       | TBD                                                   |
| Cost tag owner   | `kreneskyp`                                           |
| Cost tag deadline| `2026-05-24T00:00:00Z`                                |
| Head SHA         | TBD (recorded at run-start)                           |
| Tarball SHA256   | TBD (recorded after `make package`)                   |

## Artifacts

| # | Artifact | Stage | Command | Timestamp | Key result |
|---|----------|-------|---------|-----------|-----------|
|   |          |       |         |           |           |

## Findings

(rows added as the run progresses; F3–F6 are resolved by the plumbing
fixes committed before AWS spend — see `request.md`)
