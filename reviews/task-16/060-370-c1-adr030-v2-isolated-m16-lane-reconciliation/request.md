# Review Request: C1 ADR-030 V2 Isolated M16 Lane Reconciliation

## Context

Packets `368` and `369` used the isolated grouped-only `m=16` SQL lane:

- corpus table: `scratch_tqhnsw_real_50k_grouped_m16only_corpus`
- index: `scratch_tqhnsw_real_50k_grouped_m16only_idx`

but packet `368` cited the recall table from packet `362`, which was a
different lane:

- corpus table: `tqhnsw_real_50k_corpus`
- index: `tqhnsw_real_50k_grouped_m16_idx`

The reviewer correctly called out that this made the packet `368/369`
operating-point story unsafe. The SQL comparison also needed percentile and
repeatability support instead of a single means-only table.

## Problem

There were three open reviewer concerns across packets `367-369`:

1. reconcile the `m=16` grouped tqvector recall identity
   - packet `362` canonical grouped `m=16`: `0.900 / 0.930 / 0.936 / 0.938`
   - packet `363` isolated grouped-only `m=16`: `0.920 / 0.938 / 0.940 / 0.946`
2. extend the isolated tqvector sweep beyond `ef=320` so the ceiling is
   measured rather than inferred
3. rerun the matched-session SQL comparison with percentiles and repeatability
   before making another operating-point claim

## Planned Slice

Measurement-only response packet. No repository code changes.

1. verify the ADR-030 grouped runtime settings still match the isolated lane
   under discussion
2. rerun direct grouped tqvector `m=16` recall/latency sweeps on both:
   - the canonical grouped table from packet `362`
   - the isolated grouped-only table from packets `363/368/369`
3. extend the isolated grouped tqvector sweep to `ef=512,768,1000`
4. rerun the matched-session SQL comparison twice per side in
   `per-cell plain-server` mode and carry p50/p95 alongside mean
5. capture representative `EXPLAIN` plans so both AMs are visibly index scans

## Implementation

No code changes in this packet.

This is a measurement / interpretation-only checkpoint using existing surfaces:

- `tests.tqhnsw_debug_adr030_runtime_settings()`
- `tests.tqhnsw_graph_scan_recall_ef_sweep(...)`
- `scripts/bench_sql_latency_verified_scratch.sh`
- `scripts/bench_pgvector_sql_latency_scratch.sh`

## Validation

Scratch runtime settings at measurement time:

| grouped_build_enabled | grouped_scan_enabled | grouped_scan_window | grouped_scan_score_mode | grouped_exact_traversal_enabled |
|----------------------:|---------------------:|--------------------:|------------------------:|--------------------------------:|
| `true` | `true` | `64` | `binary` | `false` |

Live index identity check:

- isolated grouped tqvector index present:
  - `scratch_tqhnsw_real_50k_grouped_m16only_idx`
- canonical grouped tqvector index present:
  - `tqhnsw_real_50k_grouped_m16_idx`
- pgvector baseline index present:
  - `pgvector_real_50k_m16_idx`

Representative planner verification at `ef_search = 128`:

- tqvector isolated lane:
  - `Index Scan using scratch_tqhnsw_real_50k_grouped_m16only_idx on scratch_tqhnsw_real_50k_grouped_m16only_corpus`
- pgvector lane:
  - `Index Scan using pgvector_real_50k_m16_idx on pgvector_real_50k_corpus`

The matched-session launcher reruns also emitted the same per-cell
`[verified] planner uses ...` checks before timing.

## Measurements

### 1. Canonical vs isolated grouped `m=16` are different lanes

Live reruns on the same scratch cluster reproduce the two previously cited
tables exactly enough to establish lane identity rather than measurement drift:

| ef_search | canonical grouped `m=16` Recall@10 | isolated grouped-only `m=16` Recall@10 |
|----------:|-----------------------------------:|---------------------------------------:|
| 40  | `0.900` | `0.920` |
| 64  | `0.930` | `0.938` |
| 128 | `0.936` | `0.940` |
| 320 | `0.938` | `0.946` |

So the packet `362` and packet `363` recall tables were both real, but they
were never the same lane:

- packet `362` = canonical grouped `m=16`
- packets `363/368/369` = isolated grouped-only `m=16`

That is the reconciliation. Packet `368` should not have cited packet `362` as
the recall surface for the isolated SQL lane.

### 2. Isolated grouped tqvector `m=16` ceiling through `ef=1000`

Direct rerun on the isolated grouped-only lane:

| ef_search | Recall@10 | exact-quantized Recall@10 | mean query latency ms |
|----------:|----------:|--------------------------:|----------------------:|
| 40   | `0.920` | `0.920` | `1.153` |
| 64   | `0.938` | `0.920` | `1.441` |
| 128  | `0.940` | `0.920` | `2.435` |
| 320  | `0.946` | `0.920` | `5.551` |
| 512  | `0.950` | `0.920` | `8.739` |
| 768  | `0.950` | `0.920` | `14.034` |
| 1000 | `0.950` | `0.920` | `18.853` |

This closes the packet `369` ceiling gap empirically:

- the isolated tqvector `m=16` lane does improve beyond `ef=320`
- but it plateaus at `0.950` by `ef=512`
- that is still well below pgvector `m=16`'s measured floor from packet `363`
  (`0.986 @ ef=40`)

So the stronger version of the earlier claim is now supported:

- on this isolated `50k` / `m=16` / `50`-query lane, tqvector does **not**
  reach pgvector’s measured recall floor even when widened to `ef=1000`

### 3. Matched-session SQL reruns with percentiles

Both sides were rerun twice in `per-cell plain-server` mode on the same
`tqhnsw_real_50k_queries_50` subset.

