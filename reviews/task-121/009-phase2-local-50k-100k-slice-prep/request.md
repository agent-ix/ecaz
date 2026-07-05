# Review Request: Task 121 Phase 2 Local 50k/100k Slice Prep

## Scope

This packet prepares the remaining 50k and 100k execution slices of the Task 121 Phase 2 local factorial matrix. It does not execute benchmarks.

Packet `007-phase2-local-factorial-prep` prepared the full 10k/50k/100k grid, and packet `008-phase2-local-10k-slice-prep` prepared the first runnable 10k slice. This packet completes the slice set so Phase 2 can run locally scale-by-scale after reviewer sign-off or explicit override.

No AWS resources were used.

## Matrix

Each prepared slice covers the full Phase 2 interaction grid at its scale:

- scales: `50k`, `100k`
- `boundary_replica_count`: `0`, `1`, `2`, `4`
- `training_sample_rows`: `10000`, `50000`
- `nlists`: `128`, `316`
- `storage_format`: `rabitq`
- nprobe sweep: `4,8,12,16,24,32,48,64,96`
- queries per pipeline step: `200`

PQ is excluded. TurboQuant is excluded from the route-factorial slices because Stage 1 showed it was route/recall neutral; it remains a later compatibility/Pareto control if needed.

## Prepared Suite Shape

Each slice has 50 steps:

- 1 local PG18 precheck
- 16 load steps
- 16 storage steps
- 1 truth-cache recall step
- 16 `spire-pipeline` steps with recall, query metrics, cost snapshot, local-store overlap, funnel JSONL, and stage-containment JSONL

## Evidence

See `artifacts/manifest.md`.

- `target/debug/ecaz bench suite audit --config ...`: passed for both 50k and 100k, 50 steps each.
- `target/debug/ecaz ... bench suite run --dry-run --config ...`: passed for both 50k and 100k and wrote dry-run manifests.

## Next Step

After Phase 2 sign-off or explicit override, run the local slices with packet-local manifest/results/log outputs. Do not run them in AWS.
