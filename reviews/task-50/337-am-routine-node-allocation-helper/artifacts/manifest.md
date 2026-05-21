# Task 50 Packet 337 Artifact Manifest

- head SHA: `d2f494f835e041a4b0baf4aefce9d247335152f7`
- parent SHA: `cba3abe2dab7ed5b20fa9d333ffb7e94ecdcae93`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/337-am-routine-node-allocation-helper/`
- timestamp: `2026-05-21T23:04:56Z`
- lane: Task 50 unsafe burndown, AM routine boundary cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; AM routine handler construction

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: current `src` unsafe count, `IndexAmRoutine` allocation scan, and broadened boundary-signature guard
  - key lines: direct unsafe count is `1336`; direct `IndexAmRoutine` allocation now appears only in `src/am/common/routine.rs`; one guard hit remains in `src/am/ec_hnsw/options.rs`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/337-am-routine-node-allocation-helper/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/337-am-routine-node-allocation-helper src`
  - result: wrote `1336` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/337-am-routine-node-allocation-helper/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1336 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
