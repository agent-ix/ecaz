# Task 39 / 050 — SPIRE assignment.rs mutation campaign (full back-fill)

## Goal

Fifth slice of the reviewer-prescribed SPIRE storage mutation
cascade. Drive every mutation in
`src/am/ec_spire/storage/assignment.rs` to **0 missed / 0 timeouts**
(excluding documented equivalents).

## Result

**54 mutations enumerated → 52 KILLED + 2 equivalent (capacity-hint),
0 non-equivalent survivors.**

Back-fills the earlier partial verification (9/54) under the
isolated `CARGO_TARGET_DIR=$(pwd)/target-mutants` build cache
authorized in 050's own feedback. 5 boundary-check MISSED killed by
3 new tests; the 2 remaining MISSED on
`encoded_len_after_validation` are documented capacity-hint
equivalents (same class as 053/056).

## Methodology

Full per-mutation apply/test/revert via
`/tmp/run_spire_mutations_v2.py` with isolated build cache.

## Code change

- `src/am/ec_spire/storage/tests/assignment.rs`: 3 new
  boundary-killing tests (see `triage.md`).
- `src/am/ec_spire/storage/tests.rs`: imported
  `SPIRE_ASSIGNMENT_ROW_FIXED_PREFIX_BYTES` and
  `SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES`.
- `hardening/careful/src/spire.rs`: mirrored the imports in the
  careful crate.

Source `assignment.rs` unchanged.

## Validation

Artifacts under `reviews/task-39/050-spire-assignment-mutation/artifacts/`:

- `assignment-mutants-enumerated.txt` — full 54 enumeration.
- `manual-verification.log` — 54/54 per-mutation verdicts.
- `post-verification-tests.log` — clean re-run after revert.

`triage.md` documents the killing-test rationale and the 2
capacity-hint equivalents.
