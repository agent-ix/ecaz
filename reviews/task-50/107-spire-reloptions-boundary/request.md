# Task 50 Review Request: SPIRE Reloptions Boundary

## Summary

This packet reviews commit
`7c07d93689986730419edd2de43af83808f8f598`, which removes repeated internal
unsafe wrappers from SPIRE reloption parsing and routes SPIRE `amoptions`
through the shared AM callback boundary.

The slice removes `7` direct unsafe blocks from `src/` (`1750 -> 1743`).

## What Changed

- Made SPIRE `read_string_reloption` safe to call and kept the raw string
  offset plus C string reads inside that helper.
- Made `resolve_local_store_tablespace_plan` and `resolve_tablespace_name` safe
  to call, with the relcache tablespace read and PostgreSQL tablespace lookup
  retained as local residual unsafe.
- Converted SPIRE `ec_spire_amoptions` from a local `pgrx_extern_c_guard`
  unsafe block to the shared `pg_am_callback!` AM boundary.
- Removed repeated relation-options caller-side unsafe wrappers inside
  `relation_options`.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P1 FFI And Callback Boundary Contracts: SPIRE `amoptions` now uses the shared
  AM callback guard.
- P7 Reloptions And C String Contracts: SPIRE reloption string reads now have a
  safe helper boundary with a named residual owner.
- SPIRE remains the production target; this packet narrows one of the remaining
  SPIRE option/configuration surfaces before deeper coordinator and DML work.

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
| `src/` total direct unsafe blocks | 1750 | 1743 | -7 |
| `src/am/ec_spire/options/mod.rs` | 13 | 6 | -7 |
| `src/` unsafe ledger rows | 1750 | 1743 | -7 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check 7c07d936^ 7c07d936`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1743` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
