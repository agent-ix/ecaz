# Task 50 Packet 343 Artifact Manifest

- head SHA: `2f80af40e2ce94b0114535438a36a301ac3033de`
- parent SHA: `a777a868ae812e7a3fb707ec47dc319979ccf092`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/343-callback-index-info-borrow/`
- timestamp: `2026-05-21T23:36:30Z`
- lane: Task 50 unsafe burndown, AM callback IndexInfo borrow cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: AM build/index validation callback metadata

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: current `src` unsafe count, IndexInfo raw-borrow scan, and broadened boundary-signature guard
  - key lines: direct unsafe count is `1311`; targeted IndexInfo raw-borrow hits are centralized in `src/am/common/pg_ptr.rs`; raw boundary guard reports no hits
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/343-callback-index-info-borrow/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/343-callback-index-info-borrow src`
  - result: wrote `1311` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/343-callback-index-info-borrow/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1311 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
