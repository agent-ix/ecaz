# Task 97 Packet 003 Artifact Manifest

- head SHA: `13d36c051`
- task bucket: `reviews/task-97/003-qjl32-scalar-reference/`
- lane: coder-1 LUT lane, Task 97 TurboQuant QJL block kernel family
- fixture: local synthetic TurboQuant `dim=1024,bits=4,seed=42`; this is the QJL-active canonical 4-bit lane
- storage format: current production packed code `[mse_packed][qjl_packed]`
- rerank mode: exact TurboQuant QJL scoring, `ExactScoreMode::MseLutQjl`
- isolated/shared-table surface: local unit fixtures only; no table or AM storage fixture
- CI/AWS: not run

## Artifacts

### `local-cargo-test-qjl32.log`

- command: `cargo test qjl32 --lib -- --color never`
- timestamp: 2026-06-09 local
- key result:
  - `running 4 tests`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2057 filtered out`

### `local-cargo-test-candidate-batch.log`

- command: `cargo test candidate_batch --lib -- --color never`
- timestamp: 2026-06-09 local
- key result:
  - `running 17 tests`
  - `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 2044 filtered out`

## Notes

- This checkpoint implements the scalar reference and fallback-stub family only.
- The path follows the reviewer clarification that Task 97 is QJL-active canonical 4-bit TurboQuant at a non-tiled dimension such as 1024. It does not introduce a 2-bit TQ mode or any new storage surface.
- Local tests compare qjl32 scores against the existing pre-slice scalar scorer with `f32::to_bits()`.
- Counter tests verify direct rows under `quant=turboquant_qjl` and no `lut32_*` Task 87 compatibility attribution.
