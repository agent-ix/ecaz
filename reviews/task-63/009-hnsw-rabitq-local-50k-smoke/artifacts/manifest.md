# Artifact Manifest

- head SHA: `ba58dc7fb70cc7e743e3988f91d76555f1138374`
- task bucket: `reviews/task-63/009-hnsw-rabitq-local-50k-smoke/`
- lane: HNSW RaBitQ storage format
- fixture/storage format/rerank mode: local 50k HNSW smoke for
  `turboquant`, `pq_fastscan`, and `rabitq`; default scalar rerank path
- timestamp: 2026-05-26 America/Los_Angeles
- local scope: HNSW only, 50k only; not final Task 63 acceptance evidence

## Artifacts

### `local-50k-bin1/suite.json`

- command source: local `ecaz bench suite` config copied from
  `target/task63-local-50k-bin1-suite.json`
- lane / fixture / storage format / rerank mode: HNSW only, DBpedia-derived
  `ec_real_50k`, storage formats `turboquant`, `pq_fastscan`, `rabitq`;
  default scalar rerank path
- isolated/shared surface: isolated one-index-per-table prefixes
  `task63_local_50k_bin1_hnsw_*`
- note: local smoke/tuning evidence only; not final Task 63 acceptance

### `local-50k-bin1/suite-dry-run.log`

- command:
  `target/debug/ecaz bench suite run --config target/task63-local-50k-bin1-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --dry-run --manifest-output target/task63-local-50k-bin1/suite-manifest.json --results-output target/task63-local-50k-bin1/results.jsonl --log-file target/task63-local-50k-bin1/suite-dry-run.log`
- result: passed
- key result: dry run listed only 50k HNSW load, recall, latency, and storage
  steps; no 100k or non-HNSW benchmark steps

### `local-50k-bin1/suite-run.log` and `local-50k-bin1/suite-manifest.json`

- command:
  `target/debug/ecaz bench suite run --config target/task63-local-50k-bin1-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output target/task63-local-50k-bin1/suite-manifest.json --results-output target/task63-local-50k-bin1/results.jsonl --log-file target/task63-local-50k-bin1/suite-run.log`
- result: passed; 14/14 steps succeeded
- host context: PostgreSQL 18.3, socket host `/home/peter/.pgrx`, port 28818;
  `shared_buffers=128MB`, `work_mem=4MB`, `maintenance_work_mem=64MB`,
  `effective_cache_size=4GB`
- note: the successful run used escalated local execution after the sandbox
  blocked direct socket access during an earlier precheck attempt

### `local-50k-bin1/results.jsonl`

- raw suite parser output from the run command above
- key load result lines:
  - build index seconds: `turboquant` 897.12, `pq_fastscan` 934.31,
    `rabitq` 898.07
- key recall@10 result lines:
  - ef_search 40: `turboquant` 0.8700, `pq_fastscan` 0.8965,
    `rabitq` 0.7955
  - ef_search 100: `turboquant` 0.9155, `pq_fastscan` 0.9540,
    `rabitq` 0.8820
  - ef_search 200: `turboquant` 0.9315, `pq_fastscan` 0.9740,
    `rabitq` 0.9065
- key latency p50 result lines:
  - ef_search 40: `turboquant` 19.4 ms, `pq_fastscan` 24.6 ms,
    `rabitq` 48.6 ms
  - ef_search 100: `turboquant` 29.3 ms, `pq_fastscan` 36.4 ms,
    `rabitq` 88.2 ms
  - ef_search 200: `turboquant` 46.1 ms, `pq_fastscan` 53.2 ms,
    `rabitq` 157.0 ms
- key HNSW index storage result lines:
  - `turboquant`: 65.1 MiB, 1365.6 B/row
  - `pq_fastscan`: 65.2 MiB, 1368.1 B/row
  - `rabitq`: 65.1 MiB, 1365.6 B/row

### `local-50k-bin1/results-report.jsonl` and `local-50k-bin1/suite-report.log`

- command:
  `target/debug/ecaz bench suite report --manifest target/task63-local-50k-bin1/suite-manifest.json --results-output target/task63-local-50k-bin1/results-report.jsonl --log-file target/task63-local-50k-bin1/suite-report.log`
- result: passed
- key result: report summarizes completed 14, failed 0, skipped 0, dry-run 0,
  missing artifacts 0, stale 0

### `local-50k-bin1/*.log`

- packet-local raw load, recall, latency, storage, host precheck, dry-run, run,
  and report logs for the local 50k HNSW smoke
- note: artifacts preserve target paths inside suite output because they are
  emitted by `ecaz bench suite`; packet copies are the durable review evidence
