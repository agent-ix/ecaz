# Task 118 Packet 016 Artifact Manifest

- Task bucket: `reviews/task-118`
- Packet path: `reviews/task-118/016-current-head-10k-amd-diagnostics`
- Head SHA: `5ff394624d8dc8e465919f28bd78f3f0e622ab4c`
- Branch: `task-118-hnsw-quantized-recall-attribution`
- Timestamp: `2026-06-21T16:44:55-07:00`
- Host lane: AMD local development host
- Evidence status: current-head 10k diagnostic evidence only; final closeout
  still requires Intel 50k/100k evidence.

## Artifacts

### `suite-run-10k-frontier-current-head-amd.log`

- Lane: HNSW frontier containment diagnostic
- Scale: 10k
- Fixture: `data/staged-current/ec_real_10k_*`
- Storage formats: TurboQuant, PqFastScan, RaBitQ
- Build lanes: source-build and compressed-build
- Rerank mode: exact/source rerank over candidate frontier
- Isolated one-index-per-table surfaces: yes
- Command:

```bash
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database tqvector_bench --log-file reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/suite-run-10k-frontier-current-head-amd.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts --manifest-output reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/suite-manifest-10k-frontier-current-head-amd.json --results-output reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/results-10k-frontier-current-head-amd.jsonl --only frontier-10k-hnsw-turboquant --only frontier-10k-hnsw-pq-fastscan --only frontier-10k-hnsw-rabitq --only frontier-10k-hnsw-turboquant-compressed-build --only frontier-10k-hnsw-pq-fastscan-compressed-build --only frontier-10k-hnsw-rabitq-compressed-build --continue-on-error --allow-debug-backend
```

Key result lines from `results-10k-frontier-current-head-amd.jsonl`:

| step | truth@10 in frontier | truth@100 in frontier | frontier | exact rerank | dropped before exact |
| --- | ---: | ---: | ---: | ---: | ---: |
| `frontier-10k-hnsw-turboquant` | 0.9965 | 0.9545 | 200.0 | 200.0 | 0.0 |
| `frontier-10k-hnsw-pq-fastscan` | 0.9960 | 0.9543 | 200.0 | 200.0 | 0.0 |
| `frontier-10k-hnsw-rabitq` | 0.9705 | 0.9272 | 200.0 | 200.0 | 0.0 |
| `frontier-10k-hnsw-turboquant-compressed-build` | 0.9965 | 0.9545 | 200.0 | 200.0 | 0.0 |
| `frontier-10k-hnsw-pq-fastscan-compressed-build` | 0.9960 | 0.9543 | 200.0 | 200.0 | 0.0 |
| `frontier-10k-hnsw-rabitq-compressed-build` | 0.9705 | 0.9272 | 200.0 | 200.0 | 0.0 |

### `suite-run-10k-score-current-head-amd.log`

- Lane: HNSW score-correlation diagnostic
- Scale: 10k
- Fixture: `data/staged-current/ec_real_10k_*`
- Storage formats: TurboQuant, PqFastScan, RaBitQ
- Build lanes: source-build and compressed-build
- Rerank mode: exact/source rerank over candidate frontier
- Isolated one-index-per-table surfaces: yes
- Command:

```bash
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database tqvector_bench --log-file reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/suite-run-10k-score-current-head-amd.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts --manifest-output reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/suite-manifest-10k-score-current-head-amd.json --results-output reviews/task-118/016-current-head-10k-amd-diagnostics/artifacts/results-10k-score-current-head-amd.jsonl --only score-correlation-10k-hnsw-turboquant --only score-correlation-10k-hnsw-pq-fastscan --only score-correlation-10k-hnsw-rabitq --only score-correlation-10k-hnsw-turboquant-compressed-build --only score-correlation-10k-hnsw-pq-fastscan-compressed-build --only score-correlation-10k-hnsw-rabitq-compressed-build --continue-on-error --allow-debug-backend
```

Key result lines from `results-10k-score-current-head-amd.jsonl`:

| step | mean Spearman | mean abs rank shift | max abs rank shift | exact best approx rank | exact top4 max approx rank | missing cmp |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `score-correlation-10k-hnsw-turboquant` | 0.8404 | 22.79 | 175 | 1.4 | 6.1 | 0.0 |
| `score-correlation-10k-hnsw-pq-fastscan` | 0.8404 | 22.78 | 175 | 1.4 | 6.1 | 0.0 |
| `score-correlation-10k-hnsw-rabitq` | 0.9086 | 16.85 | 165 | 1.3 | 4.9 | 0.0 |
| `score-correlation-10k-hnsw-turboquant-compressed-build` | 0.8404 | 22.79 | 175 | 1.4 | 6.1 | 0.0 |
| `score-correlation-10k-hnsw-pq-fastscan-compressed-build` | 0.8404 | 22.78 | 175 | 1.4 | 6.1 | 0.0 |
| `score-correlation-10k-hnsw-rabitq-compressed-build` | 0.9086 | 16.85 | 165 | 1.3 | 4.9 | 0.0 |

## Committable Artifacts

Commit:

- `request.md`
- `artifacts/manifest.md`
- `artifacts/suite-run-10k-frontier-current-head-amd.log`
- `artifacts/suite-manifest-10k-frontier-current-head-amd.json`
- `artifacts/results-10k-frontier-current-head-amd.jsonl`
- `artifacts/frontier-10k-hnsw-{turboquant,pq-fastscan,rabitq}.log`
- `artifacts/frontier-10k-hnsw-{turboquant,pq-fastscan,rabitq}-compressed-build.log`
- `artifacts/suite-run-10k-score-current-head-amd.log`
- `artifacts/suite-manifest-10k-score-current-head-amd.json`
- `artifacts/results-10k-score-current-head-amd.jsonl`
- `artifacts/score-correlation-10k-hnsw-{turboquant,pq-fastscan,rabitq}.log`
- `artifacts/score-correlation-10k-hnsw-{turboquant,pq-fastscan,rabitq}-compressed-build.log`

Do not commit:

- raw per-query `artifacts/frontier-*.jsonl`
- raw per-query `artifacts/score-correlation-*.jsonl`
- truth caches, corpus TSV files, scratch restart logs, or operational exhaust
