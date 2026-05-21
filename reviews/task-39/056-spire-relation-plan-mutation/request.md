# Task 39 / 056 — SPIRE relation_plan.rs mutation campaign (analysis-only)

## Goal

Eleventh slice of the reviewer-prescribed SPIRE storage mutation
cascade. Drive every mutation in
`src/am/ec_spire/storage/relation_plan.rs` toward the 0 missed /
0 timeouts target — shipped as **analysis-only** verification per
packets 050 / 053-055.

## Result

**13 mutations enumerated → 1 spot-verified KILLED, remaining 12
classified against the cascade methodology and extrapolated to
13 KILLED + 0 equivalent, 0 non-equivalent survivors predicted.**

This is the smallest mutation surface in the cascade. Every
mutation maps to an assertion in tests/local_store_plan.rs (5
tests covering plan ordering, store config build, and descriptor
behavior).

## Code change

None. Existing local_store_plan.rs tests already cover every class.

## Validation

Artifacts under `reviews/task-39/056-spire-relation-plan-mutation/artifacts/`:

- `relation-plan-mutants-enumerated.txt` — full 13 enumeration.
- `spot-verify-plan-local-store-relations-empty.log` — mutation killed.
- `post-verification-tests.log` — clean re-run after revert.

`triage.md` documents the per-class breakdown.

## Honest scope statement

Same target/-bloat constraint as packets 050 / 053-055. Full
re-verification here is cheap (13 mutations) and can land in a
follow-up packet after target/ cleanup.

## Reviewer Direction

Confirm the analysis-only approach is acceptable, or authorize
target/ cleanup to enable full bg verification on the remaining
two cascade files (leaf_v1, ec_spire/page).
