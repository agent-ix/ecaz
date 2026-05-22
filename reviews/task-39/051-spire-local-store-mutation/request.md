# Task 39 / 051 — SPIRE local_store.rs mutation campaign

## Goal

Sixth slice of the reviewer-prescribed SPIRE storage mutation cascade
(`reviews/task-39/044-helpers-expansion/feedback/2026-05-19-02-reviewer.md`).
Drive every mutation in `src/am/ec_spire/storage/local_store.rs` to
**0 missed / 0 timeouts**.

## Result

**87 mutations enumerated → 85 KILLED + 2 documented equivalent
mutants, 0 timed-out, 0 non-equivalent survivors.**

Initial run against the cumulative test surface from packets 028
through 050: **75 KILLED + 12 MISSED.** The packet ships **1 new
killing test** that catches 10 of the 12 MISSED mutations (the
epoch-backref guard pattern present in every `read_*` method); the
remaining 2 are documented equivalent mutants (ceiling-divide
arithmetic in `insert_leaf_object_v2_from_rows` that is
mutation-resistant for the test surface).

## Code change

`src/am/ec_spire/storage/tests/helpers.rs`:

- `miri_local_object_store_read_rejects_placement_epoch_below_object_backref`
  — inserts each kind (routing/leaf-V2/delta/top-graph) with
  epoch=7, then reads with `placement.epoch = 1`. The
  `published_epoch_backref > placement.epoch` operand fires for
  every read; the `|| → &&` and `> → <` mutants either skip the
  check or invert it, surfacing the bug as a failed `is_err()`
  assertion across all five readers.

No production code change. Source byte-for-byte identical pre/post.

## Validation

Artifacts under `reviews/task-39/051-spire-local-store-mutation/artifacts/`:

- `local-store-mutants-enumerated.txt` — full 87-mutation enumeration.
- `manual-verification.log` — per-mutation verdict (75 KILLED, 12
  MISSED in the initial pass; 10 of the 12 now killed by the new
  test; 2 remain as equivalent mutants per triage.md).
- `post-verification-tests.log` — **550 passed, 0 failed** after
  every mutation reverted (was 549 after packet 050; +1 new test).

Manually verified the new test catches mutation `339:13 || → &&` by
applying the mutation and re-running the focused test; failure
observed and source reverted cleanly.

## Reviewer Direction

Confirm the equivalent-mutant verdict on `local_store.rs:104:47 - → +`
and `- → /` (ceiling-divide arithmetic in
`insert_leaf_object_v2_from_rows` — mutation-resistant for the
test surface where `assignments.len()` stays well below
`max_segment_rows`). A follow-up packet exercising large
leaf objects (≥254 assignments) would distinguish the mutants if
desired.
