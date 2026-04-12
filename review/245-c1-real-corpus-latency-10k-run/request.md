# Review Request: C1 Real-Corpus Latency 10k Capture

## Context

Branch:
- `main`

Prior C1 packet:
- `review/244-c1-real-corpus-latency-hardening/request.md`

This packet was opened as a live draft before the long-running measurement
finished, so the current C1 benchmark slice had a durable review surface while
the operator run was in flight.

The hardened real-corpus latency path from packet `244` is now landed on
`main`, and the local host has the canonical staged DBpedia-derived fixtures:

- `/home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_corpus.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_queries.tsv`
- `/home/peter/dev/datasets/tqhnsw_real_10k/tqhnsw_real_10k_manifest.json`

## What This Slice Does

1. start the local `pg17` pgrx cluster on port `28817`
2. create `tqvector` in `postgres`
3. load the canonical `tqhnsw_real_10k` corpus/query tables
4. build:
   - `tqhnsw_real_10k_m8_idx`
   - `tqhnsw_real_10k_m16_idx`
5. attempt the hardened `NFR-001` latency sweep:

```bash
PGHOST=/home/peter/.pgrx \
PGPORT=28817 \
PGDATABASE=postgres \
TQV_PSQL_BIN=/home/peter/.pgrx/17.9/pgrx-install/bin/psql \
bash scripts/bench_sql_latency.sh \
    --prefix tqhnsw_real_10k \
    --m 8 --m 16 \
    --ef-search 40,64,100,128,160,200 \
    --cache-state cold \
    --output /tmp/nfr1_real_10k.summary > /tmp/nfr1_real_10k.stdout
```

## Measurement Artifacts

Expected artifacts for this packet:

- `/tmp/nfr1_real_10k.stdout`
- `/tmp/nfr1_real_10k.summary`

The stdout artifact should now include:

- host OS
- CPU model
- RAM
- `shared_buffers`
- `work_mem`
- `max_parallel_workers_per_gather`
- explicit cache-state label

Per-cell summary lines should now report:

- `p50`
- `p95`
- `p99`
- `mean`
- `min`
- `max`
- `server_qps`
- `wall`

## Status

Initial operator progress:

- cluster start: complete
- extension create: complete
- `tqhnsw_real_10k` load: complete
- `m=8` / `m=16` index build: complete
- latency sweep: attempted, then aborted

The stdout banner has already been verified to contain the expected host / GUC
metadata. The sweep was then started on the first matrix cell:

- `m=8`
- `ef_search=40`

No per-cell summary line was emitted before the operator check flagged that the
run was taking unexpectedly long. A representative `EXPLAIN` then confirmed the
root cause:

```text
Limit
  ->  Sort
        Sort Key: (embedding <#> ...)
        ->  Seq Scan on tqhnsw_real_10k_corpus
```

So this attempted run was **not** a valid HNSW latency artifact. It was stopped
intentionally instead of letting it continue to time the wrong surface.

At the time of abort:

- `/tmp/nfr1_real_10k.stdout` existed and contained the expected banner
- `/tmp/nfr1_real_10k.summary` had not yet been created

This packet therefore records the operator attempt and the discovery that the
current launcher would silently benchmark `Sort -> Seq Scan` on `main`.
The matching fix lands in:

- `review/246-c1-latency-launcher-plan-verification/request.md`

## Review Focus

- Does this packet capture the right durable measurement boundary for the first
  post-A6 `NFR-001` artifact on the canonical real `10k` surface?
- Is the aborted operator attempt, plus the captured banner-only stdout and the
  representative `Sort -> Seq Scan` plan, sufficient evidence for why this did
  not become a durable latency artifact?
