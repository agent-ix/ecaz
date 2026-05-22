# Task 39 / 056 — SPIRE relation_plan.rs mutation campaign (full verification)

## Goal

Eleventh slice of the reviewer-prescribed SPIRE storage mutation
cascade. Drive every mutation in
`src/am/ec_spire/storage/relation_plan.rs` to **0 missed /
0 timeouts** (excluding documented equivalents).

## Result

**13 mutations enumerated → 11 KILLED + 2 equivalent
(reachability-restricted), 0 non-equivalent survivors. Full
per-mutation verification.**

The 2 MISSED are both on line 42's `len() > max_identifier_bytes`
guard, which is unreachable under the careful crate build:
`pg_identifier_limit_bytes()` returns 63, and the constructed
relation name is at most 36 bytes (prefix 14 + delimiters 2 +
u32 decimal digits 1-10 + 1-10). Both `> -> ==` and `> -> >=`
mutations leave the guard unreachable; they are equivalent to the
original under the careful build. The non-equivalent partner
`> -> <` correctly KILLED. See `triage.md` for the reachability
proof.

This is the same recurring equivalence class accepted by the
reviewer in 053 feedback ("reachability-restricted").

## Methodology

Full per-mutation apply/test/revert via
`/tmp/run_spire_mutations_v2.py` with
`CARGO_TARGET_DIR=$(pwd)/target-mutants` build isolation, per
053/054/055 reviewer direction.

## Code change

None.

## Validation

Artifacts under `reviews/task-39/056-spire-relation-plan-mutation/artifacts/`:

- `relation-plan-mutants-enumerated.txt` — full 13 enumeration.
- `manual-verification.log` — 13/13 per-mutation verdicts.
- `post-verification-tests.log` — clean re-run after revert.

`triage.md` documents the reachability-restricted equivalence proof.
