# Task 51 Local IVF Sidecar Real-I/O Smoke

- head SHA: `0b359e5ddbee42a7cba45042f7da577d1accf7d4`
- timestamp: `2026-05-23T15:22:19Z`
- benchmark packet: `benchmarks/task51-local-ivf-sidecar-real-io/`
- runner: `ecaz bench suite`
- SuiteConfig: `benchmarks/task51-local-ivf-sidecar-real-io/suite.json`
- suite manifest: `benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-manifest.json`
- results: `benchmarks/task51-local-ivf-sidecar-real-io/artifacts/results.jsonl`
- parsed report results: `benchmarks/task51-local-ivf-sidecar-real-io/artifacts/results-report.jsonl`
- lane: local PG18 / WSL2 only
- AWS: not used
- competitors: none; this packet is IVF/RaBitQ only
- fixture: `ec_real_50k`, reused preserved isolated prefix `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- profile: `ec_ivf`
- storage format: `rabitq`
- index reloptions: `nlists=128`, `nprobe=128`, `training_sample_rows=10000`, `quant_bits=1`, `rerank=off`
- candidate frontier: IVF approximate `LIMIT 50`, then sidecar rerank to top 10
- sidecar variants: `f16`, `rabitq8`
- sidecar read modes: `free`, `random-id`, `tid-sorted`
- recall query limit: 100 local smoke waiver
- isolated one-index-per-table surface: yes, inherited from `benchmarks/task51-local-ivf-rabitq-sidecar/`

## Commands

CLI build:

```text
cargo build -p ecaz-cli
```

Suite execution:

```text
target/debug/ecaz --log-file benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-sidecar-real-io/suite.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-dry-run.log bench suite run --config benchmarks/task51-local-ivf-sidecar-real-io/suite.json --dry-run --manifest-output benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-manifest.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-sidecar-real-io/suite.json --manifest-output benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-sidecar-real-io/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-sidecar-real-io/artifacts/results-report.jsonl
```

Sidecar table size check:

```text
target/debug/ecaz --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db tqvector_bench --socket-dir /home/peter/.pgrx --raw --sql "SELECT relname, pg_size_pretty(pg_total_relation_size(oid)) AS total_size FROM pg_class WHERE relname IN ('task51_local_50k_ivf_rabitq1_n128_sidecar_off_sidecar_f16','task51_local_50k_ivf_rabitq1_n128_sidecar_off_sidecar_rabitq8') ORDER BY relname;" --log-output benchmarks/task51-local-ivf-sidecar-real-io/artifacts/sidecar-table-sizes.log
```

## Status

`suite-status.log`:

```text
[suite:task51-local-ivf-sidecar-real-io] completed=1 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Results

F16, q=100, k=10, candidate_k=50:

| nprobe | read mode | recall@10 | sidecar I/O p50 | sidecar score p50 | sidecar total p50 | candidate SQL p50 | total p50 |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | free | 0.9810 | 0.000 ms | 2.831 ms | 2.831 ms | 62.132 ms | 65.548 ms |
| 32 | random-id | 0.9810 | 18.293 ms | 5.007 ms | 23.281 ms | 62.132 ms | 85.424 ms |
| 32 | tid-sorted | 0.9810 | 1.285 ms | 4.981 ms | 6.286 ms | 62.132 ms | 68.369 ms |
| 64 | free | 0.9960 | 0.000 ms | 2.726 ms | 2.726 ms | 104.911 ms | 107.906 ms |
| 64 | random-id | 0.9960 | 17.961 ms | 4.970 ms | 22.939 ms | 104.911 ms | 127.831 ms |
| 64 | tid-sorted | 0.9960 | 1.403 ms | 4.931 ms | 6.405 ms | 104.911 ms | 111.240 ms |
| 96 | free | 0.9980 | 0.000 ms | 2.671 ms | 2.671 ms | 150.871 ms | 153.760 ms |
| 96 | random-id | 0.9980 | 17.567 ms | 4.979 ms | 22.714 ms | 150.871 ms | 174.184 ms |
| 96 | tid-sorted | 0.9980 | 1.346 ms | 4.965 ms | 6.412 ms | 150.871 ms | 157.534 ms |
| 128 | free | 0.9980 | 0.000 ms | 2.663 ms | 2.663 ms | 208.937 ms | 211.752 ms |
| 128 | random-id | 0.9980 | 17.969 ms | 5.028 ms | 23.194 ms | 208.937 ms | 232.208 ms |
| 128 | tid-sorted | 0.9980 | 1.339 ms | 5.001 ms | 6.392 ms | 208.937 ms | 215.704 ms |

RaBitQ8, q=100, k=10, candidate_k=50:

