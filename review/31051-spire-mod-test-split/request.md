# Review Request: split oversized `src/tests/mod.rs`

## Summary

Code checkpoint: `15adc4ae` (`Split oversized test mod module`)

This checkpoint splits the large inline `src/tests/mod.rs` body into focused
same-module include files:

- `ec_ivf.rs`
- `ec_hnsw_build.rs`
- `ec_hnsw_runtime_profiles.rs`
- `ec_hnsw_runtime_comparisons.rs`
- `ec_hnsw_storage_lifecycle.rs`
- `ec_hnsw_graph_lifecycle.rs`
- `ec_hnsw_scan_gettuple.rs`
- `ec_hnsw_recall_helpers.rs`
- `ec_hnsw_recall_debug_exports.rs`
- `ec_hnsw_recall_tests.rs`

The split keeps item visibility and module scope unchanged by using `include!`
from the original `tests` module.

## Size Check

After the split:

- `src/tests/mod.rs`: 2,799 lines
- largest new split file: `src/tests/ec_hnsw_scan_gettuple.rs`: 2,844 lines
- no `src/tests/*.rs` file is over 3,000 lines

The existing largest files after this checkpoint are:

- `src/tests/remote_search/contracts.rs`: 2,864 lines
- `src/tests/ec_hnsw_scan_gettuple.rs`: 2,844 lines
- `src/tests/insert.rs`: 2,830 lines
- `src/tests/mod.rs`: 2,799 lines

## Validation

- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    unstable import grouping options.
- `cargo test -p ecaz test_binary_recv_rejects_trailing_bytes`
  - Passed.
- `cargo test -p ecaz test_ec_ivf_empty_index_build_initializes_metadata_page`
  - Passed.
- `cargo test -p ecaz test_fr020_empty_index_remains_planner_gated`
  - Passed.

The focused tests force PG18 pgrx extension rebuild/install and SQL entity
discovery through the included test module tree.

## Reviewer Focus

- Confirm the include ordering preserves the previous same-module item scope.
- Check that split boundaries are whole Rust items, especially the HNSW runtime
  and recall/debug sections.
- Confirm the chosen file groups are acceptable and stay below the repository's
  practical 3,000-line ceiling.
