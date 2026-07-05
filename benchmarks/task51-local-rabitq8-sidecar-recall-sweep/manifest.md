# Task 51 Local RaBitQ8 Sidecar Recall Sweep

- head SHA: `4bb5e6015a96a81352a18b6c5d556db85d4e86d4`
- benchmark packet: `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/`
- SuiteConfig: `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/suite.json`
- suite manifest: `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-manifest.json`
- results: `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/results.jsonl`
- parsed report: `benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/results-report.jsonl`
- timestamp: 2026-05-23

## Surface

- local PG18 socket: `/home/peter/.pgrx`, port `28818`
- fixture: preserved isolated 50k prefix `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- index shape: one `ec_ivf` index, `storage_format=rabitq`, `rerank=off`, `nlists=128`, `nprobe=128`
- lane: IVF/RaBitQ only
- storage format: current `rabitq8` sidecar, `1548` bytes/vector, `73.81 MiB` at 50k rows
- rerank mode: `tid-sorted`
- isolated one-index-per-table surface: yes

## Commands

```sh
target/release/ecaz --log-file benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-rabitq8-sidecar-recall-sweep/suite.json
target/release/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-dry-run.log bench suite run --config benchmarks/task51-local-rabitq8-sidecar-recall-sweep/suite.json --dry-run --manifest-output benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-manifest.json
target/release/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-rabitq8-sidecar-recall-sweep/suite.json --manifest-output benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-manifest.json
target/release/ecaz --log-file benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-manifest.json
target/release/ecaz --log-file benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/suite-manifest.json --results-output benchmarks/task51-local-rabitq8-sidecar-recall-sweep/artifacts/results-report.jsonl
```

## Key Results

Increasing the IVF candidate frontier did not recover recall:

| candidate_k | recall@10 | ndcg@10 | sidecar p50 | total bound p50 |
| ---: | ---: | ---: | ---: | ---: |
| 50 | 0.9480 | 0.9996 | 0.929 ms | 205.313 ms |
| 100 | 0.9480 | 0.9996 | 1.674 ms | 212.044 ms |
| 200 | 0.9480 | 0.9996 | 2.704 ms | 214.673 ms |

Conclusion: the lost recall is inside the `rabitq8` sidecar ranking, not the IVF candidate frontier size.
