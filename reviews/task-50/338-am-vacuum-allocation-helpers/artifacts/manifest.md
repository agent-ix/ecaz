# Task 50 Packet 338 Artifact Manifest

- head SHA: `6b1025cf4832996cd94092f1e5d00e0669bc9ada`
- parent SHA before slice: `2cbc84be77e92813207a41e7f4301fabec2739e9`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/338-am-vacuum-allocation-helpers/`
- timestamp: `2026-05-21T23:13:35Z`
- lane: Task 50 unsafe burndown, AM vacuum allocation boundary cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; AM vacuum allocation helpers

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: current `src` unsafe count, AM vacuum `alloc0` scan, and broadened boundary-signature guard
  - key lines: direct unsafe count is `1327`; AM vacuum `alloc0` direct use now appears only in `src/am/common/vacuum.rs`; one guard hit remains in `src/am/ec_hnsw/options.rs`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/338-am-vacuum-allocation-helpers/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/338-am-vacuum-allocation-helpers src`
  - result: wrote `1327` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/338-am-vacuum-allocation-helpers/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1327 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~2..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
