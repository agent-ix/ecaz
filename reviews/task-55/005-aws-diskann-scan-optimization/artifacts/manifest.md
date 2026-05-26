# Task 55 Packet 005 Artifact Manifest

- head SHA at code optimization: `cbf037334ce0a9f499507d206049574b8278282e`
- benchmark evidence commit: `5499ebbff16b60930aee489500b9e06ff4a71117`
- task bucket: `reviews/task-55/005-aws-diskann-scan-optimization/`
- lane: AWS low-cost Graviton, `10k` profile, `m8g.large` database host
- fixture: DBpedia/OpenAI3 `ec_real_10k` and `ec_real_100k`
- storage format: `pq_fastscan`
- rerank mode: DiskANN default
- suite runner: `ecaz bench suite` via `ecaz cloud bench`
- isolated surface: one DiskANN index per corpus table
- timestamp: 2026-05-24

## Benchmark Packets

Before/config audit:

- path: `benchmarks/task55-aws-diskann-lowcost-config-audit/`
- command shape: `target/release/ecaz cloud bench --profile 10k --suite task55-aws-diskann-lowcost-config-audit --database postgres --config benchmarks/task55-aws-diskann-lowcost-config-audit/suite.json`
- key artifact: `benchmarks/task55-aws-diskann-lowcost-config-audit/artifacts/suite-manifest.json`
- status: 21/21 steps succeeded

After/optimized:

- path: `benchmarks/task55-aws-diskann-lowcost-optimized/`
- command shape: `target/release/ecaz cloud bench --profile 10k --suite task55-aws-diskann-lowcost-optimized --database postgres --config benchmarks/task55-aws-diskann-lowcost-optimized/suite.json --ecaz-bin /usr/local/bin/ecaz`
- synced artifact path: `benchmarks/task55-aws-diskann-lowcost-optimized/artifacts/s3-sync/20260524T165309Z/`
- key artifact: `benchmarks/task55-aws-diskann-lowcost-optimized/artifacts/s3-sync/20260524T165309Z/suite-manifest.json`
- status: 21/21 steps succeeded

## Key Results

100k latency mean before:

- list_size 64: `61.9 ms`
- list_size 128: `63.1 ms`
- list_size 200: `61.7 ms`
- list_size 400: `62.9 ms`
- list_size 800: `64.8 ms`

100k latency mean after:

- list_size 64: `1.72 ms`
- list_size 128: `2.60 ms`
- list_size 200: `3.49 ms`
- list_size 400: `5.88 ms`
- list_size 800: `10.6 ms`

100k recall@10 before and after:

- list_size 64: `0.9165`
- list_size 128: `0.9625`
- list_size 200: `0.9745`
- list_size 400: `0.9855`
- list_size 800: `0.9865`

Storage:

- 100k `ec_diskann` index before: `46.1 MiB` / `483.1 B` per row
- 100k `ec_diskann` index after: `46.1 MiB` / `483.1 B` per row

Planner/config audit:

- `planner_scan_enabled = t`
- planner gate reason: `planner scan selection is live: ec_diskann cost model active`
- effective `list_size = 200` from session override in explain step
- `storage_format = pq_fastscan`

## AWS State

The AWS `10k` profile was intentionally left running after the optimized run
for additional optimization cycles.
