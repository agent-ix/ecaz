# Review Request: C1 ADR-030 V2 Gated Grouped Scan Runtime

## Context

Packet `344` persisted the grouped-PQ basis needed by runtime:

- grouped-v2 metadata now points at a persisted grouped codebook chain
- the graph layer can load that codebook model back from disk
- the shared grouped-PQ LUT builder already exists from packet `343`

The scan path still had two remaining gaps:

1. grouped-v2 runtime was rejected before any grouped approximate score could run
2. one grouped hot-path helper still allocated just to count binary sidecar words

## Problem

Without a runtime-scoped grouped query state, the grouped-v2 scan path cannot reuse the shared
packed-code scorer efficiently:

- codebooks would have to be reloaded or reinterpreted ad hoc
- grouped query LUT preparation would drift away from the shared quant helper
- the grouped score dispatch would keep erroring instead of producing an approximate score

At the same time, the external grouped runtime gate still needs to stay in place by default.

## Planned Slice

Batch the next related runtime slices together:

1. add an experimental grouped-v2 scan runtime gate, default off
2. load persisted grouped codebooks during `amrescan` and build one grouped query LUT per query
3. make grouped candidate scoring return the shared grouped-PQ approximate score
4. keep the default runtime rejection path when the gate is not enabled
5. close the lingering grouped hot-path allocation and shared-scorer contract nits from review

This slice intentionally excludes:

- no gate lift by default
- no SIMD grouped scorer yet
- no exact-rerank cutover yet
- no new recall measurements yet

## Implementation

Updated:

- `src/am/scan.rs`
- `src/am/graph.rs`
- `src/am/page.rs`
- `src/quant/grouped_pq.rs`
- `src/lib.rs`

Concrete changes:

1. added `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN` as the external grouped runtime gate
2. changed `validate_runtime_scan_format(...)` to allow grouped-v2 only when that gate is set
3. added grouped scan query preparation that:
   - loads the persisted grouped codebook model from metadata/index storage
   - reuses `PreparedQuery::rotated`
   - builds one shared grouped-PQ LUT per rescan
4. stored that grouped query preparation in scan state and freed it with the existing prepared-query
   lifetime
5. changed grouped candidate scoring to return the shared grouped-PQ approximate score from hot
   grouped search codes instead of erroring after the dispatch seam
6. kept the exact rerank helpers in place for future rerank work, but no longer forced cold rerank
   IO on the grouped approximate path
7. documented the shared scalar LUT layout and added the reviewer-requested debug assert on packed
   nibble width
8. removed the grouped `binary_word_count()` allocation by exposing a direct borrowed count helper
9. added a new pg smoke test proving:
   - grouped-v2 ordered scan still rejects by default
   - grouped-v2 ordered scan can execute when the experimental scan gate is explicitly enabled

## Measurements

This packet is still runtime wiring only, so there are no new latency or recall measurements yet.

Validation results for this checkpoint:

- `cargo test`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 grouped-v2 now has a real approximate scan runtime path behind an explicit experimental
gate instead of only stubbed grouped dispatch.

What this de-risks:

1. grouped scan query preparation now reuses the persisted codebooks and the shared LUT contract
2. grouped candidate dispatch now exercises the same shared packed-code scorer shape that future
   SIMD work will have to match
3. the default user-facing runtime rejection remains in place until the runtime gate is enabled
4. the leftover grouped hot-path allocation from earlier review is closed before the scorer gets
   any hotter

## Next Slice

The next runtime packet should focus on the first measurement-oriented grouped execution step:

1. validate the gated grouped runtime smoke test end to end
2. add a grouped exact-rerank comparison seam for top candidates behind the same gate
3. then start collecting the first recall/ordering evidence needed for gate-lift decisions
