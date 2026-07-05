# Task 131 Packet 025 Artifact Manifest

- head SHA: `c1ea446e26e17c822f1898754dd5753cf64af491`
- task bucket: `reviews/task-131`
- packet: `reviews/task-131/025-phase3-summaries-enabled-boundability`
- lane: Phase 3 summaries-enabled boundability, local four-instance PG18
- fixture: staged DBPedia `ec_real_10k`, `n128/b4`, `rabitq`, `nprobe=96`, `top_k=10`, 20 queries
- storage surface: coordinator index plus restarted remote-node storage probes
- early-stop state: OFF; this packet is diagnostic-only and does not implement worker early-stop

## Code And Test Evidence

- code commit: `c1ea446e26e17c822f1898754dd5753cf64af491` (`task 131 test materialized leaf summaries`)
- `cargo test -p ecaz --lib remote_leaf_materialization_summaries`
  - result: passed, 2 tests
  - coverage: summaries disabled keeps the materialization row shape unchanged; summaries enabled produces two RaBitQ summary blocks with expected row ranges.
- `cargo test -p ecaz-cli spire_local_multinode_step_expands_local_four_instance_lane`
  - result: passed, 1 test
- `cargo build -p ecaz-cli`
  - result: passed
  - warning: pre-existing `LoadedDistributedPlacementConfig.path` dead-code warning.

Storage layer note for the GUC-off path: `SpireLocalObjectStore::insert_leaf_object_v3_from_rows_and_summaries` delegates to `insert_leaf_object_v2_from_rows` when `summaries.is_empty()`, so disabled summaries retain the V2 object path.

## Suite Config And Run

- config: `artifacts/task131-phase3-summaries-enabled-boundability-suite.json`
- command:
  - `target/debug/ecaz bench suite run --config reviews/task-131/025-phase3-summaries-enabled-boundability/artifacts/task131-phase3-summaries-enabled-boundability-suite.json`
