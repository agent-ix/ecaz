# Task 51 Local IVF Scratch SoA Smoke

- head SHA: `a22ca84531379581855613a2968a2ca8aca14a5b`
- timestamp: `2026-05-23T15:06:04Z`
- benchmark packet: `benchmarks/task51-local-ivf-scratch-soa/`
- runner: `ecaz bench suite`
- SuiteConfig: `benchmarks/task51-local-ivf-scratch-soa/suite.json`
- suite manifest: `benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-manifest.json`
- results: `benchmarks/task51-local-ivf-scratch-soa/artifacts/results.jsonl`
- parsed report results: `benchmarks/task51-local-ivf-scratch-soa/artifacts/results-report.jsonl`
- lane: local PG18 / WSL2 only
- AWS: not used
- competitors: none; this packet is IVF/RaBitQ only
- table surface: reused preserved isolated prefix `task51_local_990k_ivf_rabitq1_n1024_w50`
- corpus: staged anchor corpus, 990000 rows, 10000 query rows, dim 1536
- profile: `ec_ivf`
- storage format: `rabitq`
- reloptions: `nlists=1024`, `nprobe=256`, `training_sample_rows=10000`, `quant_bits=1`, `rerank=heap_f32`, `rerank_width=50`, `storage_format=rabitq`
- scratch SoA mode: opt-in `ec_ivf.scratch_soa_batch_decode`, default `off`
- rerank mode: heap f32 rerank width 50
- recall query limit: 100 local smoke waiver
- latency iterations: 100, concurrency 1
- isolated one-index-per-table surface: yes, inherited from packet `benchmarks/task51-local-ivf-rabitq-990k/`

## Commands

Release local extension install before performance run:

```text
cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config
```

GUC default verification:

```text
target/debug/ecaz --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db tqvector_bench --socket-dir /home/peter/.pgrx --raw --sql "LOAD 'ecaz'; SHOW ec_ivf.scratch_soa_batch_decode;" --log-output benchmarks/task51-local-ivf-scratch-soa/artifacts/guc-scratch-soa-check.log
```

CLI build and suite execution:

```text
cargo build -p ecaz-cli
target/debug/ecaz --log-file benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-scratch-soa/suite.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-dry-run.log bench suite run --config benchmarks/task51-local-ivf-scratch-soa/suite.json --dry-run --manifest-output benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-manifest.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-scratch-soa/suite.json --manifest-output benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-scratch-soa/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-scratch-soa/artifacts/results-report.jsonl
```

## Status

`suite-status.log`:

```text
[suite:task51-local-ivf-scratch-soa] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Results

Recall, q=100, k=10:

| mode | nprobe | recall@10 | recall_p10 | NDCG@10 | mean q-time |
| --- | ---: | ---: | ---: | ---: | ---: |
| static scan | 128 | 0.9750 | 0.9000 | 0.9986 | 614.46 ms |
| scratch SoA | 128 | 0.9750 | 0.9000 | 0.9986 | 588.07 ms |

Latency, q=100, concurrency 1:

| mode | nprobe | p50 | p95 | p99 | mean |
| --- | ---: | ---: | ---: | ---: | ---: |
| static scan | 128 | 603.7 ms | 737.3 ms | 795.4 ms | 609.7 ms |
| scratch SoA | 128 | 590.5 ms | 701.9 ms | 766.7 ms | 598.5 ms |

EXPLAIN counters:

| mode | execution | posting pages | postings scored | heap TIDs scored | rerank rows | heap blocks | approx scan | exact rerank |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| static scan | 586.336 ms | 5192 | 138476 | 138476 | 50 | 48 | 576760 us | 3432 us |
| scratch SoA | 570.902 ms | 5192 | 138476 | 138476 | 50 | 48 | 561507 us | 2065 us |

## Interpretation

- Scratch SoA preserved recall on this local smoke: recall@10, recall p10, and NDCG match the static path.
- The measured p50 improvement is `603.7 ms -> 590.5 ms`, about 2.2%.
- The measured mean improvement is `609.7 ms -> 598.5 ms`, about 1.8%.
- EXPLAIN execution improved `586.336 ms -> 570.902 ms`, about 2.6%, with identical candidate/posting counts.
- This does not meet Task 51 Exp 3's local gate of at least 20% candidates/sec improvement, and no AWS run was attempted.
- Per the task rule, this local result rejects pursuing Posting Layout v2 from this scratch-SoA prototype in this round.

## Caveats

- This is local PG18/WSL2 smoke evidence only, not Graviton evidence.
- The recall query count is q=100 as a local smoke waiver.
- The prototype keeps the existing on-disk layout and copies posting tuple fields into scan-local buffers before calling the existing scalar scoring path; it does not yet implement a true chunked/vectorized bits=1 scoring kernel.
- The suite intentionally did not run vchord, pgvectorscale, or AWS.
- The suite reused the preserved 990k IVF/RaBitQ table/index from `benchmarks/task51-local-ivf-rabitq-990k/`; no corpus rebuild ran.

## Artifacts

- `suite.json`: checked-in suite config.
- `artifacts/local-pgrx-install-release.log`: release local install before performance run.
- `artifacts/guc-scratch-soa-check.log`: GUC registration/default verification.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build so suite expands scratch-SoA fields.
- `artifacts/suite-audit.log`: suite audit output.
- `artifacts/suite-dry-run.log`: dry-run showing scratch CLI flag expansion.
- `artifacts/suite-run.log`: authoritative suite run.
- `artifacts/suite-status.log`: final suite status.
- `artifacts/suite-report.log`: parsed report.
- `artifacts/suite-manifest.json`: structured suite manifest.
- `artifacts/results.jsonl`: structured results from the successful suite.
- `artifacts/results-report.jsonl`: structured results from the final report command.
- `artifacts/recall-*.log`: recall tables.
- `artifacts/latency-*.log`: latency tables.
- `artifacts/explain-*.sql`: EXPLAIN SQL fixtures.
- `artifacts/explain-*.log`: EXPLAIN JSON output with IVF counters.
- `artifacts/truth-ec-real-990k-q100-k10.json`: copied q=100/k=10 truth cache from the prior 990k packet.
