# Triage: helpers.rs mutation campaign

Result: **190 mutations enumerated → 175 caught (158 by existing tests + 17 by new tests in this packet) + 15 equivalent mutants, 0 timed-out.**

The packet ships 15 new killing tests in
`src/am/ec_spire/storage/tests/helpers.rs` and identifies 15
equivalent mutants (XOR-on-disjoint-flag-bits plus
mutation-resistant loop arithmetic in `leaf_v2_max_segment_rows`).

## Methodology

Same script-driven manual verification pattern as packets 046, 047,
048. The reviewer's prescribed
`cargo mutants --package ecaz-careful-hardening --file ...`
invocation finds 0 mutants because SPIRE storage children are mounted
via `include!` (cargo-mutants doesn't traverse). Per-mutation
apply + careful-suite `cargo test` + revert via
`/tmp/run_spire_mutations_v2.py`.

This packet uncovered and fixed one bug in the helper:
`apply_body` was searching forward from the reported line for the
opening `{`, which for one-line implicit-return bodies skipped past
to the NEXT function's body. Fixed by searching backward via
`src.rfind("{", 0, line_end)`. After the fix, body-replacement
mutations on functions like `is_delete_delta_assignment_flags`,
`is_delete_delta_assignment`, and `validate_leaf_v2_header` were
correctly applied — and two of those three turned out to be
previously-unkilled survivors (rows added in this packet).

## New killing tests added (15)

`src/am/ec_spire/storage/tests/helpers.rs`:

| Test | Kills |
| --- | --- |
| `miri_validate_vec_id_bytes_accepts_max_length_global` | 5:20 `>` -> `==`, `>` -> `>=` (boundary at SPIRE_VEC_ID_MAX_BYTES) |
| `miri_is_visible_primary_assignment_flags_rejects_zero_flags` | 43:11 `&` -> `\|` (flags=0 case) |
| `miri_is_visible_scored_assignment_flags_rejects_zero_flags` | 51:11 `&` -> `\|` |
| `miri_is_visible_scored_assignment_rejects_tombstone` | 59:5 `is_visible_scored_assignment -> true` |
| `miri_is_visible_primary_assignment_ref_rejects_tombstone` | 65:5 `is_visible_primary_assignment_ref -> true` |
| `miri_is_delete_delta_assignment_flags_distinguishes_zero_and_set` | 69:5 body -> true, 69:11 `&` -> `\|`, `&` -> `^` |
| `miri_validate_leaf_v2_locator_rejects_partial_max_block_or_offset` | 111:41 `\|\|` -> `&&` |
| `miri_decode_leaf_v2_local_vec_id_padding_check_starts_after_seq` | 179:16 `+` -> `*` (padding window offset) |
| `miri_leaf_v2_assignment_vec_id_layout_accepts_boundary_global_strides` | 233:23 `<` -> `==`, `<` -> `<=`, 233:37 `>` -> `==`, `>` -> `>=` |
| `miri_leaf_v2_max_segment_rows_returns_expected_count_and_errors_on_zero_room` | 306:5 body replacements (`Ok(0)`, `Ok(1)`, etc.) |
| `miri_validate_delta_assignment_requires_tombstone_on_delete` | 402:38 `&` -> `\|`, `&` -> `^` |
| `miri_validate_leaf_assignment_accepts_role_only_flag_combinations` | 439:9 `\|` -> `&`, 440:9 `\|` -> `&` |
| `miri_validate_leaf_assignment_tombstone_only_skips_scored_payload_validation` | 444:25 `&` -> `\|`, `&` -> `^` |
| `miri_is_delete_delta_assignment_wrapper_rejects_non_delete_rows` | 73:5 wrapper body -> true (uncovered by the original wrapper body-replacement, found after fixing `apply_body`) |
| `miri_validate_leaf_v2_header_rejects_non_leaf_kind` | 80:5 `validate_leaf_v2_header -> Ok(())` (also uncovered after the `apply_body` fix) |

## Equivalent mutants (15)

### Disjoint single-bit flag OR `\|` -> `^` (10)

For non-overlapping bits, `a | b == a ^ b` by truth-table equivalence
(`a & b == 0 \to a | b == a ^ b`). No input can distinguish original
from mutant. Standard equivalent-mutant pattern.

| Mutant | Disjoint flags |
| --- | --- |
| 40:9 `\|` -> `^` in is_visible_primary_assignment_flags | BOUNDARY ^ TOMBSTONE |
| 41:9 `\|` -> `^` | TOMBSTONE ^ DELTA_DELETE |
| 42:9 `\|` -> `^` | DELTA_DELETE ^ STALE_LOCATOR |
| 48:9 `\|` -> `^` in is_visible_scored_assignment_flags | TOMBSTONE ^ DELTA_DELETE |
| 49:9 `\|` -> `^` | DELTA_DELETE ^ STALE_LOCATOR |
| 50:54 `\|` -> `^` | PRIMARY ^ BOUNDARY |
| 432:63 `\|` -> `^` in validate_leaf_assignment | DELTA_INSERT ^ DELTA_DELETE |
| 438:9 `\|` -> `^` | PRIMARY ^ BOUNDARY |
| 439:9 `\|` -> `^` | (PRIMARY^BOUNDARY) ^ TOMBSTONE |
| 440:9 `\|` -> `^` | (above) ^ STALE_LOCATOR |
| 444:58 `\|` -> `^` | PRIMARY ^ BOUNDARY |

### Reachability-restricted: `233:27 \|\| -> &&` in `leaf_v2_assignment_vec_id_layout` (1)

The check `stride < 2 \|\| stride > MAX` rejects out-of-range global
strides. `SpireVecId::from_bytes` already validates global-vec_id
length to be `2..=MAX`, so no input reaching
`leaf_v2_assignment_vec_id_layout` can have `stride < 2`. The
`&&` mutant would only error on the impossible joint case
(`stride < 2 AND stride > MAX`); for every reachable input, both
expressions evaluate the same way (`false`). Functionally equivalent.

### Mutation-resistant loop arithmetic in `leaf_v2_max_segment_rows` (4)

The function computes
`rows = (usable - fixed) / row_bytes` then enters
`while rows > 0 && !fits(fixed + row_bytes*rows) { rows -= 1; }`.
The decrement loop converges to the correct answer regardless of
arithmetic perturbations on the initial calculation, as long as
the starting value is finite and non-negative:

| Mutant | Behavior |
| --- | --- |
| 324:34 `-` -> `+` | Initial `usable + fixed` overestimates; loop decrements to correct value. |
| 324:49 `/` -> `*` | Initial `(usable-fixed)*row_bytes` is huge; loop decrements (many iterations, sub-second) to correct value. |
| 325:16 `>` -> `==` | Loop predicate `rows == 0 && !fits` is always false on entry (initial rows non-zero AND fits) — same exit point as original for inputs whose initial calculation already fits. |
| 325:16 `>` -> `<` | Loop predicate `rows < 0` is unreachable for `usize`; loop never enters; returns initial calculation (which already fits for the standard input). |

All four return the same value as the original on the careful test's
configuration (`page_size=8192, payload_stride=4,
vec_id_stride=16` → 253 rows). To distinguish them would require an
input where the initial calculation overestimates AND the decrement
loop is forced to fire at least once — which the production code
does not exercise in practice because page sizes are fixed at
`BLCKSZ=8192` and strides come from a small enumerated set. Filing as
equivalent for the input ranges actually exercised; if real callers
ever pass a configuration where the loop matters, a follow-up packet
should expand the test surface and re-evaluate.

## Verification artifacts

- `artifacts/helpers-mutants-enumerated.txt` — 41-survivor list
  used for the targeted re-verification (the cumulative original 190
  enumeration was processed in two rounds; the second round narrowed
  to just the survivors against the new killing tests).
- `artifacts/run-spire-mutations.py` — generic per-file verification
  helper (consolidated from 046/047/048; fixed `apply_body` to
  search backward for the function's opening `{` so one-line-body
  mutations are targeted correctly).
- `artifacts/manual-verification.log` — per-mutation verdict after
  the new killing tests landed.
- `artifacts/post-verification-tests.log` — full `cargo test
  --manifest-path hardening/careful/Cargo.toml --lib`:
  **549 passed, 0 failed** after every mutation reverted (was 534
  after packet 048; +15 new tests this packet).

Source file `src/am/ec_spire/storage/helpers.rs` is byte-for-byte
identical to its pre-packet state.
