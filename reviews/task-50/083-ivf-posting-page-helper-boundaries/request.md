# Task 50 Review Request: IVF Posting Page Helper Boundaries

## Summary

This packet reviews commit `814b815e534329f174a019566c33ef46bbba63e2`, which makes IVF posting/list-directory helper APIs safe to call and removes caller-side unsafe wrappers from IVF insert, scan, vacuum, and page code.

The slice advances the comprehensive burndown plan programs P3, P4, and P6 by moving unsafe responsibility into the lower IVF page helper boundary instead of requiring every caller to wrap helper invocation in `unsafe`.

## Changed Files

- `src/am/ec_ivf/admin.rs`
- `src/am/ec_ivf/insert.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/vacuum.rs`

## Count Delta

See `artifacts/count-summary.md`.

The touched IVF files remove 8 direct unsafe blocks. The `src/` total moves from 2004 after packet 082 to 1996 after this packet.

## Validation

- `git diff --check HEAD^ HEAD`
  - log: `artifacts/git-diff-check.log`
  - result: exit code 0
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - result: exit code 0 with the existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`
  - log: `artifacts/src-unsafe-block-count-after.log`
  - result: `1996` current direct unsafe blocks under `src/`
- `make unsafe-ledger`
  - log: `artifacts/unsafe-ledger-generate.log`
  - result: generated `1996` ledger rows
- `make unsafe-ledger-check`
  - log: `artifacts/unsafe-ledger-check.log`
  - result: `ledger covers 1996 current unsafe rows`

## Residual Unsafe

Residual IVF unsafe remains in scan/page/build/vacuum/insert/admin/cost around scan descriptor access, lower page tuple and WAL primitives, relation lock/open internals, tuple/datum reads, page tuple views, and stats/debug wrappers. Those are not considered closed out by this packet; they remain under the Task 50 ledger for later deletion or residual registration.
