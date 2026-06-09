# Task 87 Phase 7 Bench Counter Capture

## Scope

This packet makes the Phase 7 scoring-share counters usable by the suite-driven real-corpus runs.

Code checkpoint:

- `52d1b251f6bbe4f17115baf119340ff539fab6de` - `Capture Task 87 counters in bench commands`

What changed:

- Added `--task87-candidate-batch-counters` to `ecaz bench latency`.
  - Each worker connection resets Task 87 counters after session setup.
  - Each worker snapshots counters after its query loop.
  - Worker snapshots are merged per sweep value and emitted as `[task87-counters]` log lines.
- Added `--task87-candidate-batch-counters` to `ecaz bench spire-pipeline`.
  - The command resets and snapshots counters on the same SPIRE pipeline connection for each sweep value.
- Added suite JSON pass-through field `task87_candidate_batch_counters: true` for latency and spire-pipeline steps.
- The emitted counter lines include surface, flushes, candidates, elapsed nanos/ms, LUT32 flushes, and LUT32 candidates.

## Why This Slice Exists

The SQL functions from packet 017 are backend-local because PostgreSQL extension statics live inside a backend process. A separate raw SQL suite step cannot prove the counters for a different backend that ran the benchmark query loop. This slice moves reset/snapshot into the benchmark commands so the final Phase 7 matrix can cite direct scoring-share counters from the same connection that ran each workload.

## Validation

Packet-local logs:

- `artifacts/cargo-test-ecaz-cli-bench-suite.log`
  - `cargo test -p ecaz-cli bench::suite --no-default-features`
  - Result: 41 passed; 0 failed.
- `artifacts/cargo-test-candidate-batch.log`
  - `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
  - Result: 4 passed; 0 failed.
- `artifacts/cargo-test-quant-lut32.log`
  - `cargo test --lib quant::lut32 --no-default-features --features pg18`
  - Result: 2 passed; 0 failed.

## Review Notes

This is not the final Phase 7 closeout. It enables valid same-backend scoring-share measurement for the upcoming real-corpus suite and HNSW batch-width decision.
