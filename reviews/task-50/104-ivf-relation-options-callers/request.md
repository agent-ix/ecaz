# Task 50 Review Request: IVF Relation Options Callers

## Summary

This packet reviews commit
`2ca55d98fbabaedd0e677a890e8ea9f7be121167`, which makes
`ec_ivf::options::relation_options` safe to call and removes the remaining
caller-side unsafe wrapper from the IVF admin diagnostic snapshot path.

The slice removes `1` direct unsafe block from `src/` (`1764 -> 1763`).

## What Changed

- Made IVF `relation_options` safe to call.
- Added a null relation guard before reading the relation descriptor.
- Kept raw `rd_options`, reloption struct casts, and string-offset reads
  centralized in `src/am/ec_ivf/options.rs`.
- Removed the IVF admin diagnostic caller-side unsafe wrapper.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P2 PostgreSQL handle views: IVF reloption reads no longer require callers to
  encode relation-pointer preconditions.
- P7 Reloptions And C String Contracts: IVF reloptions now have a safe API
  boundary and a named residual owner.
- Wave 2 IVF/RaBitQ cleanup remains active; this packet closes the reloptions
  API consistency gap before returning to larger IVF page/scan surfaces.

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
| `src/` total direct unsafe blocks | 1764 | 1763 | -1 |
| `src/am/ec_ivf/admin.rs` | 5 | 4 | -1 |
| `src/` unsafe ledger rows | 1764 | 1763 | -1 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check 2ca55d98^ 2ca55d98`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1763` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
