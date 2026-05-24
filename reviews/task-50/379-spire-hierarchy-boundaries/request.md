# Review Request: SPIRE Hierarchy Boundaries

Task: `plan/tasks/50-unsafe-burndown.md`

Code commit: `1548a3e7ba30d7b71059d3225495a545a816785d`

## Summary

This slice consolidates SPIRE coordinator hierarchy unsafe boundaries in `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`.

- Remote/local heap candidate reconstruction now resolves the indexed vector attribute and constructs the heap slot reader in one heap-read boundary.
- Coordinator fanout manifest loading now reads the epoch, object, and placement tuples inside one relation/root-control manifest boundary.
- No safe raw-pointer helper signatures were added.
- `rustfmt` also wrapped one long line in the same file.

Unsafe count movement:

- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`: 5 -> 2 direct `unsafe {` blocks.
- `src`: 1167 -> 1164 direct `unsafe {` blocks.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- `git diff --check` passed.
- `rustfmt --check src/am/ec_spire/coordinator/hierarchy_snapshots.rs` passed, with stable rustfmt's known warnings for ignored nightly-only import grouping options.
- Raw-boundary guard found no public safe raw PG boundary helper signatures.
- Unsafe ledger generated and checked: `ledger covers 1164 current unsafe rows`.

Artifacts are in `reviews/task-50/379-spire-hierarchy-boundaries/artifacts/`.
