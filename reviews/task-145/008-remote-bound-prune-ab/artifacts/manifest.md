# Task 145 Packet 008 Artifact Manifest

- Task: 145 - rerank economy at low probe
- Packet: `reviews/task-145/008-remote-bound-prune-ab`
- Head SHA: `1c1a15787156af69c2c98e77a2968b0381e16114`
- Code under review: `1c1a15787 fix(task-145): propagate remote bound prune guc`
- Date: 2026-07-06

## Slice

This packet isolates the remote bound-prune switch by A/B testing
`ec_spire.pre_materialization_prune=off` versus `on` in the production remote
scan path.

The code slice propagates `ec_spire.pre_materialization_prune` through remote
production scan session GUCs so the remote workers actually receive the
variant value.

## Suite Configuration

- Runner: `target/release/ecaz bench suite`
- Harness step kind: `spire-local-multinode`
- Install/build profile: `release`
- PostgreSQL: PG18
- Storage format: `rabitq`
- Scale cells:
  - `10k`, `nlists=128`, `boundary_replica_count=0`
  - `50k`, `nlists=1024`, `boundary_replica_count=0`
  - `100k`, `nlists=1024`, `boundary_replica_count=0`
- Query count: 200
- `top_k`: 10
- `nprobe`: 8, 16, 32, 64, 96
- Common GUCs:
  - `ec_spire.source_identity=include`
  - `ec_spire.leaf_score_only_routing=on`
  - `ec_spire.route_overfetch_multiplier=1.0`
  - `ec_spire.probe_distance_ratio=0`
  - `ec_spire.rerank_width=50`
  - `ec_spire.max_candidate_rows=100`
  - `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`
  - `ec_spire.leaf_block_pruning_max_global_blocks=0`
  - `ec_spire.max_remote_payload_bytes_per_row=16384`
- Variants:
  - `bound-prune-off`: `ec_spire.pre_materialization_prune=off`
  - `bound-prune-on`: `ec_spire.pre_materialization_prune=on`

## Commands

Focused regression:

```bash
script -q -c "cargo test --lib production_executor_compact_receive_requests_use_dispatch_state --no-default-features --features pg18 -- --nocapture" reviews/task-145/008-remote-bound-prune-ab/artifacts/cargo-test-remote-bound-prune-guc.log
```

Suite audit:

```bash
target/release/ecaz bench suite audit --config reviews/task-145/008-remote-bound-prune-ab/artifacts/task145-remote-bound-prune-ab-suite.json --log-file reviews/task-145/008-remote-bound-prune-ab/artifacts/suite-audit.log
```

Suite dry run:

```bash
target/release/ecaz bench suite run --dry-run --config reviews/task-145/008-remote-bound-prune-ab/artifacts/task145-remote-bound-prune-ab-suite.json --artifact-dir reviews/task-145/008-remote-bound-prune-ab/artifacts --manifest-output reviews/task-145/008-remote-bound-prune-ab/artifacts/suite-manifest-dry-run.json --results-output reviews/task-145/008-remote-bound-prune-ab/artifacts/suite-results-dry-run.jsonl --log-file reviews/task-145/008-remote-bound-prune-ab/artifacts/suite-dry-run.log
```

Suite run:

```bash
target/release/ecaz bench suite run --config reviews/task-145/008-remote-bound-prune-ab/artifacts/task145-remote-bound-prune-ab-suite.json --artifact-dir reviews/task-145/008-remote-bound-prune-ab/artifacts --manifest-output reviews/task-145/008-remote-bound-prune-ab/artifacts/suite-manifest.json --results-output reviews/task-145/008-remote-bound-prune-ab/artifacts/suite-results.jsonl --log-file reviews/task-145/008-remote-bound-prune-ab/artifacts/suite-run.log
```

## Artifacts

