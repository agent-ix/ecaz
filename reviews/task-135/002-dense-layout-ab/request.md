# Review request: Task 135 packet 002 — posting layout A/B (row vs dense blocks)

- Task: `plan/tasks/135-tq-ivf-posting-visit-optimization.md`
- Branch: `task-136-rank1-scorer`, evidence at head `8f7bce3cc`
- No code change under review in this packet — it measures the shipped, gated
  `dense_posting_blocks=1` reloption (Task 111/111a code) as Task 135's
  "dense-block coalescing coverage" lever, using the packet-001 sub-stage
  timer. Evidence: `artifacts/manifest.md`.

## Result: exit criterion met

Task 135's gate is "a measurable reduction of the posting_visit −
scratch_flush share at unchanged recall/storage". Measured (same session,
same binary, fresh paired loads per scale, nprobe=32):

- **posting_visit − scratch_flush: −26.6% / −29.1% / −23.9%** at 10k/50k/100k.
- **Recall/NDCG byte-identical** at every scale (0.9734 / 0.9521 / 0.8969).
- **Storage better, not just unchanged: −8.2% / −9.6% / −9.6%** index size.
- E2E mean latency −8.2% / −4.4% / −2.8%; p50/p95/p99 all improve or match.
- Build time unchanged (stage_postings within noise).

Attribution via the packet-001 `posting_page_decode` split at 100k: the win is
entry parse + scratch push (19.98 → 13.88 ms/sweep, −30.5%); page/buffer
access was already small on these fresh loads (~5.1 ms/sweep, flat).

## Counter-effect worth its own follow-up

Dense coalescing drains at row/list boundaries instead of accumulating to the
256-posting target, so flush count rises (100k: 1311 → 1781; width<32 flushes
4 → 93) and scorer_batch pays +7.2% (39.9 → 42.8 ms/sweep) — giving back
~40% of the parse+push win at 100k. A drain-policy fix (accumulate the
dense-coalesced scratch across boundaries like the row scratch does) is the
obvious next lever if more headroom is wanted; it would compound with the
Task 136 int8_approx scorer, which shrinks the scorer stage this effect
inflates.

## Levers not pursued (source-grounded, packet 001)

Prefetch, batched entry parse, and callback devirtualization on the ROW path:
per-posting parse+push is ~56 ns (near the 768-byte copy floor) and page
access is ~0.7 µs/page (near the pin/lock floor); the cost driver is entry and
page COUNT, which is the layout lever measured here.

## Scope guard

`dense_posting_blocks` default promotion remains the Task 111a-family
decision (kept gated there pending a 1M/AWS lane); this packet contributes
the current-HEAD m5-local evidence and recommends re-opening promotion with
a 1M run. No defaults were changed in this packet.

## Asks

1. Confirm the exit-criteria reading (measurable reduction achieved at
   unchanged recall and improved storage).
2. Opinion on sequencing the drain-policy follow-up vs promoting this
   packet's evidence into the Task 111a promotion decision.
