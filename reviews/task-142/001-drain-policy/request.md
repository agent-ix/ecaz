# Review request: Task 142 — scratch boundary-drain removal

- Task: `plan/tasks/142-ivf-dense-coalesce-drain-policy.md`
- Branch: `task-141-sdot-kernel` (stacked after Task 141; this A/B ran both
  cells on the default LUT scorer, so the stack does not confound it)
- Code commit: `86c05031d`
- Evidence: `artifacts/manifest.md`

## What changed

The IVF batch visit loop (`src/am/ec_ivf/scan.rs`) no longer drains either
scratch at row/dense entry or list boundaries: both the row scratch and the
dense-coalesced scratch accumulate to the 256-posting capacity and drain
there or at end-of-visit. Soundness: every pushed posting carries its own
gamma, per-posting `centroid_ip` (Task 115), and heap tids; the live-tid
budget is consumed at append time; candidates dedup by heap tid, so flush
order cannot change results. The vestigial `dense_scratch_list_id` gating
is deleted (−34 lines net).

## A/B result (dense tables, LUT scorer, before/after commit, same session)

| scale | recall@10 pre → post | latency mean pre → post | p50 pre → post |
|---|---|---|---|
| 10k | 0.9734 → 0.9734 | 0.92 → 0.90 ms (−2.2%) | 0.90 → 0.87 ms |
| 50k | 0.9521 → 0.9521 | 1.85 → 1.83 ms (−1.1%) | 1.80 → 1.75 ms |
| 100k | 0.8969 → 0.8969 | 2.71 → 2.70 ms (−0.4%) | 2.66 → 2.62 ms |

Structural gate met exactly: 100k flushes 1781 → 1311 and width<32 flushes
93 → 4 — identical to the row path. Recall byte-identical, storage
unchanged, latency ≥ baseline at every scale.

## Honest finding vs the task's expectation

The task targeted recovery of the +7.2% scorer_batch penalty from Task 135
packet 002. That did NOT materialize: scorer_batch improved only 42.47 →
42.12 ms (−0.8%) despite the flush structure being fully restored. The 135
cross-cell scorer delta was therefore mostly NOT flush-count-driven — the
residual dense-vs-row scorer gap likely lives in dense payload copy
locality or cross-table noise. The manifest records this as a
source-grounded correction; the change still ships on its merits (simpler
code, row-level flush structure, small consistent latency win, recall
byte-identical).

## Asks

1. Confirm the soundness argument for cross-list accumulation (per-posting
   centroid_ip / append-time tid budget / dedup-by-tid) matches your
   reading of `push_dense_posting` and `record_scored_posting_candidates`.
2. Accept the corrected attribution for the residual dense scorer gap (or
   direct a deeper probe); Task 143's 1m matrix proceeds either way with
   this commit as the dense candidate.
