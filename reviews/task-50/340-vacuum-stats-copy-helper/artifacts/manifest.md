# Task 50 Packet 340 Artifact Manifest

- head SHA: `65ea0caffb65f55afd373d7be9f3e256618157d5`
- parent SHA: `47981f3f9d5e2f751a732144d4ee1c0abcaf5855`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/340-vacuum-stats-copy-helper/`
- timestamp: `2026-05-21T23:22:27Z`
- lane: Task 50 unsafe burndown, AM vacuum stats copy cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; AM debug vacuum stats copying

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: current `src` unsafe count, vacuum stats raw-copy scan, and broadened boundary-signature guard
  - key lines: direct unsafe count is `1320`; targeted `unsafe { *stats }` / `&*stats` pattern is gone from the scanned AM vacuum/debug paths; one guard hit remains in `src/am/ec_hnsw/options.rs`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/340-vacuum-stats-copy-helper/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/340-vacuum-stats-copy-helper src`
  - result: wrote `1320` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/340-vacuum-stats-copy-helper/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1320 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
