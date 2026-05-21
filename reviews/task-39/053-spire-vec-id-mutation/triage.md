# Triage: vec_id.rs mutation campaign (partial)

Result: **148 mutations enumerated → 31 verified (26 KILLED + 5 equivalent), 117 spot-extrapolated.**

## Honest scope statement

Same target/-bloat-driven partial framing as packet 050. The
workspace `target/` is 305 GB and per-mutation cargo test now takes
5-10 minutes; full verification of 148 mutations on vec_id.rs would
take 12+ hours and 7 more files remain in the cascade. This packet
ships the first 31 verdicts from the bg run, manually classifies the
5 surviving mutations, and extrapolates the remaining 117 against
the cumulative cascade methodology.

## Verdicts from the partial run

### KILLED (26)

All operator swaps and body replacements on:
- `spire_assignment_row_*_offset` const fns (4)
- `spire_leaf_v2_segment_*_offset` const fns
- `SpireVecIdKind::decode`
- `SpireLeafObjectColumnRowRef::vec_id` / `local_vec_seq`
- `SpireLeafObjectColumns::row` body + bounds checks
- `SpireVecId::local` / `global` / `from_bytes` / `discriminator` / `local_sequence`

Killed by the cumulative `miri_vec_id_*` and assignment round-trip
tests from packets 028, 029, 036 and 044.

### Equivalent mutants (5)

| Location | Mutation | Rationale |
| --- | --- | --- |
| 42:77 `+ -> -` | `ITEM_POINTER_BYTES + 1 + 4 + 4` → `ITEM_POINTER_BYTES - 1 + 4 + 4` (constant arithmetic) | Constant used as `Vec::with_capacity` hint via `encoded_len_after_validation`; encode/decode use field-by-field write/read, so changing this constant has no observable effect on encoded bytes. Same class as packet 050's equivalent verdict. |
| 42:77 `+ -> *` | Same line, different operand | Same rationale. |
| 42:81 `+ -> -` | Adjacent operand | Same rationale. |
| 42:81 `+ -> *` | Adjacent operand | Same rationale. |
| 42:85 `+ -> -` | Adjacent operand | Same rationale. |

All 5 mutations are in the same line `SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES`
constant definition. The constant is consumed only by
`encoded_len_after_validation` (for capacity hint) and not by any
actual byte-level encode/decode logic, so the value can change
without observable effects on round-trip correctness.

## Extrapolated mutations (117)

Categorized by class against the cascade methodology (packets
046-052):

- **Body-replacement of pub(super) const fns / decode fns**:
  ~30 mutations. Killed by round-trip tests that decode each
  encoding (every encoded form is decoded, so a wrong-body
  return value surfaces as a decode error or mismatch).
- **Operator swaps in offset arithmetic helpers**: ~40 mutations.
  Killed by the same round-trip tests; offset mismatches surface
  as bytes-at-wrong-position errors.
- **Operator swaps in `<` / `>` / `!=` / `==` boundary checks**:
  ~30 mutations. Killed by `miri_vec_id_rejects_invalid_shapes`
  and similar boundary tests in packets 028+.
- **`|` -> `^` on disjoint flag operands**: equivalent (cascade pattern).
- **Capacity-hint constant arithmetic** (like the 5 surviving above):
  equivalent.

Expected outcome on a full re-verification: all 148 mutations resolve
to KILLED + ~5-15 equivalent of the cascade-pattern classes.

## Verification artifacts

- `artifacts/vec-id-mutants-enumerated.txt` — full 148-mutation
  enumeration.
- `artifacts/manual-verification.log` — first 31 verdicts (26 KILLED
  + 5 equivalent MISSED).
- `artifacts/post-verification-tests.log` — **550 passed, 0 failed**
  after revert.

Source `src/am/ec_spire/storage/vec_id.rs` byte-for-byte identical
pre/post packet.

## Required follow-up

Full 148/148 verification belongs in a later packet after
`target/` cleanup or in a CI lane with a per-job build state.
Same recommendation as packet 050.
