# Task 85 Packet 034: Product-Scale Closeout

## Result

Task 85 closes as a research/opt-in SPIRE improvement, not a product-default
change.

The task did find a real retained-recall SPIRE latency win:

| SPIRE surface | recall@10 | candidate_sum | heap_rerank_sum | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| packet 019 retained repeat | 0.9876 | 9,213,846 | 12,500 | 227.388 ms | 284.166 ms | 297.164 ms |
| packet 023 V5 + summary fast path repeat | 0.9876 | 9,213,846 | 12,500 | 222.692 ms | 275.769 ms | 286.980 ms |

That is a same-recall, same-candidate, same-rerank latency improvement. It is
not enough to justify a product default because IVF/RaBitQ remains much faster
and smaller at comparable or better 1M recall.

## Required Workstream Exits

| Workstream | Exit |
| --- | --- |
| object-read and physical layout | accepted in combination via packet 023 |
| summary scoring CPU | accepted via packets 022/023 |
| candidate-set-preserving rerank locality | rejected via packets 025-031 |
| candidate-surface redesign with recall preservation | rejected via packet 032 |
| benchmark harness and evidence extensions | closed sufficient for Task 85 via packets 009-025 |
| comparator and product policy gate | closed research/opt-in via packet 033 |

## Strongest Accepted Option

Packet 023 is the accepted SPIRE option. It preserves:

- `recall@10=0.9876`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`

and reports repeat latency:

- p50 `222.692 ms`
- p95 `275.769 ms`
- p99 `286.980 ms`

## Strongest Rejected Options

| Option | Decision |
| --- | --- |
| block8 geometry | rejected: no better retained-recall latency/candidate point |
| per-leaf cap | rejected: recall loss |
| block32 geometry | rejected: same-recall movement required candidate inflation |
| local heap TID fetch ordering | rejected: same recall/candidates, worse `228.595/284.140/295.823 ms` repeat latency |
| local heap block prefetch | rejected: same recall/candidates, worse `227.414/282.375/297.652 ms` repeat latency |
| bounded selected-block rescue | rejected by Task 84 evidence inherited into packet 032 |

## Product Decision

- Keep current product defaults unchanged.
- Do not add a Task 85 SPIRE default/balanced profile.
- Keep the packet 023 SPIRE shape as research/opt-in evidence.
- Prefer IVF/RaBitQ for the current 1M product path: `nprobe=256` reports
  `recall@10=0.9936` with p50/p95/p99 `66.2/72.5/75.7 ms` and a `298.0 MiB`
  index, compared with SPIRE packet 023 `0.9876` recall,
  `222.692/275.769/286.980 ms`, and an `872.1 MiB` index.

No ADR is required because Task 85 does not change defaults or external product
contracts.

## Evidence

See `artifacts/manifest.md` for packet-local provenance. AWS final status is
captured in `artifacts/cloud-status-final-closeout.log`.
