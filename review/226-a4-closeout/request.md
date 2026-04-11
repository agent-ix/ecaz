# Review Request: A4 Final Closeout

## Context

Branch:
- `main`

Prior real-corpus packets:
- `review/223-a4-real-10k-pass-and-loader-m-values/request.md`
- `review/224-a4-real-50k-directional-summary/request.md`
- `review/225-a4-cheaper-external-gate/request.md`

This packet closes Task 05 / A4 for `v0.1`.

The project spent substantial time chasing contradictory synthetic recall
results. That debugging work was still valuable, but it changed the benchmark
question: the in-repo synthetic fixtures were reproducible, yet they were not a
credible signoff surface by themselves. `NFR-003` already required DBpedia
OpenAI embeddings or a documented equivalent, so the closeout decision now uses
the real-corpus lane on `main`.

## Decision

Close A4 on the real-corpus surface required by `NFR-003`.

The release gate is:

- Recall@10 `>= 89%` at `(m=8, ef_search=128)`

The real-corpus evidence on `main` clears that comfortably:

- canonical real `10K`: `97.3%`
- broader real `50K` `50`-query gate: `94.4%`

## Final Evidence

### 1. Canonical real `10K` full gate passes strongly

Observed output from:

```sql
select * from tests.tqhnsw_graph_scan_recall_external_gate_report(
    'tqhnsw_real_10k_corpus',
    'tqhnsw_real_10k_queries',
    'tqhnsw_real_10k'
);
```

was:

```text
8   40   0.971       t
8   128  0.973  0.89 t
8   200  0.974       t
16  200  0.975       t
```

So the required gate row is not borderline. It is `8.3` percentage points above
threshold on the first canonical real subset.

### 2. Graph stays close to exact on real `10K`

At the threshold configuration on a `50`-query real slice:

- graph Recall@10: `0.972`
- exact quantized Recall@10: `0.976`

So on real `10K`, the live graph-first path is only `0.4` percentage points
below exact quantized behavior.

### 3. Broader real `50K` evidence also clears comfortably

The first directional `10`-query real `50K` gate already looked healthy:

```text
8   40   0.87        t
8   128  0.90  0.89  t
8   200  0.91        t
16  200  0.92        t
```

The threshold-point summary widened to `25` real queries stayed healthy:

- graph Recall@10: `0.936`
- exact quantized Recall@10: `0.960`

And the broader `50`-query real `50K` gate report completed successfully after
the cheaper external gate split:

```text
8   40   0.926       t
8   128  0.944  0.89 t
8   200  0.948       t
16  200  0.952       t
```

So the gate row remains comfortably above threshold even on the broader real
subset slice:

- `(m=8, ef_search=128)`: `94.4%`

## Why The Synthetic Contradiction Does Not Reopen A4

The synthetic `10K` contradiction remains documented, but it is not the
signoff surface:

1. The spec-required benchmark authority is DBpedia OpenAI embeddings or a
   documented equivalent, not the in-repo synthetic generators alone.
2. Raw reference HNSW baselines on the synthetic fixtures were also weak, which
   means those fixtures are not a credible proxy for the production-style gate.
3. The real-corpus lane is now fully implemented on `main`:
   - canonical parquet selection
   - deterministic parquet-to-TSV conversion
   - manifest verification
   - idempotent loader
   - external relation-backed gate and summary surfaces

So the remaining synthetic gap is a benchmark/debugging topic for later `C1`
work, not a blocker to closing A4.

## Readout

A4 is complete for `v0.1`.

What this unblocks:

- A5 graph-aware insert as the next runtime lane
- A6 vacuum repair after A5
- planner activation no longer being blocked on lack of recall evidence, but
  the D2 planner-activation slice still needs to be finished
- SIMD merge timing becoming a normal integration decision rather than a recall
  blocker

What remains outside A4:

- broader post-gate benchmark/report work under `C1`
- D2 planner activation itself: Condition 3 (`ef_search` wired through scan
  execution) is still only "Mostly done" in `plan/status.md` because the main
  runtime wiring landed but sentinel cleanup remains elsewhere
- synthetic-vs-real methodology follow-up
- latency/storage reporting
- insert/vacuum quality after live-write and delete churn

## Files

- [plan/tasks/05-graph-scan.md](/home/peter/dev/tqvector/plan/tasks/05-graph-scan.md)
- [plan/tasks/12-real-corpus-recall.md](/home/peter/dev/tqvector/plan/tasks/12-real-corpus-recall.md)
- [plan/status.md](/home/peter/dev/tqvector/plan/status.md)
- [plan/plan.md](/home/peter/dev/tqvector/plan/plan.md)
