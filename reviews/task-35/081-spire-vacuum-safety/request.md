# Task 35 Packet 081: SPIRE Vacuum Safety Comments

## Code Under Review

- Commit: `0f154e52874cab66519b4a85bcf97f1f5ff26539`
- Files:
  - `src/am/ec_spire/vacuum/mod.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Scope

This slice burns down the remaining unsafe-comment baseline for the SPIRE vacuum and delete-delta publication path.

The added comments document safety boundaries for:

- PostgreSQL `ambulkdelete` and `amvacuumcleanup` callback entrypoints;
- publish-lock guarded root/control reads and replacement epoch publication;
- active manifest, placement-directory, and local-store config loading;
- relation-backed object store access during vacuum reads and rewrite publication;
- PostgreSQL bulk-delete callback invocation through stack `ItemPointerData`;
- vacuum stats allocation, relation block-count reads, and stats mutation;
- pg_test debug vacuum wrappers and callback state lifetimes.

## Baseline Movement

- Global unsafe-comment baseline: `1833 -> 1799`
- `src/am/ec_spire/vacuum/mod.rs`: `34 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `1799` entries across `52` files.
- `artifacts/vacuum-baseline-after.log`: vacuum entries are `0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed.

Known unrelated warnings remain:

- unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`;
- unused SPIRE imports/re-exports in `src/am/mod.rs`.

`cargo fmt --all` emitted the repo's existing stable-rustfmt warnings for unstable rustfmt options. It also touched `hardening/careful/src/lib.rs` and `src/quant/simd.rs`; those unrelated formatting changes were restored before commit.
