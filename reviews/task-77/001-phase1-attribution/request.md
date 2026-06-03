---
task: 77
topic: phase1-attribution-hook
agent: codex
role: coder
model: GPT-5
date: 2026-05-31
---

# Task 77 Phase 1 Attribution Hook

## Request

Please review the first Task 77 checkpoint: a diagnostic-only timing hook for SPIRE candidate attribution.

This does not change production scan behavior. The new timers are only active when the existing SQL diagnostic observer is used by `ec_spire_index_scan_leaf_candidate_snapshot`; the production no-op observer leaves `wants_candidate_timing()` false.

## What Changed

- Corrected the Task 77 baseline from the superseded Task 75 `2,784,952` candidate count to the fixed `15,506,227` high-recall candidate count.
- Extended SQL-visible leaf candidate diagnostics with:
  - `leaf_object_read_nanos`
  - `candidate_score_nanos`
  - `candidate_materialize_nanos`
  - `candidate_heap_append_nanos`
- Extended `ecaz bench spire-pipeline --funnel-output` JSONL rows to carry those timing sums per query.
- Confirmed AWS remains off before starting local Phase 1 work.

## Validation

- `cargo check --all-targets --no-default-features --features pg18`
  - `artifacts/cargo-check-pg18.log`
- `cargo test -p ecaz-cli spire_pipeline --no-default-features`
  - `artifacts/cargo-test-spire-pipeline.log`
- AWS status before local work:
  - `artifacts/aws-status-1m-before-local-work.log`
  - `artifacts/aws-status-10k-medium-before-local-work.log`

## Follow-Up

The next checkpoint should install the updated PG18 extension locally and run the Task 77 Phase 1 `ecaz bench suite` packet under `benchmarks/task77-intel-local-candidate-cost-attribution/`.