- top-level generated files:
  - `artifacts/suite-manifest.json`
  - `artifacts/results.jsonl` (created but empty for this nested multinode suite; detailed result rows are in each lane's `bench-suite/results.jsonl`)
- generated TSV corpus shards were deleted before commit per repository packet rules.

## Lanes

### No-Summaries Control

- artifact dir: `artifacts/10k-n128-b4-no-summaries-control-v2`
- run id: `t131ctl66`
- ports: coordinator 40820, remotes 40821/40822/40823
- index names:
  - coordinator: `t131_p3_bound_q20_10k_n128_b4_ctl66_coord_idx`
  - remote: `t131_p3_bound_q20_10k_n128_b4_ctl66_remote_idx`
- summary GUC: not set
- key artifacts:
  - `artifacts/10k-n128-b4-no-summaries-control-v2/bench-suite/results.jsonl`
  - `artifacts/10k-n128-b4-no-summaries-control-v2/bench-suite/production-read-k10-baseline-default.log`
  - `artifacts/10k-n128-b4-no-summaries-control-v2/remote-leaf-materialization/*-timing.log`

Key result lines:

- recall/latency: recall@k `1.0000`; p50 `634.484 ms`; p95 `697.292 ms`; p99 `844.670 ms`; min `555.172 ms`; max `844.670 ms`.
- coordinator storage: index `40.9 MiB` / `42886758` bytes; total `199.7 MiB` / `209400627` bytes.
- profile aggregate: status `ready`; result source `remote_heap_candidates`; selected PID sum `1920`; returned sum `200`; strict/timeout/cancel/degraded skips all `0`.
- threshold profile totals:
  - sound bounds available `0`; sound bounds missing `1920`
  - threshold blocks available/skipped `0 / 0`
  - threshold rows available/skipped `0 / 0`

Materialization wall-time:

- node 2: `2499 ms`
- node 3: `2960 ms`
- node 4: `2943 ms`

### Summaries-On

- artifact dir: `artifacts/10k-n128-b4-leaf-block-rows64-materialized-v3`
- run id: `t131mat66`
- ports: coordinator 40830, remotes 40831/40832/40833
- index names:
  - coordinator: `t131_p3_bound_q20_10k_n128_b4_mat66_coord_idx`
  - remote: `t131_p3_bound_q20_10k_n128_b4_mat66_remote_idx`
- summary GUC:
  - suite `pgoptions`: `-c ec_spire.leaf_block_rows=64`
  - load/materialization session GUC: `ec_spire.leaf_block_rows=64`
- key artifacts:
  - `artifacts/10k-n128-b4-leaf-block-rows64-materialized-v3/bench-suite/results.jsonl`
  - `artifacts/10k-n128-b4-leaf-block-rows64-materialized-v3/bench-suite/production-read-k10-baseline-default.log`
  - `artifacts/10k-n128-b4-leaf-block-rows64-materialized-v3/remote-leaf-materialization/*-rendered.sql`
  - `artifacts/10k-n128-b4-leaf-block-rows64-materialized-v3/remote-leaf-materialization/*-timing.log`

GUC delivery evidence:

- each remote materialization log begins with `SET`
- row counts:
  - node 2: `COPY 15179`, `SELECT 15179`, materialized `43` leaves
  - node 3: `COPY 17996`, `SELECT 17996`, materialized `43` leaves
  - node 4: `COPY 16825`, `SELECT 16825`, materialized `42` leaves

Key result lines:

- recall/latency: recall@k `1.0000`; p50 `603.082 ms`; p95 `686.544 ms`; p99 `931.784 ms`; min `564.171 ms`; max `931.784 ms`.
- coordinator storage: index `42.4 MiB` / `44459622` bytes; total `201.2 MiB` / `210973491` bytes.
- profile aggregate: status `ready`; result source `remote_heap_candidates`; selected PID sum `1920`; returned sum `200`; strict/timeout/cancel/degraded skips all `0`.
- threshold profile totals:
  - sound bounds available `1920`; sound bounds missing `0`
  - threshold blocks available/skipped `12758 / 797` = `6.25%`
  - threshold rows available/skipped `754126 / 40489` = `5.37%`
  - leaf summary score nanos total `297683949`

Per-node threshold profile:

| node | sound available | missing | blocks avail | blocks skipped | rows avail | rows skipped |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 688 | 0 | 3963 | 234 | 231377 | 12407 |
| 3 | 633 | 0 | 4582 | 284 | 271907 | 14007 |
| 4 | 599 | 0 | 4213 | 279 | 250842 | 14075 |

Materialization wall-time:

- node 2: `13187 ms`
- node 3: `16607 ms`
- node 4: `15543 ms`

## Remote Storage Delta

The built-in suite storage step reports coordinator storage. Because materialized leaf summaries live on the remote nodes, this packet restarted the stopped remote PG data directories briefly and ran:

- `target/debug/ecaz --database postgres --host /home/peter/dev/ecaz/target/spire-local-multinode-sockets-t131ctl66 --port 40821 --user ecaz_coord --log-file artifacts/remote-storage/ctl66-node2-storage.log bench storage --prefix ec_real_10k_node_2`
- same for ports `40822`, `40823`, `40831`, `40832`, and `40833`

Remote index storage:

| node | control index | summaries index | delta | control B/row | summaries B/row | delta B/row |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 45.6 MiB | 47.3 MiB | +1.7 MiB | 5823.6 | 6041.2 | +217.6 |
| 3 | 48.8 MiB | 50.6 MiB | +1.8 MiB | 6055.5 | 6284.4 | +228.9 |
| 4 | 47.3 MiB | 49.2 MiB | +1.9 MiB | 5977.3 | 6213.1 | +235.8 |

Remote total storage:

| node | control total | summaries total | delta |
| --- | ---: | ---: | ---: |
| 2 | 176.2 MiB | 177.9 MiB | +1.7 MiB |
| 3 | 183.1 MiB | 184.9 MiB | +1.8 MiB |
| 4 | 179.5 MiB | 181.3 MiB | +1.8 MiB |

Storage probe artifacts:

- `artifacts/remote-storage/ctl66-node2-storage.log`
- `artifacts/remote-storage/ctl66-node3-storage.log`
- `artifacts/remote-storage/ctl66-node4-storage.log`
- `artifacts/remote-storage/mat66-node2-storage.log`
- `artifacts/remote-storage/mat66-node3-storage.log`
- `artifacts/remote-storage/mat66-node4-storage.log`

## Threshold Source

The threshold profile is the production candidate-derived global compact-candidate kth threshold, not a worker-local kth. Code path:

- CLI calls `ec_spire_remote_search_production_candidate_threshold_profile` from `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`.
- The coordinator path derives `threshold_score` through `remote_search_production_global_candidate_threshold_score_result` and `global_compact_candidate_threshold_score` in `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs`.

## Identity And Remaining Gaps

- distributed-correctness artifacts are present for both fresh lanes:
  - `artifacts/10k-n128-b4-no-summaries-control-v2/distributed-placement-plan-correctness.path`
  - `artifacts/10k-n128-b4-leaf-block-rows64-materialized-v3/distributed-placement-plan-correctness.path`
- recall parity is present: both fresh lanes report recall@k `1.0000`.
- exact result identity vs control is not proven by this run because the benchmark logs aggregate query metrics but do not emit per-query returned ID lists. This remains open.

## Dead/Stale Lane Note

- A stale-binary failed lane named `10k-n128-b4-leaf-block-rows64-materialized` was generated before the successful rerun. It stopped after remote materialization because the previous `target/debug/ecaz` still rendered the materialization SQL placeholder instead of the GUC SQL. It had no `bench-suite/results.jsonl`, was not cited as a result, and was pruned before commit.
- An earlier successful summaries-on lane named `10k-n128-b4-leaf-block-rows64-materialized-v2` was generated after the materialization fix but before materialization timing capture. It was superseded by the committed `...-materialized-v3` lane and pruned before commit.
