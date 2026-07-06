# Task 145 Packet 002 Review Request: Block Pruning A/B

Please review the block-pruning A/B suite result for Task 145.

This packet ran `ecaz bench suite` on release PG18 across 10k / 50k / 100k with
paired one-index-per-table control and block-pruning treatment prefixes. Both
variants used the packet 001 economy candidate `rerank_width=50`; the treatment
added block summaries at build time and capped global block selection at 128.

Decision requested: approve the packet as negative evidence and do not promote
this block-pruning configuration.

Key nprobe96 rows:

| scale | variant | p95 | distinct recall@k | candidates |
| --- | --- | ---: | ---: | ---: |
| 10k n128 | control | 10.238 ms | 1.0000 | 1502699 |
| 10k n128 | block | 6.942 ms | 0.9920 | 396050 |
| 50k n1024 | control | 14.805 ms | 0.9595 | 986258 |
| 50k n1024 | block | 16.329 ms | 0.9085 | 395052 |
| 100k n1024 | control | 19.179 ms | 0.9570 | 1874885 |
| 100k n1024 | block | 15.438 ms | 0.7755 | 402835 |

The treatment engages and cuts candidate counts, but recall loss is too large
at all high-probe cells and 50k p95 regresses. Storage also rises from 10.1 MiB
to 11.4 MiB at 10k, 54.4 MiB to 61.0 MiB at 50k, and 97.8 MiB to 110.9 MiB at
100k.

Evidence:

- `artifacts/manifest.md`
- `artifacts/task145-block-pruning-ab-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/suite-results.jsonl`
- packet-local `load-*`, `storage-*`, `truth-cache-*`, and `pipeline-*` logs

I intentionally did not stage generated truth-cache JSON or bulky
`leaf-block-rank-*.jsonl` dumps; the manifest records why and cites the
structured candidate sums from `suite-results.jsonl`.
