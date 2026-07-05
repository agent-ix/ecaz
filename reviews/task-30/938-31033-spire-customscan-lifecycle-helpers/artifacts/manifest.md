# Artifact Manifest

Head SHA: `cf49252f17f98b6fc01f667c13e6f07cfea60001`

Packet/topic: `31033-spire-customscan-lifecycle-helpers`

Lane / fixture / storage format / rerank mode: CustomScan lifecycle unit-test
coverage; PG18 pgrx pg_test items selected by the `custom_scan_` cargo test
filter; no storage format or rerank mode.

Timestamp: `2026-05-14T03:57:40Z`

Isolated one-index-per-table or shared-table surfaces: not applicable.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Key result: passed with the repository's existing stable-rustfmt warnings
    about unstable import options.

- `cargo-test-custom-scan.log`
  - Command: `cargo test -p ecaz custom_scan_`
  - Key result: 16 selected tests passed; this includes the new lifecycle unit
    tests plus the selected PG18 pgrx pg_test items.

- `diff-stat.log`
  - Command: `git diff --stat HEAD~1 HEAD -- src/am/ec_spire/custom_scan/begin_exec.rs src/am/ec_spire/custom_scan/tests.rs plan/tasks/task30-phase12b-spire-cleanup.md`
  - Key result: 3 files changed, 186 insertions, 40 deletions.

- `line-counts.log`
  - Command: `wc -l src/am/ec_spire/custom_scan/begin_exec.rs src/am/ec_spire/custom_scan/tests.rs src/tests/mod.rs src/tests/remote_search/mod.rs src/tests/remote_search/*.rs`
  - Key result: `src/am/ec_spire/custom_scan/tests.rs` is 437 lines;
    `src/tests/mod.rs` remains 24,517 lines; `src/tests/remote_search.rs`
    remains absent.
