# Task 145 Packet 007 Artifact Manifest

- Head SHA: `f2317c97d6d9a85e42ddc06e58c052a2a8fdf608`
- Task bucket: `reviews/task-145/007-remote-block-pruning-ab`
- Timestamp: `2026-07-06T10:14:55-07:00`
- Lane: local PG18 release, local-multinode native SPIRE, remote production-read path
- Fixture: `ec_real_10k`, `ec_real_50k`, `ec_real_100k` staged corpora from `data/staged-current`
- Storage format: `rabitq`
- Rerank mode: `ec_spire.rerank_width=50`, `ec_spire.max_candidate_rows=100`
- Shared-table vs isolated: local-multinode one coordinator table plus per-node remote tables/indexes; no shared-table benchmark surface

## Code Validation

Artifact:

- `cargo-test-remote-scan-guc-propagation.log`

Command:

```bash
script -q -c "cargo test production_executor_compact_receive_requests_use_dispatch_state --no-default-features --features pg18" reviews/task-145/007-remote-block-pruning-ab/artifacts/cargo-test-remote-scan-guc-propagation.log
```

Result:

- Passed: `1 passed; 0 failed; 2271 filtered out`.

## Suite Validation

Artifacts:

- `task145-remote-block-pruning-ab-suite.json`
- `suite-audit-r2.log`
- `suite-dry-run-r2.log`
- `suite-manifest-dry-run-r2.json`
- `suite-run-r2.log`
- `suite-manifest-r2.json`
- `suite-results-r2.jsonl` (top-level runner file is empty; nested per-cell results are authoritative)

Commands:

```bash
target/release/ecaz bench suite audit --config reviews/task-145/007-remote-block-pruning-ab/artifacts/task145-remote-block-pruning-ab-suite.json --log-file reviews/task-145/007-remote-block-pruning-ab/artifacts/suite-audit-r2.log
target/release/ecaz bench suite run --dry-run --config reviews/task-145/007-remote-block-pruning-ab/artifacts/task145-remote-block-pruning-ab-suite.json --manifest-output reviews/task-145/007-remote-block-pruning-ab/artifacts/suite-manifest-dry-run-r2.json --log-file reviews/task-145/007-remote-block-pruning-ab/artifacts/suite-dry-run-r2.log
target/release/ecaz bench suite run --config reviews/task-145/007-remote-block-pruning-ab/artifacts/task145-remote-block-pruning-ab-suite.json --manifest-output reviews/task-145/007-remote-block-pruning-ab/artifacts/suite-manifest-r2.json --results-output reviews/task-145/007-remote-block-pruning-ab/artifacts/suite-results-r2.jsonl --log-file reviews/task-145/007-remote-block-pruning-ab/artifacts/suite-run-r2.log
```

Result:

- Audit passed: 3 steps.
- Full suite completed: 10k n128, 50k n1024, 100k n1024.
- Every local-multinode step recorded `install_profile=release`.
- Every node in every step recorded `node_build_profile ... profile=release`.

## Per-Cell Artifacts

Each cell includes:

- `local-multinode.log`
- `bench-suite/local-real-production-read-suite.json`
- `bench-suite/suite-manifest.json`
- `bench-suite/suite-run.log`
- `bench-suite/results.jsonl`
- `bench-suite/storage.log`
- `bench-suite/production-read-k10-block-off-default.log`
- `bench-suite/production-read-k10-block-global128-default.log`
- `bench-suite/production-read-k10-block-off-default-identity.jsonl`
- `bench-suite/production-read-k10-block-global128-default-identity.jsonl`
- `bench-suite/summary-10k-counters.txt`, `summary-50k-counters.txt`, or `summary-100k-counters.txt`

Cell directories:

- `remote-10k-n128-block-r2`
- `remote-50k-n1024-block-r2`
- `remote-100k-n1024-block-r2`

## Key Results

At nprobe 96:

| Cell | Variant | recall | p50 | p95 | remote_heap | leaf blocks available/selected/skipped |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k n128 | block-off | 1.0000 | 136.745 ms | 143.083 ms | 30000 | 102712 / 102712 / 0 |
| 10k n128 | block-global128 | 1.0000 | 135.147 ms | 140.111 ms | 30000 | 102712 / 76800 / 25912 |
| 50k n1024 | block-off | 0.9595 | 139.848 ms | 147.403 ms | 30000 | 70264 / 70264 / 0 |
| 50k n1024 | block-global128 | 0.9595 | 139.496 ms | 144.044 ms | 30000 | 70264 / 68499 / 1765 |
| 100k n1024 | block-off | 0.9570 | 142.663 ms | 147.220 ms | 30000 | 126184 / 126184 / 0 |
| 100k n1024 | block-global128 | 0.9340 | 144.001 ms | 154.852 ms | 30000 | 126184 / 76797 / 49387 |

Identity comparison:

- 10k: `cmp=0`, 1000/1000 query rows identical.
- 50k: `cmp=0`, 1000/1000 query rows identical.
- 100k: `cmp=1`, 938/1000 rows identical, 62/1000 rows diverged.

Interpretation:

- Remote block pruning is engaged at current head, with nonzero `leaf_block_skipped_sum` in the production-read profile path.
- 10k and 50k are recall-neutral under this `global128` setting.
- 100k is recall-stressing and not recall-neutral: `block-global128` drops nprobe96 distinct recall from 0.9570 to 0.9340 and worsens p95 from 147.220 ms to 154.852 ms.
- This packet supports iterate/escalate for block-pruning thresholds, not promotion of `global128`.
