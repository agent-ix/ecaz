# Artifact Manifest

Head SHA: `9c670b44837c01d6ae09ea1c55bfe4f822b8db0c`

Packet/topic: `31034-spire-customscan-begin-state`

Lane / fixture / storage format / rerank mode: CustomScan begin-state unit-test
coverage; PG18 pgrx pg_test items selected by the `custom_scan_` cargo test
filter; no storage format or rerank mode.

Timestamp: `2026-05-14T04:05:04Z`

Isolated one-index-per-table or shared-table surfaces: not applicable.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Key result: passed with the repository's existing stable-rustfmt warnings
    about unstable import options.

- `cargo-test-custom-scan.log`
  - Command: `cargo test -p ecaz custom_scan_`
  - Key result: 17 selected tests passed, including
    `custom_scan_begin_vector_order_limit_state_initializes_plan_parts`.

- `diff-stat.log`
  - Command: `git diff --stat HEAD~1 HEAD -- src/am/ec_spire/custom_scan/begin_exec.rs src/am/ec_spire/custom_scan/tests.rs plan/tasks/task30-phase12b-spire-cleanup.md`
  - Key result: 3 files changed, 100 insertions, 11 deletions.

- `line-counts.log`
  - Command: `wc -l src/am/ec_spire/custom_scan/begin_exec.rs src/am/ec_spire/custom_scan/tests.rs src/tests/mod.rs src/tests/remote_search/mod.rs src/tests/remote_search/*.rs`
  - Key result: `src/am/ec_spire/custom_scan/tests.rs` is 474 lines;
    `src/tests/mod.rs` remains 24,517 lines; `src/tests/remote_search.rs`
    remains absent.
