# Task 85 Packet 033: Comparator And Product Policy Gate

## Result

SPIRE has an accepted same-recall latency improvement over the retained
Task 79/81 SPIRE surface, but it is not product-default ready at 1M because
the current single-node IVF/RaBitQ comparator dominates the product tradeoff.

## Comparator Table

| Engine / profile | 1M recall@10 | p50 | p95 | p99 | index size | product read |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| SPIRE packet 023 V5 + summary fast path | 0.9876 | 222.692 ms | 275.769 ms | 286.980 ms | 872.1 MiB | accepted SPIRE research/opt-in point |
| IVF/RaBitQ nprobe 128 | 0.9864 | 34.6 ms | 41.5 ms | 48.0 ms | 298.0 MiB | slightly lower recall, much faster/smaller |
| IVF/RaBitQ nprobe 256 | 0.9936 | 66.2 ms | 72.5 ms | 75.7 ms | 298.0 MiB | higher recall, much faster/smaller |
| DiskANN L800 | 0.9825 | 19.7 ms | 30.9 ms | 35.6 ms | 455.1 MiB | lower recall than SPIRE, much faster/smaller |
| HNSW AWS Graviton | unavailable at 1M | unavailable | unavailable | unavailable | unavailable | Task 61 deferred 1M due capacity; local 1M packet stopped during build |

## Policy Decision

- Keep SPIRE out of default product routing for 1M.
- Keep the accepted packet 023 SPIRE profile as research/opt-in evidence only.
- Do not introduce a new default or balanced profile from Task 85.
- Do not write an ADR to change defaults, because the comparator gate rejects
  a product-default change rather than accepting one.

This is not because Task 85 failed to improve SPIRE. It did: packet 023 beats
the retained SPIRE baseline at unchanged recall, candidates, and rerank width.
The product decision is that IVF/RaBitQ already gives better or comparable
recall at a fraction of SPIRE latency and index size on the current 1M
Graviton evidence.

## HNSW Limitation

The available HNSW comparator evidence is incomplete for 1M:

- `benchmarks/task61-aws-hnsw-graviton-baseline/` completed 10k, 50k, and 100k
  only, then explicitly deferred 1M due storage headroom on the 100 GiB
  profile.
- `benchmarks/profile-hnsw-1m/` is a local focused 1M packet, but the manifest
  shows all recall/latency/storage steps still pending and the only populated
  run log is the load/build phase.

This limitation does not block the product gate because IVF/RaBitQ already
dominates the accepted SPIRE 1M point at same-or-better recall.

## Evidence

See `artifacts/manifest.md` for source manifests and exact result lines.
