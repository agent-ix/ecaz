# Task 97 Packet 002 Artifact Manifest

- base head before packet: `036100757ad8198401d2408172cd470ebead6cc3`
- task bucket: `reviews/task-97/002-qjl32-counter-kind-and-design/`
- lane: coder-1 LUT lane, Task 97 TurboQuant QJL block kernel family
- timestamp: `2026-06-09T18:13:36-07:00`
- isolated one-index-per-table or shared-table surfaces: not applicable for
  this design/counter-kind packet

## Artifacts

### `design.md`

- command: author-written design artifact from Task 97 packet 001 feedback seq
  01-03 and ADR-076.
- key result: qjl32 remains a separate ADR-076 module family; direct counters
  use `quant=turboquant_qjl`; IVF/SPIRE/HNSW are the current QJL AM targets.

### `local-cargo-test-candidate-batch.log`

- command: `cargo test candidate_batch --lib -- --color never`
- key result: 15 tests passed, including
  `turboquant_qjl_counter_kind_has_distinct_direct_rows_without_lut32_compat`.

## Validation Summary

- Local focused Rust test passed.
- No benchmark run.
- No GitHub CI run.
- No AWS smoke or benchmark run.
