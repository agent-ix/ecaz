# Suite Report: task179-build-gate-dml-ab

- config: `reviews/task-179/057-build-gate-dml-ab/artifacts/dml-gate-suite.json`
- config_sha256: `afabfc3b5f2b400a2d360352bc0ac8ec95f982fa005a71c027bd834395f13e5d`
- dry_run: `false`
- runner_git_commit: `07371ff3f9047701bacccbc32bb9b5043414bf78`
- steps: completed 9, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0

| Step | Kind | Status | Duration ms | Artifacts |
| --- | --- | --- | ---: | --- |
| create-databases | raw | Succeeded | 247 | `reviews/task-179/057-build-gate-dml-ab/artifacts/run/create-databases.log` |
| install-extension | raw | Succeeded | 330 | `reviews/task-179/057-build-gate-dml-ab/artifacts/run/install-extension.log` |
| prepare-control | raw | Succeeded | 33 | `reviews/task-179/057-build-gate-dml-ab/artifacts/run/prepare-control.log` |
| prepare-installed | raw | Succeeded | 23 | `reviews/task-179/057-build-gate-dml-ab/artifacts/run/prepare-installed.log` |
| control-round-1 | raw | Succeeded | 915 | `reviews/task-179/057-build-gate-dml-ab/artifacts/run/control-round-1.log` |
| installed-round-1 | raw | Succeeded | 910 | `reviews/task-179/057-build-gate-dml-ab/artifacts/run/installed-round-1.log` |
| installed-round-2 | raw | Succeeded | 917 | `reviews/task-179/057-build-gate-dml-ab/artifacts/run/installed-round-2.log` |
| control-round-2 | raw | Succeeded | 910 | `reviews/task-179/057-build-gate-dml-ab/artifacts/run/control-round-2.log` |
| compare-ab | raw | Succeeded | 25 | `reviews/task-179/057-build-gate-dml-ab/artifacts/run/compare-ab.log` |

## Parsed Results

| Step | Kind | Metric | Values |
| --- | --- | --- | --- |
| create-databases | raw | dml_gate_databases | `created=2`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
| install-extension | raw | dml_gate_extension | `extension_installed=1`, `lane=installed`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
| prepare-control | raw | dml_gate_setup | `extension_installed=0`, `lane=control`, `preload_ok=1`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
| prepare-installed | raw | dml_gate_setup | `extension_installed=1`, `lane=installed`, `preload_ok=1`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
| control-round-1 | raw | dml_gate_latency | `lane=control`, `round=1`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=178.812`, `trial=1`, `us_per_statement=7.152` |
| control-round-1 | raw | dml_gate_latency | `lane=control`, `round=1`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=180.928`, `trial=2`, `us_per_statement=7.237` |
| control-round-1 | raw | dml_gate_latency | `lane=control`, `round=1`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=170.903`, `trial=3`, `us_per_statement=6.836` |
| control-round-1 | raw | dml_gate_latency | `lane=control`, `round=1`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=173.075`, `trial=4`, `us_per_statement=6.923` |
| control-round-1 | raw | dml_gate_round | `lane=control`, `median_us=7.038`, `p95_us=7.224`, `round=1`, `samples=4`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
| installed-round-1 | raw | dml_gate_latency | `lane=installed`, `round=1`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=171.510`, `trial=1`, `us_per_statement=6.860` |
| installed-round-1 | raw | dml_gate_latency | `lane=installed`, `round=1`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=181.424`, `trial=2`, `us_per_statement=7.257` |
| installed-round-1 | raw | dml_gate_latency | `lane=installed`, `round=1`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=179.159`, `trial=3`, `us_per_statement=7.166` |
| installed-round-1 | raw | dml_gate_latency | `lane=installed`, `round=1`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=171.113`, `trial=4`, `us_per_statement=6.845` |
| installed-round-1 | raw | dml_gate_round | `lane=installed`, `median_us=7.013`, `p95_us=7.243`, `round=1`, `samples=4`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
| installed-round-2 | raw | dml_gate_latency | `lane=installed`, `round=2`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=173.635`, `trial=1`, `us_per_statement=6.945` |
| installed-round-2 | raw | dml_gate_latency | `lane=installed`, `round=2`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=170.175`, `trial=2`, `us_per_statement=6.807` |
| installed-round-2 | raw | dml_gate_latency | `lane=installed`, `round=2`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=170.283`, `trial=3`, `us_per_statement=6.811` |
| installed-round-2 | raw | dml_gate_latency | `lane=installed`, `round=2`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=187.545`, `trial=4`, `us_per_statement=7.502` |
| installed-round-2 | raw | dml_gate_round | `lane=installed`, `median_us=6.878`, `p95_us=7.418`, `round=2`, `samples=4`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
| control-round-2 | raw | dml_gate_latency | `lane=control`, `round=2`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=190.604`, `trial=1`, `us_per_statement=7.624` |
| control-round-2 | raw | dml_gate_latency | `lane=control`, `round=2`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=169.429`, `trial=2`, `us_per_statement=6.777` |
| control-round-2 | raw | dml_gate_latency | `lane=control`, `round=2`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=175.402`, `trial=3`, `us_per_statement=7.016` |
| control-round-2 | raw | dml_gate_latency | `lane=control`, `round=2`, `statements=25000`, `suite_database=tqvector_bench`, `suite_host=local_socket`, `total_ms=173.978`, `trial=4`, `us_per_statement=6.959` |
| control-round-2 | raw | dml_gate_round | `lane=control`, `median_us=6.988`, `p95_us=7.533`, `round=2`, `samples=4`, `suite_database=tqvector_bench`, `suite_host=local_socket` |
| compare-ab | raw | dml_gate_ab | `control_median_us=6.988`, `control_p95_us=7.489`, `control_samples=8`, `delta_us=-0.085`, `installed_median_us=6.903`, `installed_p95_us=7.416`, `installed_samples=8`, `overhead_pct=-1.2`, `p95_ratio=0.990`, `ratio=0.988`, `suite_database=tqvector_bench`, `suite_host=local_socket` |

## Thresholds

| Name | Status | Actual | Expected |
| --- | --- | ---: | ---: |
| control has no ecaz extension | pass | 0 | Eq 0 |
| installed lane has ecaz extension | pass | 1 | Eq 1 |
| control lane is preloaded | pass | 1 | Eq 1 |
| installed lane is preloaded | pass | 1 | Eq 1 |
| control has eight measured trials | pass | 8 | Eq 8 |
| installed has eight measured trials | pass | 8 | Eq 8 |
| inactive gate median ratio is at most 1.10x | pass | 0.988 | Lte 1.1 |
| inactive gate median delta is at most one microsecond | pass | -0.085 | Lte 1 |
| inactive gate p95 ratio is at most 1.15x | pass | 0.99 | Lte 1.15 |
