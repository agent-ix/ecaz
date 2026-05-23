# Task 50 Packet 341 Artifact Manifest

- head SHA: `e24a59b7e40557225e5456a342e11db63d0c01eb`
- parent SHA: `4b63255c3ba11a7c5e606c92beda704140925db1`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/341-callback-datum-array-view/`
- timestamp: `2026-05-21T23:27:05Z`
- lane: Task 50 unsafe burndown, callback datum/isnull array read cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; AM build callback value array decoding

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: current `src` unsafe count, callback datum/isnull raw deref scan, and broadened boundary-signature guard
  - key lines: direct unsafe count is `1315`; targeted datum/isnull raw deref pattern is gone from the scanned AM directories; one guard hit remains in `src/am/ec_hnsw/options.rs`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/341-callback-datum-array-view/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/341-callback-datum-array-view src`
  - result: wrote `1315` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/341-callback-datum-array-view/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1315 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
