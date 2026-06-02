# Task 79 Review Request: RaBitQ Summary Batch-Max Scoring

## Summary

This packet reviews source checkpoint `6ece24263`, which routes SPIRE RaBitQ leaf-summary multi-representative scoring through a new prevalidated batch-max helper.

The change is narrow:

- adds `PreparedEstimator::estimate_ip_batch_max_prevalidated()` using existing RaBitQ batch kernels with stack scratch storage;
- updates `SpirePreparedAssignmentScorer::score_zero_gamma_payload_chunks_max_prevalidated()` to use that helper for RaBitQ bits 1/4/8;
- preserves scalar fallback for bits 2.

I processed the new packet 041 reviewer feedback before closing this packet. That feedback accepts the negative two-stage result and recommends either a k=3 scoring-kernel optimization or a larger architecture change. This packet is the small scoring-kernel slice.

## Validation

Packet-local focused tests passed:

- `artifacts/cargo-test-batch-max.log`: `batch_max_prevalidated_matches_scalar_max`
- `artifacts/cargo-test-summary-best-representative.log`: `leaf_block_summary_scores_best_representative_payload`

Local PG18 install/restart completed:

- backend SHA256: `da1b4b0238b03e801977d2b3b7891143a86a1874b84639389bee836f8391baf2`
- `artifacts/install-batch-max-ecaz-pg18.log`
- `artifacts/pg18-restart-batch-max.log`

`ecaz bench suite` audit/status completed with 2/2 successful steps:

- `artifacts/suite-audit.log`
- `artifacts/suite-status.log`

## Benchmark Result

Suite config: `suite-rabitq-summary-batch-max.json`

All rows are local PG18 only. No AWS was used.

Measured shape: RaBitQ, block16, k=3 summaries, nprobe 96, global block cap 1216, radius weight 0.25, rerank width 25, 200 queries.

| row | candidates | route_sum | object_bytes_sum | latency_p50_ms | latency_p95_ms | production_total_p50 | production_total_p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| batch_max | 3,877,368 | 19,200 | 14,967,100,324 | 51.917 | 62.046 | 48 | 57 | 0.9940 |

Packet 041 full-scoring baseline on the same surface was p50 52.135ms, p95 63.231ms, production p50 48ms, production p95 54ms, recall 0.9940.

## Interpretation

The implementation is behavior-preserving and directionally improves local query p50 by about 0.2ms versus the packet 041 full-scoring row, but this is not enough to close Task 79.

The packet also confirms the candidate/read surface is unchanged:

- candidates: 3,877,368
- route_sum: 19,200
- object bytes: 14,967,100,324

So this is a useful low-risk scoring-path cleanup, not a direct candidate-surface solution. The next real Task 79 work should either be deeper RaBitQ summary-scoring kernel work or an architectural prefilter that reduces routed leaf/object read surface before summary scoring.
