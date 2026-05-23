# Task 51 Packet 022 Artifacts

- head SHA: `4235b7ba12965359453c8229c0bdfa2b651ddf40`
- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/022-sidecar-concurrency-smoke/`
- timestamp: `2026-05-23T17:03:36Z`
- lane: local PG18 / WSL2
- fixture: `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- storage format: IVF/RaBitQ, `quant_bits=1`, `rerank=off`
- rerank mode: sidecar measurement harness, variants `f16` and `rabitq8`
- read modes: `random-id`, `tid-sorted`
- concurrency: `4`
- isolated one-index-per-table surface: yes, inherited preserved local fixture

## Durable Evidence

Benchmark packet:

- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/manifest.md`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/suite.json`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/results.jsonl`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/results-report.jsonl`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/cargo-test-sidecar.log`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/cargo-build-ecaz-cli.log`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-audit.log`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-dry-run.log`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-run.log`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-status.log`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-report.log`
- `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/sidecar-concurrency-c4-50k-rabitq1-n128-k50.log`

## Key Lines Cited By Request

```text
[suite:task51-local-ivf-sidecar-concurrency-smoke] completed=1 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 354 filtered out; finished in 0.00s
```

q=20, k=10, candidate_k=50, nprobe=96, concurrency=4:

```text
f16 random-id:  recall@10 1.0000, sidecar_io_p50 34.653 ms, sidecar_p50 39.615 ms
f16 tid-sorted: recall@10 1.0000, sidecar_io_p50 18.743 ms, sidecar_p50 23.733 ms
rabitq8 random-id:  recall@10 0.9450, sidecar_io_p50 26.470 ms, sidecar_p50 27.516 ms
rabitq8 tid-sorted: recall@10 0.9450, sidecar_io_p50 4.419 ms, sidecar_p50 5.552 ms
```
