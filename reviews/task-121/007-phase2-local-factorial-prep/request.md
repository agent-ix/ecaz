# Review Request: Task 121 Phase 2 Local Factorial Prep

## Scope

This packet prepares the Task 121 Phase 2 local-only factorial matrix as an `ecaz bench suite` config and dry-run manifest. It does not execute the benchmark matrix.

The Phase 2 re-plan in `reviews/task-121/005-stage1-significant-set-replan/` requests reviewer sign-off before Phase 2 benches. No feedback file for packet 005 exists yet, so this packet stops at audit/dry-run evidence rather than consuming benchmark time.

## Matrix

The prepared grid matches the packet 005 significant-set plan:

- scales: `10k`, `50k`, `100k`
- `boundary_replica_count`: `0`, `1`, `2`, `4`
- `training_sample_rows`: `10000`, `50000`
- `nlists`: `128`, `316`
- `storage_format`: `rabitq`
- nprobe sweep: `4,8,12,16,24,32,48,64,96`
- queries per pipeline step: `200`

PQ is excluded. TurboQuant is excluded from this route-factorial grid because Stage 1 showed it was route/recall neutral; it can return later as a compatibility/Pareto control if Phase 2 finds a recall-recovering route config.

## Prepared Suite Shape

The config has 148 steps:

- 1 local PG18 precheck
- 48 load steps
- 48 storage steps
- 3 truth-cache recall steps, one per scale
- 48 `spire-pipeline` steps with recall, query metrics, cost snapshot, local-store overlap, funnel JSONL, and stage-containment JSONL

All inputs point at the existing staged local corpora under `data/staged-current/`.

## Evidence

See `artifacts/manifest.md`.

- `target/debug/ecaz bench suite audit --config ...`: passed, 148 steps.
- `target/debug/ecaz ... bench suite run --dry-run --config ...`: passed and wrote `suite-phase2-local-factorial-dryrun-manifest.json`.
- Dry-run manifest confirms the expected load/storage/truth-cache/pipeline command shapes and artifact outputs.

## Next Step

After reviewer sign-off on the Phase 2 plan, run this suite locally with packet-local manifest/results/log outputs. Do not run it in AWS.
