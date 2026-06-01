# Task 79 Packet 011 Artifact Manifest

- head SHA at measurement time: `07c5e6a585316719f7f6a8079d401eca7ce787f6`
- code commit measured: `b27202e08d02dda7fee8f81dd9f81d83e5c86a8f`
- task bucket: `reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/`
- packet type: benchmark evidence for RaBitQ leaf block pruning
- lane / fixture / storage format / rerank mode: intel-local, PG18, `ec_real_100k`, `ec_spire`, RaBitQ, `nlists=128`, `recursive_fanout=8`, `nprobe=96`, `top_graph_search_list_size=96`, `rerank_width=25`, `boundary_replica_count=0`
- isolated one-index-per-table or shared-table surface: shared table `task79_surface_100k_corpus`; one active index `task79_surface_100k_idx` rebuilt in place for each block size
- timestamp: 2026-06-01T23:52:46Z

## Commands

### Suite Validation

- `jq empty reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/suite-rabitq-leaf-block-pruning.json`
- `target/debug/ecaz bench suite audit --config reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/suite-rabitq-leaf-block-pruning.json`
- `target/debug/ecaz bench suite run --dry-run --config reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/suite-rabitq-leaf-block-pruning.json --manifest-output reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/artifacts/suite-dry-run-manifest.json`

### Build And Install

- `script -q -c 'cargo build -p ecaz-cli' reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c 'target/debug/ecaz dev install ecaz-pg-test --pg 18' reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/artifacts/install-ecaz-pg18.log`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/artifacts/pg18-restart.log restart -m fast`

### Final Suite Run

- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/suite-rabitq-leaf-block-pruning.json`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/artifacts/suite-manifest.json`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/011-rabitq-leaf-block-pruning-benchmark/artifacts/report-results.jsonl`

The final suite manifest records 15 selected steps, 15 succeeded, 0 failed.
Two earlier suite run attempts failed before completion because the suite command
omitted top-level database/socket arguments; the final command above is the
successful run cited by this packet.

## Artifacts

- `suite-rabitq-leaf-block-pruning.json`: checked-in `ecaz bench suite` config.
- `artifacts/suite-dry-run-manifest.json`: dry-run manifest proving command expansion and `PGOPTIONS` for the pruning GUC.
- `artifacts/suite-manifest.json`: final successful suite manifest.
- `artifacts/results.jsonl`: raw suite results, 198 rows.
- `artifacts/report-results.jsonl`: report output, 198 rows.
- `artifacts/precheck-existing-task79-surface.log`: PG18 surface precheck.
- `artifacts/cargo-build-ecaz-cli.log`: `ecaz-cli` build log.
- `artifacts/install-ecaz-pg18.log`: PG18 extension install log.
- `artifacts/pg18-restart.log`: PG18 restart log after install.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block64.log`: RaBitQ V3 index rebuild with `ec_spire.leaf_block_rows=64`.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32.log`: RaBitQ V3 index rebuild with `ec_spire.leaf_block_rows=32`.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block128.log`: RaBitQ V3 index rebuild with `ec_spire.leaf_block_rows=128`.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-prune0.log` and matching `funnel-*.jsonl`: V3 no-prune baseline.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-prune2.log` and matching `funnel-*.jsonl`: 64-row blocks, top 2 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-prune3.log` and matching `funnel-*.jsonl`: 64-row blocks, top 3 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-prune4.log` and matching `funnel-*.jsonl`: 64-row blocks, top 4 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-prune6.log` and matching `funnel-*.jsonl`: 64-row blocks, top 6 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block32-prune6.log` and matching `funnel-*.jsonl`: 32-row blocks, top 6 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block32-prune8.log` and matching `funnel-*.jsonl`: 32-row blocks, top 8 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block32-prune10.log` and matching `funnel-*.jsonl`: 32-row blocks, top 10 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block128-prune2.log` and matching `funnel-*.jsonl`: 128-row blocks, top 2 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block128-prune3.log` and matching `funnel-*.jsonl`: 128-row blocks, top 3 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block128-prune4.log` and matching `funnel-*.jsonl`: 128-row blocks, top 4 blocks per leaf.

## Key Results

| Step | Block rows | Blocks per leaf | Candidate sum | p50 latency | p95 latency | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| block64 prune0 | 64 | 0 | 15,506,227 | 62.907 ms | 71.058 ms | 0.9975 |
| block64 prune2 | 64 | 2 | 2,236,837 | 31.361 ms | 37.082 ms | 0.6075 |
| block64 prune3 | 64 | 3 | 3,408,906 | 33.926 ms | 36.969 ms | 0.7095 |
| block64 prune4 | 64 | 4 | 4,547,347 | 37.292 ms | 43.306 ms | 0.7790 |
| block64 prune6 | 64 | 6 | 6,760,069 | 42.130 ms | 46.094 ms | 0.8685 |
| block32 prune6 | 32 | 6 | 3,523,523 | 35.352 ms | 40.180 ms | 0.7850 |
| block32 prune8 | 32 | 8 | 4,679,339 | 38.159 ms | 43.018 ms | 0.8405 |
| block32 prune10 | 32 | 10 | 5,821,849 | 40.774 ms | 44.325 ms | 0.8820 |
| block128 prune2 | 128 | 2 | 4,248,302 | 35.613 ms | 41.697 ms | 0.6740 |
| block128 prune3 | 128 | 3 | 6,436,616 | 40.860 ms | 45.935 ms | 0.8110 |
| block128 prune4 | 128 | 4 | 8,464,459 | 45.882 ms | 52.635 ms | 0.8835 |

## Interpretation

The implementation proves that leaf block pruning can mechanically reduce the
candidate surface and p50 latency in the primary RaBitQ lane. It does not meet
the Task 79 recall gate. The V3 no-prune baseline preserves the expected
15,506,227 candidate surface and 0.9975 recall@10, while every pruned
mean-summary operating point loses too much recall.

This rejects the mean-summary top-N block selector as the Task 79 solution. The
next slice should improve block selection quality, likely by scoring a
recall-preserving upper bound or multiple representatives per block rather than
a single encoded centroid mean.
