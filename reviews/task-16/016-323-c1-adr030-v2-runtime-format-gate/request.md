# Review Request: C1 ADR-030 V2 Runtime Format Gate

## Context

Packet `322` proved that the experimental ADR-030 v2 build gate writes real grouped-v2 pages and
metadata on disk.

The next narrow slice is to make the read side recognize that format explicitly, even though grouped
runtime traversal is not implemented yet.

## Problem

The scan/runtime path still assumes scalar element tuples. Without an explicit format gate, grouped-v2
indexes risk falling into scalar decode paths by accident.

We need a clear read-side boundary that:

1. recognizes `v1 scalar` vs `v2 grouped`
2. allows scalar scans to continue unchanged
3. rejects grouped-v2 scans with an explicit error until grouped runtime support exists

## Planned Slice

Add a runtime format gate that:

1. classifies graph storage format from metadata
2. validates scan startup against that format
3. rejects grouped-v2 ordered scans with a stable error message

This slice still excludes:

- no grouped runtime traversal
- no grouped scorer
- no grouped query-path fallback

## Implementation

Added explicit runtime format classification and a fail-fast grouped-v2 scan gate.

New metadata helper:

- `MetadataPage::graph_storage_format()`

New metadata enum:

- `GraphStorageFormat`

Runtime helper:

- `validate_runtime_scan_format(...)`

Behavior:

1. scan startup now classifies index graph storage from metadata
2. scalar `v1` indexes continue unchanged
3. grouped `v2` indexes are rejected at `amrescan` with a stable explicit error:
   - `tqhnsw scan runtime does not support ADR-030 grouped-v2 indexes yet`

Tests added:

- unit test that metadata format classification distinguishes `v1 scalar` from `v2 grouped`
- unit test that grouped-v2 metadata is rejected by the runtime scan gate
- pg test that builds a grouped-v2 index under the experimental gate and verifies an ordered scan
  fails with the explicit unsupported-runtime error

This is the first packet where the read side recognizes grouped-v2 intentionally instead of only by
implication from metadata fields.

## Measurements

This packet is still a format/read-boundary slice, so there are no new recall or latency
measurements.

Known validation results for this attempt:

- `cargo test metadata_graph_storage_format_distinguishes_v1_and_v2 --lib`: passed
- `cargo test validate_runtime_scan_format_rejects_grouped_v2_metadata --lib`: passed
- `cargo test test_experimental_grouped_v2_ordered_scan_rejects_runtime --no-default-features --features 'pg17 pg_test'`: passed
- `cargo test`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

ADR-030 v2 now has an explicit read-side format gate.

What this de-risks:

1. grouped-v2 indexes will no longer accidentally enter scalar scan decode paths
2. the unsupported runtime state is now deliberate and test-covered
3. the next slice can start building grouped page reads at the scan boundary without carrying
   ambiguity about the format split

## Next Slice

The next narrow slice should add grouped page read scaffolding at the graph/scan boundary:

1. grouped-hot tuple read helpers for scan/runtime use
2. grouped entry-point loading and neighbor access scaffolding
3. no grouped scoring yet, only enough read-path structure to replace the current hard rejection
