# Review Request: SPIRE Phase 1 Recall/Latency Gate

- Measurement head: `89b8a60f7d252e3664710d6a03d3dae7bc1761b4`
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Agent: coder1

## Summary

This packet closes the reviewer-requested Phase 1 measured recall/latency gate
for the current SPIRE single-level foundation on the local real 10k fixture.

The measured index is:

- corpus: real 10k fixture from `target/real-corpus/ec_hnsw_real_10k`
- rows / queries / dimensions: 10,000 / 200 / 1536
- access method: `ec_spire`
- storage format: `turboquant`
- reloptions: `nlists = 32`, `nprobe = 24`, `rerank_width = 25`,
  `local_store_count = 1`
- placement surface: single local store baseline

Raw packet-local artifacts are listed in `artifacts/manifest.md`.

## Results

Recall@10 is strong on this fixture:

| nprobe | recall@10 | ndcg@10 | recall mean q-time |
| --- | ---: | ---: | ---: |
| 8 | 0.9985 | 0.9999 | 63.91 ms |
| 16 | 1.0000 | 1.0000 | 103.66 ms |
| 24 | 1.0000 | 1.0000 | 141.72 ms |
| 32 | 1.0000 | 1.0000 | 177.17 ms |

Latency over 100 iterations:

| nprobe | mean | p50 | p95 | p99 | HWM KB |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8 | 61.5 ms | 62.1 ms | 70.2 ms | 75.8 ms | 69544 |
| 16 | 102.6 ms | 101.5 ms | 115.1 ms | 125.0 ms | 70192 |
| 24 | 143.0 ms | 140.7 ms | 156.7 ms | 178.2 ms | 70460 |
| 32 | 177.5 ms | 174.3 ms | 197.3 ms | 201.9 ms | 70816 |

## Review Focus

1. Confirm that this is enough to close the Phase 1 SPIRE measured
   recall/latency evidence gate for the local single-store foundation.
2. Check whether `nprobe = 8` should be treated as the current practical
   operating point on this fixture, given `0.9985` recall@10 and lower latency.
3. Confirm that the packet correctly separates the preceding storage fix
   (`30529`) from this measurement evidence.

## Notes

The first measurement attempt found a real build blocker: 1536-dimensional
`nlists = 32` routing objects exceeded the one-page relation tuple path. That
was fixed and reviewed separately in `30529-spire-large-routing-object-chain`
before this packet was run.

The fixture was loaded under a SPIRE-specific prefix while reusing the canonical
`ec_hnsw_real_10k` files, so the loader was run with
`--allow-manifest-mismatch`; the corpus and query file hashes are the canonical
real 10k files and the exact truth cache is stored in this packet.
