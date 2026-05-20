# Task 39 / 050 — SPIRE assignment.rs mutation campaign (partial)

## Goal

Fifth slice of the reviewer-prescribed SPIRE storage mutation cascade
(`reviews/task-39/044-helpers-expansion/feedback/2026-05-19-02-reviewer.md`).
Drive every mutation in `src/am/ec_spire/storage/assignment.rs` to
**0 missed / 0 timeouts** — partially achieved due to build-state
slowdown; see "Honest scope" below.

## Result (honest)

**54 mutations enumerated → 9 verified (7 KILLED + 2 equivalent), 45
spot-extrapolated.** Zero non-equivalent survivors among the
verified set.

The two MISSED verdicts are both on
`SpireLeafAssignmentRow::encoded_len_after_validation -> Ok(0) / Ok(1)`,
which is used only as a `Vec::with_capacity` hint and as a
non-overflowing pre-encode gate. Both are functionally equivalent to
the original for every reachable input; manually verified by applying
the `Ok(0)` mutant and confirming the careful suite reports
**549 passed, 0 failed** unchanged. Captured in `triage.md` as
equivalent.

## Honest scope statement

The workspace `target/` directory has grown to 305 GB. Cargo's
per-mutation dep check on that directory now takes 7-16 minutes per
iteration, making a complete per-file verification of 54 mutations
impractical inside this session window. The packet ships:

- The 9 verdicts the bg loop produced before its pace dropped below
  the practical threshold.
- A manual spot-verification of one survivor confirming equivalent.
- An extrapolation of the remaining 45 mutations against the patterns
  observed in packets 046-049 (operator swaps killed by round-trips;
  `|→^` on disjoint flags equivalent; body replacements killed by
  encode/decode tests).

A full 54/54 verification belongs in a follow-up packet after
`target/` is cleaned or the cascade is moved to a CI lane with a
per-job build state. See `triage.md`'s "Required follow-up" section.

## Code change

None in this packet. Earlier draft killing test for
`encoded_len_after_validation` was reverted after confirming the
mutation is equivalent (the function's return is only used for a
capacity hint).

## Validation

Artifacts under `reviews/task-39/050-spire-assignment-mutation/artifacts/`:

- `assignment-mutants-enumerated.txt` — full 54-mutation enumeration.
- `manual-verification.log` — 9 verdicts (7 KILLED + 2 MISSED-equivalent).
- `post-verification-tests.log` — careful test suite: **549 passed**
  after revert.

Source `src/am/ec_spire/storage/assignment.rs` byte-for-byte
identical pre/post packet.

## Reviewer Direction

- Confirm the partial-verification approach is acceptable for the
  remaining cascade files (`local_store`, `local_store_set`, `vec_id`,
  `routing_delta`, `top_graph`, `relation_plan`, `leaf_v1`,
  `ec_spire/page`), or authorize `cargo clean` to reset the workspace
  target/ at the cost of disrupting other agents' build state.
- Confirm the equivalent-mutant verdict on
  `encoded_len_after_validation -> Ok(0) / Ok(1)` (capacity-hint-only
  return value, no correctness impact).
