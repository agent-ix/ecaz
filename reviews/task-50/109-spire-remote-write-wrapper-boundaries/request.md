# Task 50 Review Request: SPIRE Remote Write Wrapper Boundaries

## Summary

This packet reviews commit
`00a042fb9b15db43390173de3190d48d55f18153`, which removes repeated unsafe
callers from SPIRE remote write payload preparation and dispatch wrappers.

The slice removes `12` direct unsafe blocks from `src/` (`1706 -> 1694`).

## What Changed

- Made `coordinator_insert_dispatch_plan_row` safe to call, with the relation
  OID read retained as the local residual unsafe owner in `libpq_plan.rs`.
- Made SPIRE coordinator insert prepare, insert tuple payload, insert batch,
  update, delete prepare, and select remote tuple payload wrappers safe to
  call.
- Removed all direct unsafe blocks from
  `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs`.
- Moved SQL wrappers and pg_test helpers that call those remote-write helpers
  onto safe invocation paths.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: SQL and test callers no longer need
  direct unsafe invocation for remote write helper wrappers.
- P3 SPIRE Search/Custom Scan/Remote Candidate Boundaries: remote write payload
  preparation now has a safe wrapper boundary with the raw relation OID read
  owned by `coordinator_insert_dispatch_plan_row`.
- SPIRE remains the production target; this packet clears another coordinator
  write surface before the remaining storage/custom-scan residual passes.

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
| `src/` total direct unsafe blocks | 1706 | 1694 | -12 |
| `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs` | 9 | 0 | -9 |
| `src/tests/mod.rs` | 40 | 38 | -2 |
| `src/tests/insert.rs` | 16 | 15 | -1 |
| `src/` unsafe ledger rows | 1706 | 1694 | -12 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check 00a042fb^ 00a042fb`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1694` current `src/`
  unsafe rows.

No live pgrx smoke was run for this slice because the change removes caller-side
unsafe around existing remote-write wrappers and does not add a new PostgreSQL
callback or runtime path.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
