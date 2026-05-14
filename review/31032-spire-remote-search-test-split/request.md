# Review Request: SPIRE remote-search test split

## Summary

This checkpoint responds to the oversized `src/tests/remote_search.rs` file by
removing it as a monolith and replacing it with a concern-based include
directory under `src/tests/remote_search/`.

Code checkpoints:

- `23a751e497835c62aa998fa7a4b5db0e8b9b5631`
- `f386b5f900286489e3ae603d17990babfda27362`

The shrink-list policy for this slice is:

- Do not add test bodies to `src/tests/remote_search.rs`; it is deleted.
- Do not grow `src/tests/mod.rs`; its line count stays flat and only the
  include target changes.

## Split Result

New files:

- `src/tests/remote_search/contracts.rs`
- `src/tests/remote_search/tuple_heap.rs`
- `src/tests/remote_search/coordinator_catalog.rs`
- `src/tests/remote_search/production_summary.rs`
- `src/tests/remote_search/transport_faults.rs`
- `src/tests/remote_search/receive_faults.rs`
- `src/tests/remote_search/libpq_executor.rs`
- `src/tests/remote_search/node_catalog.rs`
- `src/tests/remote_search/epoch_manifest.rs`
- `src/tests/remote_search/catalog_cleanup_policy.rs`
- `src/tests/remote_search/mod.rs`

The split preserves the existing include-based pg_test context. The content
equivalence checker reports a normalized match against the previous monolith,
accounting only for required `include_str!` relative path changes and removal
of separator-only blank lines at chunk EOFs.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz test_ec_spire_remote_search_sql_scores_selected_leaf_pids --no-run`
- `git diff --check HEAD~2 HEAD -- src/tests/mod.rs src/tests/remote_search.rs src/tests/remote_search`

Raw logs, line counts, and the strategy are in `artifacts/`.

## Reviewer Focus

- Confirm the split boundaries are reasonable for future maintenance.
- Confirm no new test bodies landed in shrink-list files.
- Confirm compile-only validation is sufficient for this mechanical include
  split.
