# Task 145 Packet 008: Remote Bound-Prune A/B

## Request

Please review the Task 145 remote bound-prune A/B slice.

Code under review:

- `1c1a15787 fix(task-145): propagate remote bound prune guc`

Packet evidence:

- `reviews/task-145/008-remote-bound-prune-ab/artifacts/manifest.md`

## What Changed

Remote production candidate scans now propagate
`ec_spire.pre_materialization_prune` in the remote scan session GUC set. Without
that, a remote A/B of the bound-prune switch could be inert because workers
would not receive the variant value.

Focused coverage:

- `cargo test --lib production_executor_compact_receive_requests_use_dispatch_state --no-default-features --features pg18 -- --nocapture`
- Log: `artifacts/cargo-test-remote-bound-prune-guc.log`

## Benchmark

The suite uses `ecaz bench suite` with release `spire-local-multinode` cells:

- 10k, `nlists=128`, `boundary_replica_count=0`
- 50k, `nlists=1024`, `boundary_replica_count=0`
- 100k, `nlists=1024`, `boundary_replica_count=0`

The isolated variant is:

- off: `ec_spire.pre_materialization_prune=off`
- on: `ec_spire.pre_materialization_prune=on`

Leaf-block pruning is disabled in both variants.

## Result

The bound-prune switch is recall-safe in this shape, but it is not a Task 145
latency win. Query identities are identical across all three scales, and recall
matches exactly for every nprobe. Candidate counts are also identical, so the
switch does not reduce the row/scoring work in this remote path.

At `nprobe=96`:

| scale | variant | recall@10 | p50 | p95 | candidates |
| --- | --- | ---: | ---: | ---: | ---: |
| 10k n128 | off | 1.0000 | 136.172 ms | 139.912 ms | 1,502,699 |
| 10k n128 | on | 1.0000 | 135.712 ms | 140.315 ms | 1,502,699 |
| 50k n1024 | off | 0.9595 | 142.317 ms | 150.252 ms | 986,258 |
| 50k n1024 | on | 0.9595 | 141.827 ms | 145.668 ms | 986,258 |
| 100k n1024 | off | 0.9570 | 144.508 ms | 148.630 ms | 1,874,885 |
| 100k n1024 | on | 0.9570 | 148.204 ms | 154.887 ms | 1,874,885 |

Decision: do not promote `ec_spire.pre_materialization_prune=on` for this
measured remote low-probe/rerank-width-50 shape. Keep the remote GUC propagation
fix because it makes this and future remote A/Bs real.

## Notes

The top-level `suite-results.jsonl` is empty for these nested
`spire-local-multinode` steps; each per-cell nested `bench-suite/results.jsonl`
contains 244 rows and is cited by the manifest. Release install/build profiles
and `HARNESS PASSED` are recorded in each cell's `local-multinode.log`.
