# Suite Report: task199-no-replica-before-10k

- config: `reviews/task-199/003-release-matrix-and-decision/artifacts/task199-no-replica-before-10k.json`
- config_sha256: `67850b1248dcf11eea155debc91588d40e9174091dceb1feff96c6b041ccee01`
- dry_run: `false`
- runner_git_commit: `9b8038bef1039c207dfc9cb01addea62f06d8010-dirty`
- steps: completed 1, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0

| Step | Kind | Status | Duration ms | Artifacts |
| --- | --- | --- | ---: | --- |
| pre-task199-no-replica-10k | distann-local-multinode | Succeeded | 138146 | `reviews/task-199/003-release-matrix-and-decision/artifacts/no-replica-before/pre-task199-no-replica-10k/distann-multinode-summary.log` |

## Parsed Results

| Step | Kind | Metric | Values |
| --- | --- | --- | --- |
| pre-task199-no-replica-10k | distann-local-multinode | drill_outcome | `drill=physical_benchmark_no_replica_insert_scale_10k`, `pass=true`, `pass_numeric=1`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
