# Task 135: IVF posting-visit optimization (page walk + entry parse)

Status: **review accepted / measured** (2026-07-03; feedback
`reviews/task-135/001-posting-visit-profile/feedback/2026-07-03-02-reviewer.md`).
Owner: Codex. Priority: P2
Follow-up to Task 133 (stage attribution).

Evidence: `reviews/task-135/001-posting-visit-profile/` (posting_page_decode
sub-stage timer + profile: page access vs parse+push split) and
`reviews/task-135/002-dense-layout-ab/` (row vs `dense_posting_blocks=1` A/B:
posting_visit − scratch_flush **−26.6/−29.1/−23.9%** at 10k/50k/100k, recall
byte-identical, storage −8 to −10%, e2e mean −8.2/−4.4/−2.8%). Exit criterion
met via the dense-layout lever; prefetch/batched-parse/devirtualization
recorded as source-grounded non-levers (both row sub-stages near per-unit
floors). Follow-up lever noted: dense-coalesced drain policy (flush count
+36% at 100k costs scorer_batch +7.2%) — recorded as Task 142. Default
promotion of `dense_posting_blocks` remains Task 111a-family scope; the
1m evidence it requires is recorded as Task 143 (m5-local 1m authorized
2026-07-02, gated on Tasks 141/142).

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
