# Task 89 Packet 001 Artifact Manifest

- Head SHA: `0719aaaab01597a4b5ee1075823d44a592f45c83`
- Task bucket: `reviews/task-89/001-ivf-tqplus-experimental-profile`
- Timestamp: 2026-06-25
- Scope: IVF-only TQ+ experimental TurboQuant calibration profile
- Lane / fixture / storage format / rerank mode: local PG18 compile and unit-test validation; no real-corpus benchmark fixture; `storage_format = turboquant`, `turboquant_calibration = tqplus_experimental`; rerank mode not exercised.
- Surface isolation: not a benchmark run; no isolated/shared table surface.

## Artifacts

### `cargo-check-pg18.log`

- Command:
  `cargo check -p ecaz --lib --no-default-features --features pg18`
- Result: pass.
- Key result line:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 2.56s`

### `cargo-test-tqplus.log`

- Command:
  `cargo test -p ecaz --lib --no-default-features --features pg18 tqplus_`
- Result: pass.
- Key result lines:
  `running 4 tests`
  `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2221 filtered out; finished in 0.04s`

### `cargo-test-metadata-roundtrip.log`

- Command:
  `cargo test -p ecaz --lib --no-default-features --features pg18 metadata_roundtrip`
- Result: pass.
- Key result lines:
  `running 11 tests`
  `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 2214 filtered out; finished in 0.00s`

### `cargo-test-size-of-assertions.log`

- Command:
  `cargo test -p ecaz --test size_of_assertions --no-default-features --features pg18`
- Result: pass.
- Key result lines:
  `running 13 tests`
  `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

## Notes

- This packet does not contain Task 89 Phase 3+ real-corpus benchmark evidence.
- This packet does not claim Task 89 closeout.
- The implementation currently gates TQ+ encoding/scoring to the existing no-QJL 4-bit lane. QJL/gamma-aware TQ+ remains a follow-up design point because it needs both residual gamma and candidate renorm scalar handling.
