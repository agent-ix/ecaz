# Task 124 packet 001 — stage2 pipeline vs rb1 champion (review request)

Status: **measured — awaiting review**. Coder: Codex. 2026-07-04.
Measurement-only packet (no code change; the pipeline under test is the
Task 130 keep-set already on main). Binary `1dda8e589`, code-identical
to the Task 146/147 baseline binary `da6101a00` (docs-only interim
commits; see `artifacts/install-stage2-dylib.log`).

## Summary

Re-baselined Task 124 question: does the landed 3-stage pipeline
(rb1 coarse → persisted TQ stage-2 over the width-50 frontier → exact
f32 rerank of 25) beat the Task 147 champion (rb1 + heap_f32 width 50)?
Plus the two controls Task 147 skipped: TQ 4-bit coarse under the SAME
rerank (true density apples-to-apples), and plain rb1 at width 25.

**Headline: no warm-cache promotion case; rb1@w50 survives.**

| finding | evidence |
|---|---|
| stage2 = tqf32 = rb1@w50 recall at ALL 18 points ≤100k | recall tables, `artifacts/cells/` |
| naive rb1@w25 loses 0.3–1.3 pp recall at every scale | same |
| stage2 −4..−10% latency at 100k, parity below | latency tables |
| at 1m stage2 pays 0.3–1.0 pp recall (n≥24); at matched recall it is pareto-EQUIVALENT (n40 0.9719 @ 6.30 ms vs rb1@w50 n32 0.9667 @ 6.21 ms) at **4.4× the index size** (1003 vs 227 MiB) | `artifacts/cells-1m/` |
| density control (tqf32): with rerank held fixed, 4-bit vs 1-bit coarse is recall-neutral and ~latency-neutral warm; the Task 147 win was primarily the RERANK stage; density's durable payoff is storage (3.2–3.5×) | E-cell rows |
| TQ stage-2 payload cost is 98% decode / 2% score (5.45 vs 0.13 ms/sweep at 100k) | stage counters in latency logs |

## Decision asks

1. Accept the warm-cache verdict: **iterate, not promote** for the
   stage-2 pipeline; rb1 + heap_f32 w50 remains the champion config.
2. Agree Phase 6 (IO-sensitive/cold-cache A/B: rb1@w50 vs stage2@25 vs
   TQ no-rerank) is now the deciding evidence for Task 124 — the
   halved-heap-fetch rationale (25 vs 50/query) is the pipeline's only
   remaining promotion path, with TQ decode optimization as the bounded
   warm-side lever if Phase 6 shows promise.
3. Note for the rb1 promotion-matrix follow-up (Task 147 ask #2): this
   packet's E-cell is the controlled density evidence it should cite.

## Evidence

`artifacts/manifest.md` (cells, shas, full tables, verdict, run log);
`artifacts/cells/` (≤100k, 37/37 steps) + `artifacts/cells-1m/` (9/9):
results.jsonl, recall/latency/storage logs with stage counters, sha
prechecks; suite configs `task124-stage2-suite.json` +
`task124-stage2-1m-suite.json` (bespoke reason in manifest). Two run
errors documented in the manifest run log (runner-level PGHOST; staged
1m corpus filename).
