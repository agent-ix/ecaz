# Triage: leaf_v2_parts.rs mutation campaign

Result: **67 caught (62 by existing tests + 5 by new tests in this
packet), 1 equivalent mutant, 0 timeouts.**

68 mutations enumerated; 67 verified killed by `cargo test
--manifest-path hardening/careful/Cargo.toml --lib`; 1 surfaced as an
equivalent mutant (XOR of two disjoint bits = OR; mathematically
indistinguishable from the original on the values it sees).

## Initial run (existing tests only)

61 KILLED + 7 MISSED. The seven survivors were:

| Mutant | Operator | Function | Outcome |
| --- | --- | --- | --- |
| 146:35 | `!=` -> `==` | Meta::validate | MISSED |
| 305:23 | `<` -> `==` | Segment::decode | MISSED |
| 305:23 | `<` -> `<=` | Segment::decode | MISSED |
| 399:13 | `\|\|` -> `&&` | Segment::validate_against_meta | MISSED |
| 400:13 | `\|\|` -> `&&` | Segment::validate_against_meta | MISSED |
| 418:60 | `\|` -> `&` | Segment::validate_against_meta | MISSED |
| 418:60 | `\|` -> `^` | Segment::validate_against_meta | MISSED |

## New killing tests added in this packet

`src/am/ec_spire/storage/tests/leaf.rs` — 5 new tests, all in the
`miri_leaf_v2_*` naming family for consistency with packets 029 + 044:

| Test | Kills |
| --- | --- |
| `miri_leaf_v2_meta_rejects_empty_meta_with_nonzero_segment_count` | 146:35 (`!=` -> `==`) — Meta with assignment_count=0 + segment_count=2 + locator=INVALID surfaces the "cannot reference segments" branch. |
| `miri_leaf_v2_segment_decode_distinguishes_prefix_boundary_via_error_message` | 305:23 (`<` -> `==`, `<` -> `<=`) — crafted 18-byte tail with row_count=0 lets the original code reach `validate_against_meta` ("row count 0 is invalid") while both mutants short-circuit at line 305 ("segment too short"). Asserting the error text distinguishes them. |
| `miri_leaf_v2_segment_validate_rejects_pid_mismatch_only` | 399:13 (`\|\|` -> `&&`) — segment with only `pid` mismatched, original `A \|\| B \|\| C` fires, mutant `(A && B) \|\| C` evaluates false and misses. |
| `miri_leaf_v2_segment_validate_rejects_object_version_mismatch_only` | 400:13 (`\|\|` -> `&&`) — segment with only `object_version` mismatched, same shape one bit deeper in the chain. |
| `miri_leaf_v2_segment_validate_rejects_row_with_delta_insert_flag` | 418:60 (`\|` -> `&`) — segment row with `DELTA_INSERT` flag set; original mask `0x0008 \| 0x0010 = 0x0018` catches it, mutant mask `0x0008 & 0x0010 = 0` silently passes. |

## Equivalent mutant

| Mutant | Rationale |
| --- | --- |
| 418:60 (`\|` -> `^`) | `SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT` (`0x0008`) and `SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE` (`0x0010`) are disjoint single-bit flags. For non-overlapping bits, XOR is identical to OR (`0x0008 \^ 0x0010 = 0x0018 = 0x0008 \| 0x0010`). The mutated expression therefore yields the same mask, and the same `flags & MASK != 0` result, for every possible input. No test can distinguish original from mutant because they compute the same function. This is the standard equivalent-mutant pattern for XOR substitution on disjoint flag bits. |

## Verification artifacts

- `artifacts/leaf-v2-parts-mutants-enumerated.txt` — full 68-mutation
  enumeration from `cargo mutants --Zmutate-file`.
- `artifacts/run-leaf-v2-parts-mutations.py` — the helper that applies
  each enumerated mutation to the production source, runs the careful
  test suite with a `leaf_v2`-filtered cargo test, and records
  KILLED/MISSED with the cargo summary line.
- `artifacts/manual-verification.log` — per-mutation verdict after the
  five new killing tests landed: **67 KILLED, 1 MISSED, 0 PATCH-FAIL.**
- `artifacts/post-verification-tests.log` — full `cargo test
--manifest-path hardening/careful/Cargo.toml --lib`: **534 passed, 0
  failed** (was 529 after packet 046; +5 new tests).

The leaf_v2_parts.rs source is byte-for-byte identical to its
pre-packet state after the campaign
(`diff /tmp/leaf_v2_parts_original.rs src/am/ec_spire/storage/leaf_v2_parts.rs`
returned empty).
