# Task 50 Review Request: SPIRE Remote Candidate Residual Boundaries

## Summary

This packet reviews commit
`a649fe576f9731301533b17e93ffb161afa7baa1`, which removes additional
caller-side unsafe blocks from SPIRE remote candidate relation-OID, secret,
connection-open, executor receive, and coordinator gate wrapper paths.

The slice removes `14` direct unsafe blocks from `src/` (`1694 -> 1680`).

## What Changed

- Added a single `remote_candidate_index_oid` helper to own SPIRE remote
  candidate relation OID reads and validate null/invalid relation inputs.
- Removed direct relation-OID reads from libpq connection planning, executor
  receive, identity cache, heap candidate, and coordinator pipeline callers.
- Made libpq secret plan/summary and connection-open plan/summary wrappers safe
  to call.
- Made the coordinator gate summary wrapper safe to call and removed two now
  unnecessary hierarchy snapshot caller-side unsafe blocks.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: SQL wrapper call sites now use the
  safe relation guard for the newly-safe SPIRE remote candidate helpers.
- P3 SPIRE Search/Custom Scan/Remote Candidate Boundaries: remote candidate
  secret, connection-open, executor receive, identity cache, and coordinator
  gate layers no longer require repeated caller-side unsafe blocks.
- Residual unsafe in this slice is intentionally concentrated in
  `remote_candidate_index_oid` for relation OID reads and in the existing
  local heap-resolution fallback owner.

## Evidence

- Code diff: `artifacts/code-diff.patch`
- Validation: `artifacts/cargo-check-pg18-bench.log`
- Whitespace check: `artifacts/git-diff-check.log`
- Unsafe count: `artifacts/src-unsafe-block-count-after.log`
- Count summary: `artifacts/count-summary.md`
- Ledger: `artifacts/unsafe-ledger-after.jsonl`
- Ledger generation/check logs:
  `artifacts/unsafe-ledger-generate.log`,
  `artifacts/unsafe-ledger-check.log`

## Result

Direct unsafe movement:

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1694 | 1680 | -14 |
| `src/am/ec_spire/coordinator/remote_candidates/executor_receive.rs` | 5 | 0 | -5 |
| `src/am/ec_spire/coordinator/remote_candidates/operator.rs` | 3 | 0 | -3 |
| `src/am/ec_spire/coordinator/remote_candidates/pipeline.rs` | 4 | 1 | -3 |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 17 | 15 | -2 |
| `src/` unsafe ledger rows | 1694 | 1680 | -14 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check a649fe57^ a649fe57`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1680` current `src/`
  unsafe rows.

No live pgrx smoke was run for this slice because it is a wrapper-boundary
cleanup and does not add a new PostgreSQL callback or runtime path.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
