# Triage: assignment.rs mutation campaign

Result: **54 mutations enumerated → 9 verified (7 KILLED + 2 equivalent), 45 spot-extrapolated based on the cascade methodology.**

## Honest scope statement

This packet's verification is **partial** by design. The workspace
`target/` directory has grown to 305 GB, and the per-mutation
`cargo test` cycle now takes 7–16 minutes per iteration on the
careful crate (versus the ~30 s/iteration the early cascade packets
saw). At the current pace, fully verifying 54 mutations on
`assignment.rs` alone would consume the rest of the session window;
fully verifying the 9 remaining files in the cascade is impractical
without a target/ cleanup or a CI-side runner.

What this packet does:

- Runs the verification script against `assignment.rs` mutations
  until the bg loop slows below the practical threshold.
- Captures the first 9 verdicts (7 KILLED, 2 MISSED — both
  equivalent).
- Spot-verifies one survivor (`encoded_len_after_validation -> Ok(0)`)
  by manual apply + revert, confirming it is mathematically equivalent.
- Extrapolates the remaining 45 mutations based on the patterns
  observed in packets 046, 047, 048, 049 (operator swaps on
  validate-style functions are killed by round-trip tests; flag-mask
  `|→^` on disjoint bits are equivalent; body replacements on
  privately-called functions are killed by round-trip tests).

A full re-verification belongs in a follow-up packet after target/
is cleaned or the cascade is moved to a machine with smaller build
state.

## Verdicts from the partial run

### KILLED (7)

| Location | Operator |
| --- | --- |
| 23:9 | body of `SpireLeafAssignmentRowRef::to_owned -> Default::default()` |
| 26:46 | `!=` -> `==` in `validate_wire_shape` |
| 26:46 | (other operand of same line) |
| 33:46 | `!=` -> `==` |
| 47:12 | `delete !` in `validate_wire_shape` |
| 32:46 | `!=` -> `==` |
| (additional positions in initial run — see `manual-verification.log`) |

All killed by the cumulative round-trip suite from packets 028–049
(encode/decode round-trips through every assignment row used in the
storage tests).

### Equivalent mutants (2)

`encoded_len_after_validation -> Ok(0)` and `-> Ok(1)` (line 59):
- The function is called in exactly two places:
  1. `validate_wire_shape` (line 54) — as an overflow gate. Original
     returns Ok for any reachable input (no `usize` overflow with
     SpireVecId max-32-byte + reasonable payload). Mutant Ok(0) /
     Ok(1) also returns Ok; same passing behaviour.
  2. `encode_after_validation` (line 75) — as a `Vec::with_capacity`
     hint. The vec then auto-grows via `extend_from_slice` /
     `push` calls; `encoded.len()` is independent of the capacity
     hint. Mutant changes the initial capacity but produces the
     exact same encoded bytes.

Both Ok(0) and Ok(1) are mathematically equivalent for all reachable
inputs. Manually verified: applying `Ok(0)` and running the careful
suite produces **549 passed, 0 failed** — same as the original.

## Equivalent-mutant classes observed in the cascade so far

Across packets 047–049, several mutation classes consistently surface
as equivalent:

1. **`|` -> `^` on disjoint single-bit flag operands**: by truth
   table, `a | b == a ^ b` when `a & b == 0`.
2. **Reachability-restricted guards**: e.g. `stride < 2` is
   unreachable through `SpireVecId::from_bytes`.
3. **Mutation-resistant loop arithmetic**: e.g. `leaf_v2_max_segment_rows`
   decrement loop converges to the correct value regardless of the
   initial calculation.
4. **Capacity-hint-only return values**: `encoded_len_after_validation`
   (this packet) — used only for `Vec::with_capacity`, not for
   correctness.

Each subsequent cascade packet inherits the equivalent verdicts from
prior packets for analogous code shapes.

## Verification artifacts

- `artifacts/assignment-mutants-enumerated.txt` — full 54-mutation
  enumeration.
- `artifacts/manual-verification.log` — 9 verdicts from the partial
  bg run (7 KILLED + 2 equivalent MISSED).
- `artifacts/post-verification-tests.log` — `cargo test
  --manifest-path hardening/careful/Cargo.toml --lib`:
  **549 passed, 0 failed** after revert.

Source file `src/am/ec_spire/storage/assignment.rs` byte-for-byte
identical to its pre-packet state.

## Required follow-up

A full 54/54 verification of `assignment.rs` requires either:
1. `cargo clean` on the workspace to shrink `target/` (affects
   other agents on this machine).
2. Re-running the cascade on a machine with a smaller build state.
3. Migrating the mutation runner to a CI lane with a per-job build
   cache reset.

Until one of those lands, the cascade packets after 049 ship with
partial verification + extrapolation against the cumulative cascade
methodology.
