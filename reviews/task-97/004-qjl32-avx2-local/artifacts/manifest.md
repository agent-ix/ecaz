# Task 97 Packet 004 Artifact Manifest

- head SHA: `b8a785736`
- task bucket: `reviews/task-97/004-qjl32-avx2-local/`
- lane: coder-1 LUT lane, Task 97 TurboQuant QJL block kernel family
- fixture: local synthetic TurboQuant `dim=1024,bits=4,seed=42`; QJL-active canonical 4-bit lane
- storage format: current production packed code `[mse_packed][qjl_packed]`
- rerank mode: exact TurboQuant QJL scoring, `ExactScoreMode::MseLutQjl`
- host ISA: local `x86_64`, AVX2 runtime dispatch
- isolated/shared-table surface: local unit and Criterion fixtures only; no table or AM storage fixture
- CI/AWS: not run

## Artifacts

### `local-cargo-test-qjl32.log`

- command: `cargo test qjl32 --lib -- --color never`
- timestamp: 2026-06-09 local
- key result:
  - `running 5 tests`
  - `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2057 filtered out`

### `local-cargo-test-candidate-batch.log`

- command: `cargo test candidate_batch --lib -- --color never`
- timestamp: 2026-06-09 local
- key result:
  - `running 17 tests`
  - `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 2045 filtered out`

### `local-cargo-bench-qjl32-avx2.log`

- command: `cargo bench --features bench --bench quant_score qjl32 -- --sample-size 10 --warm-up-time 1 --measurement-time 2`
- timestamp: 2026-06-09 local
- key result:
  - scalar `quant/qjl32_block32/scalar/d1024_b4`: `[35.394 us 36.629 us 38.119 us]`
  - dispatch `quant/qjl32_block32/dispatch/d1024_b4`: `[28.228 us 28.355 us 28.424 us]`
  - local median speedup: `36.629 / 28.355 = 1.29x`

## Notes

- The first cross-candidate AVX2 attempt was locally slower than scalar and was not packeted as the final implementation.
- This packet keeps scalar-order accumulation for the AVX2 candidate scorer and uses AVX2 for 8-dimension codebook/sign multiply chunks.
- NEON/SVE2 and Graviton 4 evidence are still deferred until AWS testing is approved.
