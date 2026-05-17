---
id: NFR-012
title: Cloud Read QPS and Live Write Throughput Targets
type: non-functional-requirement
artifact_type: NFR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/StR-007"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-044"
    type: "constrains"
    cardinality: "1:1"
---
# NFR-012: Cloud Read QPS and Live Write Throughput Targets

## Requirement

Cloud benchmark runs SHALL produce read-QPS and write-throughput
artifacts comparable across profiles, against documented targets, so
that "ecaz fits in one DB" or "ecaz needs SPIRE distribution" is an
evidence-based statement rather than a vendor-paper extrapolation.

## Read QPS Targets

Single-connection IVF + RaBitQ, nprobe=10, warm cache, no intra-query
parallelism. Targets are conservative — beat-the-target is the win
condition, not match-the-target.

| Profile | Cache state | per-query | QPS target |
|---|---|---|---|
| `1m`   | resident       | ≤ 100 µs   | ≥ 30k |
| `10m`  | resident       | ≤ 1 ms     | ≥ 15k |
| `100m` | resident       | ≤ 3 ms     | ≥ 10k |
| `1b`   | half-resident  | ≤ 10 ms    | ≥ 1.5k |
| `5b`   | NVMe-spillover | ≤ 30 ms    | ≥ 500 |
| `10b`  | NVMe-spillover | ≤ 60 ms    | ≥ 250 |

For nprobe=100 (high-recall mode), targets divide by ~10×; recorded
separately as `qps_high_recall`.

## Write Throughput Targets

Live insert throughput (post-build, with index updates), measured via
a sustained INSERT or COPY load against an already-built index.

| Profile | Single-row INSERT | COPY (8-way) | WAL ceiling |
|---|---|---|---|
| `1m`–`10m` | ≥ 3k/s | ≥ 300k/s | ~80k/s |
| `100m`–`1b` | ≥ 3k/s | ≥ 200k/s | ~80k/s |

The WAL ceiling is reported, not enforced; it identifies when SPIRE
coordinator-routed DML and placement-aware distribution become the only path to
higher write rates (`FR-059`).

## Distributed Targets (Future)

For SPIRE distributed runs using `EcSpireDistributedScan` (`FR-058`) and the
production remote executor (`FR-057`), the harness SHALL also produce these
comparison artifacts:

| Setup | Read QPS expectation vs single-node | Write throughput |
|---|---|---|
| Sharded N nodes | ~(N × single) / fan_out + coordinator_cost | ~N × single |
| Replicated N nodes | ~N × single | ~single |

The first run that emits both `1b` single-node and `1b × 3 sharded`
results closes a fundamental design question: is CustomScan remote-executor
coordination overhead small enough that sharding wins for read latency, or
only for write throughput.

## Acceptance Criteria

### NFR-012-AC-1

`ecaz cloud bench` artifacts include `read_qps.json` recording
`p50_us`, `p99_us`, `qps`, `nprobe`, `concurrency`, `cache_state`.

### NFR-012-AC-2

`ecaz cloud bench` artifacts include `write_throughput.json`
recording `single_row_per_sec`, `copy_rows_per_sec`, `wal_bytes_per_sec`.

### NFR-012-AC-3

When more than one profile run's artifacts exist in S3, the harness
SHALL emit a `comparison.md` cross-tabulating QPS and write
throughput against targets above.

### NFR-012-AC-4

Distributed runs SHALL emit `coordinator_overhead_ms` (libpq
round-trip + merge) as a separate field in `read_qps.json` so the
"sharding wins?" question is answerable from artifacts alone.
