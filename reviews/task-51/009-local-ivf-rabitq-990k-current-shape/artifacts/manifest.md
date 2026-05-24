# Artifact Manifest

- head SHA: `d72246e6cad5bab99e0889798fd75247978346a7`
- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/009-local-ivf-rabitq-990k-current-shape/`
- timestamp: `2026-05-23T08:01:32Z`
- benchmark packet: `benchmarks/task51-local-ivf-rabitq-990k/`
- SuiteConfig: `benchmarks/task51-local-ivf-rabitq-990k/suite.json`
- lane: local PG18 / WSL2
- fixture: staged 990k anchor corpus, 990000 corpus rows, 10000 query rows, dim 1536
- storage format: `rabitq`
- profile: `ec_ivf`
- rerank mode: `heap_f32`, width `50`
- table surface: isolated one-index-per-table prefix `task51_local_990k_ivf_rabitq1_n1024_w50`
- AWS: not used
- competitors: none; IVF/RaBitQ only

## Packet-Local Artifacts

- `suite-status.log`
  - command: `target/release/ecaz --log-file benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-manifest.json`
  - key line: `[suite:task51-local-ivf-rabitq-990k] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `suite-report.log`
  - command: `target/release/ecaz --log-file benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-rabitq-990k/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-rabitq-990k/artifacts/results.jsonl`
  - key lines:
    - load: total `2420.91s`, build index `1671.85s`
    - recall nprobe 128: recall@10 `0.9750`, mean q-time `561.32 ms`
    - recall nprobe 256: recall@10 `0.9850`, mean q-time `1103.46 ms`
    - latency nprobe 128: p50 `566.0 ms`, p95 `659.8 ms`
    - latency nprobe 256: p50 `1083.8 ms`, p95 `1197.9 ms`
    - storage index: `298.3 MiB`, `316.0 B` per row
- `diff-check.log`
  - command: `git diff --check`
  - key line: exited 0 with no output

## Full Benchmark Artifacts

The full raw logs live in `benchmarks/task51-local-ivf-rabitq-990k/artifacts/`:

- `suite-audit.log`
- `suite-dry-run.log`
- `suite-run.log`
- `suite-status.log`
- `suite-report.log`
- `suite-manifest.json`
- `results.jsonl`
- `load-990k-rabitq1-n1024-w50.log`
- `recall-990k-rabitq1-n1024-w50.log`
- `latency-990k-rabitq1-n1024-w50.log`
- `storage-990k-rabitq1-n1024-w50.log`
- `explain-990k-rabitq1-n1024-w50-p128.sql`
- `explain-990k-rabitq1-n1024-w50-p128.log`
- `explain-990k-rabitq1-n1024-w50-p256.sql`
- `explain-990k-rabitq1-n1024-w50-p256.log`
- `truth-ec-real-990k-q100-k10.json`

`*-debug-aborted.*` files under the benchmark artifacts record an aborted debug-binary preflight before database mutation. They are retained for provenance but are not the successful suite evidence.
