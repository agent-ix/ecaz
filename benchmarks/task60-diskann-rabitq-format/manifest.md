# Task 60 DiskANN RaBitQ Format Benchmark Packet

- task: `plan/tasks/60-ec-diskann-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- suite config: `benchmarks/task60-diskann-rabitq-format/suite.json`
- artifact directory: `benchmarks/task60-diskann-rabitq-format/artifacts/`

## Purpose

This packet is the `ecaz bench suite` measurement plan for the DiskANN
`storage_format` comparison required by Task 60. It keeps the existing
`pq_fastscan` baseline and the new `rabitq` format on the same benchmark host,
dataset, query set, breadth sweep, cache-state label, and PG18 socket.

## Matrix

| size | profile | storage formats | recall/latency sweep | cache state |
| --- | --- | --- | --- | --- |
| 100k | `ec_real_100k` | `pq_fastscan`, `rabitq` | `64,128,200,400,800` | `post_recall_warm` |
| 1M | `ec_real_ann_benchmarks_anchor` | `pq_fastscan`, `rabitq` | `64,128,200,400,800` | `post_recall_warm` |

The 1M rows prepare the same DBpedia-derived anchor fixture used by prior
DiskANN final measurements, but stage it under
`/var/lib/pgsql/18/datasets/staged-task60-diskann-rabitq/` so `ecaz bench
suite audit` can verify the full fetch -> prepare -> load dependency chain.

## Acceptance Checks

- 100k recall parity: compare the best comparable recall rows for
  `pq_fastscan` and `rabitq`; record the measured delta in this packet.
- Storage assertion: record `ecaz bench storage` output for all four prefixes.
- 1M shipping gate: `rabitq` should be at least 30% smaller than
  `pq_fastscan`, or the packet must document a not-worth-shipping decision.
- Host parity: preserve `precheck-host.log` with PostgreSQL settings, CPU, and
  memory details.

## Commands

Dry-run validation:

```sh
cargo run -p ecaz-cli -- \
  --log-file benchmarks/task60-diskann-rabitq-format/artifacts/suite-dry-run.log \
  bench suite run \
  --config benchmarks/task60-diskann-rabitq-format/suite.json \
  --dry-run \
  --manifest-output benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json
```

Full run on the benchmark host:

```sh
cargo run -p ecaz-cli -- \
  --log-file benchmarks/task60-diskann-rabitq-format/artifacts/suite-run.log \
  bench suite run \
  --config benchmarks/task60-diskann-rabitq-format/suite.json \
  --manifest-output benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json
```

Report extraction after the full run:

```sh
cargo run -p ecaz-cli -- \
  --log-file benchmarks/task60-diskann-rabitq-format/artifacts/suite-report.log \
  bench suite report \
  --manifest benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json \
  --results-output benchmarks/task60-diskann-rabitq-format/artifacts/results-report.jsonl
```

Final packet evidence should include:

- `artifacts/suite-run.log`
- `artifacts/suite-report.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl` from `suite run`
- `artifacts/results-report.jsonl` from `suite report`
- the four storage logs for 100k and 1M
- the four recall logs for 100k and 1M
- the four latency logs for 100k and 1M
- `artifacts/precheck-host.log`

The 1M shipping decision is recorded manually from the storage rows. Use the
`storage_index` rows for `storage-1m-diskann-rabitq` and
`storage-1m-diskann-pq-fastscan` in `artifacts/results-report.jsonl`, filtered
to `access method=ec_diskann`, and calculate:
`1 - (rabitq size_bytes / pq_fastscan size_bytes)`. The underlying storage logs
remain the durable source artifacts. If the calculated value is below `0.30`,
the packet records "not worth shipping" instead of treating the format as
accepted.

## Current State

The suite is scaffolded and locally dry-run validated. Measurement artifacts
are intentionally pending until the branch is installed on a PG18 benchmark host
with the staged DBpedia fixtures.
