# SPIRE Old-Epoch Physical Cleanup

## Scope

This packet adds physical cleanup for SPIRE old-epoch object tuples once the
epoch cleanup planner says retention and active-query rules permit reclamation.

Code checkpoint: `dc3a5676` (`Add SPIRE old epoch physical cleanup`)

## Changes

- Adds `ec_spire_index_epoch_cleanup_run(index_oid)`.
  - Takes the SPIRE publish lock.
  - Recomputes cleanup eligibility from latest epoch manifests.
  - Protects active and retained epoch manifest/placement/object tuple chains.
  - Deletes unprotected object tuples with PostgreSQL no-compaction line-pointer
    deletion so existing TIDs for protected tuples remain stable.
- Updates relation storage diagnostics:
  - `physical_cleanup_supported = true`
  - cleanup recommendation points at `ec_spire_index_epoch_cleanup_run`.
- Updates cleanup summary statuses:
  - `not_required`
  - `blocked_by_retention`
  - `supported`
- Adds a test-only helper to age retired manifests for PG18 cleanup coverage.
- Adds PG18 coverage proving cleanup removes tuples and a post-cleanup scan
  still returns the expected row.
- Marks the Phase 8 old-epoch physical reclamation task complete.

## Files

- `src/am/ec_spire/page.rs`
- `src/am/ec_spire/root/debug.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo test epoch_snapshot_partial_retired_residue_keeps_root_manifest_authoritative`
- `cargo pgrx test pg18 test_ec_spire_epoch_cleanup_run_reclaims_old_tuples_sql`
- `cargo pgrx test pg18 test_ec_spire_relation_storage_snapshot_sql`
- `git diff --check`

## Notes

The cleanup pass is intentionally conservative. It only deletes tuples not
referenced by the active epoch or retained epoch placement chains. It uses
`PageIndexTupleDeleteNoCompact` instead of compacting page items, preserving
line-pointer offsets for protected object TIDs.
