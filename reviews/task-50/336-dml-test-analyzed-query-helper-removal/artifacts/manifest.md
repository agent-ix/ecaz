# Task 50 Packet 336 Artifact Manifest

- head SHA: `74fe15dd8c0c36665f8afe51855d9cfa591509ef`
- parent SHA: `28d8308064ffa3b406b961ac255ab0a1a8c75462`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/336-dml-test-analyzed-query-helper-removal/`
- timestamp: `2026-05-21T23:00:36Z`
- lane: Task 50 unsafe burndown, DML test raw query helper cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; test fixture helper cleanup

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: current `src` unsafe count, broadened boundary-signature guard, and `analyzed_query` reference scan
  - key lines: direct unsafe count is `1339`; one guard hit remains in `src/am/ec_hnsw/options.rs`; no `analyzed_query(` references remain under `src/tests`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/336-dml-test-analyzed-query-helper-removal/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/336-dml-test-analyzed-query-helper-removal src`
  - result: wrote `1339` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/336-dml-test-analyzed-query-helper-removal/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1339 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
