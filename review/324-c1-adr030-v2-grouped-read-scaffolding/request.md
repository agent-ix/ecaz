# Review Request: C1 ADR-030 V2 Grouped Read Scaffolding

## Context

Packet `323` made scan startup classify `v1 scalar` vs `v2 grouped` and reject grouped-v2 indexes
explicitly.

The next narrow slice is to add real grouped-v2 tuple decode scaffolding at the graph boundary,
without enabling grouped runtime traversal or scoring yet.

## Problem

The code now recognizes grouped-v2 metadata, but all graph reads still assume legacy scalar element
tuples.

That leaves the read side with an intentional format gate, but no typed grouped read path to build
on next.

We need a bounded slice that:

1. derives grouped tuple decode shape from metadata
2. can read grouped-hot tuples intentionally from on-disk grouped-v2 indexes
3. can load grouped entry-point adjacency without falling back to scalar tuple contracts

This still excludes:

- no grouped scan traversal
- no grouped scorer
- no grouped rerank execution
- no removal of the grouped-v2 runtime rejection

## Planned Slice

Add grouped read scaffolding centered on the graph boundary:

1. metadata-derived graph storage descriptors
2. grouped-hot tuple loaders and borrowed tuple callbacks
3. one pg test that reads the grouped-v2 entry point and its neighbors from a real gated build

## Implementation

Added grouped-v2 read descriptors and grouped-hot tuple readers in `src/am/graph.rs`.

New types:

- `GroupedGraphLayout`
- `GraphStorageDescriptor`
- `GroupedGraphElement`
- `GraphTupleRef`

New helpers:

- `GraphStorageDescriptor::from_metadata(...)`
- `load_grouped_graph_element(...)`
- `with_grouped_graph_tuple(...)`
- `with_graph_storage_tuple(...)`
- `load_grouped_graph_adjacency(...)`

Behavior:

1. metadata now derives an explicit graph-storage decode contract
2. grouped-v2 descriptors compute:
   - grouped search-code bytes
   - binary sidecar word count, but only when the persisted no-QJL 4-bit binary lane is truly
     supported
   - rerank payload code length
3. grouped-hot tuples can now be decoded intentionally from a live index relation
4. scan runtime format validation now routes through `GraphStorageDescriptor::from_metadata(...)`
   before still rejecting grouped-v2 scans

Read-path validation added:

- unit test for scalar metadata -> scalar descriptor
- unit test for grouped-v2 metadata -> grouped descriptor lengths
- pg test `test_grouped_v2_graph_reads_load_entry_and_neighbors`

The new pg test:

1. enables `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD`
2. builds a source-backed grouped-v2 fixture
3. derives a grouped storage descriptor from metadata
4. opens the real index relation
5. decodes the grouped-hot entry point via both borrowed and owned grouped read helpers
6. loads entry adjacency and decodes at least one real grouped-hot neighbor

This is the first packet where grouped-v2 graph pages are intentionally read through graph helpers
rather than only being inspected as raw tuple tags.

## Measurements

This packet is still read-boundary scaffolding, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test graph_storage_descriptor_uses_grouped_lengths_for_v2_metadata --lib`: passed
- `cargo test validate_runtime_scan_format_rejects_grouped_v2_metadata --lib`: passed
- `cargo test test_grouped_v2_graph_reads_load_entry_and_neighbors --no-default-features --features 'pg17 pg_test'`: passed
- `cargo fmt --all`: passed
- `cargo test`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 v2 now has real grouped graph-read scaffolding.

What this de-risks:

1. grouped-v2 metadata now maps to an explicit decode contract instead of just a rejection branch
2. grouped-hot entry tuples and adjacency can be loaded from real on-disk grouped-v2 indexes
3. the next slice can start carrying grouped read state into scan/runtime without first inventing a
   storage descriptor

## Next Slice

The next narrow slice should move this grouped read contract into scan state:

1. persist a graph storage descriptor at scan startup
2. make cached graph-element loading format-aware
3. keep grouped-v2 scoring unsupported, but replace purely scalar cache assumptions at the scan
   boundary
