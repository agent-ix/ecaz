# Task 206 100k result summary

The registered PG18 `ecaz bench suite` run completed on 2026-08-03. The
release preflight was unanimous for extension SHA
`59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`; the three-owner fixture passed
topology, serving, and remote-owner checks with 100,000 source rows, zero
non-owned rows, and zero orphans. The external run directory was removed
after capture.

All physical arms use the same persisted head and storage layout:
`physical_generation_bytes=2,496,659,456`, `owner_graph_side_bytes=830,160,896`,
and `cluster_index_space_amplification=1.351173`. The single-index control
has recall `0.8224` and p50 `35.8 ms`.

| variant | recall | p50 latency |
| --- | ---: | ---: |
| BW32/H4 | 0.4760 | 114.8 ms |
| BW32/H5 | 0.6061 | 136.3 ms |
| BW32/H8 | 0.8361 | 174.7 ms |
| BW64/H4 | 0.7359 | 173.0 ms |
| BW64/H5 | 0.8568 | 177.6 ms |
| BW64/H8 | 0.9584 | 187.7 ms |
| BW128/H4 | 0.8470 | 184.4 ms |
| BW128/H5 | 0.9182 | 198.7 ms |
| BW128/H8 | 0.9700 | 209.5 ms |

The Pareto-relevant choices are BW64/H8 for the lower-latency physical
tradeoff and BW128/H8 for maximum recall. The complete structured source is
`run-100k-retry/results.jsonl`; per-arm recall, latency, prediction, topology,
and storage logs are under `run-100k-retry/100k/`.
