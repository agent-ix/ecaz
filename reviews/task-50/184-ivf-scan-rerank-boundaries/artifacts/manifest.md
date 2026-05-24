# Manifest: IVF Scan Rerank Boundaries

- head SHA: `3331d5408d7b7a472a9de96cf38218580539b4a4`
- task bucket: `reviews/task-50/184-ivf-scan-rerank-boundaries`
- timestamp: `2026-05-21 00:20 PDT`
- lane: Task 50 unsafe burndown
- fixture/storage/rerank: IVF scan, heap-f32 rerank setup, scan output debug helpers
- isolated one-index-per-table/shared-table: not applicable; compile/ledger validation and one targeted pgrx attempt

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed; existing `src/am/mod.rs` unused import warnings
- `cargo-check-pg18-pg-test.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,pg_test`
  - result: passed; existing Hadamard test-helper dead-code warnings
- `cargo-test-ivf-gettuple-pg18-no-run.log`
  - command: `cargo test --lib --no-default-features --features pg18,pg_test test_ec_ivf_gettuple_emits_probe_candidates_with_scores --no-run`
  - result: passed
- `cargo-pgrx-test-ivf-gettuple-pg18-blocked.log`
  - command: `cargo pgrx test pg18 test_ec_ivf_gettuple_emits_probe_candidates_with_scores`
  - result: blocked before the test body by local runtime linker error `undefined symbol: BufferBlocks`
- `rustfmt-ivf-scan-check.log`
  - command: `rustfmt --edition 2021 --check src/am/ec_ivf/scan.rs`
  - result: passed; known stable-rustfmt warnings for unstable import grouping options
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `unsafe-block-count.log`
  - command: `rg -n 'unsafe \\{' src/am/ec_ivf/scan.rs`
  - key result: `27` direct unsafe rows remain in `src/am/ec_ivf/scan.rs`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/184-ivf-scan-rerank-boundaries/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/184-ivf-scan-rerank-boundaries src`
  - key result: `1801` current direct unsafe rows under `src/`
- `unsafe-ledger-generate.log`
  - key result: `wrote 1801 unsafe ledger rows`
- `unsafe-ledger-check.log`
  - command: `python3 scripts/unsafe_ledger.py check --ledger reviews/task-50/184-ivf-scan-rerank-boundaries/artifacts/unsafe-ledger-after.jsonl src`
  - key result: `ledger covers 1801 current unsafe rows`

