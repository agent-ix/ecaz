# Task 70 / Packet 007: Tombstone Retention Regression

## Packet Scope

- Code commit: `e633b9dd05d2b8ef6fe54f1674443e11af8e6208`
- Review driver: `reviews/task-70/004-frontier-neighbor-retention/feedback/2026-05-31-001-reviewer.md`
- Related revert: `7707117f1` reverted packet 005's retained-frontier heap slice per reviewer feedback.
- Manifest: `artifacts/manifest.md`

This packet requests review for the packet-004 tombstone-safety caveat before any further frontier performance slice.

## Code Change

`src/am/ec_diskann/scan.rs` now keeps tombstoned/stripped candidates in traversal but prevents them from consuming retained result slots:

- `greedy_descent_with` still expands every popped candidate's neighbors for connectivity.
- Only `picked.emittable` candidates are inserted into the bounded retained frontier used by the rerank stage.
- The public greedy-descent doc now states that returned candidates are the emittable frontier.
- New test `sc_018_tombstoned_top_scores_do_not_starve_emittable_frontier` builds a star graph where four tombstoned nodes have the best prefilter scores and four live nodes rank behind them. It asserts the live nodes still fill the rerank/result budget.

This keeps the packet-004 bounded-retention memory fix without starving scans on tombstone-heavy graphs. No new `unsafe` was introduced.

## Validation

Commands and logs:

- `cargo fmt --check`
- `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` -> `artifacts/cargo-test-diskann-scan.log`
- `cargo check --all-targets --no-default-features --features pg18` -> `artifacts/cargo-check-pg18.log`

The focused scan module now passes 19 tests, including the new tombstone-heavy regression.

## Reviewer Notes

This is a correctness packet, not a Phase 2 performance win. Packet 005's retained-frontier heap was reverted as requested. The next Task 70 performance step should be the packet-003/004/005 requested sub-timing pass that splits frontier residual time into heap operations, visited-set work, neighbor iteration, and bounded-retention maintenance.
