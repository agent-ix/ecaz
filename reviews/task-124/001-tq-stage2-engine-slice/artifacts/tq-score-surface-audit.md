# Task 124 TurboQuant Score-Surface Audit

Head SHA: `008309ae4` before local Task 124 edits.
Packet: `reviews/task-124/001-tq-stage2-engine-slice/`
Timestamp: `2026-06-27T22:22:07Z`

## IVF Hot Path Surfaces

### IVF posting scorer, TurboQuant no-QJL 4-bit LUT

- Call path: `scan.rs` posting decode -> `IvfQuantizer::score_turboquant_batch_from_payloads` -> `score_turboquant_no_qjl_4bit_batch_for`.
- Prepared query: `IvfPreparedQuery::TurboQuantNoQjl4BitLut` when `ExactScoreMode::MseNoQjl4Bit`.
- Surface: `CandidateBatchScoringSurface::Ivf`.
- Block/SIMD status: batch/block scorer surface, with scalar-tail behavior owned by `candidate_batch`. This is not per-candidate scalar at the IVF dispatch layer.
- Task 124 relevance: candidate generation can still use RaBitQ; this surface remains relevant for TQ-only posting indexes and for verifying TQ batch telemetry.

### IVF posting scorer, TurboQuant QJL

- Call path: `IvfQuantizer::score_turboquant_batch_from_payloads` -> `score_turboquant_qjl_batch_for`.
- Prepared query: `IvfPreparedQuery::TurboQuant`.
- Surface: `CandidateBatchScoringSurface::Ivf`.
- Block/SIMD status: batch scorer surface; QJL-specific kernel behavior still needs Phase 3 telemetry if used in the final stage-2 benchmark matrix.
- Task 124 relevance: not the intended first candidate frontier, but it is a valid TQ IVF score surface.

### Index-side TQ rerank sidecar, borrowed payload refs

- Call path: `scan.rs::rerank_probe_candidates_index_side` -> `RerankScorer::score_sidecar_payload_refs_batch_with_centroid_ips` -> `RerankPayloadCodec::score_payload_refs_batch` -> `IvfQuantizer::score_turboquant_batch_from_payload_refs`.
- Prepared query: no-QJL 4-bit or QJL TQ prepared query, depending on TQ mode.
- Surface: `CandidateBatchScoringSurface::Ivf`.
- Block/SIMD status: batch scorer surface. The current implementation builds vectors of code refs and gamma values, but does not copy survivor payload bytes into a contiguous slab.
- Task 124 relevance: this is the new stage-2 hot path. The focused pgrx test asserts `rerank_payload_bytes_scored > 0` and `rerank_payload_slab_bytes_copied == 0`.

## Diagnostic / Off-Path Surfaces

### Source-diagnostic TQ rerank

- Call path: heap/source fetch -> `RerankPayloadCodec::score_sources_batch` -> encode source to TQ payload -> `score_payloads_batch`.
- Block/SIMD status: batch scorer after query-time source encoding, but source fetch + encode makes it diagnostic rather than the Task 124 latency path.
- Task 124 relevance: useful for correctness comparisons, not the product path.

### Exact-dequant TQ rerank

- Call path: `RerankPayloadCodec::score_payloads_batch` or `score_payload_refs_batch` with `RaBitQRerankScoreMode::ExactDequant`.
- Block/SIMD status: scalar per payload through `score_ip_dequantized_from_parts`.
- Task 124 relevance: off-path for the current stage-2 scorer. A latency rejection of TQ must not be based on this scalar diagnostic mode.

### HNSW / DiskANN / SPIRE TQ surfaces

- Task 124 does not touch these AMs. They remain reference surfaces only and are out of scope for this packet.

## Gaps For Phase 3

- Existing counters prove payload bytes scored, source bytes read, and whether TQ avoided the survivor slab copy.
- Existing counters do not yet split TQ stage-2 rows from final exact f32 rows; `rerank_rows` is cumulative after the new final pass.
- Existing counters do not yet expose per-query TQ block width, scalar-tail count, or ISA/kernel family through `ecaz bench suite` artifacts.
