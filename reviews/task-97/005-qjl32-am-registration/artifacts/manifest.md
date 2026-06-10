# Task 97 Packet 005 Artifact Manifest

- head SHA: `3e0ca3eaa3046414645f1c3d1e6b42fe36188dc7`
- task bucket: `reviews/task-97/005-qjl32-am-registration/`
- lane: coder-1 LUT lane, Task 97 TurboQuant QJL block kernel family
- fixture: local synthetic TurboQuant `dim=1024,bits=4,seed=42` for QJL-active AM registration; existing `dim=1536,bits=4` remains no-QJL and out of Task 97
- storage format: current production packed TurboQuant code `[mse_packed][qjl_packed]`
- rerank mode: exact TurboQuant QJL scoring, `ExactScoreMode::MseLutQjl`
- isolated/shared-table surface: not applicable; local unit tests only
- CI/AWS: not run

## Artifacts

### `local-cargo-test-qjl32.log`

- command: `cargo test qjl32 --lib -- --color never`
- timestamp: 2026-06-09 19:27:48-07:00
- key result: 9 passed; 0 failed
- covered: qjl32 scalar/block parity, AVX2 tolerance, no-QJL shape rejection, SPIRE/HNSW qjl32 AM registration tests matched by the `qjl32` filter

### `local-cargo-test-turboquant-qjl.log`

- command: `cargo test turboquant_qjl --lib -- --color never`
- timestamp: 2026-06-09 19:29:11-07:00
- key result: 6 passed; 0 failed
- covered: shared `turboquant_qjl` counter kind/helper tests plus IVF, SPIRE, and HNSW QJL registration tests

### `local-cargo-test-ivf-quantcodec-qjl.log`

- command: `cargo test common_quant_codec_turboquant_batch_is_bit_exact_with_scalar --lib -- --color never`
- timestamp: 2026-06-09 19:29:18-07:00
- key result: 1 passed; 0 failed
- covered: existing IVF generic TurboQuant `QuantCodec::score_ip_batch` branch below block width remains bit-exact with scalar

## Notes

- Local tests assert the corrected path from reviewer clarification: canonical `bits=4` TurboQuant QJL is active at non-tiled `dim=1024`; `dim=1536,bits=4` remains the no-QJL lane.
- The shared qjl32 helper now accepts `CandidateMeta::Gamma` and `CandidateMeta::GammaAndResidualSigns`, but scores residual signs from the existing packed code payload. No new storage surface or payload split is introduced.
- Direct counter rows are asserted under `quant=turboquant_qjl` for IVF, SPIRE, and HNSW, with block/tail attribution of 32 kernel candidates and 7 scalar candidates in the local 39-candidate fixtures.
