# Task 50 Packet 331 Artifact Manifest

- head SHA: `28e8b780ccec47f590fb502e8f72f7d53e34fa0c`
- parent SHA: `c96b44c1ee44d8ff81ea9dd91af6cedf67b04487`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/331-spire-dml-test-node-tag-removal/`
- timestamp: `2026-05-21T15:38:07-07:00`
- lane: Task 50 unsafe burndown, P13 DML frontdoor test helper cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; test assertion cleanup

## Artifacts

- `unsafe-counts-before-after.log`
  - command: before/after `rg -n 'unsafe\s*\{'` count for `src/tests/dml_frontdoor.rs` plus current `src/` total
  - key lines: `src/tests/dml_frontdoor.rs`: `3 -> 2`; current `src/` total: `1347`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/331-spire-dml-test-node-tag-removal/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/331-spire-dml-test-node-tag-removal src`
  - result: wrote `1347` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/331-spire-dml-test-node-tag-removal/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1347 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
