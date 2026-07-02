# Task 120 Phase 5 AWS Distributed Rerank Manifest

- Head SHA: `393688cc2c58adbb38fb7fe83e767ec7645130b8`
- Task bucket: `reviews/task-120/015-phase5-aws-distributed-rerank`
- Timestamp: `2026-06-22T05:11:41Z`
- Lane: AWS Graviton SPIRE representative run
- Fixture: `qdrant-dbpedia-openai3-large-1536-1m`
- Prepared corpus prefix: `ec_real_ann_benchmarks_anchor`
- Benchmark prefix: `ec_spire_aws_repr_1m`
- Storage format: `rabitq`
- Index reloptions: `top_graph_search_list_size=96;local_store_count=1`
- Surface: distributed SPIRE with coordinator plus two remote nodes; isolated remote tables/indexes materialized from static placement plan.

## Commands

- Build operator: `cargo build --release --bin ecaz --package ecaz-cli`
- Main AWS run/resumes: `scripts/spire-aws/with-ssm-port-forwards.sh ... -- make -C infra/spire-aws ... load-representative register-representative smoke-representative bench-representative-priority`
- Finish suite: `scripts/spire-aws/with-ssm-port-forwards.sh ... -- target/release/ecaz bench suite run --config artifacts/suite-task120-phase5-aws-finish.json --manifest-output artifacts/suite-manifest-phase5-finish.json --results-output artifacts/suite-results-phase5-finish.jsonl`
- Teardown: `env SPIRE_AWS_ALLOW_NONDEFAULT_GRAVITON_LANE=1 make -C infra/spire-aws ARTIFACT_DIR=... teardown`
- Teardown verification: `env SPIRE_AWS_ALLOW_NONDEFAULT_GRAVITON_LANE=1 make -C infra/spire-aws ARTIFACT_DIR=... preflight-state`

## Key Results

- Coordinator load/index completed for 990,000 corpus rows and 10,000 queries; coordinator RABITQ index build took `2701.75s`, total prefix load completed in `2828.33s` (`coordinator-load-retry1.log`).
- Remote node 2 loaded 504,734 rows, encoded in `219.20s`, built the remote RABITQ index in `1012.44s`, total prefix load `1457.74s` (`remote-node-2-load-representative.log`).
- Remote node 3 loaded 485,266 rows, encoded in `212.95s`, built the remote RABITQ index in `954.86s`, total prefix load `1386.23s` (`remote-node-3-load-representative.log`).
- Static remote placement publish reported `1 995 2 published_static_remote_placements` (`publish-remote-placements.log`).
- Distributed smoke used `EcSpireDistributedScan`, `remote_fanout: 2`, tuple transport `ready`, and production read profile reported `status=ready`, `result_source=remote_heap_candidates`, `final_heap_fetch_status=remote_ready`, `next_blocker=none`, `remote_heap_candidate_count=20`, `returned_candidate_count=10`, `total_elapsed_ms=89`/`105` across smoke invocations (`smoke-customscan-read.log`, `production-read-profile-smoke.log`, `aws-phase5-suite-run-resume3.log`).
- Storage for coordinator 990,000-row surface: total `16.1 GiB`, ec_spire index `784.9 MiB`, index per row `831.4 B` (`storage-1m-rabitq.log`, `storage-1m-rabitq-finish.log`).
- 1,000-query recall@10 after exact truth generation: nprobe64 `0.9646` (CI95 `0.9608..0.9680`, NDCG@10 `0.9986`, mean query time `199.21 ms`); nprobe96 `0.9716` (CI95 `0.9682..0.9747`, NDCG@10 `0.9991`, mean query time `233.64 ms`) (`recall-k10-default-nprobe64-96.log`).
- Finish latency at nprobe96: c1 p50 `234.6 ms`, p95 `259.2 ms`, p99 `286.2 ms`; c4 p50 `239.2 ms`, p95 `307.3 ms`, p99 `334.8 ms` (`latency-k10-c1-default-nprobe96-finish.log`, `latency-k10-c4-default-nprobe96-finish.log`).
- AWS teardown completed; post-teardown state preflight passed with no managed resources (`preflight-state-after-teardown.log`).

## Incomplete/Pruned Evidence

- The original 13-step representative priority suite was intentionally interrupted after storage and the first 1,000-query recall step, because continuing the full matrix would have kept the AWS cluster running too long.
- The bounded finish suite completed storage and c1/c4 latency, then the suite runner remained pending on `production-read-k10-default-nprobe96-finish` without a visible `spire-pipeline` child or output artifact. The runner was interrupted and AWS was torn down.
- Banned/generated files were pruned before commit: prepared corpus/query TSVs, truth-cache JSON, SSM tunnel state/logs, TLS private material, remote coordinator assignment TSVs, and generated row-id lists.

## Artifacts

- `suite-task120-phase5-aws.json`: original packet-local Phase 5 suite config.
- `suite-task120-phase5-aws-finish.json`: bounded finish suite config.
- `suite-representative-priority.json`, `render-check/suite-representative-priority.json`: rendered representative priority suite with prepared corpus/truth paths.
- `suite-manifest-representative-priority.json`: suite manifest showing storage and first recall succeeded before interruption.
- `suite-manifest-phase5-finish.json`: finish suite manifest showing storage/c1/c4 latency succeeded and production-read step remained pending.
- `storage-1m-rabitq.log`, `storage-1m-rabitq-finish.log`: storage outputs.
- `recall-k10-default-nprobe64-96.log`: recall output.
- `latency-k10-c1-default-nprobe96-finish.log`, `latency-k10-c4-default-nprobe96-finish.log`: bounded latency outputs.
- `smoke-customscan-read.log`, `production-read-profile-smoke.log`, `bench-spire-pipeline-smoke.log`: distributed read smoke/profile outputs.
- `coordinator-load-retry1.log`, `remote-node-2-load-representative.log`, `remote-node-3-load-representative.log`: successful load/index logs.
- `remote-node-2-inspect-representative.log`, `remote-node-3-inspect-representative.log`: remote inspect outputs.
- `distributed-representative/distributed-placement-config.json`, `distributed-representative/distributed-placement-plan.json`, `distributed-representative/remotes.jsonl`: static placement metadata.
- `remote-leaf-materialization/*`: rendered materialization SQL, observed/required leaf lists, and materialization logs; assignment TSVs were pruned.
- `remote-identities/*`: remote endpoint identity JSON.
- `publish-remote-placements.log`, `remote-node-snapshot-baseline.log`, `coordinator-placement-snapshot-after-remote-publish.log`: remote registration/publish evidence.
- `aws-topology.json`: original topology for the destroyed AWS run.
- `preflight-phase5.log`, `preflight-phase5-custom-suite.log`, `suite-audit.log`, `render-check.log`: setup/preflight/audit evidence.
- `preflight-state-after-teardown.log`: post-teardown verification.
- Remaining install/upload/package logs are retained as supporting operational evidence for the AWS run; generated payloads and private material were pruned.
