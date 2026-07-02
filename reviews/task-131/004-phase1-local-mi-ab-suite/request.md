# Task 131 Packet 004: Phase 1 Local Multi-Instance A/B Suite

## Summary

This checkpoint prepares the required Task 131 Phase 1 local multi-instance A/B matrix in the standard `ecaz bench suite` runner.

Code commit: `8d761e06a68a739428cde3b3ac81d6aa0c194e5f` (`task 131 support no-payload production read timelines`)

## Changes

- Adds `bench spire-pipeline --production-read-timeline-no-payload`, which keeps query metrics/recall intact but calls `ec_spire_remote_search_production_read_timeline(..., '{}'::text[])`.
- Adds `production_read_timeline_no_payload` to `spire-pipeline` suite steps.
- Adds `timeline_payload=none` to local-multinode production-read variants.
- Adds packet-local suite config `artifacts/task131-phase1-local-mi-ab-suite.json` for:
  - 10k, 50k, 100k
  - `n128/b4/nprobe96`
  - `n1024/b2/nprobe64`
  - baseline `ec_spire.remote_search_global_pre_heap_merge=off`
  - candidate `ec_spire.remote_search_global_pre_heap_merge=on`

## Validation

- `artifacts/cargo-test-production-read.log`: `cargo test production_read --package ecaz-cli` passed.
- `artifacts/cargo-test-local-multinode-expansion.log`: local-multinode suite expansion test passed.
- `artifacts/cargo-check-ecaz-cli.log`: `cargo check --package ecaz-cli` passed with the existing dead-code warning.
- `artifacts/git-diff-check-head.log`: committed diff check passed.
- `artifacts/suite-audit.log`: suite audit passed with six steps.
- `artifacts/dryrun-suite.log`: dry-run expanded all six local-multinode commands and preserved `timeline_payload=none` on baseline and candidate variants.

## Not Closeout Evidence

This packet is not the Phase 1 measurement result. It prepares and validates the runner path for the required local multi-instance A/B, but the actual 10k/50k/100k benchmark matrix still needs to run and produce recall/result identity, latency p50/p95/p99, heap-row, payload-byte, and storage evidence before Task 131 can make a promote/iterate/shelve decision.

