# Task 146: IVF outside-scan latency decomposition (rescan setup + executor share)

Status: **measured — awaiting review** (2026-07-03). Owner: Codex (branch
`task-146-outside-scan-profile`, stacked on `task-145-topk-collect`).
Priority: P2. Follow-up of Task 133 finding #4.
Packet: `reviews/task-146/001-outside-scan-profile/` — timers landed at
`da6101a00`, overhead neutral, decomposition complete. Headline:
executor/gettuple is ~160–180 µs/query flat (27% of e2e at 10k, 11% at
100k, 5% at 1m); probe_plan and centroid_score grow with scale (~432 /
~352 µs at 1m); query_prep is a measured non-lever (~36 µs).

## Why

Task 133 measured ~0.5–0.6 ms/query sitting OUTSIDE the approximate-scan
window (centroid scoring, LUT/query prep, executor/gettuple) and never
decomposed it. Under the new defaults that slice is proportionally the
largest unexamined territory: at 100k the e2e mean is ~1.6–1.7 ms while
the timed approximate scan accounts for roughly two thirds of it. The
1m stage budget's "unattributed" share has the same gap. If a real
lever exists there (e.g. centroid scoring is a full-nlists dot-product
sweep per query; query prep runs the rotation + int8 quantization), it
now moves double-digit percentages.

## Scope

- Add three stage timers alongside the Task 133 set (per-scan explain
  counters + process-global `ivf_stage_counters`):
  - `query_prep` — store_scan_query + prepared-query construction +
    heap-rerank-state configuration.
  - `centroid_score` — `load_centroid_scores` + probe-list selection.
  - `rescan_total` — the whole amrescan body, so
    executor/gettuple share = e2e − rescan_total, and
    rescan-internal unattributed = rescan_total − query_prep −
    centroid_score − approximate_scan.
- Timer-overhead A/B per the Task 133 precedent (before/after binary,
  e2e must be neutral).
- Measure the decomposition at 10k/50k/100k/1m on the Task 145 packet
  fixtures (already loaded), nprobe 32/40, and rank the follow-up
  levers by share.

## Out of Scope (hard)

- No behavior change to scan/scoring — timers only.
- Fixing whatever the decomposition finds (that is the follow-up task).

## Gate / Exit Criteria

- Stage counters land with neutral overhead evidence.
- A packet table attributing the outside-scan slice per stage at all
  four scales, with a ranked shortlist (or a source-grounded negative
  if nothing exceeds noise).
