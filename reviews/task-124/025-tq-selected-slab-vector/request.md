# Task 124 Packet 025: TQ selected slab vector negative result

## Summary

This packet records a rejected TurboQuant scan-path optimization attempt. The
temporary experiment replaced the selected rerank payload slab's tiny
`HashMap<ItemPointer, usize>` lookup structures with compact vectors and linear
lookup in the TQ selected-payload path.

The hypothesis was that removing small per-group hash-map allocation and hashing
would reduce TQ stage-2 materialization overhead. The 10k / 50k / 100k A/B did
not support landing it: the result is mixed and materially worse at 100k cap60.

No code is proposed for landing from this packet. The temporary diff was
reverted after measurement and preserved as:

- `artifacts/discarded-selected-slab-vector.diff`

## Validation

- `cargo fmt --check`: passed after reverting the experiment
- `ecaz bench suite audit`: passed, 18 steps
- `ecaz bench suite run`: completed, 18 succeeded / 0 failed
- `ecaz bench suite status`: completed, 18 succeeded / 0 failed
- `ecaz bench suite report`: generated

Artifact source of truth:

- `artifacts/manifest.md`
- `artifacts/task124-tq-selected-slab-vector-10-50-100-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/report-results.jsonl`
- packet-local recall, latency, and storage logs under
  `artifacts/selected-slab-vector-ab/`

## Result Versus Packet 024 Baseline

Both variants use requested `nprobe=64`, `rerank_width=75`,
`rerank_group_width=50`, and `stage2_final_rerank_width=15`.

| Scale | Variant | Packet 024 p50/p95/p99 | Selected slab vector p50/p95/p99 | Recall@10 | Outcome |
| --- | --- | ---: | ---: | ---: | --- |
| 10k | cap off | 1.14 / 1.32 / 1.52 ms | 1.17 / 1.36 / 1.66 ms | 1.0000 | worse |
| 10k | cap 60 | 1.09 / 1.25 / 1.37 ms | 1.09 / 1.21 / 1.34 ms | 1.0000 | slight tail win |
| 50k | cap off | 4.62 / 4.80 / 4.85 ms | 4.81 / 5.35 / 6.11 ms | 0.9980 | worse |
| 50k | cap 60 | 4.56 / 4.90 / 5.50 ms | 4.39 / 4.68 / 4.83 ms | 0.9980 | better |
| 100k | cap off | 8.95 / 9.22 / 9.40 ms | 9.34 / 9.60 / 9.73 ms | 1.0000 | worse |
| 100k | cap 60 | 8.59 / 8.85 / 9.03 ms | 8.99 / 10.1 / 12.0 ms | 1.0000 | worse tail |

TQ scorer counters stayed on the intended SIMD path:

- 10k cap off: `quant=turboquant isa=neon candidates=7500 scalar_candidates=0`
- 10k cap60: `quant=turboquant isa=neon candidates=7500 scalar_candidates=0`
- 50k cap off: `quant=turboquant isa=neon candidates=7500 scalar_candidates=0`
- 50k cap60: `quant=turboquant isa=neon candidates=7500 scalar_candidates=0`
- 100k cap off: `quant=turboquant isa=neon candidates=7500 scalar_candidates=0`
- 100k cap60: `quant=turboquant isa=neon candidates=7500 scalar_candidates=0`

Storage was unchanged from packet 024:

- 10k: 10.9 MiB, 1143.6 B/row
- 50k: 50.9 MiB, 1066.8 B/row
- 100k: 100.8 MiB, 1057.2 B/row

## Outcome

Reject and do not land this optimization. It is useful negative evidence: TQ
selected-payload hash lookup is not the dominant speed bottleneck for the
current stage-2 shape, and replacing it with a vector/linear lookup increases
tail risk.

Task 124 remains open. The next slice should target a larger TQ latency lever
around the stage-2 boundary, final exact fetch/materialization count, or other
measured scan-path overhead. The TQ scorer itself is already full NEON/SIMD for
this fixture (`scalar_candidates=0`), so scorer scalar fallback is not the
current blocker.
