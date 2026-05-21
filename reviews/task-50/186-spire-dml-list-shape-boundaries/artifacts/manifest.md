# Manifest: SPIRE DML List Shape Boundaries

- head SHA: `e767f92b8d097f1217364093be6a2dec227ea5e0`
- task bucket: `reviews/task-50/186-spire-dml-list-shape-boundaries`
- timestamp: `2026-05-21 00:33 PDT`
- lane: Task 50 unsafe burndown
- fixture/storage/rerank: SPIRE DML frontdoor planner list/range-table shape classification
- isolated one-index-per-table/shared-table: not applicable; compile/ledger validation and one targeted pgrx attempt

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed; existing `src/am/mod.rs` unused import warnings
- `cargo-check-pg18-pg-test.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,pg_test`
  - result: passed; existing Hadamard test-helper dead-code warnings
- `cargo-test-dml-frontdoor-predicate-pg18-no-run.log`
  - command: `cargo test --lib --no-default-features --features pg18,pg_test test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes --no-run`
  - result: passed
- `cargo-pgrx-test-dml-predicate-edge-shapes-pg18-blocked.log`
  - command: `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes`
  - result: blocked before the test body by local runtime linker error `undefined symbol: BufferBlocks`
- `rustfmt-dml-frontdoor-check.log`
  - command: `rustfmt --edition 2021 --check src/am/ec_spire/dml_frontdoor/mod.rs`
  - result: passed; known stable-rustfmt warnings for unstable import grouping options
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `unsafe-block-count.log`
  - command: `rg -n 'unsafe \\{' src/am/ec_spire/dml_frontdoor/mod.rs`
  - key result: `45` direct unsafe rows remain in `src/am/ec_spire/dml_frontdoor/mod.rs`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/186-spire-dml-list-shape-boundaries/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/186-spire-dml-list-shape-boundaries src`
  - key result: `1793` current direct unsafe rows under `src/`
- `unsafe-ledger-generate.log`
  - key result: `wrote 1793 unsafe ledger rows`
- `unsafe-ledger-check.log`
  - command: `python3 scripts/unsafe_ledger.py check --ledger reviews/task-50/186-spire-dml-list-shape-boundaries/artifacts/unsafe-ledger-after.jsonl src`
  - key result: `ledger covers 1793 current unsafe rows`

