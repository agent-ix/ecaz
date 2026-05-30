# Task 74 M5 SPIRE Overhead Gate Manifest

- head SHA: `81b7f7ea6c3902aa90c126e88d705f586811d174`
- task bucket: `reviews/task-74/001-spire-m5-overhead-gate`
- timestamp: `2026-05-30T23:37:28Z`
- lane: `m5-local`
- host: M5 laptop, 64 GB RAM
- source suite: `reviews/task-73/001-spire-m5-quality-gate/artifacts/suite.json`
- source suite manifest: `reviews/task-73/001-spire-m5-quality-gate/artifacts/suite-manifest.json`
- source suite results: `reviews/task-73/001-spire-m5-quality-gate/artifacts/results.jsonl`
- packet-local summary artifact: `reviews/task-74/001-spire-m5-overhead-gate/artifacts/overhead-summary.md`
- runner: `ecaz bench suite`
- isolation: one prefix per surface, one index per table; no shared-table benchmark surface

## Artifact Metadata

| artifact | command / source | key result |
| --- | --- | --- |
| `overhead-summary.md` | Derived from the Task 73 M5 quality-gate suite raw logs. | Records local SPIRE default, high-recall SPIRE, boundary-replica, and IVF control overhead points. |
| `../task-73/001-spire-m5-quality-gate/artifacts/pipeline-100k-tg16-b0.log` | `bench spire-pipeline`, SPIRE tg16 b0, nprobe sweep `8,16`. | Default-shape recall@10 `0.8525` at nprobe 16 with p50 `13.505 ms`. |
| `../task-73/001-spire-m5-quality-gate/artifacts/pipeline-100k-tg128-b0.log` | `bench spire-pipeline`, SPIRE tg128 b0, nprobe sweep `8,16,32,64,96,128`. | High-recall candidate recall@10 `0.9975` at nprobe 96 with p50 `75.790 ms`; ceiling recall@10 `1.0000` at nprobe 128 with p50 `95.960 ms`. |
| `../task-73/001-spire-m5-quality-gate/artifacts/pipeline-100k-tg128-b1.log` | `bench spire-pipeline`, SPIRE tg128 b1, nprobe sweep `8,16,32,64,96,128`. | Boundary replicas improve recall at lower nprobe but are slower: nprobe 64 recall@10 `0.9940`, p50 `108.444 ms`. |
| `../task-73/001-spire-m5-quality-gate/artifacts/pipeline-100k-tg128-b2.log` | `bench spire-pipeline`, SPIRE tg128 b2, nprobe sweep `8,16,32,64,96,128`. | Boundary replicas are slower still: nprobe 64 recall@10 `0.9970`, p50 `167.272 ms`. |
| `../task-73/001-spire-m5-quality-gate/artifacts/recall-100k-ivf-control.log` | `bench recall`, IVF nlists 128, heap rerank 500, nprobe sweep `48,64,80,96,128`. | IVF reaches recall@10 `0.9980` at nprobe 96 and `1.0000` at nprobe 128. |
| `../task-73/001-spire-m5-quality-gate/artifacts/latency-100k-ivf-control.log` | `bench latency`, IVF nlists 128, heap rerank 500, nprobe sweep `48,64,80,96,128`. | IVF p50 is `10.6 ms` at nprobe 96 and `12.7 ms` at nprobe 128. |

## Notes

- This Task 74 packet intentionally uses the Task 73-selected settings instead of profiling the low-recall default in isolation.
- No external profiler was installed or run. The local decision here is based on SPIRE pipeline counters, query metrics, production-read local totals, and the IVF same-host control.
- AWS profiling is justified, but should target the selected Task 73 points rather than only the current default.
