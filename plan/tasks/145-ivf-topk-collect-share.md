# Task 145: IVF top-k collect share at 1m (dedup map + ranked collect)

Status: **proposed** (2026-07-03). Owner: unassigned. Priority: P3
Discovered by the Task 143 promotion-matrix stage budget.

## Why

Under the new defaults (dense + int8/SDOT), the 1m stage budget is flat
and `topk_collect` has become a first-class slice
(`reviews/task-143/001-promotion-matrix/artifacts/latency-1m-dense-int8.log`,
per-sweep ms at nprobe=32, 16 scans): scorer_batch 31.9 (30%), parse+push
24.4 (23%), page access 22.4 (21%), **topk_collect 17.5 (17%)**,
candidate_record 7.2, probe_plan 6.9. topk_collect is
`collect_ranked_probe_candidates` over the scan's dedup map (~45k
candidates/query at 1m/nprobe=32) — a full map walk + sort per query.

## Scope

- Profile the collect: map iteration vs sort vs allocation.
- Candidate levers (one at a time): bounded top-k heap maintained during
  candidate_record (the CandidateTopK already exists for the prune
  cutoff), avoiding the full-map sort; smaller dedup map footprint.
- A/B per lever via `ecaz bench suite` at 100k/1m (recall+latency+
  storage, stage counters); recall must stay byte-identical (ordering
  levers only).

## Out of Scope (hard)

- No change to candidate dedup semantics or scoring.

## Gate / Exit Criteria

- Measurable topk_collect reduction at unchanged recall, or a
  source-grounded negative per lever.
