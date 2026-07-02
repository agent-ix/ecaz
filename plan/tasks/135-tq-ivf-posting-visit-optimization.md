# Task 135: IVF posting-visit optimization (page walk + entry parse)

Status: **proposed** (2026-07-02). Owner: unassigned. Priority: P2
Follow-up to Task 133 (stage attribution).

## Why

Task 133's per-stage attribution (reviews/task-133/001-stage-attribution)
measured the non-scorer hotspot precisely: **posting page I/O + entry parse is
42% of the approximate scan at 100k** (1.13 ms/query; 0.41/0.60/1.13 ms at
10k/50k/100k), versus scorer 46%, top-k collect 8%, dedup+heap 3%. This is the
page walk in `visit_ivf_posting_entries_for_block_sequence`
(`src/am/ec_ivf/page.rs`) plus posting entry parse and scratch pushes in
`src/am/ec_ivf/scan.rs`. Amdahl ceiling if halved: ~17% end-to-end.

## Scope

- Profile inside the visit path (extend the Task 133 stage timers or use the
  in-repo microprofile pattern) to split page access vs entry parse vs scratch
  push.
- Candidate levers, to be measured one at a time: page-level prefetch of the
  next posting page in the block sequence, batched entry parse (SoA-decode the
  page in one pass instead of per-entry callbacks), callback devirtualization /
  visit-loop hygiene, dense-block coalescing coverage (why row postings still
  dominate flush counts at nprobe=32).
- A/B per lever via `ecaz bench suite` at 10k/50k/100k IVF
  (recall+latency+storage), stage counters on (`ivf_stage_counters`).

## Out of Scope (hard)

- No on-disk posting format changes. No speculative stacking of levers —
  per-change attribution required.

## Gate / Exit Criteria

- A measurable reduction of the posting_visit − scratch_flush share at
  unchanged recall/storage, or a source-grounded negative per lever.
