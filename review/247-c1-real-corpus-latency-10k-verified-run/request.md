# Review Request: C1 Real-Corpus Latency 10k Verified Run

## Context

Branch:
- `main`

Prior packets:
- `review/245-c1-real-corpus-latency-10k-run/request.md`
- `review/246-c1-latency-launcher-plan-verification/request.md`

Packet `245` recorded the first operator attempt and the discovery that the
then-current launcher would silently benchmark `Sort -> Seq Scan`.

Packet `246` added the planner-verified launcher and updated the C1 docs/status
surfaces so they no longer claimed HNSW latency capture was already available
on `main`.

Since then:

- `origin/main` has been merged locally
- the merged planner code has been installed into the local `pg17` pgrx setup
- a representative real `10k` EXPLAIN now routes through the expected tqhnsw
  index:

```text
Index Scan using tqhnsw_real_10k_m8_idx on tqhnsw_real_10k_corpus
  Order By: (embedding <#> ...)
```

This packet is opened before the long verified run starts so the measurement
slice has a durable review surface while the artifacts are being collected.

## Fixture

Local staged dataset:

- `/home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_corpus.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_queries.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_manifest.json`

Loaded relations:

- `tqhnsw_real_10k_corpus`
- `tqhnsw_real_10k_queries`
- `tqhnsw_real_10k_m8_idx`
- `tqhnsw_real_10k_m16_idx`

Environment:

- `PGHOST=/home/peter/.pgrx`
- `PGPORT=28817`
- `PGDATABASE=postgres`
- `TQV_PSQL_BIN=/home/peter/.pgrx/17.9/pgrx-install/bin/psql`

## Planned Runs

The verified launcher requires one effective `m` per invocation, so this packet
will record two runs:

### 1. `m=8`

```bash
bash scripts/bench_sql_latency_verified.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8.summary > /tmp/nfr1_real_10k_m8.stdout
```

### 2. `m=16`

```bash
bash scripts/bench_sql_latency_verified.sh \
    --prefix tqhnsw_real_10k \
    --m 16 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m16.summary > /tmp/nfr1_real_10k_m16.stdout
```

## Expected Artifacts

- `/tmp/nfr1_real_10k_m8.stdout`
- `/tmp/nfr1_real_10k_m8.summary`
- `/tmp/nfr1_real_10k_m16.stdout`
- `/tmp/nfr1_real_10k_m16.summary`

## Status

At packet-open time:

- planner-verified launcher: landed on `main`
- local planner preflight: passes for `tqhnsw_real_10k_m8_idx`
- verified benchmark runs: pending

The measured summary lines and the read against `NFR-001` will be added to this
packet once the runs finish.

## Run Update: 2026-04-11

Since packet-open:

- packet `248` landed the runtime fix that allows planner-routed ordered scans
  to execute when PostgreSQL passes a non-null zero-qual key buffer into
  `amrescan`
- the verified scratch launcher also now auto-detects the active local pg17
  socket between `/tmp/tqvector_pgrx_home` and `${HOME}/.pgrx`

### Observed `m=8` results

Command used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8.summary
```

Representative preflight:

```text
[verified] representative EXPLAIN uses tqhnsw_real_10k_m8_idx
```

Completed cells written to `/tmp/nfr1_real_10k_m8.summary`:

```text
m=8   ef_search=40   n=200   p50=140.133ms p95=155.791ms p99=175.270ms mean=140.982ms min=122.753ms max=185.471ms server_qps=7.09 wall=29.15s
m=8   ef_search=64   n=200   p50=190.189ms p95=202.887ms p99=211.387ms mean=189.651ms min=169.224ms max=215.236ms server_qps=5.27 wall=37.65s
m=8   ef_search=100  n=200   p50=263.911ms p95=280.042ms p99=292.275ms mean=262.439ms min=234.273ms max=331.911ms server_qps=3.81 wall=52.18s
m=8   ef_search=128  n=200   p50=322.025ms p95=339.410ms p99=343.483ms mean=320.426ms min=282.956ms max=373.059ms server_qps=3.12 wall=63.78s
m=8   ef_search=160  n=200   p50=386.370ms p95=404.463ms p99=452.724ms mean=384.200ms min=345.348ms max=466.840ms server_qps=2.60 wall=75.32s
m=8   ef_search=200  n=200   p50=6276.095ms p95=6481.173ms p99=6862.115ms mean=6295.632ms min=6044.156ms max=8153.601ms server_qps=0.16 wall=1209.01s
```

### Current completion state

- `m=8, ef_search=40..200`: complete
- `m=16` sweep: not started yet

## Interim Read

The repaired C1 surface now demonstrates the key unblock:

- planner verification passes against the expected tqhnsw index
- runtime ordered tqhnsw scans execute successfully for the real `10k` corpus
- the observed `m=8` latency curve is monotonic, but the `ef_search=200`
  point is dramatically worse than the rest of the surface

That tail is not a minor regression. It is a major behavior change:

- `ef_search=160`: mean `384.200ms`
- `ef_search=200`: mean `6295.632ms`

So C1 is now blocked by performance, not by planner/runtime correctness.

The packet does **not** yet constitute a final `NFR-001` closeout because the
`m=16` run is still outstanding and the `m=8, ef_search=200` tail now needs
follow-up analysis rather than just more collection.

## Review Focus

- Is this the right measurement boundary for the first durable HNSW latency
  artifacts on the canonical real `10k` surface?
- Once the artifacts are captured, should this packet inline the full summary
  table, or just attach the artifact paths and a short operator digest?
