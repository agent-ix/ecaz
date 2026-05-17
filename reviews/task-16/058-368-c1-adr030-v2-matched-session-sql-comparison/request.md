# Review Request: C1 ADR-030 V2 Matched-Session SQL Comparison

## Context

Earlier SQL-level comparisons between tqvector grouped-v2 and pgvector were
too harsh on tqvector because the session shape was not matched.

Packet `364` compared:

- tqvector through a per-query SQL launcher
- pgvector through a per-query SQL launcher

and concluded pgvector was faster at the SQL layer even though tqvector still
won in the direct harness.

Packets `366` and `367` then changed the diagnosis:

- heap fetch / simple projection were not the bottleneck
- the large tqvector “SQL gap” mostly disappeared once the SQL leg reused a
  backend session and used server-side timing

That made the next measurement question straightforward:

> if tqvector and pgvector are both timed with the same `per-cell plain-server`
> session shape, what does the SQL-level comparison actually look like?

## Problem

The branch had:

- tqvector direct-harness latency and recall from packets `362` and `367`
- pgvector direct-harness latency and recall from packet `363`
- an old SQL-level comparison from packet `364`

But packet `364` was no longer trustworthy after packet `367` showed the
measurement surface was mixing per-query overhead with per-cell internal
profiles.

## Planned Slice

Do one narrow measurement-only batch:

1. rerun the isolated tqvector grouped `m=16` lane with the verified launcher
   in `per-cell plain-server` mode
2. rerun the pgvector `m=16` lane on the same `50`-query subset with the same
   `per-cell plain-server` shape
3. compare the resulting SQL means cell-by-cell
4. line those results up with the existing direct-harness recall reads from
   packets `362` and `363`

## Implementation

No code changes in this packet.

This is a measurement / interpretation-only checkpoint using the already
landed harnesses:

- `scripts/bench_sql_latency_verified.sh`
- `scripts/bench_sql_latency_verified_scratch.sh`
- `scripts/bench_pgvector_sql_latency.sh`
- `scripts/bench_pgvector_sql_latency_scratch.sh`

## Validation

No new code landed in this packet.

The live measurement commands below both completed successfully on the scratch
cluster and wrote their summary files.

## Measurements

### tqvector grouped `m=16` SQL latency (`per-cell plain-server`)

Command:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix scratch_tqhnsw_real_50k_grouped_m16only \
  --m 16 \
  --corpus-table scratch_tqhnsw_real_50k_grouped_m16only_corpus \
  --query-table tqhnsw_real_50k_queries_50 \
  --index-name scratch_tqhnsw_real_50k_grouped_m16only_idx \
  --ef-search 40,64,128,320 \
  --query-limit 50 \
  --cache-state warm \
  --warmup-passes 1 \
  --session-mode per-cell \
  --timing-mode plain-server \
  --output /tmp/tqvector_grouped_m16only_sql_percell_plain_cmp.summary
```

Observed means:

| ef_search | tqvector SQL mean ms |
|----------:|---------------------:|
| 40  | `0.959` |
| 64  | `1.525` |
| 128 | `2.163` |
| 320 | `4.360` |

### pgvector `m=16` SQL latency (`per-cell plain-server`)

Command:

```bash
bash scripts/bench_pgvector_sql_latency_scratch.sh \
  --corpus-table pgvector_real_50k_corpus \
  --query-table tqhnsw_real_50k_queries_50 \
  --index-name pgvector_real_50k_m16_idx \
  --dim 1536 \
  --ef-search 40,64,128,320 \
  --query-limit 50 \
  --cache-state warm \
  --warmup-passes 1 \
  --session-mode per-cell \
  --timing-mode plain-server \
  --output /tmp/pgvector_real_50k_m16_sql_percell_plain_cmp.summary
```

Observed means:

| ef_search | pgvector SQL mean ms |
|----------:|---------------------:|
| 40  | `1.641` |
| 64  | `1.775` |
| 128 | `3.101` |
| 320 | `6.443` |

### Matched-session SQL comparison

With the same `per-cell plain-server` session shape, tqvector is faster at
every measured `ef_search` point:

| ef_search | tqvector mean ms | pgvector mean ms | pgvector / tqvector |
|----------:|-----------------:|-----------------:|--------------------:|
| 40  | `0.959` | `1.641` | `1.711x` |
| 64  | `1.525` | `1.775` | `1.164x` |
| 128 | `2.163` | `3.101` | `1.434x` |
| 320 | `4.360` | `6.443` | `1.478x` |

### Recall context from earlier packets

The recall tradeoff does **not** change here; this packet only corrects the
SQL timing surface.

From packet `362`, isolated grouped tqvector `m=16` on the same `50`-query
subset was:

| ef_search | tqvector Recall@10 |
|----------:|-------------------:|
| 40  | `0.900` |
| 64  | `0.930` |
| 128 | `0.936` |
| 320 | `0.938` |

From packet `363`, pgvector `m=16` on the same subset was:

| ef_search | pgvector Recall@10 |
|----------:|-------------------:|
| 40  | `0.986` |
| 64  | `0.992` |
| 128 | `0.998` |
| 320 | `0.998` |

So the corrected SQL read is:

- tqvector grouped-v2 `m=16` is faster through SQL on this lane
- pgvector remains much more accurate on the same query subset

## Interpretation

This packet supersedes packet `364`’s SQL-speed conclusion on the isolated
grouped-v2 lane.

The old read was:

- pgvector faster through SQL
- tqvector faster only in the direct harness

The corrected matched-session read is:

- tqvector is faster in the direct harness
- tqvector is also faster through SQL once both sides use the same
  `per-cell plain-server` measurement shape
- pgvector still wins decisively on recall

That is a more coherent operating-point story:

1. tqvector grouped-v2 is now a real latency-first lane, not just a direct
   harness artifact
2. pgvector remains the near-exact / quality-first baseline
3. the real product question is no longer “does tqvector’s SQL integration
   erase its scan-core win?” It does not, on the matched lane.

## Risk / Follow-up

This packet does not change product behavior, but it does change what the
branch should optimize next.

Immediate implications:

1. stop using the old per-query SQL comparison as evidence that tqvector loses
   its latency advantage at the SQL layer
2. when comparing tqvector and pgvector going forward, keep the session shape
   matched by default
3. focus the next runtime question on operating-point quality, not on a
   supposed SQL-layer regression

The next useful batch should likely quantify the corrected same-latency /
same-recall tradeoff explicitly, now that the SQL measurement surface is
honest. 
