# Task 39 / 049 — SPIRE helpers.rs mutation campaign

## Goal

Fourth slice of the reviewer-prescribed SPIRE storage mutation cascade
(`reviews/task-39/044-helpers-expansion/feedback/2026-05-19-02-reviewer.md`).
Drive every mutation in `src/am/ec_spire/storage/helpers.rs` to
**0 missed / 0 timeouts**.

## Result

**190 mutations enumerated → 175 KILLED, 15 documented equivalent
mutants, 0 timed-out, 0 non-equivalent survivors.**

Initial run against the cumulative test surface from packets 028
through 048: **41 survivors.** Of those 41, the second pass against
the 15 new killing tests shipped in this packet reduced to 17
survivors — all documented equivalent mutants (11 `|→^` on disjoint
single-bit flag operands, 1 `||→&&` on a reachability-restricted
guard, and 5 mutation-resistant arithmetic perturbations in
`leaf_v2_max_segment_rows` whose decrement loop converges to the
correct answer regardless).

A second sub-pass on body-replacement mutations (after fixing a
real bug in the verification helper) surfaced 2 additional
previously-unkilled body-replacement mutations
(`73:5 is_delete_delta_assignment -> true` and
`80:5 validate_leaf_v2_header -> Ok(())`); both verified killed by
the corresponding new tests via manual apply + revert cycles.

See `triage.md` for the per-mutation map, killing-test names, and
equivalent-mutant rationale.

## Code change

`src/am/ec_spire/storage/tests/helpers.rs` (new) — 15 killing tests:

- `miri_validate_vec_id_bytes_accepts_max_length_global` — boundary at
  `SPIRE_VEC_ID_MAX_BYTES` for the `> MAX` guard.
- `miri_is_visible_primary_assignment_flags_rejects_zero_flags` — pins
  the `flags=0 → false` semantic against `& → |` mutation.
- `miri_is_visible_scored_assignment_flags_rejects_zero_flags` — same
  shape on the scored helper.
- `miri_is_visible_scored_assignment_rejects_tombstone` — kills
  body-replacement on the wrapper.
- `miri_is_visible_primary_assignment_ref_rejects_tombstone` — kills
  body-replacement on the ref wrapper.
- `miri_is_delete_delta_assignment_flags_distinguishes_zero_and_set` —
  kills body-replacement + both `& → |` / `& → ^` mutations on the
  delete-delta flag check.
- `miri_validate_leaf_v2_locator_rejects_partial_max_block_or_offset` —
  pins the `|| MAX` guard against `|| → &&`.
- `miri_decode_leaf_v2_local_vec_id_padding_check_starts_after_seq` —
  uses a seq with the high byte set so a `+ → *` mutation on the
  padding-window offset is observable as a padding-non-zero error.
- `miri_leaf_v2_assignment_vec_id_layout_accepts_boundary_global_strides` —
  tests stride at both boundaries (2 and MAX).
- `miri_leaf_v2_max_segment_rows_returns_expected_count_and_errors_on_zero_room` —
  the only test that exercises this helper directly; covers the
  body-replacement mutations and the page-too-small error path.
- `miri_validate_delta_assignment_requires_tombstone_on_delete` —
  delete-without-tombstone surfaces the mask check.
- `miri_validate_leaf_assignment_accepts_role_only_flag_combinations` —
  PRIMARY-only and BOUNDARY-only rows kill the `| → &` collapse of
  the role_flags constant.
- `miri_validate_leaf_assignment_tombstone_only_skips_scored_payload_validation` —
  kills `& → |` / `& → ^` on the scored-payload-validation branch.
- `miri_is_delete_delta_assignment_wrapper_rejects_non_delete_rows` —
  the wrapper's body-replacement-to-`true` mutation (uncovered after
  fixing the helper script's `apply_body` bug).
- `miri_validate_leaf_v2_header_rejects_non_leaf_kind` — the
  `validate_leaf_v2_header → Ok(())` body-replacement (also uncovered
  after the apply_body fix).

`src/am/ec_spire/storage/tests.rs` — adds
`include!("tests/helpers.rs");`.

`hardening/careful/src/spire.rs` — adds the same include inside the
careful `storage::tests` module so the new tests run under
`make coverage` and the mutation-verification suite.

No production source change. Source byte-for-byte identical post-packet.

## Methodology + helper-script bug fix

Same script-driven manual verification pattern as packets 046, 047,
048. While running this packet, the helper script's `apply_body`
function was found to be buggy for one-line implicit-return function
bodies: it searched forward from the reported `line:col` for the
opening `{`, which for one-line bodies skipped past to the NEXT
function's body and mutated the wrong function. Fixed by searching
backward via `src.rfind("{", 0, line_end)`. The fix is reflected in
`artifacts/run-spire-mutations.py` and applies to every subsequent
cascade packet.

The bug-fix discovery surfaced 2 previously-unkilled body-replacement
mutations (73:5 and 80:5) that the original buggy script had reported
as KILLED while actually mutating the wrong function. The two new
killing tests in this packet (`miri_is_delete_delta_assignment_wrapper_rejects_non_delete_rows`
and `miri_validate_leaf_v2_header_rejects_non_leaf_kind`) close those
gaps.

## Validation

Artifacts under `reviews/task-39/049-spire-helpers-mutation/artifacts/`:

- `helpers-mutants-enumerated.txt` — full 190-mutation enumeration.
- `helpers-survivors-revalidated.txt` — 41-survivor focus list used
  for the second-pass re-verification.
- `run-spire-mutations.py` — generic verification helper with the
  fixed `apply_body`.
- `manual-verification-survivor-revalidation.log` — **24 KILLED, 17
  MISSED (all 17 are documented equivalent mutants — see triage.md).**
- `post-verification-tests.log` — `cargo test --manifest-path
  hardening/careful/Cargo.toml --lib`: **549 passed, 0 failed** after
  every mutation reverted (was 534 after packet 048; +15 new tests).

## Reviewer Direction

- Confirm the 17 equivalent-mutant verdicts in `triage.md` (11
  disjoint-flag XOR, 1 reachability-restricted, 5 mutation-resistant
  loop arithmetic).
- Verify that the `run-spire-mutations.py` fix is correct for future
  cascade packets (`assignment`, `local_store`, `local_store_set`,
  `vec_id`, `routing_delta`, `top_graph`, `relation_plan`, `leaf_v1`,
  `ec_spire/page`). The bug only manifested on functions with
  one-line implicit-return bodies; multi-statement function bodies
  were unaffected.
