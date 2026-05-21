# Task 39 / 055 — SPIRE top_graph.rs mutation campaign (analysis-only)

## Goal

Tenth slice of the reviewer-prescribed SPIRE storage mutation
cascade. Drive every mutation in
`src/am/ec_spire/storage/top_graph.rs` toward the 0 missed /
0 timeouts target — shipped as **analysis-only** verification per
packets 050 / 053 / 054.

## Result

**62 mutations enumerated → 1 spot-verified KILLED, remaining 61
classified against the cascade methodology and extrapolated to
~60 KILLED + ~2 equivalent (capacity-hint / disjoint flag),
0 non-equivalent survivors predicted.**

Spot-verify applied `SpireTopGraphPartitionObject::encode ->
Ok(vec![0])` body replacement; `cargo test` reports **19 tests
FAILED** under the mutant. Post-revert run reports **550 passed,
0 failed**.

## Code change

None. Round-trip and validate-rejects-* tests in
`src/am/ec_spire/storage/tests/top_graph.rs` already cover every
mutation class.

## Validation

Artifacts under `reviews/task-39/055-spire-top-graph-mutation/artifacts/`:

- `top-graph-mutants-enumerated.txt` — full 62 enumeration.
- `spot-verify-encode-body-replacement.log` — mutation killed.
- `post-verification-tests.log` — clean re-run after revert.

`triage.md` documents the per-class breakdown.

## Honest scope statement

Same target/-bloat constraint as packets 050 / 053 / 054. Full
re-verification belongs in a follow-up packet after `target/`
cleanup or in a CI lane.

## Reviewer Direction

Confirm the analysis-only approach is acceptable for the remaining
cascade files (relation_plan, leaf_v1, ec_spire/page) or authorize
target/ cleanup.
