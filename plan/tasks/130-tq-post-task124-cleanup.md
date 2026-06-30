# Task 130: TurboQuant Post-Task-124 Cleanup

Status: **review requested — cleanup slice complete** (2026-06-30; packet `reviews/task-130/001-task124-cleanup/`).
Owner: coder (to be assigned). One coder, one branch.
Priority: P1 before any Task 124 landing or closeout PR merge.

## Why

Task 124 finally did the TurboQuant scorer/compute-path work, but its last
round left two cleanup hazards:

- The closeout can blur the validated production-TQ result with smaller,
  recall-broken experimental formats. Reviewer feedback requires those
  categories to stay separate.
- The reduced-dimension validation added a production-facing
  `rerank_format=turboquant2_768` only to prove the workload result. The result
  failed the recall contract at 50k/100k, so that reloption must not land as a
  product-facing format.
- `ecaz bench suite` emits regenerable `truth-*-k10.json` files, but the repo
  ignore rules only covered `truth-cache/` directories. That mismatch leaves
  packet worktrees full of untracked truth caches and makes accidental commits
  more likely.

## Goal

Cleanly land the useful Task 124 outcome without landing failed experimental
surfaces as product options:

- preserve the validated production-TQ result as exactly that: `-5.4%`
  in-engine scorer elapsed at 100k from peripheral scorer-path changes;
- preserve TQ2 and reduced-dimension measurements as evidence only, clearly
  labelled separate smaller formats that fail recall;
- remove or gate any failed validation-only reloption that should not be exposed
  to users;
- fix packet hygiene so regenerable truth caches are ignored consistently.

## Scope

Required cleanup:

1. Remove `rerank_format=turboquant2_768` from the production-facing IVF
   reloption surface after packet 037 records its real-index evidence.
2. Keep packet 037 as evidence in history and task documentation; do not erase
   the failed result.
3. Update `.gitignore` so `reviews/**/truth-*.json` and
   `benchmarks/**/truth-*.json` are ignored alongside `truth-cache/`.
4. Update Task 124 / Task 130 docs so no summary can claim that Category-B
   smaller formats are validated production TurboQuant speedups.
5. Produce a Task 130 review packet documenting the cleanup and validation.

Optional follow-up, only with a separate packet:

- Gate or remove older Task 124 experimental formats (`turboquant2`,
  `turboquant_binary`) if the owner decides they should not remain callable.
  This task's first cleanup slice does not change them because they predate
  packet 037 and have their own evidence history.

## Non-Goals

- Do not re-benchmark TQ2 or reduced-dimension variants unless code changes
  their behavior.
- Do not relitigate Task 124's optimization outcome.
- Do not delete packet evidence, committed logs, or review feedback.
- Do not delete local untracked benchmark caches without explicit operator
  approval; ignore rules are the safe repo-level fix.

## Acceptance Criteria

1. Source no longer accepts or advertises `rerank_format=turboquant2_768`.
2. Focused options/rerank checks pass after the cleanup.
3. `git check-ignore` confirms `reviews/**/truth-*.json` is ignored.
4. Task 124 and/or Task 130 documentation preserves the category split:
   production 4-bit TQ has the validated `-5.4%` result; TQ2/reduced-dim
   numbers are separate smaller-format evidence and recall-broken where
   measured.
5. A review packet under `reviews/task-130/` records the cleanup, validation,
   and any intentionally retained untracked local artifacts.

## References

- `plan/tasks/124-ivf-tq-stage2-rerank-pipeline.md`
- `reviews/task-124/035-post-scorer-product-suite/feedback/2026-06-30-03-reviewer.md`
- `reviews/task-124/036-tq2-real-index-validation/`
- `reviews/task-124/037-tq2-dim768-real-index/`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
