# Review Request: SPIRE Pipeline Benchmark Counters

## Summary

Phase 10.7 now has a first-class CLI surface for route-budget and remote-fanout
counter collection:

- `ecaz bench spire-pipeline`

Code/docs checkpoint: `8b221c93c5d9f99f56b33268b4d19fa8da6fae13`
(`Add SPIRE pipeline benchmark counters`)

## Scope

- Adds `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`.
- Wires the command into `ecaz bench`.
- Supports SPIRE `nprobe` sweeps plus session overrides for:
  - `ec_spire.rerank_width`;
  - `ec_spire.max_candidate_rows`;
  - `ec_spire.adaptive_nprobe`;
  - `ec_spire.adaptive_nprobe_score_gap_micros`.
- Aggregates routing-budget rows from
  `ec_spire_index_scan_routing_snapshot`.
- Aggregates local route/candidate/heap/remote-fanout rows from
  `ec_spire_index_scan_pipeline_snapshot`.
- Optionally calls `ec_spire_remote_pipeline_steps` with selected remote PIDs
  to capture remote PID fanout counters.
- Updates the Phase 9 and Phase 10 detailed task files to complete local
  architecture status while preserving the Phase 8 scale-packet gate for
  product-scale claims.

## Validation

- `cargo test -p ecaz-cli spire_pipeline`
  - 5 passed; 0 failed.
- `cargo fmt --check`
  - exit 0; only the existing stable-rustfmt warnings for unstable import
    grouping settings.
- `cargo run -p ecaz-cli -- bench spire-pipeline --help`
  - exit 0; command is visible under `ecaz bench`.
- `git diff --cached --check`
  - exit 0 before commit.

## Notes

This does not claim a new product-scale performance result. It closes the
operator harness gap: future packets can now collect routing, candidate,
heap-rerank, local remote-fanout, and remote PID fanout counters through `ecaz`
instead of hand-written SQL.
