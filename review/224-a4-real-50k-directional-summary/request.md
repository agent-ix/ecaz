# Review Request: A4 Real 50k Directional Summary

## Context

Branch:
- `main`

Prior real-corpus packets:
- `review/218-a4-real-corpus-recall-lane/request.md`
- `review/219-a4-real-corpus-loader-smoke/request.md`
- `review/220-a4-real-corpus-metric-contract-followup/request.md`
- `review/221-a4-real-corpus-subset-manifest-contract/request.md`
- `review/222-a4-real-corpus-fetch-and-schema-alignment/request.md`
- `review/223-a4-real-10k-pass-and-loader-m-values/request.md`

This packet records the first directional A4 measurements on the default real
`50k` DBpedia-derived subset after the loader and canonical subset pipeline were
made usable on `main`.

It is intentionally a progress packet, not an A4-closeout packet. The live
`50k` directional numbers are now in hand, but only on small real-query slices;
the full `50k` gate-sized sweep is still too expensive to treat as the default
interactive loop.

## What This Slice Shows

### 1. The real `50k` four-config gate report is healthy on the first slice

With both `tqhnsw_real_50k_m8_idx` and `tqhnsw_real_50k_m16_idx` present, the
live external gate report over the first `10` real queries returned:

```text
8   40   0.87        t
8   128  0.90  0.89  t
8   200  0.91        t
16  200  0.92        t
```

So the default real `50k` subset now has an actual four-row gate-shaped read,
and every A4 configuration is above water on the first real slice. The gate
row itself clears threshold at `90.0%`.

### 2. Real `50k` at the gate point looks healthy

Using the staged real subset:

- corpus table: `tqhnsw_real_50k_corpus` (`50,000` rows)
- query table: `tqhnsw_real_50k_queries`
- index: `tqhnsw_real_50k_m8_idx`

The gate-point external summary on a `10`-query real slice at
`(m=8, ef_search=128)` returned:

```text
8  128  50000  10  0.9  0.889  0.9367415  0.0058907466  0.80121213  0.94  2  3
```

Interpreted:

- graph Recall@10: `0.900`
- graph Recall@100: `0.889`
- exact quantized Recall@10: `0.940`
- graph-below-exact queries: `2`
- worst exact gap: `3`

That means the A4 threshold config is already above the `0.89` gate on the
first real `50k` directional slice.

### 3. Widening the real query slice still stays above the gate

The same gate point over a `25`-query real slice returned:

```text
8  128  50000  25  0.936  0.9104  0.96071154  0.005804238  0.88993937  0.96  4  3
```

Interpreted:

- graph Recall@10: `0.936`
- exact quantized Recall@10: `0.960`

So widening from `10` queries to `25` did not reveal a collapse. The real
`50k` gate point still looks comfortably above threshold.

### 4. Lower `ef_search` behaves like a normal quality/speed tradeoff

On the same `10`-query real slice:

```text
8  40   50000  10  0.87  0.4    0.91623175  0.0060257167  0.7187879   0.94  4  4
8  200  50000  10  0.91  0.922  0.9447237   0.0058951     0.8521212   0.94  2  2
```

That is the expected shape:

- `ef_search=40` is lower-quality and slightly below the A4 threshold
- `ef_search=128` clears the threshold
- `ef_search=200` moves a bit higher still

This looks like a healthy operating curve rather than the old synthetic-path
contradiction.

### 5. Real `50k` `m=16` also looks healthy on the first slice

The remaining non-gate config at `(m=16, ef_search=200)` over the first `10`
real queries returned:

```text
16  200  50000  10  0.92  0.938  0.953522  0.005921532  0.9327272  0.94  2  1
```

Interpreted:

- graph Recall@10: `0.920`
- exact quantized Recall@10: `0.940`

So the real `50k` directional matrix is now complete at the "small but real"
slice level, and all four A4 configurations look sane.

## Evidence

### Four-config gate report, `10` real queries

Observed output from:

```sql
select * from tests.tqhnsw_graph_scan_recall_external_gate_report(
    'tqhnsw_real_50k_corpus',
    'tqhnsw_real_50k_queries_10',
    'tqhnsw_real_50k'
);
```

was:

```text
8   40   0.87        t
8   128  0.90  0.89  t
8   200  0.91        t
16  200  0.92        t
```

### Gate point, `10` real queries

Observed output from:

```sql
select * from tests.tqhnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_50k_corpus',
    'tqhnsw_real_50k_queries_10',
    'tqhnsw_real_50k_m8_idx',
    8,
    128
);
```

was:

```text
8  128  50000  10  0.9  0.889  0.9367415  0.0058907466  0.80121213  0.94  2  3
```

### Gate point, `25` real queries

Observed output from:

```sql
select * from tests.tqhnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_50k_corpus',
    'tqhnsw_real_50k_queries_25',
    'tqhnsw_real_50k_m8_idx',
    8,
    128
);
```

was:

```text
8  128  50000  25  0.936  0.9104  0.96071154  0.005804238  0.88993937  0.96  4  3
```

### Lower and higher `ef_search` points, `10` real queries

Observed outputs from:

```sql
select * from tests.tqhnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_50k_corpus',
    'tqhnsw_real_50k_queries_10',
    'tqhnsw_real_50k_m8_idx',
    8,
    40
);

select * from tests.tqhnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_50k_corpus',
    'tqhnsw_real_50k_queries_10',
    'tqhnsw_real_50k_m8_idx',
    8,
    200
);
```

were:

```text
8  40   50000  10  0.87  0.4    0.91623175  0.0060257167  0.7187879   0.94  4  4
8  200  50000  10  0.91  0.922  0.9447237   0.0058951     0.8521212   0.94  2  2
```

### `m=16, ef_search=200`, `10` real queries

Observed output from:

```sql
select * from tests.tqhnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_50k_corpus',
    'tqhnsw_real_50k_queries_10',
    'tqhnsw_real_50k_m16_idx',
    16,
    200
);
```

was:

```text
16  200  50000  10  0.92  0.938  0.953522  0.005921532  0.9327272  0.94  2  1
```

## What Is Still Missing

### 1. Wider real `50k` slices are still expensive

The external summary surface is still paying for:

- brute-force fp32 truth
- exact quantized top-10
- live graph scan

for the same query set inside one wrapper. That makes wider real `50k` slices
expensive even after the one-time index builds are done.

This is now a harness-cost issue, not evidence of a correctness failure.

### 2. The wider four-config gate rerun hit a client/session hiccup

An attempted rerun of the four-config gate report over `25` real queries lost
its client session while the backend query continued running. The scratch
cluster itself stayed healthy, and the underlying backend remained active on
CPU until it was manually stopped.

This is a tooling/session-management problem around the interactive scratch
workflow, not a signal that the graph path or the dataset failed.

## Readout

The live real-data picture is now materially different from the synthetic lane:

- real `10k` already passed strongly
- real `50k` now has an actual four-config gate report on a `10`-query real
  slice, and every row passes there
- real `50k` at the threshold point is also above the gate on the first `10`
  and `25` query slices
- real `50k` at `(m=16, ef_search=200)` also looks healthy on the first `10`
  real queries
- graph remains below exact quantized, but by a small enough margin that the
  main story is no longer "A4 is fundamentally broken on the live path"

The next useful step is no longer "get any `50k` number at all". It is to
decide whether the next round should be:

- a broader real `50k` query slice to harden the read, or
- cheaper real-data harness tooling so broader slices stop costing several
  minutes of fp32 truth work each time.
