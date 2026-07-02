# Task 120 Phase 5 AWS Distributed Rerank Evidence

Please review this AWS distributed SPIRE measurement packet for Task 120 Phase 5.

## Scope

This packet records a coordinator + two-remote AWS representative run on the
990k/1M `qdrant-dbpedia-openai3-large-1536-1m` fixture using
`storage_format=rabitq`, `top_graph_search_list_size=96`, and
`local_store_count=1`.

The code checkpoint immediately before this packet is:

- `393688cc2` `Harden SPIRE AWS representative resume paths`

## Result

The distributed setup reached a real read-ready state:

- coordinator loaded/indexed 990,000 rows;
- remote node 2 loaded/indexed 504,734 rows;
- remote node 3 loaded/indexed 485,266 rows;
- static remote placements published for 995 leaves across 2 remote nodes;
- distributed smoke used `EcSpireDistributedScan` with `remote_fanout: 2`;
- production read profile reported `status=ready`, `result_source=remote_heap_candidates`, `final_heap_fetch_status=remote_ready`, and `next_blocker=none`.

Measured evidence captured:

- storage: total `16.1 GiB`, ec_spire index `784.9 MiB`, `831.4 B` per row;
- recall@10 over 1,000 queries:
  - nprobe64: `0.9646`, NDCG@10 `0.9986`, mean query time `199.21 ms`;
  - nprobe96: `0.9716`, NDCG@10 `0.9991`, mean query time `233.64 ms`;
- nprobe96 bounded latency:
  - c1: p50 `234.6 ms`, p95 `259.2 ms`, p99 `286.2 ms`;
  - c4: p50 `239.2 ms`, p95 `307.3 ms`, p99 `334.8 ms`.

The final production-read finish-suite step did not complete: after c4 latency,
the suite manifest remained pending on `production-read-k10-default-nprobe96-finish`
without a visible `spire-pipeline` child or output artifact. I stopped that
runner and tore AWS down instead of continuing to burn resources. The earlier
smoke/profile artifact still proves the distributed read path reached
remote-heap-ready behavior, but this packet should be treated as partial Phase 5
evidence, not Task 120 closeout.

AWS teardown completed and post-teardown state preflight passed.

## Evidence

- Artifact manifest:
  `reviews/task-120/015-phase5-aws-distributed-rerank/artifacts/manifest.md`
- Suite configs/manifests:
  `artifacts/suite-task120-phase5-aws.json`
  `artifacts/suite-task120-phase5-aws-finish.json`
  `artifacts/suite-manifest-representative-priority.json`
  `artifacts/suite-manifest-phase5-finish.json`
- Load/index:
  `artifacts/coordinator-load-retry1.log`
  `artifacts/remote-node-2-load-representative.log`
  `artifacts/remote-node-3-load-representative.log`
- Distributed registration/smoke:
  `artifacts/publish-remote-placements.log`
  `artifacts/smoke-customscan-read.log`
  `artifacts/production-read-profile-smoke.log`
  `artifacts/bench-spire-pipeline-smoke.log`
- Measurements:
  `artifacts/storage-1m-rabitq.log`
  `artifacts/recall-k10-default-nprobe64-96.log`
  `artifacts/latency-k10-c1-default-nprobe96-finish.log`
  `artifacts/latency-k10-c4-default-nprobe96-finish.log`
- Teardown verification:
  `artifacts/preflight-state-after-teardown.log`

## Review Notes

This is not a request to close Task 120. It is a Phase 5 AWS evidence packet plus
the harness resume fixes needed to salvage the already-provisioned run.
