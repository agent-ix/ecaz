# Review Request: C1 ADR-030 V2 Cold Rerank Fetch Seam

## Context

Reviewer feedback on the ADR-030 grouped-v2 lane asked for a real cold-payload access seam before
the grouped scorer grows further.

At this point the grouped read path already exposes:

- grouped hot tuple reads
- cached grouped search-code payloads
- cached `reranktid`

But there was still no dedicated helper that could turn `reranktid` into the actual cold rerank
tuple payload.

## Problem

Without an explicit cold rerank fetch seam, the future grouped pipeline would still have to solve two
problems at once:

1. grouped search scoring
2. decoding the cold rerank payload from raw page tuples

That is the wrong dependency shape for the next scorer slice.

## Planned Slice

Add a graph-side grouped rerank payload read boundary:

1. typed grouped rerank payload struct
2. borrowed tuple helper for grouped rerank reads
3. owned grouped rerank payload loader from `reranktid`
4. pg coverage proving a grouped-v2 index can load the cold rerank payload from disk

This slice intentionally excludes:

- no scorer implementation yet
- no rerank stage execution yet
- no scan-path cutover to use the cold payload

## Implementation

Updated:

- `src/am/graph.rs`
- `src/lib.rs`

Concrete changes:

1. added `GroupedRerankPayload { tid, gamma, code }`
2. added `with_grouped_rerank_tuple(...)`
3. added `load_grouped_rerank_payload(...)`
4. added a pg test that:
   - enables the experimental grouped-v2 build gate
   - builds a source-backed grouped-v2 index
   - loads the grouped-hot entry point
   - follows `entry.reranktid`
   - decodes the cold rerank tuple
   - verifies payload length and basic payload validity

## Measurements

This packet is a read-boundary seam, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test test_grouped_v2_graph_reads_load_cold_rerank_payload --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: one full run hit
    `pg_test_tqhnsw_successor_candidate_from_entry_adjacency`; isolated rerun passed, so this
    attempt records that as an existing suite flake rather than a rerank-fetch regression
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 grouped-v2 now has an explicit cold rerank payload read seam.

What this de-risks:

1. grouped runtime work can fetch cold rerank payloads through one graph helper instead of reopening
   raw tuple decode paths
2. the eventual tiny-rerank stage now has a real storage boundary to target
3. grouped search scoring and cold rerank fetch can evolve independently

## Next Slice

The next high-signal slice should use this seam without lifting the runtime gate:

1. carry a typed cold rerank payload view into the grouped scorer boundary
2. strengthen grouped metadata/runtime validation around rerank codec assumptions
3. then move toward end-to-end `binary -> grouped -> rerank` measurement
