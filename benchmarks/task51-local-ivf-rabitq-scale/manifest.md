# Manifest: task51-local-ivf-rabitq-scale

- packet: `benchmarks/task51-local-ivf-rabitq-scale`
- head SHA at packet write: `807d2389a5f1bd5d128fea2fe67ba27e15b4b891`
- timestamp: `2026-05-23T06:05:36Z`
- task: 51
- lane: local IVF/RaBitQ geometry scale-up with counters
- runner: `ecaz bench suite`
- config: `suite.json`
- structured manifest: `artifacts/suite-manifest.json`
- structured results: `artifacts/results.jsonl`
- PostgreSQL: local PG18 scratch, socket `/home/peter/.pgrx`, port `28818`
- database: `tqvector_bench`
- AWS: not used
- vchord: not run
- pgvectorscale: not run

This packet responds to reviewer feedback on
`reviews/task-51/006-local-ivf-rabitq-geometry` by moving the IVF/RaBitQ
geometry screen to larger, non-saturated local fixtures and adding
representative EXPLAIN counter steps. It does not choose an AWS promotion
winner. Local WSL2 results do not measure Graviton v4 / Neoverse NEON
byte-LUT behavior, so AWS remains the final gate after local methodology
and code work are complete.

## Surface

- fixtures:
  - `target/real-corpus/staged-task50/ec_real_50k_{corpus,queries}.tsv`
  - `target/real-corpus/staged-task50/ec_real_100k_{corpus,queries}.tsv`
- access method: `ec_ivf`
- table layout: isolated one-index-per-table surfaces
- suite-level `bits=4`: canonical `ecvector` corpus/query encoding and
  query scoring width expected by `encode_to_ecvector(source, 4, 42)`
- index storage: `storage_format=rabitq`, `quant_bits=1`
- rerank: `heap_f32`, `rerank_width=50`
- list counts: `nlists={64,128}`
- q-count: 200 local queries
- latency iterations: 200 per nprobe cell, concurrency 1

The q-count is intentionally lower than the Task 51 AWS floor of 500 because
this is a local methodology and scale-up screen. The final AWS packet must
use q-count >= 500 unless it records a separate cost waiver.

## Commands

```bash
target/release/ecaz bench suite audit --config benchmarks/task51-local-ivf-rabitq-scale/suite.json --log-file benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-audit.log
target/release/ecaz bench suite run --dry-run --config benchmarks/task51-local-ivf-rabitq-scale/suite.json --host /home/peter/.pgrx --port 28818 --database tqvector_bench --log-file benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-dry-run.log
target/release/ecaz bench suite run --config benchmarks/task51-local-ivf-rabitq-scale/suite.json --host /home/peter/.pgrx --port 28818 --database tqvector_bench --log-file benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-run.log
target/release/ecaz bench suite status --manifest benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-manifest.json --log-file benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-status.log
target/release/ecaz bench suite report --manifest benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-manifest.json --log-file benchmarks/task51-local-ivf-rabitq-scale/artifacts/suite-report.log
```

Status:

