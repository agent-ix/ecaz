# Review Request: SPIRE Phase 13e Production Read Overlap

## Summary

This slice adds production-read timeline evidence for Phase 13e. The new SQL surface `ec_spire_remote_search_production_read_timeline(...)` exposes per-node candidate and heap receive timings from the same async production read executor used by the distributed CustomScan path. It is not backed by the older diagnostic pg-test transport probe.

The local PG18 1-coordinator plus 3-remote fixture now locks node 2's remote heap table during a typed tuple-payload production read and asserts that the other remote heap receives complete first. The passing run proves the fast remotes are not serialized behind the slow remote in the production heap receive phase.

## Key Evidence

- `artifacts/phase13e-production-read-overlap.log`
  - `profile_summary=ready|3|3|3|3|6`
  - `production_timeline_summary=3|3|620|31|0`
  - `strict_remote_failure_exit_code=3`
  - `degraded_profile_summary=degraded_ready|3|2|2|2|1|0|0|6|none`
  - `SPIRE Phase 13e static remote placement PG18 fixture passed`
- `artifacts/production-read-timeline.tsv`
  - candidate receive rows: 3 ready
  - heap receive rows: 3 ready
  - slow node 2 heap completion: `620 ms`
  - fastest non-slow heap completion: `31 ms`

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo fmt --all -- --check`
- `bash -n scripts/run_spire_phase13e_static_remote_placement_pg18.sh`
- `scripts/run_spire_phase13e_static_remote_placement_pg18.sh --artifact-dir reviews/task-30/972-spire-phase13e-production-read-overlap/artifacts --smoke-log reviews/task-30/972-spire-phase13e-production-read-overlap/artifacts/phase13e-production-read-overlap.log`

## Remaining Phase 13e Work

- Drive the remaining read matrices through `ecaz bench suite`.
- Capture AWS correctness-tier profile fields.
- Capture representative AWS p50/p95/p99 latency and recall.
- Evaluate connection pooling only after corrected AWS profile evidence.
