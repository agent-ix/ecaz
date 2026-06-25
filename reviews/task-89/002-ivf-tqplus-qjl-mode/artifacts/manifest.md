# Task 89 Packet 002 Artifact Manifest

- Head SHA: `a6b0970011fb64bde036c3f7a07321c0ab570885`
- Task bucket: `reviews/task-89/002-ivf-tqplus-qjl-mode`
- Timestamp: 2026-06-25
- Scope: IVF TQ+ QJL/gamma-aware mode implementation checkpoint
- Lane / fixture / storage format / rerank mode: local PG18 compile and unit-test validation; QJL-active 32-dimensional TurboQuant unit fixture plus existing no-QJL 1536-dimensional unit fixture; `storage_format = turboquant`, `turboquant_calibration = tqplus_experimental`; rerank mode not exercised.
- Surface isolation: not a benchmark run; no isolated/shared table surface.

## Artifacts

### `cargo-check-pg18.log`

- Command:
  `cargo check -p ecaz --lib --no-default-features --features pg18`
- Result: pass.
- Key result line:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 0.08s`

### `cargo-test-tqplus.log`

- Command:
  `cargo test -p ecaz --lib --no-default-features --features pg18 tqplus_`
- Result: pass.
- Key result lines:
  `running 6 tests`
  `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2221 filtered out; finished in 0.05s`

## Notes

- This packet extends packet 001 by adding QJL/gamma-aware TQ+ support.
- QJL-active TQ+ keeps residual gamma in the IVF posting gamma field and appends a 4-byte candidate renormalization scalar to the experimental TQ+ code bytes.
- No-QJL TQ+ remains width-neutral: the IVF posting gamma field carries candidate renormalization and the code bytes are unchanged.
- This packet still does not contain Task 89 Phase 3+ real-corpus benchmark evidence.
