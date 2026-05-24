# Artifact Manifest: IVF Scratch SoA Chunked Bits1

- head SHA: `50a0ec81a`
- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/019-ivf-scratch-chunked-bits1/`
- code commit under review: `a756fe0a5`
- benchmark packet: `benchmarks/task51-local-ivf-scratch-soa-chunked/`
- benchmark commit: `50a0ec81a`
- timestamp: `2026-05-23T17:05:00Z`
- lane: local PG18 / WSL2 only
- fixture: preserved `task51_local_990k_ivf_rabitq1_n1024_w50`
- storage format: `rabitq`, `quant_bits=1`
- rerank mode: `heap_f32`, `rerank_width=50`
- isolated one-index-per-table surface: yes
- AWS: not used
- competitors: none; IVF/RaBitQ only

## Artifacts

| artifact | command | notes |
| --- | --- | --- |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/manifest.md` | manual packet manifest | Source of truth for the local measurement packet. |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/suite.json` | checked-in SuiteConfig | Drives the static vs chunked scratch recall, latency, and EXPLAIN steps. |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/local-pgrx-install-release.log` | `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config` | Installs the release extension build before PG18 runtime measurement. |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-run.log` | `ecaz bench suite run --config benchmarks/task51-local-ivf-scratch-soa-chunked/suite.json` | Full suite execution log. |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-status.log` | `ecaz bench suite status --manifest .../suite-manifest.json` | Completed 6, failed 0. |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-report.log` | `ecaz bench suite report --manifest .../suite-manifest.json` | Parsed result summary. |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/results.jsonl` | emitted by `suite run` | Structured step results. |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/results-report.jsonl` | emitted by `suite report` | Parsed metric rows. |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/explain-static-990k-rabitq1-n1024-w50-p128.log` | suite `explain` step | Static scan counters. |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/explain-chunked-scratch-990k-rabitq1-n1024-w50-p128.log` | suite `explain` step with `ec_ivf.scratch_soa_batch_decode=on` | Chunked scratch counters. |

## Key Lines Cited

Suite status:

```text
[suite:task51-local-ivf-scratch-soa-chunked] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Recall:

```text
static:  recall@k=0.9750 recall_p10=0.9000 recall_worst=0.5000 ndcg@k=0.9986 mean q-time=584.59 ms
chunked: recall@k=0.9750 recall_p10=0.9000 recall_worst=0.5000 ndcg@k=0.9986 mean q-time=660.99 ms
```

Latency:

```text
static:  p50=549.1 ms p95=636.9 ms p99=660.3 ms mean=553.1 ms
chunked: p50=653.4 ms p95=765.6 ms p99=1095.3 ms mean=668.2 ms
```

EXPLAIN counters:

```text
static:  Postings Scored=138476 Approximate Scan Elapsed Us=610717 Execution Time=622.097 ms
chunked: scratch_soa_batch_decode=on Postings Scored=138476 Approximate Scan Elapsed Us=622809 Execution Time=630.776 ms
```

Derived candidates/sec:

```text
static:  138476 / 0.610717 = 226740 candidates/sec
chunked: 138476 / 0.622809 = 222337 candidates/sec
```

## Validation Notes

Focused `ecaz-cli` tests passed:

```text
cargo test -p ecaz-cli --no-default-features explain_sql_can_enable_ivf_scratch_soa
cargo test -p ecaz-cli --no-default-features expands_recall_with_defaults
```

Focused `ecaz` lib tests compile but cannot run in the standalone harness because of the existing pgrx backend symbol issue:

```text
cargo test -p ecaz --lib bits1_batch --no-run --no-default-features --features pg18
cargo test -p ecaz --lib posting_scratch_soa_reuses_capacity_when_payload_len_matches --no-run --no-default-features --features pg18
```

Runtime scan behavior is covered by the local PG18 suite packet above.
