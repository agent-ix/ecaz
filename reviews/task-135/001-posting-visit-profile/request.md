# Review request: Task 135 packet 001 — posting-visit sub-stage profile

- Task: `plan/tasks/135-tq-ivf-posting-visit-optimization.md`
- Branch: `task-136-rank1-scorer` (Task 135 slices stack after the Task 136
  GUC-gated scorer, which defaults off and is inert here)
- Code commit: `8f7bce3cc` — new `posting_page_decode` stage timer
- Evidence: `artifacts/manifest.md`

## What changed

Task 135's first scope item: profile inside the posting-visit path.
`visit_ivf_posting_entries_for_block_sequence` now returns the wall time spent
inside per-page decode callbacks (excluding read-stream buffer acquisition),
and both IVF scan paths record it as the `posting_page_decode` stage (global
stage counters + per-scan EXPLAIN counter). Timer overhead: two `Instant::now()`
calls per posting page. No behavior change to scan results by construction.

## Profile result (100k, per-sweep, LUT scorer, row-layout tables)

posting_visit 86.0 ms = page/buffer access **22.1** + entry parse/scratch push
**18.7** + flush 45.2 (scorer 42.2 + record 2.7 + SoA bookkeeping 0.3).

- Parse+push is ~56 ns/posting — near the 768-byte payload-copy floor.
- Page access is ~0.69 µs/page across ~1k row-layout pages/query — near the
  buffer pin/lock floor.
- Flush widths are healthy (avg ~253, width≥32 for 1307/1311), so the
  "row postings dominate flush counts" observation from Task 133 is about
  entry/page COUNT, not narrow flushes.

## Conclusion → next lever

Neither sub-stage has meaningful in-place headroom; the cost driver is the
row posting layout itself (~10 postings/page ⇒ ~1k page visits and ~10.4k
entry parses per query at 100k). The shipped, gated dense-block layout
(`dense_posting_blocks=1` reloption) with the landed Task 111a scan-side
coalescing measured **28.2 vs 32.4 ms p50 (−13%) at 100k** against row in
`reviews/task-111a/004-all-dense-options-benchmark` at equal recall and
smaller storage. Packet 002 will A/B row vs dense-block loads on the standard
TQ IVF fixture at 10k/50k/100k (recall+latency+storage, stage counters on) to
measure the posting-visit reduction on current HEAD — per the task's
"dense-block coalescing coverage" lever. Promotion of the reloption default
remains Task 111a-family scope; 135 contributes the measurement.

## Asks

1. Sanity-check the sub-stage accounting (page-access column includes the
   ≤2-per-scan drain flushes; noted in the manifest).
2. Flag if you want the prefetch or batched-parse levers profiled deeper
   before the layout A/B — the numbers above suggest they are second-order.
