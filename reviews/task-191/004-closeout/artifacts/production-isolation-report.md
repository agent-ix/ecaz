# Suite Report: task191-production-build-isolation

- config: `reviews/task-191/004-closeout/artifacts/production-isolation-suite.json`
- config_sha256: `b3bb9e945d6fd3c2f952ad7d8678725825285a35515c76dfcb3654126f333389`
- dry_run: `false`
- runner_git_commit: `3ceaba9615bef4dcd9c4ca7c49c2dd07256b9e51`
- steps: completed 1, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0

| Step | Kind | Status | Duration ms | Artifacts |
| --- | --- | --- | ---: | --- |
| normal-release-lazy10-smoke | distann-local-multinode | Succeeded | 145520 | `reviews/task-191/004-closeout/artifacts/production-isolation-run/normal-release-lazy10-smoke/distann-multinode-summary.log` |

## Parsed Results

| Step | Kind | Metric | Values |
| --- | --- | --- | --- |
| normal-release-lazy10-smoke | distann-local-multinode | physical_topology | `control_bytes=8192`, `directory_bytes=131072`, `graph_bytes=25133056`, `node=1`, `non_owned=0`, `orphans=0`, `phase=ready`, `records=3323`, `row_bytes=55427072`, `rows=3323`, `state=Ready`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `topology_ok=true` |
| normal-release-lazy10-smoke | distann-local-multinode | physical_topology | `control_bytes=8192`, `directory_bytes=131072`, `graph_bytes=25575424`, `node=2`, `non_owned=0`, `orphans=0`, `phase=ready`, `records=3391`, `row_bytes=56582144`, `rows=3391`, `state=Ready`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `topology_ok=true` |
| normal-release-lazy10-smoke | distann-local-multinode | physical_topology | `control_bytes=8192`, `directory_bytes=122880`, `graph_bytes=24797184`, `node=3`, `non_owned=0`, `orphans=0`, `phase=ready`, `records=3286`, `row_bytes=54837248`, `rows=3286`, `state=Ready`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `topology_ok=true` |
| normal-release-lazy10-smoke | distann-local-multinode | physical_topology | `control_bytes=8192`, `directory_bytes=131072`, `graph_bytes=25133056`, `node=1`, `non_owned=0`, `orphans=0`, `phase=published`, `records=3323`, `row_bytes=55427072`, `rows=3323`, `state=Published`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `topology_ok=true` |
| normal-release-lazy10-smoke | distann-local-multinode | physical_topology | `control_bytes=8192`, `directory_bytes=131072`, `graph_bytes=25575424`, `node=2`, `non_owned=0`, `orphans=0`, `phase=published`, `records=3391`, `row_bytes=56582144`, `rows=3391`, `state=Published`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `topology_ok=true` |
| normal-release-lazy10-smoke | distann-local-multinode | physical_topology | `control_bytes=8192`, `directory_bytes=122880`, `graph_bytes=24797184`, `node=3`, `non_owned=0`, `orphans=0`, `phase=published`, `records=3286`, `row_bytes=54837248`, `rows=3286`, `state=Published`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `topology_ok=true` |
| normal-release-lazy10-smoke | distann-local-multinode | drill_outcome | `drill=physical_serving`, `pass=true`, `pass_numeric=1`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
| normal-release-lazy10-smoke | distann-local-multinode | drill_outcome | `drill=physical_topology_gate`, `pass=true`, `pass_numeric=1`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
