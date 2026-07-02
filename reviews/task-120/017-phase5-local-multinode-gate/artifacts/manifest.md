# Artifact Manifest: Task 120 Phase 5 Local Multi-Node Gate

- Head SHA: `c5448e08c781893ae6919b2325625f6248336d7b`
- Branch: `task-120-spire-coarse-rerank-measurement-program`
- Task bucket: `reviews/task-120/017-phase5-local-multinode-gate`
- Created: `2026-06-22T15:10:28Z`
- Lane: local multi-node PG18 distributed SPIRE gate
- Fixture: DBPedia staged real-corpus 10k/50k/100k plus one tiny static remote smoke
- Storage format: `rabitq`
- Rerank mode: production-read path with `rerank_width=25`
- Surface: isolated one-index-per-table per scale and per worker shard; no shared-table current-lane surface
- AWS/cloud: not used. This packet used local PostgreSQL instances only.

## Local Multi-Node Requirement

This is a local multi-node distributed test. Each real-corpus run used one
coordinator PostgreSQL instance plus three worker PostgreSQL instances on the
same physical machine. The workers were distinct SPIRE node identities
`2`, `3`, and `4`; running them on the same host was intentional and still
exercised the distributed path through static remote placements and remote
dispatch. A single-node local scan was not used for the gate evidence.

| Run | Coordinator port | Worker ports | Prefix | Artifact path |
| --- | ---: | --- | --- | --- |
| tiny smoke | 39710 | 39711, 39712, 39713 | `ec_spire_phase13e_static` | `artifacts/static-remote-smoke/` |
| 10k | 39720 | 39721, 39722, 39723 | `t120p5r10k` | `artifacts/real10k-valid/` |
| 50k | 39730 | 39731, 39732, 39733 | `t120p5r50k` | `artifacts/real50k/` |
| 100k | 39740 | 39741, 39742, 39743 | `t120p5r100k` | `artifacts/real100k/` |

## Commands

Static remote smoke:

```sh
scripts/run_spire_phase13e_static_remote_placement_pg18.sh \
  --artifact-dir reviews/task-120/017-phase5-local-multinode-gate/artifacts/static-remote-smoke \
  --run-id task120-017-local-smoke \
  --skip-install \
  --coord-port 39710 --remote1-port 39711 --remote2-port 39712 --remote3-port 39713 \
  --fixture-rows 12 --bench-top-k 6 --bench-queries-limit 1 --bench-sweep 3
```

10k local multi-node run:

```sh
env SPIRE_AWS_SKIP_REPRESENTATIVE_PREPARE=1 \
  SPIRE_AWS_STORAGE_FORMAT=rabitq \
  SPIRE_AWS_REMOTE_CORPUS_OUTPUT_DIR=target/t120p5-real10k-shards \
  SPIRE_AWS_COORD_RELOPTIONS='nlists=128;recursive_fanout=8;nprobe=24;rerank_width=25;boundary_replica_count=0;top_graph_enabled=1;top_graph_degree=32;top_graph_build_list_size=100;top_graph_search_list_size=96' \
  SPIRE_AWS_REMOTE_RELOPTIONS='nlists=128;recursive_fanout=8;nprobe=24;rerank_width=25;boundary_replica_count=0;top_graph_enabled=1;top_graph_degree=32;top_graph_build_list_size=100;top_graph_search_list_size=96' \
  scripts/run_spire_phase13e_aws_harness_local_pg18.sh \
    --artifact-dir reviews/task-120/017-phase5-local-multinode-gate/artifacts/real10k-valid \
    --run-id r10k3 --run-dir target/t120r10k3 \
    --skip-install --skip-fault-drills \
    --tier representative --prefix t120p5r10k \
    --prepared-prefix ec_real_10k --prepared-dir data/task106_intel_dbpedia_staged \
    --bench-top-k 10 --bench-queries-limit 200 \
    --bench-sweep 64,96 --bench-rowcap-sweep 96 \
    --bench-truth-corpus-file data/task106_intel_dbpedia_staged/ec_real_10k_corpus.tsv \
    --coord-port 39720 --remote1-port 39721 --remote2-port 39722 --remote3-port 39723
```

