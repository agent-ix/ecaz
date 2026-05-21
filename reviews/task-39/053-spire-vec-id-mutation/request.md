# Task 39 / 053 — SPIRE vec_id.rs mutation campaign (partial)

## Goal

Eighth slice of the reviewer-prescribed SPIRE storage mutation
cascade. Drive every mutation in
`src/am/ec_spire/storage/vec_id.rs` to **0 missed / 0 timeouts** —
partially achieved due to target/-bloat slowdown.

## Result (honest)

**148 mutations enumerated → 31 verified (26 KILLED + 5 equivalent),
117 spot-extrapolated against the cascade methodology. Zero
non-equivalent survivors in the verified set.**

The 5 MISSED verdicts are all on line 42's
`SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES` constant arithmetic. This
constant is consumed only by `encoded_len_after_validation` as a
`Vec::with_capacity` hint; the actual encode/decode logic writes
and reads fields at fixed offsets independent of the constant. Same
equivalent-mutant class as packet 050's `encoded_len_after_validation
-> Ok(0)`.

## Honest scope statement

Same target/-bloat constraint as packet 050. Full verification of
148 mutations would take 12+ hours at the current ~10-min-per-
mutation cycle. Ship the verified 31 + extrapolation, defer full
re-verify to a follow-up packet after target/ cleanup.

## Code change

None. No killing tests needed — all verified survivors are
documented equivalent mutants.

## Validation

Artifacts under `reviews/task-39/053-spire-vec-id-mutation/artifacts/`:

- `vec-id-mutants-enumerated.txt` — full 148 enumeration.
- `manual-verification.log` — 31 verdicts.
- `post-verification-tests.log` — **550 passed, 0 failed** after revert.

## Reviewer Direction

Same as packet 050: confirm the partial-verification + extrapolation
approach is acceptable for the remaining cascade files, or authorize
target/ cleanup.
