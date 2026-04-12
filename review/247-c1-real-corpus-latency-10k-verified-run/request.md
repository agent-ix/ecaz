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

## Review Focus

- Is this the right measurement boundary for the first durable HNSW latency
  artifacts on the canonical real `10k` surface?
- Once the artifacts are captured, should this packet inline the full summary
  table, or just attach the artifact paths and a short operator digest?
