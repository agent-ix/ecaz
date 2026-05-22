# Review Request: SPIRE Diagnostics Manifest Boundary

Task: `plan/tasks/50-unsafe-burndown.md`

Code commit: `fcd544d8e`

## Summary

This slice consolidates SPIRE boundary-replica diagnostics manifest reads in `src/am/ec_spire/coordinator/diagnostics.rs`.

- The epoch, object, and placement manifest tuple reads now share one relation/root-control boundary.
- The SAFETY comment names the shared invariant: all tuple IDs come from the same root/control state read from the same live SPIRE index relation, and each page helper returns owned bytes before decoded manifests are cross-checked.
- No safe raw-pointer helper signatures were added.

Unsafe count movement:

- `src/am/ec_spire/coordinator/diagnostics.rs`: 3 -> 1 direct `unsafe {` blocks.
- `src`: 1158 -> 1156 direct `unsafe {` blocks.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- `git diff --check` passed.
- `rustfmt --check src/am/ec_spire/coordinator/diagnostics.rs` passed, with stable rustfmt's known warnings for ignored nightly-only import grouping options.
- Raw-boundary guard found no public safe raw PG boundary helper signatures.
- Unsafe ledger generated and checked: `ledger covers 1156 current unsafe rows`.

Artifacts are in `reviews/task-50/382-spire-diagnostics-manifest-boundary/artifacts/`.
