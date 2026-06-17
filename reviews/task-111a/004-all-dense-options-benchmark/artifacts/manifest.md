# Task 111a All Dense Options Benchmark Manifest

Task bucket: `reviews/task-111a/`

Packet: `reviews/task-111a/004-all-dense-options-benchmark/`

Head SHA: `c543e7a969f91d9574037d309cfe904aaf5db352`

Branch: `task-111-ivf-dense-posting-block-layout`

Date: 2026-06-17

## Scope

This packet measures row postings, dense-old, dense-a, dense-typed, dense-b,
and dense-b-typed for TurboQuant and RaBitQ on real 50k and 100k corpora. The
suite uses isolated one-index-per-table surfaces, not shared-table surfaces.

Suite config:

- `artifacts/task111a-all-dense-options-suite.json`
- config SHA256 from suite report:
  `e5c465048b03cf10b33566e9ba055285ca015945ce85faa281c726623c156a63`

Installed PG18 extension:

- command:
  `target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-111a/004-all-dense-options-benchmark/artifacts/install-ecaz-pg18-release.log`
- backend hash:
  `96daa6f4f390c7891f55cfbfba611172064142b6b8b5d21f07dfcbc8caead484`

## Commands

Release build:

```text
cargo build --release -p ecaz-cli --bin ecaz
```

Suite audit:

```text
target/release/ecaz --log-file reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite-audit.log bench suite audit --config reviews/task-111a/004-all-dense-options-benchmark/artifacts/task111a-all-dense-options-suite.json --database task111a_dense_bench --host /home/peter/.pgrx --port 28818
```

Suite dry run:

```text
target/release/ecaz --log-file reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite-dry-run.log bench suite run --config reviews/task-111a/004-all-dense-options-benchmark/artifacts/task111a-all-dense-options-suite.json --dry-run --database task111a_dense_bench --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite-dry-run-manifest.json
```

Suite run:

```text
target/release/ecaz --log-file reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite-run.log bench suite run --config reviews/task-111a/004-all-dense-options-benchmark/artifacts/task111a-all-dense-options-suite.json --database task111a_dense_bench --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite/suite-manifest.json --results-output reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite/results.jsonl
```

Suite status:

```text
target/release/ecaz --log-file reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite-status.log bench suite status --manifest reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite/suite-manifest.json
```

Suite report:

```text
target/release/ecaz --log-file reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite-report.log bench suite report --manifest reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite/suite-manifest.json --results-output reviews/task-111a/004-all-dense-options-benchmark/artifacts/suite/results-report.jsonl
```

## Artifacts

Primary structured artifacts:

- `artifacts/suite/suite-manifest.json`
- `artifacts/suite/results.jsonl`
- `artifacts/suite/results-report.jsonl`
- `artifacts/summary.md`
- `artifacts/suite-status.log`
- `artifacts/suite-report.log`
- `artifacts/suite-audit.log`
- `artifacts/suite-dry-run.log`
- `artifacts/suite-dry-run-manifest.json`

Raw logs cited by the summary are the `artifacts/suite/latency-*.log`,
`artifacts/suite/recall-*.log`, and `artifacts/suite/storage-*.log` files for
the 24 measured scale/quant/variant combinations.

Regenerable truth cache files are intentionally not committed:

- `artifacts/suite/truth-50k-k10.json`
- `artifacts/suite/truth-100k-k10.json`

## Key Results

Suite status:

```text
[suite:task111a-all-dense-options-benchmark-gate] completed=120 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Latency p50 at nprobe 32:

```text
50k  TQ     row 15.0 ms  dense-old 17.7 ms  dense-a 13.1 ms  dense-typed 22.0 ms  dense-b 14.6 ms  dense-b-typed 13.8 ms
50k  RaBitQ row 7.29 ms  dense-old 6.06 ms  dense-a 6.17 ms  dense-typed 5.99 ms  dense-b 7.14 ms  dense-b-typed 7.31 ms
100k TQ     row 32.4 ms  dense-old 38.4 ms  dense-a 28.2 ms  dense-typed 37.7 ms  dense-b 29.0 ms  dense-b-typed 30.4 ms
100k RaBitQ row 14.7 ms  dense-old 12.2 ms  dense-a 12.6 ms  dense-typed 12.2 ms  dense-b 14.0 ms  dense-b-typed 13.9 ms
```

Recall at nprobe 32/64 is unchanged across layout variants:

```text
50k  TQ     nprobe32 recall@k=0.9420 ndcg@k=0.9994; nprobe64 recall@k=0.9420 ndcg@k=0.9996
50k  RaBitQ nprobe32 recall@k=0.7750 ndcg@k=0.9896; nprobe64 recall@k=0.7770 ndcg@k=0.9899
100k TQ     nprobe32 recall@k=0.9370 ndcg@k=0.9966; nprobe64 recall@k=0.9560 ndcg@k=0.9997
100k RaBitQ nprobe32 recall@k=0.7630 ndcg@k=0.9875; nprobe64 recall@k=0.7750 ndcg@k=0.9906
```

Storage, primary ANN index only:

```text
50k  TQ     row 44.1 MiB; dense-old/a/typed 39.8 MiB; dense-b/b-typed 49.2 MiB
50k  RaBitQ row 15.2 MiB; dense-old/a/typed 11.6 MiB; dense-b/b-typed 12.9 MiB
100k TQ     row 87.6 MiB; dense-old/a/typed 78.9 MiB; dense-b/b-typed 98.1 MiB
100k RaBitQ row 29.7 MiB; dense-old/a/typed 22.5 MiB; dense-b/b-typed 25.1 MiB
```

Batch-width explanation for the TQ dense-old regression at nprobe 32:

```text
50k row        flushes=9,147   candidates=2,328,863  width>=32=9,134
50k dense-old  flushes=233,854 candidates=2,328,462  width8-15=232,161 width>=32=0
50k dense-a    flushes=10,699  candidates=2,328,864  width>=32=10,389
100k row       flushes=20,379  candidates=5,203,807  width>=32=20,363
100k dense-old flushes=521,755 candidates=5,203,613  width8-15=519,350 width>=32=0
100k dense-a   flushes=21,778  candidates=5,203,752  width>=32=21,443
```

## Notes

The suite created local truth cache files under the packet artifact directory.
They are regenerable and intentionally excluded from git per repository review
packet policy.

