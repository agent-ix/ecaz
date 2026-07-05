# Task 51 Local IVF/RaBitQ Sidecar Upper-Bound

- head SHA: `ee876d09089dbc67a2faa824f7545e92227c3a8d`
- benchmark path: `benchmarks/task51-local-ivf-rabitq-sidecar`
- task bucket: `reviews/task-51`
- lane: local PG18 Exp 7 sidecar upper-bound preflight
- fixture: `ec_real_50k`, 50,000 corpus rows, 200 queries
- access method: `ec_ivf`
- storage format: `rabitq`
- index reloptions: `nlists=128`, `nprobe=128`, `training_sample_rows=10000`, `quant_bits=1`, `rerank=off`
- candidate frontier: IVF approximate `LIMIT 50`, then local sidecar rerank to top 10
- sidecar variants: `f32`, `f16`, `rabitq8`
- isolated one-index-per-table surface: yes, `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- AWS: not used
- vchord / pgvectorscale: not used

## Measurement Scope

This packet is a free-I/O upper-bound measurement. The CLI builds each
sidecar representation in process memory before timed reranking, so
`sidecar_p50` measures local scoring over resident source bytes. It does not
measure product sidecar storage latency, random id lookup, TID-sorted sidecar
fetch, prefetch behavior, or an in-index read path. A real-I/O sidecar
microbenchmark is still owed before Exp 7 can support a product decision.

`candidate_sql_p50` is the current `rerank=off` IVF approximate query returning
the top 50 candidate ids to the client. It is not a full posting-frontier
materialization.

## Commands

Validation:

```text
cargo check -p ecaz-cli
cargo test -p ecaz-cli expands_sidecar_rerank_with_variants
target/debug/ecaz bench suite run --config benchmarks/task51-local-ivf-rabitq-sidecar/suite.json --dry-run --manifest-output benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-manifest.json
git diff --check
```

Benchmark:

```text
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-rabitq-sidecar/suite.json --manifest-output benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/results.jsonl
target/debug/ecaz --log-file benchmarks/task51-local-ivf-rabitq-sidecar/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-rabitq-sidecar/suite.json
```

## Artifacts

- `suite.json` - checked-in SuiteConfig.
- `artifacts/suite-manifest.json` - suite execution manifest.
- `artifacts/results.jsonl` - structured parsed results.
- `artifacts/suite-run.log` - authoritative run log.
- `artifacts/suite-status.log` - status summary.
- `artifacts/suite-report.log` - parsed report.
- `artifacts/suite-audit.log` - suite audit.
- `artifacts/load-50k-rabitq1-n128-rerank-off.log` - load/index build log.
- `artifacts/sidecar-50k-rabitq1-n128-k50.log` - sidecar measurement table.
- `artifacts/storage-50k-rabitq1-n128-rerank-off.log` - storage accounting.

## Result Summary

Suite status:

```text
completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Key sidecar rows:

| nprobe | variant | recall@10 | candidate SQL p50 | sidecar p50 | total bound p50 | sidecar size |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 32 | f32 | 0.9800 | 57.225 ms | 1.533 ms | 58.800 ms | 292.97 MiB |
| 32 | f16 | 0.9800 | 57.225 ms | 2.556 ms | 59.890 ms | 146.48 MiB |
| 32 | rabitq8 | 0.9415 | 57.225 ms | 1.130 ms | 58.360 ms | 73.81 MiB |
| 64 | f32 | 0.9940 | 107.509 ms | 1.552 ms | 109.450 ms | 292.97 MiB |
| 64 | f16 | 0.9940 | 107.509 ms | 2.561 ms | 110.184 ms | 146.48 MiB |
| 64 | rabitq8 | 0.9505 | 107.509 ms | 1.120 ms | 108.628 ms | 73.81 MiB |
| 96 | f32 | 0.9975 | 166.570 ms | 1.662 ms | 168.543 ms | 292.97 MiB |
| 96 | f16 | 0.9975 | 166.570 ms | 2.654 ms | 169.267 ms | 146.48 MiB |
| 96 | rabitq8 | 0.9535 | 166.570 ms | 1.103 ms | 167.665 ms | 73.81 MiB |
| 128 | f32 | 0.9975 | 234.021 ms | 1.548 ms | 235.618 ms | 292.97 MiB |
| 128 | f16 | 0.9975 | 234.021 ms | 2.550 ms | 236.581 ms | 146.48 MiB |
| 128 | rabitq8 | 0.9535 | 234.021 ms | 1.102 ms | 235.189 ms | 73.81 MiB |

Storage:

```text
ec_ivf index size: 15.9 MiB
f32 sidecar estimate: 292.97 MiB
f16 sidecar estimate: 146.48 MiB
rabitq8 sidecar estimate: 73.81 MiB
```

## Interpretation and Caveats

- This is a local preflight and upper-bound harness, not a real IVF sidecar implementation. It eliminates per-query heap/vector fetch from the timed rerank path by loading sidecar data into the CLI process.
- The measured local candidate SQL time is the existing rerank-off IVF approximate scan. It still dominates p50, especially at high nprobe.
- f32 and f16 preserve the same recall because both rerank the same 50-candidate approximate frontier closely enough at this fixture. The f16 scalar path is slower on this local x86/WSL2 host; that is not Graviton NEON evidence.
- `rabitq8` is much smaller and slightly faster in local rerank CPU, but loses recall at this candidate width on this fixture.
- `recall_p10 = 0.9000` at nprobe=32 for all variants is a candidate-frontier floor: the approximate `LIMIT 50` candidate set is missing true neighbors for the hardest queries before exact sidecar scoring runs.
- Query count is 200 as a local cost waiver. AWS promotion must raise q-count per Task 51 rules.
