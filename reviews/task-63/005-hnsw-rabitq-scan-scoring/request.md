# Task 63 Review Request: HNSW RaBitQ Scan Scoring

## Scope

This checkpoint wires HNSW RaBitQ scans through the shared RaBitQ scorer. RaBitQ
graph payloads now prepare a RaBitQ query scorer and use it for grouped hot
search-code scoring during traversal.

Code commit under review:

- `80309f865888321015698a9f77023a7d25cd2869` - Add HNSW RaBitQ scan scoring

## Implementation Notes

- Added a concrete `RaBitQQuantizer::prepare_ip_query` helper that returns the
  reusable `RaBitQScorer`; the trait method now delegates to that helper.
- `TqScanOpaque` owns a scan-local `RaBitQScorer` for RaBitQ indexes.
- RaBitQ grouped hot payloads now produce a `GroupedScoreShape` and dispatch
  approximate traversal scoring through `RaBitQScorer`.
- Live rerank buffering is enabled for RaBitQ so final output can use the cold
  scalar-quantized rerank payload as the comparison score.

## Deliberate Limits

Insert and vacuum still fail closed for RaBitQ. This packet also does not add a
PG18 SQL smoke; that should land with the remaining insert/vacuum runtime
decision and SQL validation slice.

## Validation

- `cargo check -q --lib` passed.
- `cargo test -q --lib graph_storage_descriptor_uses_rabitq_code_len_for_v4_metadata --no-run`
  passed. The command emitted existing test warning noise about unnecessary
  `unsafe` blocks and unused Hadamard test helpers.

Artifact manifest:

- `artifacts/manifest.md`
