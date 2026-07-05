# Task 65b Packet 008: Worker-Zero Fallback Evidence

## Summary

This packet records a corpus-scale worker-zero fallback run for DiskANN build using the current Task 65b branch. The run is driven by `ecaz bench suite` and covers real10k plus real100k with `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, and `alpha=1.2`.

Worker-zero was enforced in the suite through both:

- `PGOPTIONS="-c max_parallel_maintenance_workers=0 -c max_parallel_workers=0"`
- table reloption `parallel_workers=0`

I removed the suite runner's `capture_parallel_workers` option from this packet because that parser is currently IVF-specific; keeping it made the DiskANN suite fail after otherwise successful steps. The dry-run log and manifest show the worker-zero controls that were used instead.

## Results

The full 9-step suite completed successfully. The graph steps were rerun after rebuilding the CLI so the packet-local graph logs include the new digest rows.

Real10k worker-zero:

- load completed prefix in `9.78s`
- recall@10 at L64/L128/L200: `0.9965`, `0.9970`, `0.9975`
- mean q-time at L64/L128/L200: `0.61 ms`, `0.68 ms`, `0.78 ms`
- DiskANN index size: `4.7 MiB`, `494.0 B` per row
- reachable live fraction: `1.000000`
- neighbor refs: `257058`
- digest baseline:
  - live TID digest `b476ea9f9a43d92eff12389fab3a013060d0a1cfdc47665af859194b4764d1bd`
  - adjacency digest `af9fe980fb9d0f6149d4102a82d561af0fc7e9b2fde422f47acc5e1e3cf7f0b5`
  - first 256 node digest `da8ab263ef126cffc5e62ddd42969e86f58b75e860f8b87f1327649246e2a667`

Real100k worker-zero:

- built index in `243.15s`
- completed prefix in `411.92s`
- recall@10 at L64/L128/L200: `0.9190`, `0.9640`, `0.9755`
- mean q-time at L64/L128/L200: `0.96 ms`, `1.11 ms`, `1.40 ms`
- DiskANN index size: `46.1 MiB`, `483.1 B` per row
- reachable live fraction: `0.999890`
- neighbor refs: `3101446`
- digest baseline:
  - live TID digest `5739d9a6040ccf6fe041e297d201a5a25537d18955398d9054c378926d81de53`
  - adjacency digest `683af2fb14938b475054f2d735d14e89a162947e93dba795d0077c5f492b5a12`
  - first 256 node digest `e332f9a4cba1318e4563adc9e2802d33ffefd161be3c76abf14eed503c31b4f7`

The real10k and real100k recall rows match packet `001` worker-zero recall exactly. Real100k build time is effectively unchanged versus packet `001` (`243.15s` here vs `243.29s` there).

## Evidence

- Manifest: `reviews/task-65b/008-worker-zero-fallback/artifacts/manifest.md`
- Suite config: `reviews/task-65b/008-worker-zero-fallback/suite.json`
- Suite audit: `reviews/task-65b/008-worker-zero-fallback/artifacts/suite-audit.log`
- Suite dry-run: `reviews/task-65b/008-worker-zero-fallback/artifacts/suite-dry-run.log`
- Full suite run: `reviews/task-65b/008-worker-zero-fallback/artifacts/suite-run.log`
- Suite manifest: `reviews/task-65b/008-worker-zero-fallback/artifacts/suite-manifest.json`
- Graph digest rerun: `reviews/task-65b/008-worker-zero-fallback/artifacts/suite-graph-rerun.log`
- Per-step logs under `reviews/task-65b/008-worker-zero-fallback/artifacts/`

Note: `results.jsonl` is present but empty after the graph-only digest rerun rewrote that configured output path. The durable result sources for this packet are the suite log, suite manifest, per-step logs, and artifact manifest.

## Review Ask

Please review this as the worker-zero fallback evidence packet for Task 65b. It establishes that the current branch preserves worker-zero corpus-scale recall/storage/timing behavior and now has graph digests for future deterministic comparisons. It does not close the full Task 65b acceptance set; worker tuning, flush/batch sizing, and final performance/scaling evidence are still pending.
