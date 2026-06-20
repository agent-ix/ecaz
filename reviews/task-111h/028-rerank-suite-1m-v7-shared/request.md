# Review Request: Task 111h 1M v7 Shared-Table Rerank Sweep

This packet adds the 1M shared-table benchmark evidence for the v7 coarse-rerank format/width sweep.

The run used `ecaz bench suite` only. It completed 124/124 steps with no failures:

- 20 cells: `source/f32`, `index/f16`, `index/rabitq4`, `index/rabitq8`, and `index/turboquant`, each at widths `32,64,128,256`.
- Shared table prefix `task111h028_1m_shared`.
- One active IVF index per cell, with drop-before/drop-after steps.
- Recall and latency sweeps at `nprobe=8,16,32,64,128,200`.
- 990,000 corpus rows, 10,000 query rows.

Primary artifacts:

- `artifacts/manifest.md`
- `artifacts/summary.md`
- `artifacts/task111h-1m-rerank-format-width-v7-shared-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/results-report.jsonl`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/suite/*.log`

The generated truth cache `artifacts/suite/truth-1m-k10.json` is intentionally not committed.

Main result:

- `source/f32 w64` is the best normal operating point in this sweep: at `nprobe=32`, recall is `0.9570`, formal latency mean is `12.4 ms`, and the IVF index is `226.8 MiB`.
- `source/f32 w128` has the best high-recall endpoint: at `nprobe=200`, recall is `0.9910` and formal latency mean is `43.9 ms`.
- Current `index/f16` does not win: it matches source/f32 recall, but adds roughly `3.1-3.3 GiB` of IVF index storage and gets slower as width increases.
- `index/rabitq4` is recall-limited: best endpoint is `0.9370`.
- `index/rabitq8` and `index/turboquant` improve over rabitq4, but only reach about `0.95` recall at `nprobe=200`, while source/f32 reaches `0.9880-0.9910`.

Please review the suite config, artifact manifest, and summary against `results.jsonl`. The claim I want checked is deliberately narrow: this v7 shared-table sweep does not show a quantized index-side rerank mode beating source/f32 on recall/latency/storage.
