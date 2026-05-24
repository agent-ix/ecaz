# Task 50 Review Request: SPIRE Remote Candidate Wrapper Boundaries

## Summary

This packet reviews commit
`76eb15e6c9f61b66b9ae83dbcf8480c87b590d26`, which removes repeated internal
unsafe wrappers from SPIRE remote candidate fanout, libpq planning, executor
receive, and production scan output paths.

The slice removes `37` direct unsafe blocks from `src/` (`1743 -> 1706`).

## What Changed

- Made the SPIRE remote-search request, readiness, execution, libpq request,
  libpq connection, libpq dispatch, production executor-state, degraded-skip,
  production session, scan handoff, heap-resolution summary, and read-profile
  wrapper rows safe to call.
- Added a safe SQL wrapper relation guard macro for AM helpers whose raw
  PostgreSQL access has been pushed behind a lower-level owner.
- Removed caller-side unsafe wrappers from SPIRE remote candidate fanout,
  libpq dispatch/bind/executor paths, production scan output, hierarchy
  snapshot summary, and custom scan tuple payload result streaming.
- Kept residual unsafe blocks in lower-level relation/page/descriptor helpers
  where the raw PostgreSQL pointer reads still live.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: SQL wrapper call sites now have a
  safe-call guard for helpers that no longer require unsafe invocation.
- P3 SPIRE Search/Custom Scan/Remote Candidate Boundaries: remote candidate
  fanout, libpq planning, executor receive, and scan-output wrappers no longer
  require repeated caller-side unsafe blocks.
- P6 SPIRE Storage And Metadata Helpers: hierarchy snapshot result summary
  callers now use a safe helper boundary.
- SPIRE remains the production target; this packet continues shrinking the
  coordinator and production scan surfaces before deeper residual-owner passes.

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
| `src/` total direct unsafe blocks | 1743 | 1706 | -37 |
| `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` | 16 | 2 | -14 |
| `src/am/ec_spire/coordinator/remote_candidates/executor_receive.rs` | 12 | 5 | -7 |
| `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs` | 7 | 2 | -5 |
| `src/am/ec_spire/coordinator/remote_candidates/fanout.rs` | 8 | 4 | -4 |
| `src/` unsafe ledger rows | 1743 | 1706 | -37 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check 76eb15e6^ 76eb15e6`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1706` current `src/`
  unsafe rows.

No live pgrx smoke was run for this slice because the change is a safe wrapper
boundary cleanup rather than a new PostgreSQL callback or runtime behavior path.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
