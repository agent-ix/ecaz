# Task 50 Packet 339 Artifact Manifest

- head SHA: `44aca7d57af00b23256d2a97ba1bedf07d2a11f9`
- parent SHA: `5fbf109cb3160d98aa1b78281af3a5e5b7e17735`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/339-callback-heap-tid-decoder/`
- timestamp: `2026-05-21T23:18:59Z`
- lane: Task 50 unsafe burndown, callback heap TID pointer-copy cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; AM build callback TID decoding

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: current `src` unsafe count, callback heap TID deref scan, and broadened boundary-signature guard
  - key lines: direct unsafe count is `1324`; targeted `item_pointer_get_both(unsafe { *tid })` pattern is gone from the four touched build paths; one guard hit remains in `src/am/ec_hnsw/options.rs`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/339-callback-heap-tid-decoder/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/339-callback-heap-tid-decoder src`
  - result: wrote `1324` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/339-callback-heap-tid-decoder/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1324 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
