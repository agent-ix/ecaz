# Task 51 AWS RaBitQ8 Sidecar Warm Sweep

- Branch: `aws-optimization-ivf-rabitq-spire`
- Task bucket: `reviews/task-51`
- Benchmark packet: `benchmarks/task51-aws-rabitq8-sidecar-warm-sweep`
- Scope: warm AWS 1M IVF/RaBitQ sidecar-only rerun
- Variants: `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- Excluded: vchord, pgvectorscale/DiskANN, unchanged comparator reruns
- AWS profile: `10k-medium`
- Preserved snapshot: `snap-0b72153293b0b749b`

## Intended Run

This packet reruns the new-sidecar sweep after the cold run in
`benchmarks/task51-aws-rabitq8-sidecar-full-sweep`.

Differences from the cold run:

- `warmup_queries=200` before timed candidate and sidecar metrics.
- `rebuild_sidecar_table=false` so complete sidecar measurement tables are reused.
- Remote binary includes the one-variant-at-a-time sidecar harness change from `0429af2ab`.

## Evidence

Pending AWS rerun.