| nprobe | read mode | recall@10 | sidecar I/O p50 | sidecar score p50 | sidecar total p50 | candidate SQL p50 | total p50 |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | free | 0.9390 | 0.000 ms | 1.150 ms | 1.150 ms | 62.132 ms | 63.692 ms |
| 32 | random-id | 0.9390 | 16.654 ms | 1.116 ms | 17.798 ms | 62.132 ms | 80.636 ms |
| 32 | tid-sorted | 0.9390 | 0.885 ms | 1.058 ms | 1.961 ms | 62.132 ms | 63.982 ms |
| 64 | free | 0.9470 | 0.000 ms | 1.125 ms | 1.125 ms | 104.911 ms | 105.999 ms |
| 64 | random-id | 0.9470 | 16.655 ms | 1.119 ms | 17.799 ms | 104.911 ms | 123.684 ms |
| 64 | tid-sorted | 0.9470 | 0.942 ms | 1.071 ms | 2.068 ms | 104.911 ms | 107.062 ms |
| 96 | free | 0.9480 | 0.000 ms | 1.157 ms | 1.157 ms | 150.871 ms | 152.383 ms |
| 96 | random-id | 0.9480 | 17.146 ms | 1.121 ms | 18.272 ms | 150.871 ms | 170.114 ms |
| 96 | tid-sorted | 0.9480 | 0.919 ms | 1.053 ms | 1.981 ms | 150.871 ms | 153.272 ms |
| 128 | free | 0.9480 | 0.000 ms | 1.109 ms | 1.109 ms | 208.937 ms | 210.027 ms |
| 128 | random-id | 0.9480 | 17.354 ms | 1.119 ms | 18.523 ms | 208.937 ms | 228.018 ms |
| 128 | tid-sorted | 0.9480 | 0.902 ms | 1.060 ms | 1.976 ms | 208.937 ms | 210.997 ms |

Table sizes after rebuild:

```text
task51_local_50k_ivf_rabitq1_n128_sidecar_off_sidecar_f16     | 197 MB
task51_local_50k_ivf_rabitq1_n128_sidecar_off_sidecar_rabitq8 | 79 MB
```

## Interpretation

- The real-I/O harness closes the packet 008 reviewer gap at local-smoke scale:
  sidecar read cost is now measured separately from sidecar scoring cost.
- Naive random-id lookup is not a good product shape. It adds about 17-18 ms
  p50 I/O for 50 candidates on this local fixture.
- TID-sorted batch fetch is much closer to the free-I/O bound. It adds about
  0.9-1.4 ms p50 sidecar I/O for the measured f16/rabitq8 tables.
- F16 preserves the 50-candidate frontier recall in this fixture and reaches
  recall@10 0.9980 by nprobe 96/128.
- RaBitQ8 remains much smaller but still loses recall at this candidate width
  (`0.9470-0.9480` at nprobe 64-128).

## Caveats

- This is local PG18/WSL2 smoke evidence only, not Graviton evidence.
- q=100 is below the final Task 51 evidence bar and is used here as a local
  real-I/O screen.
- The `total_bound_*` column name is inherited from the original upper-bound
  harness. For `random-id` and `tid-sorted` rows it is `candidate_sql_* +
  measured sidecar_*`, not a free-I/O product forecast.
- The TID-sorted mode fetches sidecar rows in physical `ctid` order from a
  separate fixed-width `bytea` table. It is still a microbenchmark, not an
  in-index sidecar storage implementation.
- TID-sorted assumptions are load-bearing:
  - static corpus snapshot
  - single-threaded measurement
  - sidecar table built in corpus/id order immediately before measurement
  - no concurrent insert/update/delete churn
  - local 50k sidecar tables fit comfortably in OS cache (`197 MB` f16,
    `79 MB` rabitq8)
  - candidate frontiers are still generated by approximate score; the harness
    asks PostgreSQL to return matching sidecar rows in physical `ctid` order
- The current `sidecar_io_*` columns include the real DB fetch and the
  `ORDER BY ctid` work for `tid-sorted`. There is no separate client-side
  `sidecar_sort_*` column yet because this harness does not have a product
  heap-TID frontier to sort before sidecar fetch.
- The suite intentionally did not run vchord, pgvectorscale, or AWS.

## Artifacts

- `suite.json`: checked-in suite config.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build before suite execution.
- `artifacts/suite-audit.log`: suite audit output.
- `artifacts/suite-dry-run.log`: dry-run showing expanded read modes.
- `artifacts/suite-run.log`: authoritative suite run.
- `artifacts/suite-status.log`: final suite status.
- `artifacts/suite-report.log`: parsed report.
- `artifacts/suite-manifest.json`: structured suite manifest.
- `artifacts/results.jsonl`: structured results from the successful suite.
- `artifacts/results-report.jsonl`: structured results from the final report command.
- `artifacts/sidecar-real-io-50k-rabitq1-n128-k50.log`: measurement table.
- `artifacts/sidecar-table-sizes.log`: real PostgreSQL table sizes after rebuild.
