---
id: StR-007
title: Repeatable Scale Benchmarking on Managed Cloud
type: stakeholder-requirement
status: PROPOSED
derived_usecases:
  - US-021
---
# StR-007: Repeatable Scale Benchmarking on Managed Cloud

## Need

Ecaz benchmark evidence today is bounded by what fits on an operator
workstation. The committed corpus stops at 1M × 1536-dim vectors
(`Qdrant/dbpedia-entities-openai3-…-1M`), but real product decisions
(index choice, quantizer profile, page layout, planner cost model)
need data at 10M and 100M before they can be trusted. Recreating that
state by hand on AWS is slow, error-prone, and an unbounded cost risk
when an instance is forgotten.

## Expectation

An operator SHALL be able to run a complete ecaz benchmark cycle
(provision → install → load corpus → bench → tear down) at a chosen
profile size — `10k`, `dev`, `1m`, `10m`, `100m` — by issuing a single
sequence of `ecaz cloud …` commands, with no AWS console access, no
manual SSH, and no manual SQL. The harness SHALL preserve loaded
corpora across iterations via pause/resume and EBS snapshots so that
re-running benchmarks does not require re-loading hundreds of millions
of rows.

## Rationale

- Standard AWS RDS does not load custom pgrx extensions (ecaz declares
  `superuser = true` and `_PG_init` hooks); the harness therefore
  targets EC2 self-managed Postgres 18, with the door open to RDS
  Custom in the future.
- Graviton (ARM64) instance families (`m7g`/`c7g`/`r7g`/`r8g`) are
  ~20% cheaper per vCPU than equivalent Intel families and run pure
  Rust workloads natively; cost matters at 10M+ scale.
- Public ANN benchmarks (Qdrant, Weaviate, ann-benchmarks.com, NeurIPS
  Big-ANN) are published on a small set of canonical Hugging Face and
  S3-hosted datasets. Comparability requires running on the same data.
- 100M × 1536-dim fp32 ≈ 600 GB raw before indexes; loading over the
  public internet is impractical, so corpus staging and load SHALL
  happen inside the database VPC.

## Success Criteria

- `ecaz cloud up --profile 10k` reaches a queryable ecaz database in
  under 10 minutes from a clean slate.
- `ecaz cloud bench --profile 1m --suite smoke` produces a recall +
  latency + storage artifact bundle uploaded to S3, with no manual
  intervention.
- `ecaz cloud pause` reduces the running profile's compute cost to
  zero while preserving loaded data; `ecaz cloud resume` returns the
  same profile to a queryable state.
- `ecaz cloud down` followed by `ecaz cloud status` reports zero
  paid resources for the profile.
- The same harness drives `10m` and `100m` profiles without code
  changes, only `tfvars` selection.
