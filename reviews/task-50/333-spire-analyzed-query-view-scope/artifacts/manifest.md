# Task 50 Packet 333 Artifact Manifest

- head SHA: `803cfb908c4dadade6770cd162ffeca29ac4f78a`
- parent SHA: `3f9e2797ee940748eab43944925fbb3f44a918e7`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/333-spire-analyzed-query-view-scope/`
- timestamp: `2026-05-21T15:47:13-07:00`
- lane: Task 50 unsafe burndown, P11 analyzed Query view scope
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; SQL diagnostic query analysis boundary

## Artifacts

- `unsafe-counts-and-guard.log`
  - command: before/after `src` unsafe count and broadened boundary-signature guard
  - key lines: `1347 -> 1345`; guard hits drop from four to three because `src/storage/query.rs::analyze_single_query` no longer returns `*mut pg_sys::Query`
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/333-spire-analyzed-query-view-scope/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/333-spire-analyzed-query-view-scope src`
  - result: wrote `1345` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/333-spire-analyzed-query-view-scope/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1345 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
