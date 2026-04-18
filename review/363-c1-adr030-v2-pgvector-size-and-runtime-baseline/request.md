# Review Request: C1 ADR-030 V2 pgvector Size And Runtime Baseline

## Context

The branch had converged on a credible grouped-v2 runtime lane:

- deterministic grouped graph builds from packet `361`
- canonical `50k` grouped-v2 `m=16` direct frontier from packet `362`
- grouped-v2 binary traversal remains materially faster than scalar tqvector,
  but trails scalar on recall

The remaining question from the user was the external baseline:

> are these tqvector indexes actually much smaller than an equivalent pgvector
> HNSW index, and what does the runtime comparison look like on the same real
> corpus?

The scratch cluster did not have `pgvector` installed, so the branch needed a
small reproducible install seam before it could answer that with measurements.

## Problem

The repo already contains storage expectations like “~9 tuples per page vs ~1
for pgvector,” but there was no live same-corpus `pg_relation_size(...)`
comparison on the current scratch cluster.

There was also no current pgvector runtime read on the same `50k` real-corpus
query subset, which meant the branch could not answer whether tqvector’s
latency/recall trade-off was merely internally consistent or actually
competitive against the obvious external baseline.

## Planned Slice

Batch the baseline setup and measurement together:

1. add a tiny script to install a local pgvector checkout into the same pg17
   scratch tree used by tqvector
2. create the `vector` extension in scratch
3. materialize a pgvector copy of the real `50k` corpus from the same raw
   `source` vectors already present in `tqhnsw_real_50k_corpus`
4. build pgvector HNSW indexes at `m=8` and `m=16` with the same
   `ef_construction=128`
5. compare index sizes directly against the existing tqvector indexes
6. probe pgvector `m=16` runtime on the same `50`-query subset
7. compare that read against the isolated tqvector grouped `m=16` lane

## Implementation

Updated:

- `scripts/install_pgvector_pg17_scratch.sh`

Concrete changes:

1. added a reproducible scratch install wrapper for a local pgvector checkout
   (`/home/peter/dev_bak/pgvector` by default)
2. the script installs pgvector into the same pg17 pgrx/Postgres tree already
   used by the scratch cluster:
   - `PG_CONFIG=/home/peter/.pgrx/17.9/pgrx-install/bin/pg_config`
3. the script does not download anything; it only builds and installs the local
   checkout if present

Scratch-only measurement setup:

1. created `vector` extension version `0.8.2`
2. created `pgvector_real_50k_corpus` from the same raw `source` vectors:
   - `select id, source::vector(1536) as embedding from tqhnsw_real_50k_corpus`
3. built:
   - `pgvector_real_50k_m8_idx`
   - `pgvector_real_50k_m16_idx`
4. created an isolated grouped tqvector copy for planner-unambiguous runtime
   checks:
   - `scratch_tqhnsw_real_50k_grouped_m16only_corpus`
   - `scratch_tqhnsw_real_50k_grouped_m16only_idx`

## Validation

Script validation:

- `bash -n scripts/install_pgvector_pg17_scratch.sh`

Install validation:

- `./scripts/install_pgvector_pg17_scratch.sh`
- `CREATE EXTENSION vector`
- `SELECT extname, extversion FROM pg_extension WHERE extname = 'vector'`

Observed installed extension:

- `vector 0.8.2`

## Measurements

### Same-corpus index sizes

All sizes below come from live `pg_relation_size(...)` on the current scratch
cluster.

| Index | Bytes | Size |
|------|------:|-----:|
| `pgvector_real_50k_m8_idx` | `409,608,192` | `391 MB` |
| `pgvector_real_50k_m16_idx` | `409,608,192` | `391 MB` |
| `tqhnsw_real_50k_m8_idx` | `59,711,488` | `57 MB` |
| `tqhnsw_real_50k_grouped_m8_idx` | `65,028,096` | `62 MB` |
| `tqhnsw_real_50k_m16_idx` | `68,280,320` | `65 MB` |
| `tqhnsw_real_50k_grouped_m16_idx` | `68,403,200` | `65 MB` |

Simple readout:

- pgvector `50k` HNSW is about `6x` larger than tqvector on this corpus
  surface
- against the current grouped-v2 `m=16` lane:
  - `391 MB / 65 MB ≈ 6.0x`

So the answer to the user’s size question is yes: not merely “somewhat
smaller,” but materially smaller in the measured database.

### pgvector `m=16` runtime on the 50-query subset

Using the same `tqhnsw_real_50k_queries_50` query subset and an exact
top-10 truth table built from brute-force pgvector scans:

| ef_search | Recall@10 | mean query latency ms |
|----------:|----------:|----------------------:|
| 40  | `0.9860` | `2.4338` |
| 64  | `0.9920` | `2.3840` |
| 128 | `0.9980` | `3.7039` |
| 320 | `0.9980` | `6.8600` |

### Isolated tqvector grouped `m=16` direct runtime

On the isolated grouped-only `m=16` tqvector table built from the same corpus:

| ef_search | Recall@10 | mean query latency ms |
|----------:|----------:|----------------------:|
| 40  | `0.9200` | `0.9811` |
| 64  | `0.9380` | `1.3198` |
| 128 | `0.9400` | `2.2270` |
| 320 | `0.9460` | `5.6927` |

Interpretation:

- tqvector grouped-v2 remains materially smaller on disk
- tqvector grouped-v2 also remains faster at the tested points
- but pgvector’s recall is much higher on the same query subset

Representative `m=16` comparison:

- tqvector grouped `ef=128`: `0.940 @ 2.227ms`
- pgvector `ef=128`: `0.998 @ 3.704ms`

That is a real trade-off, not a clean win:

- tqvector grouped: smaller and faster
- pgvector: more accurate

### tqvector SQL-planner caveat

I also tried to benchmark the isolated grouped `m=16` table through a raw SQL
`ORDER BY embedding <#> ... LIMIT 10` probe, but the plain planner path still
chose a sequential scan even with `enable_seqscan = off`.

The usable tqvector SQL timing surface therefore remains the existing verified
launcher (`scripts/bench_sql_latency_verified_scratch.sh`), which did succeed
on the isolated grouped table:

| ef_search | mean SQL latency ms |
|----------:|--------------------:|
| 40  | `4.569` |
| 64  | `5.446` |
| 128 | `7.225` |
| 320 | `11.249` |

That SQL surface is valid, but it is not methodologically identical to the
pgvector batch probe above, so the fairest direct comparison in this packet is
the same-corpus size baseline plus the direct-runtime readout.

## Risk / Follow-up

This packet answers the storage question cleanly:

- yes, tqvector is dramatically smaller than an equivalent pgvector HNSW index
  on the same real `50k` corpus

It also sharpens the runtime question:

- tqvector grouped-v2 is faster but lower-recall
- pgvector is slower but near-exact on this lane

The next useful follow-up is not more size work. It is one of:

1. build a proper pgvector latency harness that mirrors the tqvector verified
   SQL launcher more closely
2. improve the plain tqvector planner lane so raw SQL probes actually use the
   isolated grouped index without the special verified harness
3. make the product call explicitly: latency-first compressed ANN versus
   higher-recall full-vector ANN

