# Manifest: task51-local-ivf-rabitq-geometry

- packet: `benchmarks/task51-local-ivf-rabitq-geometry`
- head SHA at run start: `d4be1037f50dfa4f8357c849404df37ca084620c`
- timestamp: `2026-05-23T05:25:55Z`
- task: 51
- lane: local IVF/RaBitQ geometry sweep
- runner: `ecaz bench suite`
- config: `suite.json`
- structured manifest: `artifacts/suite-manifest.json`
- structured results: `artifacts/results.jsonl`
- PostgreSQL: local PG18 scratch, socket `/home/peter/.pgrx`, port `28818`
- database: `tqvector_bench`
- AWS: not used
- vchord: not run
- pgvectorscale: not run

This packet is the first local Task 51 IVF/RaBitQ-only experiment. It sweeps
IVF geometry over `nlists={32,64,128}` on the local DBpedia 10k fixture, using
`storage_format=rabitq`, `quant_bits=1`, `rerank=heap_f32`, and
`rerank_width=50`.

## Surface

- fixture: `target/real-corpus/staged-task50/ec_real_10k_{corpus,queries}.tsv`
- corpus rows: 10000
- query rows: 200
- access method: `ec_ivf`
- column/query encoding: `ecvector` canonical `bits=4`, `seed=42`
- compact index storage: `storage_format=rabitq`, `quant_bits=1`
- rerank mode: `heap_f32`
- rerank width: 50
- load reloptions:
  - `nlists=32,nprobe=32,training_sample_rows=10000,quant_bits=1,rerank=heap_f32,rerank_width=50`
  - `nlists=64,nprobe=64,training_sample_rows=10000,quant_bits=1,rerank=heap_f32,rerank_width=50`
  - `nlists=128,nprobe=128,training_sample_rows=10000,quant_bits=1,rerank=heap_f32,rerank_width=50`
- recall sweeps:
  - nlists 32: `nprobe={8,16,24,32}`
  - nlists 64: `nprobe={8,16,24,32,48,64}`
  - nlists 128: `nprobe={8,16,32,48,64,96,128}`
- latency sweeps: same `nprobe` cells, 200 iterations, concurrency 1
- isolated one-index-per-table surfaces: yes

## Commands

```bash
target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file benchmarks/task51-local-ivf-rabitq-geometry/artifacts/install-ecaz-pg18.log
target/release/ecaz dev scratch restart --pg 18 --log-file benchmarks/task51-local-ivf-rabitq-geometry/artifacts/restart-pg18.log
target/release/ecaz dev sql --pg 18 --raw --sql "CREATE EXTENSION IF NOT EXISTS ecaz; SELECT extversion FROM pg_extension WHERE extname='ecaz';" --log-output benchmarks/task51-local-ivf-rabitq-geometry/artifacts/create-extension-pg18.log
target/release/ecaz bench suite audit --config benchmarks/task51-local-ivf-rabitq-geometry/suite.json --log-file benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-audit-after-bits-fix.log
target/release/ecaz bench suite dry-run --config benchmarks/task51-local-ivf-rabitq-geometry/suite.json --host /home/peter/.pgrx --port 28818 --database tqvector_bench --log-file benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-dry-run-after-bits-fix.log
target/release/ecaz bench suite run --config benchmarks/task51-local-ivf-rabitq-geometry/suite.json --host /home/peter/.pgrx --port 28818 --database tqvector_bench --log-file benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-run-final.log
target/release/ecaz bench suite status --manifest benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-manifest.json --log-file benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-status.log
target/release/ecaz bench suite report --manifest benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-manifest.json --log-file benchmarks/task51-local-ivf-rabitq-geometry/artifacts/suite-report.log
```

The earlier `suite-run.log` failed because the child commands lacked an
explicit host/socket. The earlier `suite-run-hosted.log` failed because the
first config used `bits=1` for the `ecvector` column/query encoding; `ecvector`
load expects canonical `(bits,seed)=(4,42)`. Those failed logs are retained for
traceability and are superseded by `suite-run-final.log`.

## Authoritative Artifacts

| File | Role |
| --- | --- |
| `suite.json` | checked-in `SuiteConfig` for the run |
| `artifacts/suite-manifest.json` | structured suite execution manifest |
| `artifacts/results.jsonl` | structured per-step results |
| `artifacts/suite-run-final.log` | final successful suite run log |
| `artifacts/suite-status.log` | suite status: `completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0` |
| `artifacts/suite-report.log` | parsed report with load, recall, latency, and storage tables |
| `artifacts/truth-ec-real-10k-q200-k10.json` | recall truth cache |
| `artifacts/load-10k-rabitq1-n{32,64,128}-w50.log` | per-geometry load/index logs |
| `artifacts/recall-10k-rabitq1-n{32,64,128}-w50.log` | per-geometry recall sweeps |
| `artifacts/latency-10k-rabitq1-n{32,64,128}-w50.log` | per-geometry latency sweeps |
| `artifacts/storage-10k-rabitq1-n{32,64,128}-w50.log` | per-geometry storage measurements |

## Key Results

| Geometry | Best low-nprobe recall cell | Latency p50 at that cell | Full-recall cell | Index size |
| --- | ---: | ---: | ---: | ---: |
| `nlists=32` | `nprobe=8`, recall@10 `0.9985` | `13.5 ms` | `nprobe=16`, recall@10 `1.0000` | `3.3 MiB` |
| `nlists=64` | `nprobe=8`, recall@10 `0.9970` | `7.91 ms` | `nprobe=16`, recall@10 `1.0000` | `3.6 MiB` |
| `nlists=128` | `nprobe=16`, recall@10 `0.9970` | `7.95 ms` | `nprobe=32`, recall@10 `1.0000` | `4.4 MiB` |

The local 10k result favors larger `nlists` for low-nprobe latency. At
approximately matched high recall, `nlists=64,nprobe=8` cuts p50 latency from
`13.5 ms` to `7.91 ms` versus `nlists=32,nprobe=8`, with index storage rising
from `3.3 MiB` to `3.6 MiB`. `nlists=128,nprobe=8` reaches `5.52 ms` p50 at
recall@10 `0.9935`, and `nlists=128,nprobe=16` reaches recall@10 `0.9970` at
`7.95 ms` p50.

## Interpretation

This is a local geometry screen, not an AWS closeout. The result supports
promoting `nlists=64` and `nlists=128` into the next larger local IVF/RaBitQ
suite before using AWS as the final gate.

