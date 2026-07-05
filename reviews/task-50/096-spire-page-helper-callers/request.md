# Task 50 Review Request: SPIRE Page Helper Callers

## Summary

This packet reviews commit
`128d3324024c6e8f052cb99f8e975b2b7f805560`, which makes SPIRE page and
publish helper APIs safe to call and removes broad caller-side unsafe across
SPIRE build, publish, debug, snapshot, scan, insert, update, and vacuum paths.

The slice removes `89` direct unsafe blocks from `src/` (`1915 -> 1826`).

## What Changed

- Made SPIRE root/control page initialization and read helpers safe to call.
- Made object tuple append/read, object tuple scan, same-length rewrite, and
  no-compact delete helpers safe at the page API boundary.
- Made publish helper APIs for manifest bundles, retired manifests,
  replacement epochs, and placement entries safe to call.
- Rolled those safe APIs through SPIRE callers, including production build,
  insert, update publish, vacuum, coordinator snapshots, planner placement
  eligibility, relation scan loading, and debug helpers.

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
| `src/` total direct unsafe blocks | 1915 | 1826 | -89 |
| `src/am/ec_spire/build/publish.rs` | 9 | 0 | -9 |
| `src/am/ec_spire/build/recursive.rs` | 3 | 0 | -3 |
| `src/am/ec_spire/coordinator/debug.rs` | 29 | 9 | -20 |
| `src/am/ec_spire/page.rs` | 27 | 19 | -8 |
| `src/am/ec_spire/update/publish/relation.rs` | 9 | 3 | -6 |
| `src/` unsafe ledger rows | 1915 | 1826 | -89 |

See `artifacts/count-summary.md` for the complete touched-file count table.

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1826` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.
