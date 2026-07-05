# Task 97 Packet 004 Artifact Manifest

- head SHA: `b0efa19d9`
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
  - `running 7 tests`
  - `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2057 filtered out`

### `local-cargo-test-candidate-batch.log`

- command: `cargo test candidate_batch --lib -- --color never`
- timestamp: 2026-06-09 local
- key result:
  - `running 17 tests`
  - `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 2047 filtered out`

### `local-cargo-bench-qjl32-avx2.log`

- command: `cargo bench --features bench --bench quant_score qjl32 -- --sample-size 10 --warm-up-time 1 --measurement-time 2`
- timestamp: 2026-06-09 local
- key result:
  - scalar `quant/qjl32_block32/scalar/d1024_b4`: `[34.330 us 35.000 us 35.754 us]`
  - dispatch `quant/qjl32_block32/dispatch/d1024_b4`: `[28.089 us 28.645 us 29.658 us]`
  - local median speedup: `35.000 / 28.645 = 1.22x`

## Notes

- The first cross-candidate AVX2 attempt was locally slower than scalar and was not packeted as the final implementation.
- This packet keeps scalar-order accumulation for the AVX2 candidate scorer and uses AVX2 for 8-dimension codebook/sign multiply chunks.
- The production-dispatched QJL AVX2 path now uses the same scalar-order 3-bit
  QJL strategy, satisfying the reviewer-required production-dispatch tolerance
  pair before ISA evidence is cited.
- NEON/SVE2 and Graviton 4 evidence are still deferred until AWS testing is approved.
