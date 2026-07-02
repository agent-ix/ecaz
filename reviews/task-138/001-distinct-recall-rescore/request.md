# Review Request: Task 138 — Distinct-Recall Metric And Historical Evidence Audit

Status: measurement complete; requesting review of the metric slice, the
re-score evidence, and the conclusion re-ranking.

## Phase 0 — Metric (commit `70ab4dfd0`)

- `distinct_recall@k` = |distinct(returned top-k) ∩ truth top-k| / k;
  duplicates count once. Emitted alongside the unchanged duplicate-tolerant
  `recall@k` on the `ecaz bench recall` table and the spire-pipeline
  coordinator query metrics; identity JSONL records gain
  `distinct_returned_count`; `distinct_returned_{min,mean}` accompany both
  tables. Definitions live next to the metric in
  `crates/ecaz-cli/src/commands/bench/recall.rs`.
- New `ecaz bench rescore-identity` re-scores historical
  `spire_result_identity` JSONL against brute-force truth from the packet's
  corpus/queries TSVs (runner extension per FR-038; no forked scripts; no
  historical artifact edited).
- Cross-validation, two independent paths: (a) the re-scorer reproduces
  packet 027's published duplicate-tolerant recall to 4 decimals (0.9985 @
  10k, 1.0000 @ 50k); (b) on the fresh 10k n1024/b2 cell the live runner's
  new columns and the offline re-scorer agree exactly (0.9975 / 0.4930 /
  mean 4.94).

## Phase 1 — Inventory

`artifacts/inventory.md`. Summary: Task 131 packet 027 is the only re-scorable
returned-ID evidence in the repo; n1024/b2 had no ID artifacts anywhere and is
re-run here; Task 121's DOE ran on the local single-instance lane, which
dedupes by vec_id inside one index (`src/tests/build.rs`) and is therefore not
duplicate-exposed; pre-027 multi-instance packets are unrecoverable for exact
re-scoring but their shapes are covered by the cells here.

## Phase 2 — Corrected recall table (multi-instance lane, k=10)

Current metric = duplicate-tolerant recall@10 as historically published.
Distinct = `distinct_recall@10`. `dup q` = queries whose top-10 contains at
least one duplicated corpus id. All cells `rabitq`, production read.

| Scale | Shape | nprobe | Queries | Current | Distinct | distinct_returned min/mean | dup q | Source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 10k | n128/b4 | 96 | 200 | 0.9985 | **0.5195** | 4 / 5.21 | 183/200 | packet 027 re-score (`artifacts/rescore/rescore-10k-n128-b4-threshold-{off,on}.log`, arms byte-identical) |
| 50k | n128/b4 | 96 | 1000 | 1.0000 | **0.4146** | 4 / 4.15 | 1000/1000 | packet 027 re-score (`artifacts/rescore/rescore-50k-n128-b4-threshold-{off,on}.log`) |
| 10k | n1024/b2 | 64 | 200 | 0.9975 | **0.4930** | 4 / 4.94 | 199/200 | fresh cell `artifacts/10k-n1024-b2/` + `artifacts/rescore/rescore-10k-n1024-b2.log` |
| 50k | n1024/b2 | 64 | 200 | 0.9980 | **0.5095** | 4 / 5.11 | 200/200 | fresh cell `artifacts/50k-n1024-b2/` + `artifacts/rescore/rescore-50k-n1024-b2.log` |

Fresh-cell latency (context, not a comparison claim): 10k n1024/b2 p50
537.928 ms, 50k n1024/b2 p50 660.366 ms at nprobe=64, 200 queries, this host.
The 50k p50 lands within 0.6% of the Task 131 packet 024 before-arms baseline
(663.809 ms), so the historical n1024/b2 latency numbers transfer to these
cells; only their recall labels change.

Bottom line across all four cells: every multi-instance shape measured returns
4-5 distinct rows on average for k=10 while the current metric reports
0.9975-1.0000. The duplicate defect is shape-independent (b2 duplicates as
pervasively as b4 — 2 replicas still put one row on up to 3 nodes), so no
historical multi-instance recall figure can be rehabilitated by shape choice;
they must be re-measured with the Task 137 fix engaged.

## Phase 3 — Conclusion re-ranking (survive / weaken / flip)

1. **Task 121: "boundary_replica_count is the primary route-recovery lever" —
   SURVIVES for what it measured, WEAKENS as a distributed-default
   recommendation.** The Phase 2 DOE cells (`reviews/task-121/011..018`) ran
   on the local single-instance lane, where boundary replicas share one
   allocated vec_id inside one index and the scan dedupes them
   (`scan_dedupe_mode=vec_id`; `src/tests/build.rs` asserts distinct
   results). Those recall gains are real, not duplicate inflation. However,
   the lever's transfer to the multi-instance surface — where Tasks 123/131
   consumed it as `b4` — is exactly the mechanism that manufactures
   cross-node duplicates (ADR-083). Any multi-instance recall credited to b4
   under the current metric is part real recovery, part duplicate inflation;
   the corrected split at the representative shape is the difference between
   1.0000 and 0.4146 at 50k. Until re-measured with the Task 137 fix on, b4
   must not be cited as a distributed recall lever.
2. **Task 123/131 baseline "recall 1.0 at nprobe=96 n128/b4" (multi-instance)
   — FLIPS.** Distinct recall at that operating point is 0.5195 (10k) /
   0.4146 (50k). A user issuing k=10 receives on average ~5 (10k) / ~4 (50k)
   distinct neighbors. The published 0.9985-1.0000 figures measure duplicate
   hits.
3. **Task 123/131 baseline "recall ~1.0 at nprobe=64 n1024/b2"
   (multi-instance) — FLIPS.** Fresh cell: current 0.9975 vs distinct 0.4930
   at 10k; 50k row TBD in this table. The fine-list shape's *latency*
   advantage (4-7x, Task 131 packet 024 / Task 123 packets 019/020) is
   identity-agnostic and stands, but its "matched recall" qualifier was
   matched-with-duplicates on both sides.
4. **Task 131 packet 027 inter-arm "matched recall" (threshold off vs on) —
   SURVIVES.** The off/on identity JSONL was byte-identical at both scales
   (verified again by identical re-score outputs for both arms), so the
   structural no-op conclusion and the shelve decision are untouched. The
   qualifier stays: matched under duplicate-tolerant metrics AND matched
   distinct — the arms returned the same rows.

## What this packet does NOT do

- No dedupe fix (Task 137 owns it; see
  `reviews/task-137/001-source-identity-dedupe-ab/` and ADR-083).
- No routing tuning (Task 139 consumes this metric).
- No historical artifact edited; all re-scoring is additive.

## Artifacts

See `artifacts/manifest.md`.