```text
[suite:task51-local-ivf-rabitq-scale] completed=20 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Authoritative Artifacts

| File | Role |
| --- | --- |
| `suite.json` | checked-in `SuiteConfig` |
| `artifacts/suite-manifest.json` | structured suite execution manifest |
| `artifacts/results.jsonl` | structured parsed results, 100 rows |
| `artifacts/suite-run.log` | final successful suite run |
| `artifacts/suite-status.log` | suite status, 20 complete / 0 failed |
| `artifacts/suite-report.log` | parsed report with load, recall, latency, storage, planner-cost rows |
| `artifacts/explain-*-rabitq1-n*-w50.log` | EXPLAIN counter evidence |
| `artifacts/truth-ec-real-50k-q200-k10.json` | 50k truth cache |
| `artifacts/truth-ec-real-100k-q200-k10.json` | 100k truth cache |
| `artifacts/local-index-catalog-check.log` | pre-run catalog check proving local built indexes/database existed |

## Key Results

### 50k

| Geometry | Recall cell | Recall@10 | p50 | p95 | Index size |
| --- | ---: | ---: | ---: | ---: | ---: |
| `nlists=64` | `nprobe=48` | `0.9950` | `140.6 ms` | `164.2 ms` | `15.2 MiB` |
| `nlists=64` | `nprobe=64` | `0.9975` | `190.2 ms` | `214.7 ms` | `15.2 MiB` |
| `nlists=128` | `nprobe=64` | `0.9940` | `91.6 ms` | `111.7 ms` | `15.9 MiB` |
| `nlists=128` | `nprobe=96` | `0.9975` | `138.5 ms` | `162.6 ms` | `15.9 MiB` |

At matched recall@10 `0.9975`, `nlists=128,nprobe=96` is 27% lower p50 than
`nlists=64,nprobe=64` on local 50k (`138.5 ms` vs `190.2 ms`) with modest
index growth (`15.9 MiB` vs `15.2 MiB`).

### 100k

| Geometry | Recall cell | Recall@10 | p50 | p95 | Index size |
| --- | ---: | ---: | ---: | ---: | ---: |
| `nlists=64` | `nprobe=48` | `0.9955` | `306.5 ms` | `350.3 ms` | `29.7 MiB` |
| `nlists=64` | `nprobe=64` | `0.9985` | `379.5 ms` | `419.3 ms` | `29.7 MiB` |
| `nlists=128` | `nprobe=64` | `0.9870` | `196.6 ms` | `225.1 ms` | `30.5 MiB` |
| `nlists=128` | `nprobe=96` | `0.9970` | `290.3 ms` | `325.1 ms` | `30.5 MiB` |
| `nlists=128` | `nprobe=128` | `0.9985` | `377.7 ms` | `418.0 ms` | `30.5 MiB` |

At top matched recall@10 `0.9985`, `nlists=128,nprobe=128` is effectively
tied with `nlists=64,nprobe=64` on local 100k (`377.7 ms` vs `379.5 ms`).
At slightly lower recall, `nlists=128,nprobe=96` gives recall@10 `0.9970`
at `290.3 ms` p50.

## Counter Summary

Representative EXPLAIN steps use `nprobe=64` for each geometry.

| Cell | Posting pages | Postings scored | Rerank rows | Heap blocks | Approx scan | Exact rerank | Execution |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k `n64,p64` | 1886 | 50000 | 50 | 34 | `184.3 ms` | `2.2 ms` | `188.6 ms` |
| 50k `n128,p64` | 900 | 23505 | 50 | 34 | `87.4 ms` | `2.5 ms` | `91.9 ms` |
| 100k `n64,p64` | 3734 | 100000 | 50 | 33 | `376.2 ms` | `2.0 ms` | `380.2 ms` |
| 100k `n128,p64` | 1823 | 48351 | 50 | 33 | `223.3 ms` | `3.2 ms` | `229.4 ms` |

The counters show this local geometry experiment is dominated by approximate
posting scan time, not exact heap rerank time. Raising `nlists` roughly halves
postings scored at fixed `nprobe=64`, but at 100k the highest recall point
requires `nprobe=128`, which gives back the scan-volume win.

## Interpretation

This packet closes the local methodology gaps from the 10k screen enough to
avoid a premature AWS decision:

- sub-knee nprobe cells are present on 50k and 100k;
- recall is not saturated across every cell;
- representative scan counters are captured;
- headline rows are not treated as Graviton evidence;
- no vchord or pgvectorscale run was added.

For AWS, this packet suggests carrying both `nlists=64` and `nlists=128`
forward rather than promoting only one. The 50k local fixture clears the
25% p50 matched-recall gate for `nlists=128`, but the 100k fixture does not
at the highest recall band. The next local work should process Experiment 7
sidecar measurements and any reviewer feedback before opening the AWS final
gate.

