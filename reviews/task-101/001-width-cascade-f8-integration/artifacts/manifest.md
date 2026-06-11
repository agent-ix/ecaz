# Task 101 Packet 001 Artifact Manifest

- head SHA: `11f8fc38113c08614c8ddca2073e54adcb018d81`
- task bucket: `reviews/task-101/001-width-cascade-f8-integration/`
- lane / fixture / storage: local Rust unit and PG18 grouped-PQ pg_test coverage for the shared width-cascade driver, counter split, and all migrated quant/index families
- host class: local Intel AVX2
- isolated one-index-per-table surfaces: not applicable for unit tests; grouped-PQ packet 94 suite covers local IVF/DiskANN benchmark surfaces
- timestamp: 2026-06-10

## Artifacts

### `cargo-test-candidate-batch.log`

- command: `script -q -c "cargo test --lib candidate_batch" reviews/task-101/001-width-cascade-f8-integration/artifacts/cargo-test-candidate-batch.log`
- key result: `18 passed; 0 failed`
- coverage: counter storage split, distinct `TurboQuantTiledLut` / `TurboQuantInt8` rows, prevalidation no-output-mutation paths, shared cascade counter attribution

### `cargo-test-grouped-pq.log`

- command: `script -q -c "cargo test --lib grouped_pq" reviews/task-101/001-width-cascade-f8-integration/artifacts/cargo-test-grouped-pq.log`
- key result: `35 passed; 0 failed`
- coverage: grouped-PQ partial dispatch, IVF/DiskANN/HNSW batch registration, AVX2 parity, PG18 grouped-PQ pg_test

### `cargo-test-qjl32.log`

- command: `script -q -c "cargo test --lib qjl32" reviews/task-101/001-width-cascade-f8-integration/artifacts/cargo-test-qjl32.log`
- key result: `11 passed; 0 failed`
- coverage: QJL 32-block plus octet/scalar remainder behavior after driver migration

### `cargo-test-hamming32.log`

- command: `script -q -c "cargo test --lib hamming32" reviews/task-101/001-width-cascade-f8-integration/artifacts/cargo-test-hamming32.log`
- key result: `3 passed; 0 failed`
- coverage: binary sidecar block and partial exactness after driver migration

### `cargo-test-int8-approx32.log`

- command: `script -q -c "cargo test --lib int8_approx32" reviews/task-101/001-width-cascade-f8-integration/artifacts/cargo-test-int8-approx32.log`
- key result: `2 passed; 0 failed`
- coverage: int8 approximate block and partial exactness after driver migration

### `cargo-test-tiled-lut32.log`

- command: `script -q -c "cargo test --lib tiled_lut32" reviews/task-101/001-width-cascade-f8-integration/artifacts/cargo-test-tiled-lut32.log`
- key result: `1 passed; 0 failed`
- coverage: tiled LUT run exactness after counter-kind split and driver migration

## Key Result Lines

- Shared driver layout is now `src/am/common/candidate_batch/{mod.rs,counters.rs,drivers.rs}`.
- The driver path covers TurboQuant no-QJL, TurboQuant QJL, TurboQuant tiled LUT, TurboQuant int8 approximate, RaBitQ bits=1, grouped-PQ, and binary hamming.
- Counter kind split is covered by `block_kernel_counter_api_keeps_turboquant_exact_modes_distinct`.
- The Task 94 packet `reviews/task-94/026-f8-width-cascade-integration/` carries the benchmark suite evidence for grouped-PQ IVF/DiskANN counter coverage.