50k local multi-node run:

```sh
env SPIRE_AWS_SKIP_REPRESENTATIVE_PREPARE=1 \
  SPIRE_AWS_STORAGE_FORMAT=rabitq \
  SPIRE_AWS_REMOTE_CORPUS_OUTPUT_DIR=target/t120p5-real50k-shards \
  SPIRE_AWS_COORD_RELOPTIONS='nlists=128;recursive_fanout=8;nprobe=24;rerank_width=25;boundary_replica_count=0;top_graph_enabled=1;top_graph_degree=32;top_graph_build_list_size=100;top_graph_search_list_size=96' \
  SPIRE_AWS_REMOTE_RELOPTIONS='nlists=128;recursive_fanout=8;nprobe=24;rerank_width=25;boundary_replica_count=0;top_graph_enabled=1;top_graph_degree=32;top_graph_build_list_size=100;top_graph_search_list_size=96' \
  scripts/run_spire_phase13e_aws_harness_local_pg18.sh \
    --artifact-dir reviews/task-120/017-phase5-local-multinode-gate/artifacts/real50k \
    --run-id r50k --run-dir target/t120r50k \
    --skip-install --skip-fault-drills \
    --tier representative --prefix t120p5r50k \
    --prepared-prefix ec_real_50k --prepared-dir data/task111a_real50k \
    --bench-top-k 10 --bench-queries-limit 200 \
    --bench-sweep 64,96 --bench-rowcap-sweep 96 \
    --bench-truth-corpus-file data/task111a_real50k/ec_real_50k_corpus.tsv \
    --coord-port 39730 --remote1-port 39731 --remote2-port 39732 --remote3-port 39733
```

100k local multi-node run:

```sh
env SPIRE_AWS_SKIP_REPRESENTATIVE_PREPARE=1 \
  SPIRE_AWS_STORAGE_FORMAT=rabitq \
  SPIRE_AWS_REMOTE_CORPUS_OUTPUT_DIR=target/t120p5-real100k-shards \
  SPIRE_AWS_COORD_RELOPTIONS='nlists=128;recursive_fanout=8;nprobe=24;rerank_width=25;boundary_replica_count=0;top_graph_enabled=1;top_graph_degree=32;top_graph_build_list_size=100;top_graph_search_list_size=96' \
  SPIRE_AWS_REMOTE_RELOPTIONS='nlists=128;recursive_fanout=8;nprobe=24;rerank_width=25;boundary_replica_count=0;top_graph_enabled=1;top_graph_degree=32;top_graph_build_list_size=100;top_graph_search_list_size=96' \
  scripts/run_spire_phase13e_aws_harness_local_pg18.sh \
    --artifact-dir reviews/task-120/017-phase5-local-multinode-gate/artifacts/real100k \
    --run-id r100k --run-dir target/t120r100k \
    --skip-install --skip-fault-drills \
    --tier representative --prefix t120p5r100k \
    --prepared-prefix ec_real_100k --prepared-dir data/task106_full_sweep_100k \
    --bench-top-k 10 --bench-queries-limit 200 \
    --bench-sweep 64,96 --bench-rowcap-sweep 96 \
    --bench-truth-corpus-file data/task106_full_sweep_100k/ec_real_100k_corpus.tsv \
    --coord-port 39740 --remote1-port 39741 --remote2-port 39742 --remote3-port 39743
```

## Suite Artifacts

Each real-corpus run was driven by `ecaz bench suite` through the checked-in
local harness. The suite configs and results are:

| Scale | Suite config | Suite manifest | Results |
| --- | --- | --- | --- |
| 10k | `artifacts/real10k-valid/bench-suite/local-real-production-read-suite.json` | `artifacts/real10k-valid/bench-suite/suite-manifest.json` | `artifacts/real10k-valid/bench-suite/results.jsonl` |
| 50k | `artifacts/real50k/bench-suite/local-real-production-read-suite.json` | `artifacts/real50k/bench-suite/suite-manifest.json` | `artifacts/real50k/bench-suite/results.jsonl` |
| 100k | `artifacts/real100k/bench-suite/local-real-production-read-suite.json` | `artifacts/real100k/bench-suite/suite-manifest.json` | `artifacts/real100k/bench-suite/results.jsonl` |

