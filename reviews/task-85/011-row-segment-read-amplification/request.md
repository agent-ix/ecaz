# Task 85 Review Request: Row Segment Read Amplification Metrics

## Scope

This checkpoint extends the packet-009/010 funnel metrics so Task 85 can
measure actual selected row-segment read amplification, not only total routed
leaf storage footprint.

Packet 010 showed the retained block16/global1152 warm run spending p50
`181.330 ms` in object reads and reporting p50 `610,463,408` row bytes/query.
Static inspection showed that field is total routed row-object storage bytes,
not the subset of physical row segments read after global block selection.
Without a touched-segment metric, Task 85 cannot decide whether a V5
block-aligned layout is worth implementing.

## Code Change

- Adds scan diagnostics fields:
  - `leaf_row_segment_read_count`
  - `leaf_row_segment_read_bytes`
- Records selected V2 row segments after
  `read_leaf_object_v2_segments_for_row_ranges(...)`.
- Adds `SpireLeafPartitionObjectV2Segment::encoded_len(...)` so the observed
  selected segment bytes match the encoded tuple layout.
- Exposes the fields through
  `ec_spire_index_scan_leaf_candidate_snapshot(...)`.
- Carries the fields through `ecaz bench spire-pipeline` leaf rows and
  funnel JSONL.
- Extends the existing CLI carry-through and SQL-contract tests.

## Validation

- `cargo fmt --check` passed.
- Focused compile validation was attempted with
  `timeout 20 cargo test -p ecaz-cli spire_pipeline --locked --offline --no-run`.
  It timed out before printing compiler output. Earlier attempts with longer
  `cargo check`/`cargo test` invocations showed Cargo CPU-bound before
  spawning `rustc`; no Rust compiler diagnostics were reached.

Because this crosses Rust module and SQL result shape boundaries, this packet
should be reviewed with the validation caveat in mind. The next checkpoint
must re-run focused compile/tests before using the metrics in AWS.

## Evidence

- `artifacts/cargo-fmt-check.log`
- `artifacts/cargo-test-ecaz-cli-spire-pipeline-no-run-timeout.log`
- `artifacts/cargo-process-status-after-timeout.log`
- `artifacts/manifest.md`
