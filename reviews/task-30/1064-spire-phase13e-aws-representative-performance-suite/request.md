# Review Request

This packet preserves the interrupted attempt to start the suite-driven AWS representative performance proof on the preserved packet 1062 Graviton cluster.

It is intentionally not a completion packet. The representative smoke checks completed, then the operator requested nightly shutdown shortly after the run entered the first representative suite step (`13a3a-recall-k10`). The benchmark/tunnel process group was stopped, all running `us-west-2` EC2 instances were stopped, and a final process check found no remaining tunnel, benchmark, or cloud-down process.

Evidence:

- `artifacts/smoke-customscan-read.log`: representative smoke still uses `Custom Scan (EcSpireDistributedScan)` and `remote_heap_candidates`.
- `artifacts/bench-spire-pipeline-smoke.log`: q=5 smoke production-read rows remain healthy, with zero socket opens on the warm pooled path and no timeout/cancel/degraded skips.
- `artifacts/suite-manifest-representative-priority.json`: rendered representative suite commands are present, but selected steps remained `pending` after interruption.
- `artifacts/manifest.md`: records the shutdown boundary, instance IDs, stopped states, and the exact resume work.

Remaining Phase 13e proof work is unchanged: run `bench-representative-priority`, `bench-representative-pooling`, `summarize-representative-performance`, and `verify-representative-performance-summary` on the Graviton lane to capture representative p50/p95/p99 latency, recall, production profile, and suite-gated pooling A/B evidence.
