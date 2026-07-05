# Task 131 Packet 003: Production Read P99 Evidence Columns

## Summary

This checkpoint improves the evidence surface needed for Task 131 Phase 1 A/B measurement. Production-read profile and per-node timeline reports now include p99 timing columns, so `ecaz bench suite` result extraction can preserve p50/p95/p99 timing evidence for candidate receive, heap receive, payload decode, merge, and total production-read phases.

Code commit: `bf8128ca116e51ce9e862960fc1d067b41203031` (`task 131 add p99 production read metrics`)

## Changes

- Adds p99 columns to the aggregated production-read profile table:
  - `connect_p99`
  - `endpoint_identity_p99`
  - `candidate_p99`
  - `heap_p99`
  - `payload_decode_p99`
  - `merge_p99`
  - `total_p99`
- Adds p99 columns to the per-node production-read timeline table:
  - `elapsed_p99`
  - `payload_decode_p99`
- Extends the CLI renderer test to cover the new p99 fields and keep the Task 131 global pre-heap columns covered.

## Validation

- `artifacts/cargo-test-ecaz-cli-production-read.log`: `cargo test production_read --package ecaz-cli` passed, 3 tests.
- `artifacts/cargo-check-ecaz-cli.log`: `cargo check --package ecaz-cli` passed with the existing dead-code warning for `LoadedDistributedPlacementConfig::path`.
- `artifacts/git-diff-check-head.log`: `git diff --check HEAD~1..HEAD` passed with no output.

## Not Closeout Evidence

This packet does not include a live multi-instance benchmark. It only makes the upcoming Task 131 `ecaz bench suite` A/B output strong enough to cite p50/p95/p99 production-read phase timings, including heap receive. Task 131 still needs local multi-instance 10k/50k/100k evidence before any promotion, shelve, or closeout decision.

