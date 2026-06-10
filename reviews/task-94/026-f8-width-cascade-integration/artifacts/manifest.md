# Task 94 Packet 026 Artifact Manifest

- head SHA: `11f8fc38113c08614c8ddca2073e54adcb018d81`
- task bucket: `reviews/task-94/026-f8-width-cascade-integration/`
- lane / fixture / storage: LUT lane / local PG18 / IVF PqFastScan rerank-off 10k, 25k, 100k plus forced DiskANN grouped-PQ 50k, 100k / `storage_format=pq_fastscan`
- host class: local Intel AVX2
- isolated one-index-per-table surfaces: yes for Task 94 IVF fixtures; DiskANN reused existing `task67_local_fullq_*_diskann` fixtures with `ec_diskann.prefilter_kind=grouped_pq`
- timestamp: 2026-06-10

## Test Artifacts

### `cargo-test-candidate-batch.log`

- command: `script -q -c "cargo test --lib candidate_batch" reviews/task-101/001-width-cascade-f8-integration/artifacts/cargo-test-candidate-batch.log`
- copied into this packet for Task 94 locality
- key result: `18 passed; 0 failed`

### `cargo-test-grouped-pq.log`

- command: `script -q -c "cargo test --lib grouped_pq" reviews/task-101/001-width-cascade-f8-integration/artifacts/cargo-test-grouped-pq.log`
- copied into this packet for Task 94 locality
- key result: `35 passed; 0 failed`; includes PG18 `pg_test_pq_fastscan_binary_score_mode_bypasses_grouped_pq_scoring`

## Suite Artifacts

### `task94-local-pqfastscan-matrix-suite.json`

- local `ecaz bench suite` config copied from packet 025 and retargeted to this packet's artifact dir
- steps: 14
- surfaces: IVF PqFastScan 10k/25k/100k recall+latency, forced DiskANN grouped-PQ 50k/100k latency

### `suite-audit-cli.log`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/026-f8-width-cascade-integration/artifacts/suite-audit-cli.log bench suite audit --config reviews/task-94/026-f8-width-cascade-integration/artifacts/task94-local-pqfastscan-matrix-suite.json`
- key result: `[suite:task94-local-pqfastscan-matrix] audit passed: 14 steps`

### `suite-run-cli.log`, `suite-manifest.json`, `results.jsonl`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/026-f8-width-cascade-integration/artifacts/suite-run-cli.log bench suite run --config reviews/task-94/026-f8-width-cascade-integration/artifacts/task94-local-pqfastscan-matrix-suite.json --artifact-dir reviews/task-94/026-f8-width-cascade-integration/artifacts --manifest-output reviews/task-94/026-f8-width-cascade-integration/artifacts/suite-manifest.json --results-output reviews/task-94/026-f8-width-cascade-integration/artifacts/results.jsonl`
- key result: suite completed and wrote 38 parsed result rows

### `suite-status-cli.log`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/026-f8-width-cascade-integration/artifacts/suite-status-cli.log bench suite status --manifest reviews/task-94/026-f8-width-cascade-integration/artifacts/suite-manifest.json`
- key result: `[suite:task94-local-pqfastscan-matrix] completed=14 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

### `suite-report-cli.log`, `results-report.jsonl`

- command: `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/026-f8-width-cascade-integration/artifacts/suite-report-cli.log bench suite report --manifest reviews/task-94/026-f8-width-cascade-integration/artifacts/suite-manifest.json --results-output reviews/task-94/026-f8-width-cascade-integration/artifacts/results-report.jsonl`
- key result: report emitted 38 parsed rows, including direct `block_kernel_counters`

## Key Result Lines

- Recall parity: IVF batch-off and batch-on recall rows match exactly at 10k, 25k, and 100k for both `nprobe=32` and `nprobe=64`.
- Direct counter coverage:
  - IVF 10k/25k/100k batch-on rows have `surface=ivf quant=grouped_pq isa=avx2 scalar_candidates=0`.
  - DiskANN 50k/100k forced grouped-PQ rows have `surface=diskann quant=grouped_pq isa=avx2 scalar_candidates=0`.
- Local latency: this run is not a speedup claim. IVF batch-on p50 was slower than batch-off at all measured local cells; DiskANN cells were forced grouped-PQ attribution runs without a paired batch-off comparison.
