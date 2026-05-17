# Artifact Manifest

Head SHA: `86fae0f50fd12749f97d96d0d377dac281f0ebb6`

Packet/topic: `31035-spire-customscan-read-cancel`

Lane / fixture / storage format / rerank mode: CustomScan read-path
cancellation and receive-layer local-cancel pg_test fixtures; PG18; rabitq
fixture indexes; rerank mode not applicable.

Timestamp: `2026-05-14T04:24:16Z`

Isolated one-index-per-table or shared-table surfaces: isolated one-index
fixtures.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Key result: passed with the repository's existing stable-rustfmt warnings
    about unstable import options.

- `cargo-test-customscan-read-cancel.log`
  - Command: `cargo test -p ecaz test_ec_spire_customscan_read_cancel_releases_transport`
  - Key result: selected CustomScan read-cancel pg_test passed as a
    should-panic query-cancel boundary test.

- `cargo-test-receive-local-cancel.log`
  - Command: `cargo test -p ecaz test_ec_spire_prod_receive_local_cancel_remote_cancel`
  - Key result: selected receive-layer local-cancel pg_test passed, including
    `local_query_cancelled` categorization and governance lock release.

- `diff-stat.log`
  - Command: `git diff --stat HEAD~1 HEAD -- src/am/ec_spire/coordinator/remote_candidates/production_transport.rs src/tests/custom_scan.rs src/tests/remote_search/receive_faults.rs plan/tasks/task30-phase12b-spire-cleanup.md`
  - Key result: 4 files changed, 121 insertions, 5 deletions.

- `line-counts.log`
  - Command: `wc -l src/tests/custom_scan.rs src/tests/remote_search/receive_faults.rs src/tests/mod.rs src/tests/remote_search/mod.rs src/tests/remote_search/*.rs`
  - Key result: `src/tests/mod.rs` remains 24,517 lines;
    `src/tests/remote_search.rs` remains absent; touched test files remain
    below the split threshold.
