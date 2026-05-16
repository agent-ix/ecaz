# Artifact Manifest: SPIRE Phase 13 AWS Verification Parent Packet

Head SHA: `544129dc3442a0686a1c56ebf907d7bb6f4a9bfe`
Packet/topic: `768-c1-spire-phase13-aws-verification`
Lane: Phase 13 AWS verification parent packet
Fixture: parent packet skeleton only; no AWS resources provisioned
Storage format: n/a
Rerank mode: n/a
Surface: Phase 13 AWS verification parent evidence container
Timestamp: 2026-05-15
Isolated one-index-per-table or shared-table surfaces: n/a

## Parent Packet State

This manifest intentionally contains no AWS runtime artifacts yet. It exists so
the first AWS provisioning attempt has a packet-local destination before any
external resources are allocated.

## Pre-AWS Evidence Already Available

1. Phase 13c AWS-readiness follow-up packet: `review/765-c1-spire-phase13c-aws-readiness/`
   - Covers remote libpq TLS support and PK SELECT schema-drift enforcement.

2. Phase 13d read-efficiency/profile packet: `review/766-c1-spire-phase13d-read-efficiency-observability/`
   - Covers production read profile rows, candidate/heap session reuse, cheaper
     default diagnostics, and local PG18 CustomScan smoke evidence.

3. Phase 13 AWS no-spend preflight packet: `review/767-c1-spire-phase13-aws-preflight/`
   - Covers `make -C infra/spire-aws preflight`, Terraform validation, suite
     JSON parsing, shell syntax checks, `ecaz dev sql` AWS connection support,
     and dry-run Make target expansion.

## Required Future Artifact Classes

- `aws-topology.json` from `make -C infra/spire-aws provision`.
- Install logs from `make -C infra/spire-aws install-extension`.
- Registration and GUC logs from `make -C infra/spire-aws register-remotes`.
- Corpus load/inspect logs for every executed tier.
- CustomScan smoke logs, including `production-read-profile-*.log`.
- Suite manifests and JSONL result files.
- Fault drill transcripts and recovery diagnostics.
- Teardown transcript and final cost-tag/no-live-resource report.
