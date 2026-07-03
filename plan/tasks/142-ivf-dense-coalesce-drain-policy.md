# Task 142: IVF dense-coalesced scratch drain policy

Status: **review accepted / measured** (2026-07-03; feedback
`reviews/task-142/001-drain-policy/feedback/2026-07-03-02-reviewer.md`).
Owner: Codex. Priority: P2

Evidence: `reviews/task-142/001-drain-policy/` (code `86c05031d`): flush
structure fully restored to row levels (100k flushes 1781→1311, width<32
93→4), recall byte-identical, latency mean −2.2/−1.1/−0.4%. Finding: the
+7.2% scorer recovery target was falsified — scorer_batch improved only
−0.8% with flush structure restored, so the Task 135 dense scorer gap is
mostly NOT flush-count-driven (correction recorded in the packet).
Follow-up to Task 135 packet 002 (dense-layout A/B) and Task 111a
(scan-side dense coalescing).

## Why

Task 135 packet 002 (`reviews/task-135/002-dense-layout-ab/`) measured
`dense_posting_blocks=1` winning posting_visit−scratch_flush by −24/−29%
at byte-identical recall, but giving back ~40% of the parse+push win at
100k on the scorer side: the dense-coalesced scratch drains at every Row
entry and list switch (`process_dense_coalesced_postings` call sites in
`src/am/ec_ivf/scan.rs`, ~1595 and ~1655 at `8f7bce3cc`), so flush count
rose 1311→1781 per sweep, width<32 flushes 4→93, and scorer_batch paid
+7.2% (39.9→42.8 ms/sweep). The row scratch does not have this problem —
it accumulates across pages and lists up to
`IVF_POSTING_SCRATCH_SOA_BATCH_POSTINGS` (256).

## Scope

- Change the dense-coalesced drain policy to accumulate across row/list
  boundaries up to the 256-posting target, mirroring the row scratch.
- Soundness check to confirm in implementation: `centroid_ips` is already
  per-posting in `IvfPostingScratchSoa` (Task 115), and the live-tid budget
  is consumed at append time, so cross-list accumulation should be
  score-identical — verify no per-flush list-scoped assumption remains
  (e.g. `dense_scratch_list_id` gating).
- A/B per change: dense-with-fix vs dense-without-fix, same session, IVF
  10k/50k/100k recall+latency+storage, stage + flush-width counters.
- If the row-entry drain must stay for correctness, record the reason as
  the source-grounded outcome and fix only the list-switch drain.

## Out of Scope (hard)

- No on-disk format change (scan-side policy only).
- No change to the row scratch path or its flush target.
- Default promotion of `dense_posting_blocks` (that is Task 143 / the
  Task 111a family decision).

## Gate / Exit Criteria

- Recall byte-identical to the current dense path at all three scales.
- Flush-width histogram under dense restored to ~row levels
  (width≥32 share ≥ 99% at 100k/nprobe=32).
- scorer_batch and e2e latency at least match the current dense path at
  every scale (target: recover the +7.2% 100k scorer penalty), or a
  source-grounded negative recording why the drain is required.
