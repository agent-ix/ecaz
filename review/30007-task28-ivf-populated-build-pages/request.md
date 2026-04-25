# Review Request: Task 28 IVF Populated Build Pages

Scope: Phase 3 populated-write checkpoint. Non-empty `ec_ivf` builds now flush
the staged centroid, posting, and directory tuples to index data pages and
rewrite metadata with trained build state.

Task: `plan/tasks/28-ivf-access-method.md` Phase 3

Branch: `task28-ivf`

Head SHA: `0cb6aafa5d6e8c00b2d92d619edd874fa30befb1`

Owner: coder2

Files:

- `src/am/ec_ivf/build.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo test`
- `git diff --check`

Validation notes:

- `cargo test` was run before the checkpoint policy changed and completed
  cleanly under the default PG18-backed pgrx test path.
- The PG18 main suite included
  `pg_test_ec_ivf_non_empty_index_build_writes_staged_pages`, which passed.
- Additional broad PG18/PG17/full clippy gates were not run after the AGENTS
  update because the repository now asks agents to avoid tests unless they are
  necessary and to focus validation on PG18.

## Summary

This slice enables the first populated physical build path:

- Non-empty `ambuild` now trains centroids, stages the IVF build plan, flushes
  staged data pages, and returns `index_tuples` equal to staged postings.
- Adds `write_data_pages` for IVF using the same GenericXLog full-image page
  append pattern used by the existing HNSW build writer.
- Rewrites metadata after data-page flush with trained dimensions, resolved
  `nlists`, training version, centroid head, directory head, and live tuple
  count.
- Adds a PG test that creates a non-empty `ec_ivf` index and verifies trained
  metadata plus relation growth beyond the metadata page.
- Adds a debug metadata helper for PG tests to inspect populated IVF metadata
  without exposing a user-facing SQL surface.
- Updates the task plan to note that populated writes are now present while
  populated scans remain Phase 4 work.

## Review Focus

Please review for:

- Whether appending all staged data pages before rewriting metadata is the
  right first crash-safety boundary for this AM.
- Whether the directory shape using block-level head/tail refs should change
  before scan routing starts.
- Whether `index_tuples` should count posting tuples, heap tuples, or a future
  duplicate-coalesced posting count.
- Whether the test should inspect actual centroid/directory/posting tuples from
  disk before Phase 4, or whether metadata and relation growth is enough for
  this checkpoint.
- Whether `write_data_pages` should be promoted to a shared helper instead of
  duplicating the HNSW build writer pattern.

## Non-Goals

This packet does not implement populated IVF scans, nearest-list routing,
candidate scoring, live insert, vacuum, planner costing, list-directory
in-place updates, or any measurement claim.
