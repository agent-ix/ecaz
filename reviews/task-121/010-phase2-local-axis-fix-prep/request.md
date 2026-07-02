# Review Request: Task 121 Phase 2 Local Axis-Fix Prep

## Scope

This packet resolves the axis-selection feedback on packets 007/008/009. It
regenerates the Phase 2 local-only `ecaz bench suite` configs and dry-run
manifests with the corrected factorial grid.

No benchmark cells were executed in this packet.

## Feedback Addressed

Reviewer feedback requested:

- do not carry screen-negative `nlists=316` as a co-equal Phase 2 axis
- add the marginal screened lever `recursive_fanout=16`
- regenerate the full grid and 10k/50k/100k slice dry-run scaffold

The corrected grid is:

- scales: `10k`, `50k`, `100k`
- `boundary_replica_count`: `0`, `1`, `2`, `4`
- `training_sample_rows`: `10000`, `50000`
- `recursive_fanout`: `8`, `16`
- `nlists`: fixed at `128`
- `storage_format`: `rabitq`
- nprobe sweep: `4,8,12,16,24,32,48,64,96`
- queries per pipeline step: `200`

PQ remains excluded. TurboQuant remains out of this route-factorial grid because
Stage 1 showed it was route/recall neutral; it can return later as a
compatibility/Pareto control if Phase 2 finds a recall-recovering route config.

## Prepared Suite Shape

The corrected full config has 148 steps:

- 1 local PG18 precheck
- 48 load steps
- 48 storage steps
- 3 truth-cache recall steps, one per scale
- 48 `spire-pipeline` steps with recall, query metrics, cost snapshot,
  local-store overlap, funnel JSONL, and stage-containment JSONL

Each scale slice has 50 steps: 1 precheck, 16 load, 16 storage, 1 truth-cache,
and 16 pipeline steps.

## Evidence

See `artifacts/manifest.md`.

- `target/debug/ecaz bench suite audit --config ...`: passed for full, 10k,
  50k, and 100k configs.
- `target/debug/ecaz ... bench suite run --dry-run --config ...`: passed for
  full, 10k, 50k, and 100k configs.
- Dry-run manifests confirm `dry_run=true`, `nlists=128`, and
  `recursive_fanout=8/16` across the expanded commands.

## Next Step

After reviewer acceptance of this corrected grid, run the local 10k slice first
as the first real Phase 2 measurement. Do not run it in AWS.
