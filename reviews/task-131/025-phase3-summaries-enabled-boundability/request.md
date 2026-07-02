# Task 131 Review Request: Phase 3 Summaries-Enabled Boundability

This packet reruns the Phase 3 boundability cell with leaf-block summaries actually materialized on remote leaf objects. It is not a closeout and does not implement worker early-stop. It answers the immediate reviewer question: when summaries are enabled in the local four-instance surface, are sound bounds available and selective enough to justify the next gated early-stop A/B?

Artifacts are under `reviews/task-131/025-phase3-summaries-enabled-boundability/artifacts/`.

## What Changed

- Committed `c1ea446e26e17c822f1898754dd5753cf64af491`:
  - factors remote materialization summary building through `remote_leaf_materialization_block_summaries`;
  - adds direct unit coverage for summaries disabled/enabled materialization rows;
  - records per-node remote materialization wall-time in the local multinode fixture.
- Reran a fresh two-lane 10k `n128/b4` suite:
  - `t131ctl66`: no summaries control;
  - `t131mat66`: `ec_spire.leaf_block_rows=64` through `PGOPTIONS` and load/materialization session GUC.

## Result

Bounds are available with summaries materialized, and the diagnostic global-threshold profile finds modest skip potential:

| lane | recall@k | p50 | p95 | sound bounds | threshold rows skipped |
| --- | ---: | ---: | ---: | ---: | ---: |
| no summaries | 1.0000 | 634.484 ms | 697.292 ms | 0 available / 1920 missing | 0 |
| summaries on | 1.0000 | 603.082 ms | 686.544 ms | 1920 available / 0 missing | 40489 / 754126 = 5.37% |

The threshold used here is the production candidate-derived global compact-candidate kth threshold, not worker-local kth. See `artifacts/manifest.md`.

Materialization and storage costs are now packet-local:

- materialization wall-time, control: node 2 `2499 ms`, node 3 `2960 ms`, node 4 `2943 ms`;
- materialization wall-time, summaries: node 2 `13187 ms`, node 3 `16607 ms`, node 4 `15543 ms`;
- remote index storage delta: about `+1.7` to `+1.9 MiB` per remote node, or `+217.6` to `+235.8 B/row`.

## Interpretation

This unblocks the Phase 3 viability decision but does not settle it. The bound source is real and complete on the summaries-on surface. The selectivity ceiling at this scale is only about 5% of remote rows, so the next A/B should be narrow and default-off: implement the first recall-safe scan-time early-stop gate using the existing candidate-derived global kth seed, then measure matched recall and latency with the gate on/off.

## Remaining Gaps

- Exact result identity vs no-summary control is still not proven. The run proves distributed-correctness and recall parity, but the benchmark artifacts do not emit returned ID lists for an identity diff.
- Phase 1 bytes-avoided and `100k n1024/b2` remain acknowledged gaps.
- Phase 2 normal-fixture scale latency remains unmeasured.
- Phase 4 metadata format/version/maintenance design remains deferred unless the early-stop A/B justifies promoting summary metadata.
