# Review Request: SPIRE Registered Page Operations

- task: Task 50 unsafe burndown
- packet: `reviews/task-50/384-spire-registered-page-operations`
- code commit: `06b8c36c3f1aad5faf385a302f59005102c50af0`
- scope: `src/am/ec_spire/page.rs`

## Summary

This slice consolidates SPIRE registered-page page primitives into operation-shaped helpers:

- `init_with_special` now performs metadata-page initialization and special-area copy under one checked boundary.
- `add_item_if_space` now combines page free-space read, FSM update for insufficient space, and tuple append.
- `record_current_free_space` records the current FSM entry without exposing a caller-composed `free_space` value.
- `delete_no_compact_checked` combines max-offset validation and no-compact tuple deletion for the same registered page.

This removes the separate low-level `init`, `copy_to_special`, `free_space`, `record_free_space`, `add_item`, `max_offset`, and `delete_no_compact` call surface inside `SpireRegisteredPage`.

## Unsafe Count

- before this slice: `1156`
- after this slice: `1154`

## Validation

Artifacts are packet-local under `reviews/task-50/384-spire-registered-page-operations/artifacts/`.

- `cargo-check-pg18-bench.log`: passed; only the pre-existing unused SPIRE DML re-export warning in `src/am/mod.rs`.
- `rustfmt-page.log`: passed; only existing stable-toolchain warnings for unstable rustfmt settings.
- `git-diff-check.log`: passed.
- `raw-boundary-guard.log`: passed; no matches.
- `src-unsafe-count.log`: `1154`.
- `unsafe-ledger-generate.log`: generated 1154 ledger rows.
- `unsafe-ledger-check.log`: passed; ledger covers 1154 current unsafe rows.
