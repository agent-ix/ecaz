# Task 50 Packet 342 Artifact Manifest

- head SHA: `d3a75515e2a328d54590288d5464f6c38c04471c`
- parent SHA: `3703cb787bec229f7a35e4a8d5ea5318276ef341`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/342-hnsw-reloptions-nonnull-handle/`
- timestamp: `2026-05-21T23:32:25Z`
- lane: Task 50 unsafe burndown, HNSW reloptions boundary-signature cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: HNSW relation options

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: current `src` unsafe count and broadened boundary-signature guard
  - key lines: direct unsafe count is `1314`; raw boundary guard reports no hits
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/342-hnsw-reloptions-nonnull-handle/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/342-hnsw-reloptions-nonnull-handle src`
  - result: wrote `1314` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/342-hnsw-reloptions-nonnull-handle/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1314 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
