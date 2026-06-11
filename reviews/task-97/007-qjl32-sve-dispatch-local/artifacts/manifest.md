# Task 97 Packet 007 Artifact Manifest

- head SHA: `69b9d82fb5b2f4468becb6d0bfd57ad901c0fe22`
- task bucket: `reviews/task-97/007-qjl32-sve-dispatch-local/`
- lane: coder-1 LUT lane, Task 97 TurboQuant QJL block kernel family
- fixture: local synthetic TurboQuant `dim=1024,bits=4,seed=42`; QJL-active `ExactScoreMode::MseLutQjl`
- storage format: current production packed TurboQuant code `[mse_packed][qjl_packed]`
- rerank mode: exact TurboQuant QJL scoring
- isolated/shared-table surface: not applicable; local unit/toolchain commands only
- CI/AWS: not run

## Artifacts

### `local-cargo-test-qjl32.log`

- command: `cargo test qjl32 --lib -- --color never`
- timestamp: 2026-06-09 19:44:25-07:00
- key result: 10 passed; 0 failed
- covered: qjl32 scalar/block parity, AVX2 tolerance, optional SVE parity test hook, and AM registration tests matched by the `qjl32` filter
- host note: on this x86 host, `qjl32_sve_block32_matches_pre_slice_scorer_tolerance_when_available` returns early because no SVE backend is available

### `local-aarch64-toolchain-check.log`

- command: `rustup target list --installed`; `command -v aarch64-linux-gnu-gcc`; `command -v clang`
- timestamp: 2026-06-09 19:44:45-07:00
- key result: Rust target `aarch64-unknown-linux-gnu` is installed; `aarch64-linux-gnu-gcc` and `clang` are missing
- scope: explains why local AArch64 cargo checking is unavailable in this environment without installing toolchain components

## Notes

- `src/quant/qjl32/sve.rs` now mirrors the grouped-PQ SVE runtime pattern:
  `sve2` is preferred over `sve`, and the function returns the ISA actually
  selected.
- The SVE helper is vector-length agnostic. It multiplies f32 arrays with an
  SVE predicated loop, then sums products in Rust scalar order to preserve the
  existing qjl32 tolerance contract.
- The test hook exposes runtime SVE vector lanes for future Graviton packets.
- This packet does not claim Graviton 4 runtime dispatch, vector length, or
  SVE2 counter rows. Those remain deferred until AWS validation is approved.
