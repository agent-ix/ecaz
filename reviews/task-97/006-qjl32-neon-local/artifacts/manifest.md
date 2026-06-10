# Task 97 Packet 006 Artifact Manifest

- head SHA: `d11a0287e4a3c5732760b1568f1e963deeda5a09`
- task bucket: `reviews/task-97/006-qjl32-neon-local/`
- lane: coder-1 LUT lane, Task 97 TurboQuant QJL block kernel family
- fixture: local synthetic TurboQuant `dim=1024,bits=4,seed=42`; QJL-active `ExactScoreMode::MseLutQjl`
- storage format: current production packed TurboQuant code `[mse_packed][qjl_packed]`
- rerank mode: exact TurboQuant QJL scoring
- isolated/shared-table surface: not applicable; local unit/check commands only
- CI/AWS: not run

## Artifacts

### `local-cargo-test-qjl32.log`

- command: `cargo test qjl32 --lib -- --color never`
- timestamp: 2026-06-09 19:36:51-07:00
- key result: 9 passed; 0 failed
- covered: qjl32 scalar/block parity, AVX2 tolerance, AM registration tests matched by the `qjl32` filter; on this x86 host the NEON module is target-gated and not executed

### `local-cargo-check-aarch64-blocked.log`

- command: `cargo check --target aarch64-unknown-linux-gnu --lib`
- timestamp: 2026-06-09 19:37:02-07:00
- key result: blocked before the ecaz crate by local cross-toolchain setup
- blocker line: `failed to find tool "aarch64-linux-gnu-gcc": No such file or directory`
- scope: this is not NEON parity or Graviton dispatch evidence; it records why local AArch64 type checking could not be completed in this environment

## Notes

- `src/quant/qjl32/neon.rs` now has a target-gated NEON implementation that returns `Isa::Neon` only when `is_aarch64_feature_detected!("neon")` succeeds. Other hosts keep the scalar fallback behavior.
- The NEON inner loop mirrors the already-reviewed AVX2 local structure: it processes one candidate at a time, vector-multiplies four dimensions per chunk, and then adds products back in lane order to preserve the scalar accumulation contract.
- Graviton 4 runtime dispatch (`Isa::Sve2`) and measured vector length remain deferred until AWS validation is approved.
