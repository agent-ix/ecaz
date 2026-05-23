# Task 50 Review Request: Planner Cost Soundness Follow-Up

## Summary

Code commit: `d888b912fd4371be8919478d0f587988619d3af0`

This packet addresses reviewer soundness finding #1 from
`reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-01-reviewer.md`
and the cross-posted finding in packet 100.

Change:

- `current_planner_cost_constants` is now `unsafe fn`
- `current_cpu_tuple_cost` is now `unsafe fn`
- all HNSW, IVF, SPIRE, DiskANN, and SPIRE custom-scan cost callers now
  explicitly acknowledge the planner/backend context before reading
  PostgreSQL backend-local planner cost globals

This is a soundness follow-up, not a count-reduction packet. Direct unsafe count
increased from packet 140's `1541` to `1551` because previously hidden
backend-global reads are now explicit at the call sites.

## Validation

- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - passed with the known pre-existing `src/am/mod.rs` unused import warning
- `make unsafe-block-count`
  - `unsafe_blocks 1551`
  - `files 124`
- `make unsafe-ledger`
- `make unsafe-ledger-check`
  - `ledger covers 1551 current unsafe rows`

## Artifacts

- `artifacts/code-stat.log`
- `artifacts/code-diff.patch`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/src-unsafe-block-count-after.log`
- `artifacts/count-summary.md`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
