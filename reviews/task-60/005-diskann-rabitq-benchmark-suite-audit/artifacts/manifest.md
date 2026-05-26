# Task 60 DiskANN RaBitQ Benchmark Suite Audit Artifacts

- Head SHA: `1f9b75bae381a7181e2688819e1d017ec270d969`
- Task bucket: `reviews/task-60/005-diskann-rabitq-benchmark-suite-audit/`
- Timestamp: 2026-05-25
- Lane: benchmark suite readiness, dry-run only
- Fixture: DBpedia OpenAI3 1M fetch with `ec_real_100k` and `ec_real_ann_benchmarks_anchor` prepares
- Storage formats: `pq_fastscan`, `rabitq`
- Rerank mode: default DiskANN benchmark recall/latency flow
- Shared-table surface: no; suite uses one prefix per size and storage format

## Artifacts

### `suite-audit.log`

Command:

```sh
cargo run -p ecaz-cli -- bench suite audit --config benchmarks/task60-diskann-rabitq-format/suite.json
```

Key result:

```text
[suite:task60-diskann-rabitq-format] audit passed: 24 steps
```

### `suite-dry-run.log`

Command:

```sh
cargo run -p ecaz-cli -- bench suite run --config benchmarks/task60-diskann-rabitq-format/suite.json --dry-run --manifest-output benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json
```

Key result:

```text
[suite:task60-diskann-rabitq-format] wrote benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json
[suite:task60-diskann-rabitq-format] prepare-ec-real-1m-anchor -> --database tqvector_bench corpus prepare --profile ec_real_ann_benchmarks_anchor --parquet /var/lib/pgsql/18/datasets/dbpedia-openai3-1m/data --output-dir /var/lib/pgsql/18/datasets/staged-task60-diskann-rabitq
```
