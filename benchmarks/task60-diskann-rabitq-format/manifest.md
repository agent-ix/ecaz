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
cargo run -p ecaz-cli -- bench suite run \
  --config benchmarks/task60-diskann-rabitq-format/suite.json \
  --dry-run \
  --manifest-output benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json
```

Full run on the benchmark host:

```sh
cargo run -p ecaz-cli -- bench suite run \
  --config benchmarks/task60-diskann-rabitq-format/suite.json \
  --manifest-output benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json
```

Report extraction after the full run:

```sh
cargo run -p ecaz-cli -- bench suite report \
  --manifest benchmarks/task60-diskann-rabitq-format/artifacts/suite-manifest.json \
  --results-output benchmarks/task60-diskann-rabitq-format/artifacts/results.jsonl
```

## Current State

The suite is scaffolded and locally dry-run validated. Measurement artifacts
are intentionally pending until the branch is installed on a PG18 benchmark host
with the staged DBpedia fixtures.
