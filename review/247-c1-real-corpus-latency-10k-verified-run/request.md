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

Valid HNSW cells written to `/tmp/nfr1_real_10k_m8.summary` before the later
planner fallback was corrected:

```text
m=8   ef_search=40   n=200   p50=140.133ms p95=155.791ms p99=175.270ms mean=140.982ms min=122.753ms max=185.471ms server_qps=7.09 wall=29.15s
m=8   ef_search=64   n=200   p50=190.189ms p95=202.887ms p99=211.387ms mean=189.651ms min=169.224ms max=215.236ms server_qps=5.27 wall=37.65s
m=8   ef_search=100  n=200   p50=263.911ms p95=280.042ms p99=292.275ms mean=262.439ms min=234.273ms max=331.911ms server_qps=3.81 wall=52.18s
m=8   ef_search=128  n=200   p50=322.025ms p95=339.410ms p99=343.483ms mean=320.426ms min=282.956ms max=373.059ms server_qps=3.12 wall=63.78s
m=8   ef_search=160  n=200   p50=386.370ms p95=404.463ms p99=452.724ms mean=384.200ms min=345.348ms max=466.840ms server_qps=2.60 wall=75.32s
```

### Invalidated `ef_search=200` observation

Packet `249` followed up on the `ef_search=200` cliff and found the launcher
bug that caused it:

- the earlier verified launcher only checked one optimistic preflight before
  the sweep
- at `ef_search=200`, the planner actually flipped to `Sort -> Seq Scan`
- with `enable_seqscan = off`, the same `m=8, ef_search=200` query still ran
  as an index scan in about `429.576ms`

So the previously recorded `~6295ms` `ef_search=200` line was **not** a valid
HNSW latency artifact and should not be read as tqhnsw scan runtime.

### Current completion state

- `m=8, ef_search=40..160`: complete and valid
- `m=8, ef_search=200`: restored after packets `249` and `250`; full rerun now
  records `mean=454.021ms`, `p95=490.704ms`, `p99=560.238ms`, `wall=89.30s`
- `m=16` sweep: attempted, but blocked immediately by index selection

## Run Update: 2026-04-11 (planner-cost follow-up)

Packet `250` tuned the FR-020 CPU term so the representative real-`10k`
`m=8, ef_search=200` query now stays on the index:

- `tqhnsw_index_cost_snapshot('tqhnsw_real_10k_m8_idx')` at
  `SET tqhnsw.ef_search = 200` now reports `modeled_startup_cost = 1403.52`
- plain `EXPLAIN` again shows:

```text
Index Scan using tqhnsw_real_10k_m8_idx on tqhnsw_real_10k_corpus
```

- the verified scratch launcher now succeeds for a one-query smoke at
  `ef_search=200`:

```text
m=8   ef_search=200  n=1  p50=413.156ms mean=413.156ms wall=0.43s
```

That restores the planner side of the `ef_search=200` cell, but this packet
still needs the full 200-query rerun plus the pending `m=16` sweep before it
can serve as a durable C1 closeout.

## Run Update: 2026-04-11 (m16 launcher attempt)

With the `m=8, ef_search=200` cell repaired, the next step was the planned
verified `m=16` sweep on the shared canonical table:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 16 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m16.summary
```

The run aborted at the first cell exactly as the verified guard is supposed to:

- expected index: `tqhnsw_real_10k_m16_idx`
- actual planner choice at `ef_search=40`: `tqhnsw_real_10k_m8_idx`

Representative plan:

```text
Limit  (cost=302.72..336.58 rows=10 width=12)
  ->  Index Scan using tqhnsw_real_10k_m8_idx on tqhnsw_real_10k_corpus
```

That confirmed the remaining topology problem: on the shared canonical table,
the planner naturally prefers the cheaper `m=8` tqhnsw index.

## Run Update: 2026-04-11 (isolated m16 surface)

To measure `m=16` honestly without lying to the planner, the same staged TSVs
were loaded into an isolated one-index prefix:

```bash
./scripts/load_real_corpus_scratch.sh \
    --prefix tqhnsw_real_10k_m16only \
    --corpus-file /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_corpus.tsv \
    --queries-file /home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_queries.tsv \
    --m 16 \
    --allow-manifest-mismatch
```

That creates:

- `tqhnsw_real_10k_m16only_corpus`
- `tqhnsw_real_10k_m16only_queries`
- `tqhnsw_real_10k_m16only_m16_idx`

The verified sweep then ran against that isolated prefix:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k_m16only \
    --m 16 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m16only.summary
```

Completed cells:

```text
m=16  ef_search=40   n=200   p50=148.902ms p95=162.914ms p99=168.812ms mean=148.659ms min=130.837ms max=172.814ms server_qps=6.73 wall=30.64s
m=16  ef_search=64   n=200   p50=195.037ms p95=214.783ms p99=253.494ms mean=194.966ms min=165.693ms max=268.405ms server_qps=5.13 wall=38.67s
m=16  ef_search=100  n=200   p50=266.061ms p95=287.915ms p99=321.608ms mean=263.784ms min=221.468ms max=367.158ms server_qps=3.79 wall=52.42s
m=16  ef_search=128  n=200   p50=319.995ms p95=346.969ms p99=401.160ms mean=317.408ms min=264.918ms max=410.527ms server_qps=3.15 wall=63.17s
m=16  ef_search=160  n=200   p50=383.110ms p95=408.949ms p99=424.347ms mean=377.647ms min=327.830ms max=437.783ms server_qps=2.65 wall=75.23s
m=16  ef_search=200  n=200   p50=462.573ms p95=496.152ms p99=529.941ms mean=457.512ms min=380.804ms max=643.833ms server_qps=2.19 wall=89.95s
```

## Current read

Using the repaired and isolated measurement surfaces:

- `m=8` remains slightly faster at `ef_search=40`, `64`, `100`, and `200`
- `m=16` is effectively tied at `128`
- `m=16` is slightly faster at `160`

The difference is small enough that `m=16` does **not** currently look like a
clear latency win. The larger remaining opportunity is still raw scan runtime,
not “switch from `m=8` to `m=16`.”

## Interim Read

The repaired C1 surface now demonstrates the key unblock:

- runtime ordered tqhnsw scans execute successfully for the real `10k` corpus
- valid `m=8` HNSW latency is currently captured through `ef_search=160`
- the old `ef_search=200` cliff was a benchmark-integrity bug, not a proven
  runtime-collapse datapoint

The remaining tail issue is planner behavior, not harness ambiguity:

- `ef_search=160`: mean `384.200ms`
- `ef_search=200` with seqscan forced off: `429.576ms` on a representative
  query
- `ef_search=200` with the live planner: falls back to `Sort -> Seq Scan`

So C1 is now blocked by planner-cost crossover and performance follow-up, not
by runtime correctness or benchmark integrity.

The packet does **not** yet constitute a final `NFR-001` closeout because the
`m=16` run is still outstanding and the `m=8, ef_search=200` cell now needs
rerun under the stricter per-cell planner guard.

## Review Focus

- Is this the right measurement boundary for the first durable HNSW latency
  artifacts on the canonical real `10k` surface?
- Once the artifacts are captured, should this packet inline the full summary
  table, or just attach the artifact paths and a short operator digest?
