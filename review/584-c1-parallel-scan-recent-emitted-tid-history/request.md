# Review Request: Parallel Scan Recent Emitted TID History

Current head: `81ed9368b466c697ee4a34dd3ddc659b61acf68c`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`

Problem:
- The live parallel duplicate suppression path only compared a foreign worker's
  `last_emitted_heap_tid`.
- A fast worker could emit heap TID A, then B, then C before a slower worker
  reached A. The slower worker would no longer see A in the foreign worker's
  last-emitted slot and could emit a duplicate heap TID.
- This is an advisory cross-worker dedupe path, but the observed forced-live
  failure mode made the single-slot history too narrow for worker skew.

What changed:
- Added a bounded per-worker recent emitted heap-TID ring to the parallel DSM
  worker runtime snapshot and bumped the DSM layout version to 13.
- Scan workers now maintain the ring locally whenever they mark an output
  emitted, publish it with the rest of the worker runtime snapshot, and reset it
  on rescan.
- Foreign-worker duplicate suppression now checks both the legacy
  `last_emitted_heap_tid` and the recent emitted heap-TID ring.
- Expanded fixed-size parallel-scan unit-test backing buffers to match the
  larger worker-slot layout.
- Strengthened the handoff duplicate regression so the suppressed duplicate is
  no longer the foreign worker's most recent emitted heap TID.

Artifact:
- `artifacts/pg18-parallel-scan.log` captures the safe PG18 CLI preflight after
  the change.
- The log shows ordered `ec_hnsw` candidate IDs match the serial IDs. The
  default plan still remains a serial `Index Scan`, while the control plan shows
  PostgreSQL can launch 4 workers for the same fixture.

Validation:
- Passed:
  - `cargo test`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`
  - `cargo test try_take_parallel_scan_handoff_output_suppresses_already_emitted_foreign_heap_tid --lib`
  - `cargo test try_take_parallel_scan_next_output --lib`
  - `cargo test am::common::parallel --lib`
  - `cargo test try_take_parallel_scan_handoff_output --lib`
  - `cargo test parallel_scan --lib`
  - `cargo fmt --check`
  - `git diff --check`

Review focus:
- Whether the bounded recent-emitted ring is the right minimum coordination
  primitive before enabling the AM planner path.
- Whether a capacity of 32 is enough for the current forced-live skew surface.
- Whether this should stay as advisory snapshot state or move later to a
  stronger global emitted heap-TID table if forced-live validation still finds
  duplicate seams.
