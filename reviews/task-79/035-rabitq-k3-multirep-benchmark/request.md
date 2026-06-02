# Task 79 Packet 035: RaBitQ k=3 Multi-Representative Benchmark

This packet tests the reviewer-recommended k=3 cluster-mean representative path on the local Task 79 100k RaBitQ surface.

It does not pass Task 79. k=3 recovers useful recall, but current scoring remains too slow at the caps needed to hit the recall gate.

AWS was not used.

## Result

| global blocks | candidates | p50 | p95 | recall@10 | returned | gate |
| ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 512 | 3,239,966 | 44.708 ms | 52.527 ms | 0.9855 | 2000 | fail recall |
| 640 | 4,050,130 | 49.910 ms | 60.820 ms | 0.9910 | 2000 | fail recall+p50 |
| 704 | 4,454,827 | 48.199 ms | 56.595 ms | 0.9920 | 2000 | fail recall+p50 |
| 768 | 4,860,209 | 49.252 ms | 59.120 ms | 0.9925 | 2000 | fail p50 |

The best recall row, global768, hits recall and candidate-count gates but misses the p50 gate by about 4.25ms.

## Evidence

- `artifacts/manifest.md`: packet metadata, commands, artifacts, and key result summary.
- `artifacts/k3-cluster-mean.patch`: temporary local source patch used for k=3 measurement.
- `artifacts/compact-results.tsv`: compact row table.
- `artifacts/cargo-test-k3-leaf-block.log`: focused k=3 summary construction test.
- `artifacts/install-k3-ecaz-pg18.log`: local patched backend install log.
- `artifacts/suite-status.log`: suite status, `completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: structured report output.
- `artifacts/pipeline-*.log`: per-row local pipeline logs.
- `suite-rabitq-k3-multirep-block32.json`: checked-in SuiteConfig used by `ecaz bench suite`.

## Interpretation

k=3 is not a direct fix, but it is useful signal. It proves the remaining misses can be recovered by richer block summaries without exceeding the candidate cap. The blocker is latency in the current scoring/selection path.

The next local direction should preserve global allocation and reduce scoring cost: either build-time/per-leaf score calibration with k=2, or a two-stage summary score that uses cheap scoring to shortlist blocks and k=3 scoring only for final block selection.
