# Artifact Manifest

- head SHA: `dd9626447b8f4316052de97f8024253c24a5f36c`
- task bucket: `reviews/task-63/008-hnsw-rabitq-binary-search-code/`
- lane: HNSW RaBitQ storage format
- fixture/storage format/rerank mode: compile validation plus local 10k HNSW
  smoke for `turboquant`, `pq_fastscan`, and `rabitq`
- timestamp: 2026-05-26 America/Los_Angeles

## Artifacts

### `cargo-check-lib.log`

- command: `cargo check -q --lib`
- result: passed; log is empty because `-q` emitted no warnings or errors

### `cargo-test-hnsw-no-run.log`

- command: `cargo test -q --lib hnsw --no-run`
- result: passed compile/no-run validation
- key result: command exited 0
- notes: log contains pre-existing unused/unsafe warnings

### `cargo-test-rabitq-binary-search-code-runtime.log`

- command: `cargo test -q --lib rabitq_flush_output_uses_binary_search_codes_and_scalar_rerank`
- result: local runtime execution failed before test body with dynamic symbol error
- key result: `undefined symbol: LockBuffer`
- notes: this is the same local pgrx-linked runtime limitation seen in adjacent HNSW work; use the no-run compile result plus PG18 SQL packet coverage for review context

### `local-10k-bin1/suite.json`

- command source: checked-in `ecaz bench suite` config copied from
  `target/task63-local-10k-bin1-suite.json`
- lane / fixture / storage format / rerank mode: HNSW only, DBpedia-derived
  `ec_real_10k`, storage formats `turboquant`, `pq_fastscan`, `rabitq`;
  default scalar rerank path
- isolated/shared surface: isolated one-index-per-table prefixes
  `task63_local_10k_bin1_hnsw_*`
- note: local smoke/tuning evidence only; not final Task 63 acceptance

### `local-10k-bin1/suite-manifest.json`

- command:
  `target/debug/ecaz bench suite run --config target/task63-local-10k-bin1-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output target/task63-local-10k-bin1/suite-manifest.json --results-output target/task63-local-10k-bin1/results.jsonl --log-file target/task63-local-10k-bin1/suite-run.log`
- result: passed; 14/14 steps succeeded
- timestamp: 2026-05-26 21:22 America/Los_Angeles
- head SHA: `9526aea0a7da2661829d0ba40fa1b3cca222a032`
- host context: PostgreSQL 18.3, socket host `/home/peter/.pgrx`, port 28818;
  `shared_buffers=128MB`, `work_mem=4MB`, `maintenance_work_mem=64MB`,
  `effective_cache_size=4GB`

### `local-10k-bin1/results.jsonl`

- raw suite parser output from the run command above
- key load result lines:
  - build index seconds: `turboquant` 97.93, `pq_fastscan` 109.60,
    `rabitq` 99.14
- key recall@10 result lines:
  - ef_search 40: `turboquant` 0.8845, `pq_fastscan` 0.8945,
    `rabitq` 0.8135
  - ef_search 100: `turboquant` 0.9445, `pq_fastscan` 0.9635,
    `rabitq` 0.9205
  - ef_search 200: `turboquant` 0.9700, `pq_fastscan` 0.9940,
    `rabitq` 0.9365
- key latency p50 result lines:
  - ef_search 40: `turboquant` 15.3 ms, `pq_fastscan` 19.6 ms,
    `rabitq` 42.4 ms
  - ef_search 100: `turboquant` 24.6 ms, `pq_fastscan` 32.3 ms,
    `rabitq` 89.0 ms
  - ef_search 200: `turboquant` 38.2 ms, `pq_fastscan` 44.0 ms,
    `rabitq` 156.7 ms
- key HNSW index storage result lines:
  - `turboquant`: 13.0 MiB, 1366.4 B/row
  - `pq_fastscan`: 13.1 MiB, 1377.9 B/row
  - `rabitq`: 13.0 MiB, 1366.4 B/row

### `local-10k-bin1/results-report.jsonl` and `local-10k-bin1/suite-report.log`

- command:
  `target/debug/ecaz bench suite report --manifest target/task63-local-10k-bin1/suite-manifest.json --results-output target/task63-local-10k-bin1/results-report.jsonl --log-file target/task63-local-10k-bin1/suite-report.log`
- result: passed
- key result: report summarizes completed 14, failed 0, skipped 0, dry-run 0,
  missing artifacts 0, stale 0

### `local-10k-bin1/*.log`

- packet-local raw load, recall, latency, storage, host precheck, audit, run,
  and report logs for the local 10k HNSW smoke
- note: artifacts preserve target paths inside suite output because they are
  emitted by `ecaz bench suite`; packet copies are the durable review evidence