- `task145-remote-bound-prune-ab-suite.json`: checked-in suite config.
- `suite-audit.log`: audit output for the suite config.
- `suite-dry-run.log`: dry-run expansion log.
- `suite-manifest-dry-run.json`: dry-run manifest.
- `suite-run.log`: top-level suite run log.
- `suite-manifest.json`: top-level suite manifest.
- `suite-results.jsonl`: top-level result sink. It is empty for these
  nested `spire-local-multinode` steps; per-cell nested results are authoritative.
- `cargo-test-remote-bound-prune-guc.log`: focused regression proving compact
  receive requests include the remote scan session GUC set.
- `latency-recall-summary.txt`: compact recall and latency table.
- `leaf-score-summary.txt`: compact leaf candidate and scoring cost table.
- `identity-comparison.txt`: per-scale identity equality summary.
- `remote-10k-n128-bound-r1/local-multinode.log`: release install/build and
  harness proof for the 10k cell.
- `remote-50k-n1024-bound-r1/local-multinode.log`: release install/build and
  harness proof for the 50k cell.
- `remote-100k-n1024-bound-r1/local-multinode.log`: release install/build and
  harness proof for the 100k cell.
- `remote-*/bench-suite/local-real-production-read-suite.json`: nested suite
  config emitted by each local-multinode cell.
- `remote-*/bench-suite/suite-manifest.json`: nested suite manifests.
- `remote-*/bench-suite/suite-run.log`: nested suite run logs.
- `remote-*/bench-suite/results.jsonl`: nested recall/latency/storage results.
- `remote-*/bench-suite/storage.log`: storage measurements.
- `remote-*/bench-suite/production-read-k10-bound-prune-*-default.log`:
  production read result logs.
- `remote-*/bench-suite/production-read-k10-bound-prune-*-default-identity.jsonl`:
  identity traces used by `identity-comparison.tsv`.

Generated corpus TSVs, correctness data TSVs, PostgreSQL server logs, load
logs, and remote materialization operational logs are intentionally not
committed.

## Release Proof

Each local-multinode cell records release install/build and a passing harness:

```text
remote-10k-n128-bound-r1/local-multinode.log: install_profile=release, node_build_profile=release for coord/remote1/remote2/remote3, HARNESS PASSED
remote-50k-n1024-bound-r1/local-multinode.log: install_profile=release, node_build_profile=release for coord/remote1/remote2/remote3, HARNESS PASSED
remote-100k-n1024-bound-r1/local-multinode.log: install_profile=release, node_build_profile=release for coord/remote1/remote2/remote3, HARNESS PASSED
```

## Key Results

At `nprobe=96`:

| scale | variant | recall@10 | distinct_recall@10 | p50 | p95 | candidates | truncated |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k n128 | off | 1.0000 | 1.0000 | 136.172 ms | 139.912 ms | 1,502,699 | 1,496,699 |
| 10k n128 | on | 1.0000 | 1.0000 | 135.712 ms | 140.315 ms | 1,502,699 | 1,496,699 |
| 50k n1024 | off | 0.9595 | 0.9595 | 142.317 ms | 150.252 ms | 986,258 | 980,258 |
| 50k n1024 | on | 0.9595 | 0.9595 | 141.827 ms | 145.668 ms | 986,258 | 980,258 |
| 100k n1024 | off | 0.9570 | 0.9570 | 144.508 ms | 148.630 ms | 1,874,885 | 1,868,885 |
| 100k n1024 | on | 0.9570 | 0.9570 | 148.204 ms | 154.887 ms | 1,874,885 | 1,868,885 |

Identity traces are byte-identical by query result set at all scales:

```text
10k-n128    1000 compared, 0 different, 1000 same
50k-n1024   1000 compared, 0 different, 1000 same
100k-n1024  1000 compared, 0 different, 1000 same
```

Decision: `ec_spire.pre_materialization_prune=on` is recall-safe in this
remote shape, but it does not reduce leaf candidate or truncated candidate
counts. It is neutral to slightly negative on latency, with the 100k nprobe96
cell regressing from 144.508 ms p50 / 148.630 ms p95 to 148.204 ms p50 /
154.887 ms p95. Do not promote this switch as a Task 145 win for the measured
shape.
