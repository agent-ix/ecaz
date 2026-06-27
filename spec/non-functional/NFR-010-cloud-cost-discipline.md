---
id: NFR-010
title: Cloud Cost Discipline
type: NFR
quality_attribute: compliance
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/StR-007"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-044"
    type: "constrains"
    cardinality: "1:1"
---
# NFR-010: Cloud Cost Discipline

## Statement

The cloud harness SHALL make ongoing AWS spend visible, bounded, and
reversible. A forgotten profile SHALL NOT be able to silently accrue
material spend.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Paid compute resources after `ecaz cloud down` | zero (only intentionally-retained snapshot/bucket remain) | zero compute resources | `ecaz cloud status` after teardown |
| Cost visibility fields in status output | `estimated_hourly_usd` and `retained_monthly_usd` populated | non-null numeric values | `ecaz cloud status --json` |
| Instance compute charges while paused | $0/hr with EBS data retained, no auto-resume | zero compute charges | `ecaz cloud status` cost report |
| Oversized-profile provisioning guard | profiles larger than `dev` require `--confirm-cost` matching projected $/day | provisioning refused without matching flag | attempted `ecaz cloud up` without the flag |
| NAT gateway resources in Terraform plan | 0 | 0 | Terraform plan audit |
| S3 raw-parquet retention | lifecycle expiry at configured retention (default 30 days), bench artifacts retained indefinitely under a separate prefix | lifecycle rule present | S3 lifecycle configuration audit |

## Policy

1. `ecaz cloud down` SHALL fully destroy all paid resources for a
   profile (EC2, EBS, snapshots if `--delete-snapshots`, S3 if
   `--delete-bucket`). Default behavior retains snapshots and S3
   for cost-vs-data-retention safety.
2. `ecaz cloud pause` SHALL drop instance compute charges to zero
   while retaining EBS data. `pause` SHALL NOT auto-resume
   (contrast with RDS Stop's 7-day auto-restart).
3. `ecaz cloud status` SHALL report estimated $/hr for running
   resources and $/mo for retained storage, sourced from a
   committed cost table per instance type and EBS size.
4. When a stack has been paused for >7 days, `status` SHALL
   recommend `snapshot` + `down` to drop EBS charges.
5. The S3 bucket SHALL have a lifecycle rule expiring raw parquet
   after a configurable retention (default 30 days). Bench
   artifacts SHALL be retained indefinitely under a separate
   prefix.
6. The Terraform module SHALL NOT provision a NAT gateway. All
   in-VPC AWS access uses VPC endpoints (S3, SSM).
7. The harness SHALL refuse to provision a profile larger than
   `dev` without an explicit `--confirm-cost` flag whose argument
   matches the projected $/day for that profile.

## Verification

Compliance is checked by exercising the harness commands and inspecting
infrastructure outputs: tearing down a profile via `ecaz cloud down` and
confirming `ecaz cloud status` reports zero compute resources and only the
intentionally-retained snapshot/bucket; checking `ecaz cloud status --json`
for non-null `estimated_hourly_usd` and `retained_monthly_usd` sourced from
the committed cost table; attempting to `up` a profile larger than `dev`
without `--confirm-cost` and confirming refusal; and auditing the Terraform
plan for zero NAT gateway resources and the S3 bucket for the parquet
lifecycle rule.

## Acceptance Criteria

### NFR-010-AC-1

A teardown of any profile via `ecaz cloud down` followed by
`ecaz cloud status` reports zero compute resources and only the
intentionally-retained snapshot/bucket.

### NFR-010-AC-2

`ecaz cloud status --json` includes `estimated_hourly_usd` and
`retained_monthly_usd` fields with non-null numeric values.

### NFR-010-AC-3

A profile larger than `dev` cannot be `up`'d without
`--confirm-cost <usd>` matching the projected daily cost.

### NFR-010-AC-4

Terraform plan for any profile contains zero NAT gateway resources.
