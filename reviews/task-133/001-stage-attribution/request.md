---
task: 133
topic: stage-attribution
requester: codex
date: 2026-07-01
code_commit: a05babf74
base_commit: eced7e1bc561cf64b54b2e72ef36a0399a9c76e5
---

# Review Request: Task 133 — IVF per-stage latency attribution (the non-scorer ~40%)

Task 133 asked where the non-scorer ~40% of IVF no-QJL 4-bit query latency goes
(heap/top-k vs page/posting I/O vs materialization vs decode), with a committed
per-stage breakdown at 10k/50k/100k and a ranked follow-up shortlist.

## What landed

- `27518cbb2` — suite provenance stamping (runner git sha + backend
  `ecaz_build_git_sha()` + dylib sha where derivable). Reviewer carry-over from
  task-125/002; every suite manifest in this packet is stamped.
- `a05babf74` — per-stage timers: `stats_{probe_plan, posting_visit,
  scratch_flush, scorer_batch, candidate_record, topk_collect}_elapsed_us` in
  `IvfExplainCounters` (EXPLAIN-surfaced, per scan) mirrored into process-global
  accumulators surfaced by `ec_ivf_stage_scoring_snapshot()/_reset()` and
  `ecaz bench latency --ivf-stage-counters` (suite step option
  `ivf_stage_counters`). Timer granularity is per flush/visit/scan — no
  per-candidate clocks.

## Evidence (artifacts/, manifest.md is source of truth)

- Attribution (per query, `-rerun` + `-quiet` passes agree on shares):

| stage | 10k | 50k | 100k | 100k share of scan |
|---|---|---|---|---|
| approximate_scan | 951 µs | 1653 µs | 2697 µs | 100% |
| scorer_batch | 433 | 836 | 1252 | **46%** |
| page I/O + entry parse (visit − flush) | 409 | 599 | 1126 | **42%** |
| topk_collect | 72 | 142 | 213 | 8% |
| probe_plan (outside scan window) | 48 | 89 | 127 | — |
| candidate_record (dedup map + bounded heap) | 28 | 58 | 84 | 3% |
| SoA copy (flush − scorer − record) | 8 | 16 | 19 | <1% |

  Coverage sanity: `approximate_scan ≈ posting_visit + topk_collect` within
  ~3 µs at every scale; `scorer_batch` agrees with the independent block-kernel
  counters within dispatch overhead (40.05 vs 39.49 ms @100k).
- Recall unchanged at all scales (0.9734 / 0.9521 / 0.8969 — identical to
  task-125 packet 001), storage untouched.
- **Timer overhead A/B: neutral.** Pre-timer dylib (`eced7e1bc`, worktree
  build) vs with-timers dylib (`a05babf74`), same tables, same quiet machine:
  1.21/2.05/3.23 ms vs 1.09/1.99/3.21 ms mean at 10k/50k/100k
  (`results-pretimer.jsonl` vs `results-latency-quiet.jsonl`). Kernel 39.16 vs
  39.71 ms. Differences are inside session noise; the with-timer build is not
  slower.
- Session-noise caveat: absolute latencies this session run ~10–18% above the
  task-125 packet numbers (e.g. 3.21 vs 2.73 ms @100k) on BOTH dylibs — tables
  were re-created after an extension drop, so physical layout differs. Stage
  shares and A/B deltas within this session are unaffected.

## Findings and ranked follow-ups (the deliverable)

1. **Posting page I/O + entry parse is the non-scorer hotspot: 42% of the scan
   at 100k (1.13 ms/query), growing with scale** (43% → 36% → 42% of scan at
   10k/50k/100k in absolute terms 0.41/0.60/1.13 ms). This is the page walk in
   `visit_ivf_posting_entries_for_block_sequence` + posting entry parse +
   scratch pushes. Amdahl ceiling if halved: ~17% e2e. Follow-up candidates:
   posting-page layout/prefetch, batched entry parse, page-visit loop hygiene.
   → propose as the next optimization task.
2. **Top-k collect (final sort of the dedup map) is 8%** (213 µs @100k) —
   `collect_ranked_probe_candidates` builds + drains a fresh BinaryHeap over
   all deduped candidates. Ceiling if halved: ~3% e2e. Possible cheap win
   (partial select instead of full heap), but second priority.
3. **The dedup HashMap + bounded-heap record loop is NOT a hotspot** (3%).
   The "attack the heap" hypothesis from the task file is a confirmed negative.
4. **~0.5–0.6 ms/query sits outside the approximate-scan window** (centroid
   scoring, LUT query prep, executor/gettuple) — not yet decomposed; worth one
   more timer (centroid scoring) if follow-up work targets it.
5. Scorer remains 46% of the scan — consistent with task-125's ~60%-of-latency
   estimate now measured precisely. Further scorer wins (e.g. the rank-1
   in-register kernel idea from the TQ evaluation) still carry the largest
   single-stage ceiling.

## Requested review

- Confirm the stage decomposition + derived shares are sound (esp. the
  visit−flush and flush−scorer−record derivations).
- Confirm the timer-overhead A/B supports keeping the timers always-on (no
  debug GUC gate).
- Concur with the ranked shortlist (page I/O + parse first) so follow-up tasks
  can be filed.
