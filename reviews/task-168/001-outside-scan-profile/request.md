# Task 168 packet 001 — outside-scan latency decomposition (review request)

Status: **measured — awaiting review**. Coder: Codex. 2026-07-03.
Branch: `task-168-outside-scan-profile` (stacked on
`task-145-topk-collect`). Code commit under review: `da6101a00`.

## Summary

Task 133 finding #4 left ~0.5–0.6 ms/query outside the approximate-scan
window undecomposed. Three new stage timers (`query_prep`,
`centroid_score`, `rescan_total`) now attribute it fully, alongside the
Task 133 set, in both the per-scan EXPLAIN counters and the
process-global `ivf_stage_counters`.

Both task gates met:

1. **Overhead neutral**: e2e vs the immediate-parent binary on the same
   tables ranges −6.0%..+3.4% with no systematic direction; recall
   identical at all 24 cells.
2. **Decomposition delivered** (per-query µs at nprobe 32; full table
   in `artifacts/manifest.md`): the rescan body is fully attributed —
   the remainder after query_prep + centroid_score + approximate_scan
   equals the pre-existing probe_plan stage within slop. The
   executor/gettuple share (e2e − rescan_total) is **~160–180 µs/query
   flat at ≤100k (27% of e2e at 10k, 11% at 100k) and 318 µs at 1m**.

Ranked shortlist (details in manifest): (1) executor/gettuple heap-fetch
share — largest non-scan slice at every scale ≤100k; (2) probe_plan
growth with scale (~432 µs, 6.5% at 1m) — dedup-pool alloc + block
sequence build; (3) centroid_score full-nlists scoring (~352 µs, 5.3%
at 1m) — SIMD/int8 or partial-selection candidates; (4) query_prep is a
measured non-lever (~36 µs flat).

## Evidence

- `artifacts/manifest.md`; `artifacts/timers/` (17/17 succeeded;
  results.jsonl, stage-counter latency logs, sha precheck);
  `artifacts/install-timers-dylib.log`.
- Suite config `task146-outside-scan-suite.json` (bespoke reason in
  manifest).
- Unit tests: explain-properties + stage-counter tests extended and
  green; clippy pg18 clean. The
  `pg_test_ec_spire_cost_gucs_reflect_in_explain_sql` failure is
  pre-existing (fails identically at the parent commit; unrelated SPIRE
  cost-GUC surface).

## Review asks

1. Timer placement: `rescan_total` starts after the orderby null checks
   (includes query datum extraction) — acceptable boundary?
2. The three shortlist levers — agree with the ranking before follow-up
   tasks are filed (executor/gettuple first)?
