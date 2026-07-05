# Review Request: IVF Scratch SoA Chunked Bits1

- task: `plan/tasks/51-ivf-rabitq-second-optimization-round.md`
- packet: `reviews/task-51/019-ivf-scratch-chunked-bits1/`
- code commit: `a756fe0a5` Add IVF RaBitQ bits1 scratch batch scoring
- benchmark packet: `benchmarks/task51-local-ivf-scratch-soa-chunked/`
- benchmark commit: `50a0ec81a` Add local IVF scratch chunked benchmark packet
- branch: `aws-optimization-ivf-rabitq-spire`

## Scope

This addresses reviewer feedback on packet 015: the previous scratch SoA path only copied posting payloads into a SoA buffer and then reused the normal per-posting scalar dispatch. This slice wires the opt-in scratch path to a RaBitQ bits=1 batch scoring entrypoint over the contiguous scratch payload slab.

Changes:

- Added `PreparedEstimator::estimate_ip_bits1_batch` for contiguous bits=1 RaBitQ code slabs.
- Added IVF quantizer dispatch for RaBitQ bits=1 batch scoring only; non-bits=1 and non-RaBitQ profiles decline and keep the previous path.
- Routed `ec_ivf.scratch_soa_batch_decode=on` scratch batches through the batch scorer.
- Extended `ecaz bench suite` `explain` steps with `ivf_scratch_soa_batch_decode` so counter evidence can be produced by the suite runner.

## Validation

Compile/static validation:

```text
cargo test -p ecaz --lib bits1_batch --no-run --no-default-features --features pg18
cargo test -p ecaz --lib posting_scratch_soa_reuses_capacity_when_payload_len_matches --no-run --no-default-features --features pg18
cargo test -p ecaz-cli --no-default-features explain_sql_can_enable_ivf_scratch_soa
cargo test -p ecaz-cli --no-default-features expands_recall_with_defaults
cargo build -p ecaz-cli --no-default-features
```

The `ecaz` lib tests compile, but standalone runtime is still blocked by the existing pgrx symbol lookup issue:
`undefined symbol: CacheRegisterRelcacheCallback`.

Benchmark validation:

```text
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-scratch-soa-chunked/suite.json --manifest-output benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/results-report.jsonl
```

Suite status:

```text
[suite:task51-local-ivf-scratch-soa-chunked] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

Recall is byte-equivalent:

| mode | recall@10 | recall_p10 | recall_worst | NDCG@10 |
| --- | ---: | ---: | ---: | ---: |
| static scan | 0.9750 | 0.9000 | 0.5000 | 0.9986 |
| chunked scratch SoA | 0.9750 | 0.9000 | 0.5000 | 0.9986 |

Performance is negative locally:

| mode | latency p50 | latency p95 | latency p99 | approx scan | candidates/sec |
| --- | ---: | ---: | ---: | ---: | ---: |
| static scan | 549.1 ms | 636.9 ms | 660.3 ms | 610717 us | 226740 |
| chunked scratch SoA | 653.4 ms | 765.6 ms | 1095.3 ms | 622809 us | 222337 |

Conclusion: Exp 3 fails the local promotion gate. Do not promote scratch SoA or use it as a Layout v2 gate-open signal. The local result is now a real chunked bits=1 scratch measurement, not just the layout-copy half from packet 015.

## Artifacts

Packet-local manifest:

- `reviews/task-51/019-ivf-scratch-chunked-bits1/artifacts/manifest.md`

Benchmark packet:

- `benchmarks/task51-local-ivf-scratch-soa-chunked/manifest.md`
- `benchmarks/task51-local-ivf-scratch-soa-chunked/suite.json`
- `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/suite-report.log`
- `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/results-report.jsonl`
- `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/explain-chunked-scratch-990k-rabitq1-n1024-w50-p128.log`