The tiny static remote smoke also has suite artifacts under
`artifacts/static-remote-smoke/bench-suite/`.

## Corpus Provenance

Corpus/query TSVs are not committed per repository policy. The local staged
inputs and loader SHA lines are recorded in coordinator load logs:

| Scale | Staged corpus | Corpus SHA | Query SHA |
| --- | --- | --- | --- |
| 10k | `data/task106_intel_dbpedia_staged/ec_real_10k_corpus.tsv` | `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` | `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` |
| 50k | `data/task111a_real50k/ec_real_50k_corpus.tsv` | `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` |
| 100k | `data/task106_full_sweep_100k/ec_real_100k_corpus.tsv` | `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` |

Remote shard TSVs were generated under:

- `target/t120p5-real10k-shards`
- `target/t120p5-real50k-shards`
- `target/t120p5-real100k-shards`

Those shard TSVs are not committed.

## Key Results

See `artifacts/phase5-local-multinode-summary.md` for the compact result table.

All real-corpus suite runs passed and reported `HARNESS PASSED`.

| Scale | Step | nprobe | recall@10 | p50 | p95 | p99 | Storage total | SPIRE index |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | default | 64 | 0.9850 | 42.148 ms | 49.005 ms | 57.794 ms | 168.4 MiB | 9.4 MiB |
| 10k | default | 96 | 0.9855 | 44.638 ms | 50.035 ms | 54.638 ms | 168.4 MiB | 9.4 MiB |
| 10k | rowcap25k | 96 | 0.9855 | 48.339 ms | 68.922 ms | 77.558 ms | 168.4 MiB | 9.4 MiB |
| 50k | default | 64 | 0.9850 | 54.194 ms | 62.366 ms | 71.122 ms | 835.6 MiB | 40.7 MiB |
| 50k | default | 96 | 0.9900 | 59.243 ms | 82.910 ms | 87.674 ms | 835.6 MiB | 40.7 MiB |
| 50k | rowcap25k | 96 | 0.9900 | 59.481 ms | 64.358 ms | 67.666 ms | 835.6 MiB | 40.7 MiB |
| 100k | default | 64 | 0.9730 | 78.257 ms | 117.404 ms | 125.179 ms | 1.6 GiB | 79.7 MiB |
| 100k | default | 96 | 0.9880 | 98.876 ms | 113.926 ms | 134.894 ms | 1.6 GiB | 79.7 MiB |
| 100k | rowcap25k | 96 | 0.9880 | 95.085 ms | 108.528 ms | 114.650 ms | 1.6 GiB | 79.7 MiB |

Remote dispatch evidence for every real-corpus suite row:

- `status=ready`
- `result_source=remote_heap_candidates`
- `local_pid_sum=0`
- `remote_pid_sum=12800` for nprobe `64`, `19200` for nprobe `96`
- `dispatch_sum=600`
- `remote_heap_candidate_sum=6000`
- `merge_input_sum=6000`
- `merge_output_sum=2000`
- `timeout_sum=0`
- `cancel_sum=0`
- `degraded_skip_sum=0`

## Build And Load Times

| Scale | Coordinator index build | Coordinator load total | Worker index builds |
| --- | ---: | ---: | --- |
| 10k | 2.34s | 14.85s | node 2: 719.99ms; node 3: 772.84ms; node 4: 722.13ms |
| 50k | 7.30s | 71.88s | node 2: 3.81s; node 3: 3.54s; node 4: 3.11s |
| 100k | 12.40s | 130.43s | node 2: 4.83s; node 3: 4.67s; node 4: 6.60s |

## Caveats

- The rowcap25k step did not bind at these route counts because the nprobe `96`
  rows selected `19200` PIDs, below the 25k cap.
- The smoke handoff summary still reports `requires_remote_heap_resolution` for
  full-row handoff. The production-read suite verifies the compact remote
  heap-candidate path with `id` projection.
- NDCG was not emitted by the current `spire-pipeline` suite step.
