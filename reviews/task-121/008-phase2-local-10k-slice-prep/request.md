# Review Request: Task 121 Phase 2 Local 10k Slice Prep

## Scope

This packet prepares a 10k-only execution slice of the Task 121 Phase 2 local factorial matrix. It does not execute the benchmark slice.

Packet `007-phase2-local-factorial-prep` prepared and dry-ran the full 10k/50k/100k grid. This packet derives the first runnable 10k slice from that full config so the initial Phase 2 run can be launched, monitored, and reviewed independently after reviewer sign-off or explicit override.

No AWS resources were used.

## Matrix

The prepared 10k slice covers the full Phase 2 interaction grid at the 10k scale:

- `boundary_replica_count`: `0`, `1`, `2`, `4`
- `training_sample_rows`: `10000`, `50000`
- `nlists`: `128`, `316`
- `storage_format`: `rabitq`
- nprobe sweep: `4,8,12,16,24,32,48,64,96`
- queries per pipeline step: `200`

PQ is excluded. TurboQuant is excluded from this route-factorial slice for the same reason as packet 007: it was recall-neutral in Stage 1 and belongs later as a compatibility/Pareto control if needed.

## Prepared Suite Shape

The 10k slice has 50 steps:

- 1 local PG18 precheck
- 16 load steps
- 16 storage steps
- 1 truth-cache recall step
- 16 `spire-pipeline` steps with recall, query metrics, cost snapshot, local-store overlap, funnel JSONL, and stage-containment JSONL

## Evidence

See `artifacts/manifest.md`.

- `target/debug/ecaz bench suite audit --config ...`: passed, 50 steps.
- `target/debug/ecaz ... bench suite run --dry-run --config ...`: passed and wrote `suite-phase2-local-10k-slice-dryrun-manifest.json`.

## Next Step

After Phase 2 sign-off or explicit override, run this 10k slice locally with packet-local manifest/results/log outputs. Do not run it in AWS.
