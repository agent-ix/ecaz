# Task 100: ec_ivf Plain-Scan Planner Guard

Status: proposed (2026-06-10)
Owner: unassigned
Priority: 3 (operator-facing robustness)

## Why

During Task 93 packet 003 benching (M5 local lane), a plain
`SELECT count(*)` over a corpus table that carries an `ec_ivf` index failed
with:

```text
ERROR: ec_ivf scan currently requires exactly one ORDER BY query
```

The planner chose the `ec_ivf` index for a non-ORDER-BY query (an
index-only-style count path) and the AM's scan entry rejected it. Any
operator running ordinary SQL against an indexed table can hit this; it also
breaks `ecaz corpus load` re-runs against an existing prefix (the loader's
row-count probe), which is how it surfaced. First recorded in
`reviews/task-93/003-rabitq32-neon/artifacts/manifest.md` (run note).

## Scope

1. Reproduce with a minimal fixture: indexed table, `count(*)`, planner
   choosing the ec_ivf path (likely needs the cost model to be cheap enough
   or seqscan disabled).
2. Decide the fix shape:
   - make the AM refuse plain-scan paths at planning time (cost/amcanorder
     surface) so the planner never selects it without ORDER BY; or
   - support a degraded full-scan mode in `amgettuple`/`amgetbitmap`.
   The first is likely correct: the AM exists only to serve ordered ANN
   scans.
3. Audit ec_hnsw / ec_diskann / ec_spire for the same exposure.
4. Regression test at the SQL level (`pg_test`) plus a loader re-run test.

## Acceptance criteria

- `count(*)` (and other non-ORDER-BY statements) over indexed tables plan
  and execute without error for all four AMs.
- `ecaz corpus load` re-runs cleanly against an existing prefix.
- `pg_test` regression coverage for the planner path.

## Coordination

- Independent of the Task 93-99 kernel lanes; touches planner/cost surfaces
  owned by the planner lane (`am/cost.rs`).
