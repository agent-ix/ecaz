# Task 50 Review Request: SPIRE AM OID Helper Boundary

## Summary

This packet reviews commit
`404f44b7d78773682d03ea497674bff178453036`, which centralizes the SPIRE access
method OID lookup behind one safe helper.

The slice removes `1` net direct unsafe block from `src/` (`1676 -> 1675`).

## What Changed

- Added `ec_spire_access_method_oid` as the single SPIRE owner for the
  `get_index_am_oid("ec_spire")` syscache lookup.
- Removed duplicate inline unsafe lookups from DML frontdoor and custom-scan
  planner code.
- Kept the syscache call as the named residual owner in `src/am/ec_spire/mod.rs`.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: one PostgreSQL syscache boundary now
  owns SPIRE AM OID lookup instead of duplicated caller unsafe.
- P2 PostgreSQL Handle Views: DML/custom-scan catalog callers now use a typed
  helper for AM lookup disposition.

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
| `src/` total direct unsafe blocks | 1676 | 1675 | -1 |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 25 | 24 | -1 |
| `src/am/ec_spire/custom_scan/planner.rs` | 9 | 8 | -1 |
| `src/am/ec_spire/mod.rs` | 0 | 1 | +1 |
| `src/` unsafe ledger rows | 1676 | 1675 | -1 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check 404f44b7^ 404f44b7`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1675` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
