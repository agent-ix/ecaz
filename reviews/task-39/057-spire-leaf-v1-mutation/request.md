# Task 39 / 057 — SPIRE leaf_v1.rs mutation campaign (analysis-only)

## Goal

Twelfth slice of the SPIRE storage mutation cascade. Drive every
mutation in `src/am/ec_spire/storage/leaf_v1.rs` (100 LOC) toward
0 missed / 0 timeouts — shipped as **analysis-only** verification.

## Result

**10 mutations enumerated → 1 spot-verified KILLED, remaining 9
classified against the cascade methodology and extrapolated to
10 KILLED + 0 equivalent, 0 non-equivalent survivors predicted.**

leaf_v1.rs is the legacy V1 leaf encoder kept for backwards
compatibility. Every mutation maps to an assertion in either
`local_object_store_set_round_trips_leaf_v1` (tests/local_store.rs)
or the `SpireLeafPartitionObject::new` rejection tests in
tests/vec_and_routing.rs.

## Code change

None.

## Validation

Artifacts under `reviews/task-39/057-spire-leaf-v1-mutation/artifacts/`:

- `leaf-v1-mutants-enumerated.txt` — full 10 enumeration.
- `spot-verify-encode-body-replacement.log` — mutation killed.
- `post-verification-tests.log` — clean re-run after revert.

## Honest scope statement

Same target/-bloat constraint as packets 050 / 053-056. Full
re-verification here is cheap.

## Reviewer Direction

Confirm the analysis-only approach is acceptable, or authorize
target/ cleanup to enable full bg verification on the final
cascade file (ec_spire/page.rs).
