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

## Run Update: 2026-04-12 (score-cache rerun planned)

Packet `254` landed and pushed a scan-local ordered-scan score cache. On the
same representative real `10k` query (`id=10000`, `m=8`), that slice already
shifted the hot path materially:

- `ef_search=40`: representative SQL `Execution Time` dropped
  `126.200ms -> 95.759ms`
- `ef_search=200`: representative SQL `Execution Time` dropped
  `418.902ms -> 186.563ms`

That is large enough that the previously recorded C1 summary lines are now
stale as a performance read. The next active step is to rerun the verified
surface on top of the score-cache checkpoint.

Planned rerun commands:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8_scorecache.summary

scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k_m16only \
    --m 16 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m16only_scorecache.summary
```

Expected artifacts for this rerun:

- `/tmp/nfr1_real_10k_m8_scorecache.summary`
- `/tmp/nfr1_real_10k_m16only_scorecache.summary`

The completed summary lines and revised C1 read will be appended here once the
rerun finishes.

## Run Update: 2026-04-12 (score-cache rerun complete)

The verified rerun completed on top of packet `254`'s score-cache checkpoint.

### Canonical `m=8` rerun

Command used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8_scorecache.summary
```

Completed cells:

```text
m=8   ef_search=40   n=200   p50=88.588ms  p95=98.193ms  p99=103.261ms mean=89.089ms  min=76.797ms  max=106.311ms server_qps=11.22 wall=19.96s
m=8   ef_search=64   n=200   p50=106.081ms p95=122.159ms p99=131.747ms mean=106.258ms min=91.629ms  max=153.262ms server_qps=9.41  wall=22.01s
m=8   ef_search=100  n=200   p50=125.762ms p95=141.822ms p99=153.593ms mean=125.833ms min=106.770ms max=158.239ms server_qps=7.95  wall=25.91s
m=8   ef_search=128  n=200   p50=141.625ms p95=164.515ms p99=173.066ms mean=141.723ms min=116.648ms max=182.332ms server_qps=7.06  wall=29.11s
m=8   ef_search=160  n=200   p50=156.947ms p95=176.708ms p99=190.143ms mean=156.208ms min=128.881ms max=200.697ms server_qps=6.40  wall=31.99s
m=8   ef_search=200  n=200   p50=173.301ms p95=204.956ms p99=234.492ms mean=173.680ms min=140.074ms max=265.483ms server_qps=5.76  wall=35.49s
```

### Isolated `m=16` rerun

Command used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k_m16only \
    --m 16 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m16only_scorecache.summary
```

Completed cells:

```text
m=16  ef_search=40   n=200   p50=88.795ms  p95=101.055ms p99=109.004ms mean=89.637ms  min=74.567ms  max=118.637ms server_qps=11.16 wall=20.03s
m=16  ef_search=64   n=200   p50=105.408ms p95=123.099ms p99=130.242ms mean=106.521ms min=87.499ms  max=146.175ms server_qps=9.39  wall=22.06s
m=16  ef_search=100  n=200   p50=127.171ms p95=143.737ms p99=152.691ms mean=127.250ms min=106.440ms max=162.147ms server_qps=7.86  wall=26.17s
m=16  ef_search=128  n=200   p50=142.072ms p95=164.259ms p99=176.806ms mean=142.539ms min=117.579ms max=185.011ms server_qps=7.02  wall=29.25s
m=16  ef_search=160  n=200   p50=157.133ms p95=177.546ms p99=188.151ms mean=156.759ms min=127.955ms max=205.981ms server_qps=6.38  wall=32.08s
m=16  ef_search=200  n=200   p50=174.553ms p95=202.650ms p99=215.789ms mean=174.326ms min=141.134ms max=226.704ms server_qps=5.74  wall=34.19s
```

### Artifact paths

- `/tmp/nfr1_real_10k_m8_scorecache.summary`
- `/tmp/nfr1_real_10k_m16only_scorecache.summary`

## Revised C1 read

The score-cache checkpoint materially changed the C1 latency surface.

Compared to the prior verified `m=8` surface on the same lane:

- `ef_search=40`: mean `140.982ms -> 89.089ms`
- `ef_search=64`: mean `189.651ms -> 106.258ms`
- `ef_search=100`: mean `262.439ms -> 125.833ms`
- `ef_search=128`: mean `320.426ms -> 141.723ms`
- `ef_search=160`: mean `384.200ms -> 156.208ms`
- `ef_search=200`: mean `454.021ms -> 173.680ms`

So the new checkpoint cut the canonical `m=8` mean surface by roughly
`37% -> 62%`, depending on `ef_search`.

Across the rerun itself, `m=8` and isolated `m=16` are now effectively tied.
`m=8` is still slightly faster at every measured `ef_search`, but the gap is
well under `2ms` mean at every cell.

Against `NFR-001`, this is still not a closeout:

- requirement baseline: `p50 < 5ms` for `m=8, ef_search=40`
- measured baseline: `p50 = 88.588ms`, `p99 = 103.261ms`

So the runtime is now dramatically better and the artifacts are durable, but
the latency target is still missed by a wide margin. C1 has moved from
"runtime and planner integrity bring-up" into a pure optimization/readout lane.

## Run Update: 2026-04-12 (greedy upper-layer seeding)

Packet `256` replaced scan-time upper-layer result-window seeding with greedy
descent. The change stayed within the scan runtime and passed the required
checkpoint gate:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

The verified canonical `m=8` rerun on the shared real-`10k` prefix used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8_greedy_upper.summary
```

Completed cells:

```text
m=8   ef_search=40   n=200   p50=69.562ms  p95=79.380ms  p99=83.530ms  mean=69.855ms  min=55.491ms  max=84.006ms  server_qps=14.32 wall=14.71s
m=8   ef_search=64   n=200   p50=79.898ms  p95=90.619ms  p99=98.609ms  mean=79.753ms  min=62.327ms  max=104.349ms server_qps=12.54 wall=16.71s
m=8   ef_search=100  n=200   p50=92.714ms  p95=106.607ms p99=116.384ms mean=92.465ms  min=71.284ms  max=117.682ms server_qps=10.81 wall=20.62s
m=8   ef_search=128  n=200   p50=102.767ms p95=114.942ms p99=119.196ms mean=101.467ms min=79.701ms  max=134.937ms server_qps=9.86  wall=21.02s
m=8   ef_search=160  n=200   p50=113.216ms p95=129.379ms p99=141.886ms mean=112.132ms min=87.779ms  max=162.703ms server_qps=8.92  wall=23.18s
m=8   ef_search=200  n=200   p50=126.960ms p95=145.479ms p99=150.469ms mean=124.238ms min=93.581ms  max=156.663ms server_qps=8.05  wall=25.57s
```

Compared to the post-fast-hash canonical `m=8` surface from packet `255`:

- `ef_search=40`: mean `88.360ms -> 69.855ms`
- `ef_search=64`: mean `103.883ms -> 79.753ms`
- `ef_search=100`: mean `125.027ms -> 92.465ms`
- `ef_search=128`: mean `139.747ms -> 101.467ms`
- `ef_search=160`: mean `153.566ms -> 112.132ms`
- `ef_search=200`: mean `174.147ms -> 124.238ms`

That makes this the current best verified canonical `m=8` surface on `main`.

The isolated `m=16` surface has not yet been rerun after packets `255` and
`256`, so the newest apples-to-apples verified artifact is still the shared
canonical `m=8` lane above. The next honest comparison step would be an
isolated rerun of `tqhnsw_real_10k_m16only` on the current code, but the
larger remaining latency gap is still on raw runtime, not `m` selection.

## Run Update: 2026-04-12 (QJL-disabled 4-bit score fast path)

Packet `257` kept the graph runtime unchanged and instead optimized the
QJL-disabled `bits = 4` score path used by the real-corpus 1536-dim lane. The
change passed the required checkpoint gate:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

The verified canonical `m=8` rerun used:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
    --prefix tqhnsw_real_10k \
    --m 8 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k_m8_noqjl_score.summary
```

Completed cells:

```text
m=8   ef_search=40   n=200   p50=50.283ms p95=53.238ms  p99=55.862ms  mean=50.521ms min=46.541ms max=60.999ms  server_qps=19.79 wall=10.83s
m=8   ef_search=64   n=200   p50=54.331ms p95=91.486ms  p99=104.949ms mean=57.345ms min=48.861ms max=123.289ms server_qps=17.44 wall=13.77s
m=8   ef_search=100  n=200   p50=57.820ms p95=63.342ms  p99=70.617ms  mean=57.997ms min=51.008ms max=75.340ms  server_qps=17.24 wall=12.35s
m=8   ef_search=128  n=200   p50=60.312ms p95=64.450ms  p99=66.955ms  mean=60.150ms min=54.430ms max=69.580ms  server_qps=16.63 wall=14.13s
m=8   ef_search=160  n=200   p50=63.722ms p95=68.823ms  p99=70.548ms  mean=63.575ms min=56.444ms max=75.351ms  server_qps=15.73 wall=13.46s
m=8   ef_search=200  n=200   p50=68.254ms p95=77.307ms  p99=83.182ms  mean=68.260ms min=58.277ms max=85.272ms  server_qps=14.65 wall=15.80s
```

Compared to the post-greedy canonical `m=8` surface from packet `256`:

- `ef_search=40`: mean `69.855ms -> 50.521ms`
- `ef_search=64`: mean `79.753ms -> 57.345ms`
- `ef_search=100`: mean `92.465ms -> 57.997ms`
- `ef_search=128`: mean `101.467ms -> 60.150ms`
- `ef_search=160`: mean `112.132ms -> 63.575ms`
- `ef_search=200`: mean `124.238ms -> 68.260ms`

That makes this the new best verified canonical `m=8` surface on `main`.

Even after this step-change, `NFR-001` still remains open by a wide margin:

- requirement baseline: `p50 < 5ms` at `m=8, ef_search=40`
- current measured baseline: `p50 = 50.283ms`, `p99 = 55.862ms`

So C1 is still an optimization lane, but the bottleneck mix has shifted again:
the 4-bit score path is no longer the clean first target it was before packet
`257`.

## Run Update: 2026-04-12 (verified warm-cache seam)

Packet `261` extended the verified launcher so warm-cache runs can prime the
query set and, critically, keep each `(m, ef_search)` cell inside one backend
session:

- `--warmup-passes N`
- `--session-mode per-query|per-cell`

The planner/index guard still runs before any warmup or timing, so this warm
surface remains a verified tqhnsw artifact rather than a forced-plan seam.

Representative `tqhnsw_real_10k`, `m=8`, `ef_search=40` results:

```text
per-query  warmup=1: p50=50.246ms p95=53.903ms p99=57.319ms mean=50.496ms
per-cell   warmup=1: p50=15.883ms p95=24.149ms p99=32.691ms mean=16.843ms
per-cell   warmup=3: p50=14.315ms p95=16.350ms p99=17.613ms mean=14.194ms
```

That establishes two things:

1. backend/session reuse matters materially for honest warm-cache reporting
2. even the stronger warm per-cell surface still misses `NFR-001`

So the corrected C1 read is no longer “warm is already solved.” The more
accurate read is:

- canonical cold verified `10K` surface: still far above target
- verified warm per-cell `10K` surface: much better, but still about `3x`
  above the `p50 < 5ms` target and slightly above the `p99 < 15ms` target
