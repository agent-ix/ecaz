# Task 50 Packet 334 Artifact Manifest

- head SHA: `459817bf72b644492eeabeeb7df99397d840dfb7`
- parent SHA: `ea661ba7ec243c9811deb913c22b89b9b861e625`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/334-ivf-debug-vacuum-owned-stats/`
- timestamp: `2026-05-21T15:50:20-07:00`
- lane: Task 50 unsafe burndown, P13 IVF debug helper cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; test/debug vacuum stats API

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: before/after `src` unsafe count and broadened boundary-signature guard
  - key lines: `1345 -> 1344`; guard hits drop from three to two because `debug_ec_ivf_vacuum_stats` no longer returns `pg_sys::IndexBulkDeleteResult`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/334-ivf-debug-vacuum-owned-stats/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/334-ivf-debug-vacuum-owned-stats src`
  - result: wrote `1344` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/334-ivf-debug-vacuum-owned-stats/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1344 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
