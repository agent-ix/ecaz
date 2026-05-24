# Task 50 Packet 180 Artifact Manifest

- head SHA: `610172d0e4e8e7ca7d36edd8d49b50faa8623bea`
- task bucket: `reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation`
- lane: RaBitQ / quant Hadamard unsafe burndown
- fixture / storage format / rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable
- timestamp: 2026-05-20 23:57 America/Los_Angeles

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `script -q -e -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/cargo-check-pg18-bench.log`
  - result: passed
  - key lines: `Finished dev profile`; existing `src/am/mod.rs` unused-import warning remains.

- `cargo-check-pg18-pg-test.log`
  - command: `script -q -e -c "cargo check --all-targets --no-default-features --features pg18,pg_test" reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/cargo-check-pg18-pg-test.log`
  - result: passed
  - key lines: `Finished dev profile`; existing `src/quant/hadamard.rs` test-helper dead-code warnings remain.

- `cargo-test-hadamard-pg18-no-run.log`
  - command: `script -q -e -c "cargo test --lib --no-default-features --features pg18,pg_test quant::hadamard --no-run" reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/cargo-test-hadamard-pg18-no-run.log`
  - result: passed
  - key line: `Executable unittests src/lib.rs`.

- `cargo-test-hadamard-pg18-run-blocked.log`
  - command: `script -q -c "cargo test --lib --no-default-features --features pg18,pg_test quant::hadamard" reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/cargo-test-hadamard-pg18-run-blocked.log`
  - result: blocked before tests ran
  - key line: `undefined symbol: LockBuffer`.

- `git-diff-check.log`
  - command: `script -q -e -c "git diff --check HEAD~1..HEAD" reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/git-diff-check.log`
  - result: passed

- `rustfmt-hadamard-check.log`
  - command: `script -q -e -c "rustfmt --edition 2021 --check src/quant/hadamard.rs" reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/rustfmt-hadamard-check.log`
  - result: passed
  - key lines: rustfmt emitted existing stable-toolchain warnings for unstable `imports_granularity` / `group_imports` config keys.

- `unsafe-block-count.log`
  - command: `script -q -e -c "make unsafe-block-count" reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/unsafe-block-count.log`
  - result: passed
  - key line: `src/quant/hadamard.rs` now `31`.

- `unsafe-ledger-generate.log`
  - command: `script -q -e -c "make UNSAFE_LEDGER=reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation unsafe-ledger" reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/unsafe-ledger-generate.log`
  - result: passed
  - key line: `wrote 1829 unsafe ledger rows`.

- `unsafe-ledger-check.log`
  - command: `script -q -e -c "make UNSAFE_LEDGER=reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/unsafe-ledger-check.log`
  - result: passed
  - key line: `ledger covers 1829 current unsafe rows`.

- `unsafe-ledger-after.jsonl`
  - result: generated ledger snapshot after `610172d0e4e8e7ca7d36edd8d49b50faa8623bea`
  - key result: `1829` current unsafe rows.