#### tqvector isolated grouped-only `m=16`

Run 1:

| ef_search | p50 ms | p95 ms | mean ms |
|----------:|-------:|-------:|--------:|
| 40  | `1.037` | `1.556` | `1.081` |
| 64  | `1.344` | `1.789` | `1.361` |
| 128 | `2.049` | `2.594` | `2.068` |
| 320 | `4.663` | `6.815` | `4.666` |

Run 2:

| ef_search | p50 ms | p95 ms | mean ms |
|----------:|-------:|-------:|--------:|
| 40  | `1.051` | `1.471` | `1.090` |
| 64  | `1.439` | `1.995` | `1.486` |
| 128 | `2.262` | `3.229` | `2.265` |
| 320 | `5.087` | `6.529` | `4.964` |

Observed tqvector mean band across the two reruns:

| ef_search | tqvector SQL mean band |
|----------:|-----------------------:|
| 40  | `1.081 .. 1.090 ms` |
| 64  | `1.361 .. 1.486 ms` |
| 128 | `2.068 .. 2.265 ms` |
| 320 | `4.666 .. 4.964 ms` |

#### pgvector `m=16`

Run 1:

| ef_search | p50 ms | p95 ms | mean ms |
|----------:|-------:|-------:|--------:|
| 40  | `1.259` | `2.019` | `1.277` |
| 64  | `1.719` | `2.406` | `1.789` |
| 128 | `2.952` | `3.903` | `2.942` |
| 320 | `6.537` | `9.887` | `6.540` |

Run 2:

| ef_search | p50 ms | p95 ms | mean ms |
|----------:|-------:|-------:|--------:|
| 40  | `1.593` | `2.531` | `1.602` |
| 64  | `1.641` | `2.084` | `1.610` |
| 128 | `2.992` | `4.110` | `3.000` |
| 320 | `6.383` | `9.479` | `6.432` |

Observed pgvector mean band across the two reruns:

| ef_search | pgvector SQL mean band |
|----------:|-----------------------:|
| 40  | `1.277 .. 1.602 ms` |
| 64  | `1.610 .. 1.789 ms` |
| 128 | `2.942 .. 3.000 ms` |
| 320 | `6.432 .. 6.540 ms` |

## Interpretation

This packet responds directly to the reviewer concerns on packets `368` and
`369`.

### 1. The lane-identity issue is resolved

Packet `368` mixed:

- isolated SQL timings from the `scratch_tqhnsw_real_50k_grouped_m16only_*`
  lane
- canonical recall numbers from packet `362`

That was incorrect. The isolated lane is stronger than the canonical lane, and
the two should not be conflated.

The corrected attribution is:

- packet `362` = canonical grouped `m=16`
- packet `363` direct grouped table = isolated grouped-only `m=16`
- packet `368` SQL timings = isolated grouped-only `m=16`
- packet `369` should therefore have used only the isolated grouped-only
  recall surface, and should be treated as superseded

### 2. The recall ceiling claim is now empirical

The isolated grouped tqvector `m=16` lane does **not** stop at `0.946`.

It continues to:

- `0.950 @ ef=512`
- and then plateaus through `ef=1000`

That still leaves a large gap to pgvector’s measured `0.986` floor. So the
revised claim is stronger than packet `369`’s inference, not weaker:

- tqvector does not reach pgvector’s low-end recall floor on this lane even
  after widening the measured sweep to the GUC ceiling

### 3. The SQL verdict must be banded, not point-estimated

The repeated matched-session SQL runs support two robust claims:

1. tqvector is still faster on the isolated lane at the higher measured
   operating points:
   - `ef=128`: tqvector `2.068 .. 2.265 ms` vs pgvector `2.942 .. 3.000 ms`
   - `ef=320`: tqvector `4.666 .. 4.964 ms` vs pgvector `6.432 .. 6.540 ms`
2. the low-end boundary is softer than packet `369` claimed:
   - tqvector `ef=40` is clearly faster than pgvector’s observed `ef=40` band
     (`1.081 .. 1.090 ms` vs `1.277 .. 1.602 ms`)
   - but tqvector `ef=64` overlaps pgvector `ef=40` enough that the earlier
     “tqvector owns everything below `1.6 ms`” verdict is not decision-grade on
     this `50`-query sample

So the safe operating-point read is now:

- tqvector clearly owns a sub-`~1.1 ms` point at `Recall@10 = 0.920`
- tqvector very likely remains faster at its `0.938` point, but the SQL margin
  versus pgvector’s `ef=40` floor is too narrow and too variable to headline as
  a hard boundary on this sample
- pgvector still dominates the higher-recall region because tqvector never
  reaches `0.986` even by `ef=1000`

## Risk / Follow-up

This packet supersedes the strong form of packet `369`’s verdict.

What is now safe to carry forward:

1. the canonical and isolated grouped `m=16` lanes are different and must be
   cited separately
2. the isolated grouped tqvector `m=16` lane has a measured recall ceiling of
   about `0.950`
3. the earlier “tqvector owns everything below `1.6 ms`” line was too strong
   for the available variance band
4. the broader “latency-first compressed lane versus higher-recall full-vector
   lane” framing still looks directionally right, but should remain provisional
   until it is rerun on a larger query subset than `50`

The next useful measurement batch is therefore narrower than packet `369`:

- if the goal is a product verdict, rerun the matched isolated lane on
  `tqhnsw_real_50k_queries_200_adr031` or the full `tqhnsw_real_50k_queries`
  table so the low-end SQL boundary is no longer sample-noise-sized
- if the goal is an ADR decision, do **not** promote packet `369`’s old
  threshold language; use this packet’s softer banded read instead
