# Review Request: A4 Real 50k — 200-Query Gate

## Context

Branch:
- `main`

Prior real-corpus packets:
- `review/224-a4-real-50k-directional-summary/request.md`
- `review/225-a4-cheaper-external-gate/request.md`

The `225` packet established:

- the gate path is now cheap (exact-quantized excluded)
- the `50`-query real `50k` gate completes in ~3 minutes
- all four A4 configs pass at `50` queries (`94.4%` at the gate point)

The 50-query slice is a meaningful checkpoint but not a full signoff. The
canonical query table (`tqhnsw_real_50k_queries`) contains `1,000` queries. A
`200`-query gate is the next useful step: wide enough to be statistically
meaningful, cheap enough to complete in a single session given the current
runtime baseline.

## What Is Requested

Run the real `50k` external gate report over the first `200` real queries using
the detached scratch helper, and record the result here.

### Setup

The `200`-query query table does not yet exist as a staged fixture. It needs to
be created from the full `tqhnsw_real_50k_queries` table:

```sql
CREATE TABLE tqhnsw_real_50k_queries_200 AS
SELECT * FROM tqhnsw_real_50k_queries
ORDER BY id
LIMIT 200;
```

This follows the same pattern as `tqhnsw_real_50k_queries_10` and
`tqhnsw_real_50k_queries_25`.

### Gate Run

Using the detached scratch helper from `225`:

```bash
scripts/run_real_corpus_recall_scratch.sh \
    --mode gate \
    --corpus  tqhnsw_real_50k_corpus \
    --queries tqhnsw_real_50k_queries_200 \
    --prefix  tqhnsw_real_50k \
    --detach
```

or equivalent direct SQL captured via server-side COPY.

### Expected Output Shape

```text
m   ef_search  recall_at_10  gate_recall_at_10  passes_gate
8   40         ...                              t|f
8   128        ...            0.89              t|f
8   200        ...            0.93              t|f
16  200        ...            0.97              t|f
```

## What to Record

- full gate report output
- wall-clock completion time
- whether the result is stable relative to the `50`-query read

## Readout Criteria

A clean pass at `200` queries — all four rows `t`, gate row at `≥ 89%` — would
be the strongest real-corpus A4 evidence recorded to date and a credible
signoff-level checkpoint.

If any row fails at `200` queries that passed at `50`, record the exact numbers
and flag the regression; do not paper over it.
