# Task 50 Review Request: SPIRE Manifest Loader Callers

## Summary

This packet reviews commit
`29e734e59768904a4c18496762d3c907975bacb9`, which makes SPIRE coordinator
manifest-loader helpers safe to call and removes direct caller-side unsafe from
debug, diagnostics, active snapshot, remote-candidate fanout, production fault
matrix, and production scan-output paths.

The slice removes `9` direct unsafe blocks from `src/` (`1826 -> 1817`).

## What Changed

- Made `load_relation_epoch_manifests_for_coordinator_fanout` safe to call.
- Made `load_relation_epoch_manifests_for_boundary_placement_diagnostics` safe
  to call.
- Removed the corresponding caller-side unsafe wrappers across the SPIRE
  coordinator and remote-candidate surfaces.
- Kept the residual page/TID boundary centralized in
  `page::read_object_tuple`, which owns the pinned object tuple copy contract.

## Plan Coverage

This advances the comprehensive Task 50 plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

- P2 PostgreSQL handle views: callers no longer encode manifest tuple-read
  preconditions locally.
- P11 Planner, Node, List, And Custom Scan Views: SPIRE remote-candidate
  coordinator paths shed repeated unsafe wrappers around manifest loading.
- Wave 2 item 20: SPIRE remote-candidate coordinator views.

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
| `src/` total direct unsafe blocks | 1826 | 1817 | -9 |
| `src/am/ec_spire/coordinator/debug.rs` | 9 | 8 | -1 |
| `src/am/ec_spire/coordinator/diagnostics.rs` | 4 | 2 | -2 |
| `src/am/ec_spire/coordinator/remote_candidates/fanout.rs` | 10 | 8 | -2 |
| `src/am/ec_spire/coordinator/remote_candidates/fault_matrix.rs` | 2 | 1 | -1 |
| `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` | 22 | 20 | -2 |
| `src/am/ec_spire/coordinator/snapshots.rs` | 11 | 10 | -1 |
| `src/` unsafe ledger rows | 1826 | 1817 | -9 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1817` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan.
