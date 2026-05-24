# Task 51 Local IVF Scratch SoA Chunked Bits1

- head SHA: `a756fe0a5`
- timestamp: `2026-05-23T16:55:00Z`
- benchmark packet: `benchmarks/task51-local-ivf-scratch-soa-chunked/`
- runner: `ecaz bench suite`
- SuiteConfig: `benchmarks/task51-local-ivf-scratch-soa-chunked/suite.json`
- suite manifest: `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-manifest.json`
- results: `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/results.jsonl`
- parsed report: `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/results-report.jsonl`
- lane: local PG18 / WSL2 only
- AWS: not used
- competitors: none; IVF/RaBitQ only
- table surface: reused preserved isolated prefix `task51_local_990k_ivf_rabitq1_n1024_w50`
- corpus: staged anchor corpus, 990000 rows, 10000 query rows, dim 1536
- profile: `ec_ivf`
- storage format: `rabitq`
- reloptions: `nlists=1024`, `nprobe=256`, `training_sample_rows=10000`, `quant_bits=1`, `rerank=heap_f32`, `rerank_width=50`, `storage_format=rabitq`
- scratch mode: opt-in `ec_ivf.scratch_soa_batch_decode`, default `off`
- scratch implementation under test: contiguous scratch SoA payload slab scored through `PreparedEstimator::estimate_ip_bits1_batch`
- rerank mode: heap f32 rerank width 50
- recall query limit: 100 local smoke waiver
- latency iterations: 100, concurrency 1
- isolated one-index-per-table surface: yes, inherited from `benchmarks/task51-local-ivf-rabitq-990k/`

## Commands

Compile/static validation:

```text
cargo test -p ecaz --lib bits1_batch --no-run --no-default-features --features pg18
cargo test -p ecaz --lib posting_scratch_soa_reuses_capacity_when_payload_len_matches --no-run --no-default-features --features pg18
cargo test -p ecaz-cli --no-default-features explain_sql_can_enable_ivf_scratch_soa
cargo test -p ecaz-cli --no-default-features expands_recall_with_defaults
cargo build -p ecaz-cli --no-default-features
```

The focused `ecaz` lib tests compiled but standalone execution is blocked by the existing pgrx symbol lookup issue:
`undefined symbol: CacheRegisterRelcacheCallback`.

Release install and local PG18 restart:

```text
script -q -e -c "cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config" benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/local-pgrx-install-release.log
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 restart -m fast
script -q -e -c "/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 status" benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/local-pg18-status-after-restart.log
```

Suite run:

```text
target/debug/ecaz --log-file benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-scratch-soa-chunked/suite.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-dry-run.log bench suite run --config benchmarks/task51-local-ivf-scratch-soa-chunked/suite.json --dry-run --manifest-output benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-manifest.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-scratch-soa-chunked/suite.json --manifest-output benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/results-report.jsonl
```

## Status

`suite-status.log`:

```text
[suite:task51-local-ivf-scratch-soa-chunked] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Results

Recall, q=100, k=10:

| mode | nprobe | recall@10 | recall_p10 | recall_worst | NDCG@10 | mean q-time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| static scan | 128 | 0.9750 | 0.9000 | 0.5000 | 0.9986 | 584.59 ms |
| chunked scratch SoA | 128 | 0.9750 | 0.9000 | 0.5000 | 0.9986 | 660.99 ms |

Latency, q=100, concurrency 1:

| mode | nprobe | p50 | p95 | p99 | mean |
| --- | ---: | ---: | ---: | ---: | ---: |
| static scan | 128 | 549.1 ms | 636.9 ms | 660.3 ms | 553.1 ms |
| chunked scratch SoA | 128 | 653.4 ms | 765.6 ms | 1095.3 ms | 668.2 ms |

EXPLAIN counters:

| mode | scratch GUC | execution | posting pages | postings scored | heap TIDs scored | rerank rows | heap blocks | approx scan | candidates/sec |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| static scan | off | 622.097 ms | 5192 | 138476 | 138476 | 50 | 48 | 610717 us | 226740 |
| chunked scratch SoA | on | 630.776 ms | 5192 | 138476 | 138476 | 50 | 48 | 622809 us | 222337 |

Candidates/sec is `Postings Scored / (Approximate Scan Elapsed Us / 1e6)`.

## Conclusion

The chunked bits=1 scratch path preserves recall byte-equivalence but fails the Exp 3 promotion gate locally:

- candidates/sec regressed from about 226.7k/s to 222.3k/s, a 1.9% decrease
- latency p50 regressed from 549.1 ms to 653.4 ms, an 19.0% slowdown
- p99 regressed from 660.3 ms to 1095.3 ms

Do not promote scratch SoA to AWS or Layout v2 from this evidence. Exp 3 can be closed as negative locally unless a separate Graviton-only microbench is explicitly requested for the NEON-specific kernel question.
