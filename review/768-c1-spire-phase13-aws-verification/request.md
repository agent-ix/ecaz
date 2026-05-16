# Review Request: SPIRE Phase 13 AWS Verification Parent Packet

**Requester:** coder1
**Date:** 2026-05-15
**Head SHA for pre-AWS prep:** `544129dc3442a0686a1c56ebf907d7bb6f4a9bfe`
**Review focus:** parent packet skeleton for the upcoming AWS verification pass.

## Purpose

This packet is the parent evidence container for Phase 13 AWS verification. It
exists before any AWS resources are provisioned so every later correctness,
performance, operations, fault, and teardown child packet can cite a stable
parent and packet-local artifact convention.

No AWS resources have been provisioned for this packet yet.

## Current Prep State

- Phase 13c local AWS-readiness fixes are implemented in packet `765`.
- Phase 13d production-read profiling and read-efficiency fixes are implemented
  in packet `766`.
- Phase 13 AWS no-spend preflight prep is implemented in packet `767`.
- `make -C infra/spire-aws preflight` passed in packet `767`.
- The runbook/operator surface now exposes `preflight`, `provision`,
  `install-extension`, `register-remotes`, `pass-correctness`, and
  `pass-representative`.

## Required Before Provisioning

- [ ] Reviewer accepts Phase 13a design decisions or records explicit
  deferrals.
- [ ] Reviewer accepts Phase 13b runbook/operator surface or records explicit
  deferrals.
- [ ] Reviewer accepts Phase 13d read-profile packet `766`.
- [ ] Reviewer accepts Phase 13 AWS preflight packet `767`.
- [ ] AWS account quota for `r6i.4xlarge` plus three `r6i.2xlarge` instances is
  confirmed in the selected region.
- [ ] `infra/spire-aws/terraform.tfvars` is created from
  `terraform.tfvars.example` with real `region`, `availability_zone`, `ami_id`,
  `owner`, and `auto_stop_at`.
- [ ] AWS CLI credentials are active for a role that can manage EC2, VPC,
  SSM, Secrets Manager, S3, and IAM in the target region.
- [ ] The ecaz extension tarball for the exact head SHA to test is uploaded to
  the Terraform-created artifact bucket before `install-extension`.

## Planned Child Packets

- Correctness pass: provision, install, register, load correctness corpus,
  CustomScan smoke, correctness suite, degraded/strict fault drills, teardown.
- Representative read pass: representative corpus, smoke/profile capture,
  recall/latency suite, transport sweep, profile rowsets, local baseline pair.
- Representative write pass: INSERT/UPDATE/DELETE rows plus orphaned 2PC and
  missing-GUC drills.
- Stage E subset: selected lifecycle/fault fixtures against AWS topology.
- Optional stress pass: reviewer-gated only.
- Final closeout: aggregate child packets, teardown/cost-tag evidence, accepted
  caveats, and Phase 13 exit-status update.

## Artifact Rules

Every child packet must keep raw logs under its own `artifacts/` directory and
record:

- head SHA;
- packet/topic;
- command used;
- timestamp;
- dataset identity;
- AWS region and AZ;
- AMI ID;
- sanitized instance IDs;
- sanitized secret ARNs/names;
- extension version;
- storage format and rerank mode;
- isolated one-index-per-table versus shared-table surface;
- key result lines cited by `request.md`.

## Known Limits

This parent packet is intentionally a skeleton. It does not yet contain AWS
runtime evidence, quota proof, tfvars, tarball upload proof, or teardown proof.
Those are required before Phase 13 can close.
