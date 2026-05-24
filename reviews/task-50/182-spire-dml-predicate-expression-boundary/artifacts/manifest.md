# Task 50 Packet 182 Artifact Manifest

- head SHA: `6d06e0b03a0714aa647c8ae748ca60dc7f044205`
- task bucket: `reviews/task-50/182-spire-dml-predicate-expression-boundary`
- lane: SPIRE DML frontdoor unsafe burndown
- fixture / storage format / rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable
- timestamp: 2026-05-21 00:38 America/Los_Angeles

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `script -q -e -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/cargo-check-pg18-bench.log`
  - result: passed
  - key lines: `Finished dev profile`; existing `src/am/mod.rs` unused-import warning remains.

- `cargo-check-pg18-pg-test.log`
  - command: `script -q -e -c "cargo check --all-targets --no-default-features --features pg18,pg_test" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/cargo-check-pg18-pg-test.log`
  - result: passed
  - key lines: `Finished dev profile`; existing `src/quant/hadamard.rs` test-helper dead-code warnings remain.

- `cargo-test-dml-frontdoor-predicate-pg18-no-run.log`
  - command: `script -q -e -c "cargo test --lib --no-default-features --features pg18,pg_test test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes --no-run" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/cargo-test-dml-frontdoor-predicate-pg18-no-run.log`
  - result: passed
  - key line: `Executable unittests src/lib.rs`.

- `cargo-pgrx-test-dml-predicate-edge-shapes-pg18-blocked.log`
  - command: `script -q -c "cargo pgrx test pg18 test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/cargo-pgrx-test-dml-predicate-edge-shapes-pg18-blocked.log`
  - result: blocked before tests ran
  - key line: `undefined symbol: BufferBlocks`.

- `cargo-pgrx-test-dml-const-coercion-pg18-blocked.log`
  - command: `script -q -c "cargo pgrx test pg18 test_ec_spire_dml_frontdoor_const_coercion_and_cte" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/cargo-pgrx-test-dml-const-coercion-pg18-blocked.log`
  - result: blocked before tests ran
  - key line: `undefined symbol: BufferBlocks`.

- `git-diff-check.log`
  - command: `script -q -e -c "git diff --check HEAD~1..HEAD" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/git-diff-check.log`
  - result: passed

- `rustfmt-dml-frontdoor-check.log`
  - command: `script -q -e -c "rustfmt --edition 2021 --check src/am/ec_spire/dml_frontdoor/mod.rs" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/rustfmt-dml-frontdoor-check.log`
  - result: passed
  - key lines: rustfmt emitted existing stable-toolchain warnings for unstable `imports_granularity` / `group_imports` config keys.

- `unsafe-block-count.log`
  - command: `script -q -e -c "make unsafe-block-count" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/unsafe-block-count.log`
  - result: passed
  - key line: `src/am/ec_spire/dml_frontdoor/mod.rs` now `48`.

- `unsafe-ledger-generate.log`
  - command: `script -q -e -c "make UNSAFE_LEDGER=reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/182-spire-dml-predicate-expression-boundary unsafe-ledger" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/unsafe-ledger-generate.log`
  - result: passed
  - key line: `wrote 1816 unsafe ledger rows`.

- `unsafe-ledger-check.log`
  - command: `script -q -e -c "make UNSAFE_LEDGER=reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/182-spire-dml-predicate-expression-boundary/artifacts/unsafe-ledger-check.log`
  - result: passed
  - key line: `ledger covers 1816 current unsafe rows`.

- `unsafe-ledger-after.jsonl`
  - result: generated ledger snapshot after `6d06e0b03a0714aa647c8ae748ca60dc7f044205`
  - key result: `1816` current unsafe rows.
