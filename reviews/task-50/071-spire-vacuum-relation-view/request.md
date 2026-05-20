# Task 50 Review Request: SPIRE Vacuum Relation View

## Summary

This packet starts a SPIRE production vacuum fanout slice.

The code change introduces `SpireVacuumIndexRelation`, a private live relation view used by vacuum and delete-delta publish code for:

- root/control page reads;
- active epoch manifest loads;
- local store config loads;
- read/write object-store set opens;
- placement row writes;
- replacement epoch publication.

This removes repeated caller-owned unsafe blocks around the same live index relation invariant while leaving callback entry, publish locking, vacuum stats, and heap-dead callback boundaries explicit.

## Counts

- `src/am/ec_spire/vacuum/mod.rs`: 31 -> 26 direct unsafe blocks
- `src/` total: 2074 -> 2069 direct unsafe blocks

See `artifacts/count-summary.md`.

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`
- unsafe ledger generated and checked:
  - `unsafe-ledger-after.jsonl` contains 2069 current `src/` rows
  - `unsafe-ledger-check.log`: `ledger covers 2069 current unsafe rows`

## Residual / Follow-Up

This is not SPIRE vacuum closeout. The file still has 26 direct unsafe blocks, including PostgreSQL callback entry, publish-lock acquisition, heap-dead callback invocation, vacuum stats allocation/mutation, debug test callbacks, and the centralized live relation view internals.
