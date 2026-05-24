# Artifact Manifest

- head SHA: `40f750f928c34089fe9fd95b1ff12574d73ad9c7`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/253-hadamard-safe-simd-dispatch`
- timestamp: `2026-05-21T15:10:31Z`
- lane: RabitQ/quant FWHT SIMD dispatch
- fixture: local compile/static validation
- storage format: not applicable
- rerank mode: not applicable
- table/index isolation: not applicable

## Artifacts

- `artifacts/rustfmt-hadamard.log`
  - Command: `rustfmt --edition 2021 --check src/quant/hadamard.rs`
  - Result: passed; stable rustfmt emitted existing unstable-option warnings.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; existing unused SPIRE re-export warning in `src/am/mod.rs`.
- `artifacts/cargo-test-hadamard-pg18-bench-no-run.log`
  - Command: `cargo test --lib quant::hadamard --no-default-features --features pg18,bench --no-run`
  - Result: passed.
- `artifacts/cargo-test-hadamard-pg18-bench-runtime-attempt.log`
  - Command: `cargo test --lib quant::hadamard --no-default-features --features pg18,bench`
  - Result: failed before running test bodies with local PostgreSQL symbol
    lookup error: `undefined symbol: LockBuffer`.
- `artifacts/unsafe-counts.log`
  - Command: counted `unsafe` lines in `src` and `src/quant/hadamard.rs`
    before and after the code change.
  - Key lines:
    - `HEAD src unsafe count: 2412`
    - `working src unsafe count: 2409`
    - `HEAD hadamard unsafe count: 37`
    - `working hadamard unsafe count: 34`
