# Review Request: C1 ADR-030 V2 Grouped Metadata Payload Validation

## Context

Grouped-v2 runtime reads now rely on all of these being true together:

- grouped search code exists in the hot tuple
- cold rerank payload exists behind `reranktid`
- grouped metadata advertises both of those payload classes

Before this slice, `GraphStorageDescriptor::from_metadata(...)` validated grouped codec kinds and
shape, but not whether the required payload flags were actually set.

## Problem

If grouped-v2 metadata claims the grouped format but omits either:

- `PAYLOAD_FLAG_GROUPED_SEARCH_CODE`
- `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD`

the runtime could accept an index whose actual payload contract is incomplete.

That would fail later and less clearly than necessary.

## Planned Slice

Strengthen grouped-v2 metadata validation:

1. reject grouped-v2 metadata missing grouped-search payload flag
2. reject grouped-v2 metadata missing cold-rerank payload flag
3. keep the existing grouped codec and shape checks unchanged

This slice intentionally excludes:

- no scorer implementation yet
- no new storage format
- no binary-prefilter runtime work

## Implementation

Updated:

- `src/am/graph.rs`

Concrete changes:

1. `GraphStorageDescriptor::from_metadata(...)` now rejects grouped-v2 metadata that does not
   advertise grouped search-code payloads
2. the same helper now rejects grouped-v2 metadata that does not advertise cold rerank payloads
3. added unit tests covering both rejection paths

## Measurements

This packet is metadata/runtime validation only, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test graph_storage_descriptor_rejects_grouped_v2_missing_grouped_payload_flag --lib`: passed
  - `cargo test graph_storage_descriptor_rejects_grouped_v2_missing_cold_rerank_flag --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

Grouped-v2 runtime metadata acceptance is stricter and better aligned with the actual payload
contract.

What this de-risks:

1. grouped runtime code will fail fast on incomplete grouped-v2 metadata
2. the grouped scorer path can rely on both hot grouped search codes and cold rerank payloads being
   part of the declared format contract
3. metadata validation now matches the reviewer-requested tightening around grouped-v2 runtime
   assumptions

## Next Slice

The next narrow slice should keep the grouped scorer runway moving:

1. use the stricter metadata contract plus merged hot/cold payload seam at the scorer boundary
2. then build the first real grouped scorer implementation behind the existing runtime gate
