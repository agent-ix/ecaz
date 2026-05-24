# Task 55 M5 DiskANN Baseline Manifest

Post-burndown DiskANN reference numbers on the M5 Pro laptop.
Establishes a new M5 DiskANN baseline that subsequent Task 55 / Task
33 / Task 56 / Task 57 packets compare against.

DiskANN does not have a prior M5 baseline; this manifest is the
first.

## Head and host

| Field | Value |
| --- | --- |
| HEAD SHA | (filled by run) |
| Captured | 2026-05-23 (America/Los_Angeles) |
| Host | Peters-MBP (Apple Silicon M5 Pro, 64 GiB, macOS 26.4.1) |
| PostgreSQL | 18 (pgrx local install, socket `/Users/peter/.pgrx`, port 28818) |
| Extension build | `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` |

## Scope

DiskANN only. Corpora `ec_real_10k` (10k corpus / 200 queries) and
`ec_real_100k` (100k / 1000) from the local DBpedia/OpenAI3 fixtures
at `fixtures/m5_diskann_real{10k,100k}/`. 1536-dim, ip metric.

DiskANN scan sweep: `list_size ∈ {64, 128, 200, 400, 800}`, `k = 10`.
Build options at profile defaults (`graph_degree_r`, `alpha`,
`max_search_list_size`, etc. — emitted via `ec_diskann` profile).

## Re-run

```sh
/Users/peter/.cargo/bin/ecaz \
  --host /Users/peter/.pgrx --port 28818 --database tqvector_bench \
  bench suite run \
  --config benchmarks/task-55-m5-diskann-baseline/suite.json \
  --log-file benchmarks/task-55-m5-diskann-baseline/artifacts/suite-run.log
```

The suite expands to **8 steps** (`load`, `recall`, `latency`,
`storage` × 2 sizes).

## Artifacts

| Step | Log |
| --- | --- |
| load 10k DiskANN | `artifacts/corpus-load-ec_real_10k-diskann.log` |
| recall 10k DiskANN | `artifacts/recall-ec_real_10k-diskann.log` |
| latency 10k DiskANN | `artifacts/latency-ec_real_10k-diskann.log` |
| storage 10k DiskANN | `artifacts/storage-ec_real_10k-diskann.log` |
| load 100k DiskANN | `artifacts/corpus-load-ec_real_100k-diskann.log` |
| recall 100k DiskANN | `artifacts/recall-ec_real_100k-diskann.log` |
| latency 100k DiskANN | `artifacts/latency-ec_real_100k-diskann.log` |
| storage 100k DiskANN | `artifacts/storage-ec_real_100k-diskann.log` |
| Suite | `artifacts/results.jsonl`, `artifacts/suite-manifest.json`, `artifacts/suite-run.log` |
