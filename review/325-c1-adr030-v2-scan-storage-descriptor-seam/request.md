# Review Request: C1 ADR-030 V2 Scan Storage Descriptor Seam

## Context

Packet `324` added grouped-v2 graph-read scaffolding and proved that grouped-hot entry tuples and
neighbors can be decoded intentionally from a real gated grouped-v2 index.

The next narrow slice is to carry that storage contract into scan state so the scan cache no longer
assumes every graph tuple is a scalar element tuple.

## Problem

Even after `324`, the scan runtime still cached graph elements through scalar-only tuple callbacks:

1. scan startup classified format, but discarded the decode contract
2. cached graph-element loading still called `with_graph_element_tuple(...)`
3. cached element extraction still assumed scalar tuple fields directly

That keeps the read boundary narrow, but it means scan state still has scalar-only assumptions baked
into its cache layer.

We need a bounded seam that:

1. stores the graph storage descriptor on scan startup
2. routes graph-element cache loads through the typed graph tuple abstraction
3. keeps grouped-v2 scoring unsupported instead of pretending grouped tuples are scoreable already

This still excludes:

- no grouped-v2 traversal enablement
- no grouped search scoring
- no grouped rerank path
- no removal of the grouped-v2 runtime rejection at scan startup

## Planned Slice

Add one scan-state seam:

1. `amrescan` keeps the scalar-v1 storage descriptor returned by format validation
2. cached graph-element loading becomes descriptor-aware
3. exact-score extraction remains scalar-only and fails explicitly if ever asked to score grouped
   tuples

## Implementation

Added shared tuple-field accessors on `graph::GraphTupleRef` in
`src/am/graph.rs`:

- `level()`
- `deleted()`
- `heaptid_count()`
- `collect_heaptids()`
- `neighbortid()`
- `binary_word_count()`
- `collect_binary_words()`
- `exact_payload()`

Updated `src/am/scan.rs`:

1. `validate_runtime_scan_format(...)` now returns a scalar graph storage descriptor rather than
   just `()`
2. `TqScanOpaque` now stores `scan_graph_storage`
3. `amrescan` records that descriptor in scan state
4. `CachedGraphElement::from_graph_tuple_ref(...)` now builds cached graph headers from
   `graph::GraphTupleRef`
5. `cached_graph_element(...)` now loads through `with_graph_storage_tuple(...)`
6. binary sidecar extraction is now tuple-kind-aware:
   - scalar tuples can still derive or read binary words
   - grouped tuples can carry persisted binary words into cache
7. exact payload extraction remains scalar-only through `exact_payload()`
8. if exact-score storage is ever requested for a grouped tuple, the code still errors with the
   existing grouped-v2 unsupported-runtime message

New unit test:

- `cached_graph_element_from_grouped_tuple_ref_keeps_header_and_binary_words`

This is intentionally a seam packet, not a capability packet. Grouped-v2 scans are still rejected at
startup, but the scan cache no longer hardcodes scalar tuple decoding internally.

## Measurements

This packet is still runtime-boundary scaffolding, so there are no new latency or recall
measurements.

Known validation results for this attempt:

- `cargo test cached_graph_element_from_grouped_tuple_ref_keeps_header_and_binary_words --lib`: passed
- `cargo test validate_runtime_scan_format_rejects_grouped_v2_metadata --lib`: passed
- `cargo test`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 v2 scan state now preserves the graph storage descriptor and uses typed graph tuple access
for cached header loads.

What this de-risks:

1. grouped-v2 read contracts now reach scan state rather than stopping at graph helpers
2. the scan cache can ingest grouped tuple headers and binary sidecars without pretending grouped
   tuples are scalar payloads
3. the next slice can make more of the scan cache format-aware without first untangling scalar-only
   cache assumptions

## Next Slice

The next narrow slice should push this further into scan/cache behavior:

1. make loaded-state bookkeeping explicitly reflect `exact payload unavailable` vs `exact payload
   present`
2. gate any grouped tuple exact-score requests before they reach generic score helpers
3. prepare the cache seam needed for a future grouped approximate scorer without enabling traversal
   yet
