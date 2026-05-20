# Task 50 Review Request: SPIRE Relation Options Callers

## Summary

This packet reviews commit
`44c1f2beddcb7f29185c5330c9dcc8c0ba3c3903`, which makes
`options::relation_options` safe to call for SPIRE and removes caller-side
unsafe wrappers from reloption consumers.

The slice removes `9` direct unsafe blocks from `src/` (`1810 -> 1801`).

## What Changed

- Made `src/am/ec_spire/options/mod.rs::relation_options` safe to call.
- Added a null relation guard before reading the relation descriptor.
- Kept the raw `rd_options` read, reloptions struct cast, and C-string offset
  reads centralized in the SPIRE options module.
- Removed simple caller-side unsafe wrappers in cost, insert, active snapshot,
  custom scan explain, endpoint identity, and production scan-output paths.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P2 PostgreSQL handle views: live relation reloption reads are centralized
  instead of repeated at call sites.
- P7 Reloptions And C String Contracts: SPIRE reloptions access now has a safe
  API boundary and a named residual owner.
- Wave 2 item 20: SPIRE remote-candidate coordinator views.
- Wave 2 item 24: IVF/SPIRE-style reloptions cleanup pattern, applied here to
  SPIRE before crossing into the remaining AMs.

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
| `src/` total direct unsafe blocks | 1810 | 1801 | -9 |
| `src/am/ec_spire/coordinator/remote_candidates/endpoint_identity.rs` | 2 | 1 | -1 |
| `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` | 18 | 16 | -2 |
| `src/am/ec_spire/coordinator/snapshots.rs` | 9 | 8 | -1 |
| `src/am/ec_spire/cost/mod.rs` | 18 | 15 | -3 |
| `src/am/ec_spire/custom_scan/explain.rs` | 3 | 2 | -1 |
| `src/am/ec_spire/insert.rs` | 12 | 11 | -1 |
| `src/` unsafe ledger rows | 1810 | 1801 | -9 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1801` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
