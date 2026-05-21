# Triage: vec_id.rs mutation analysis (back-filled with wire-format assertions)

Result: **148 mutations enumerated → 140 KILLED + 8 equivalent,
0 non-equivalent survivors. Full per-mutation verification under
isolated CARGO_TARGET_DIR, plus 6 new compile-time wire-format
byte-layout assertions.**

Improved from the previous back-fill (133 KILLED + 15 equivalent)
in response to reviewer feedback 2026-05-21-03 which raised the
concern that "encoder/decoder-symmetric constant" equivalents might
mask wire-format regressions.

## Methodology

Full per-mutation apply/test/revert via
`/tmp/run_spire_mutations_v2.py` with
`CARGO_TARGET_DIR=$(pwd)/target-mutants` build isolation.

## Wire-format compile-time assertions

Added `const _: () = assert!(...)` lines pinning the expected
value of each byte-layout constant. Any mutation that changes the
constant's *value* now fails at compile time with `exit=101`,
killing it. Mutations that change the *expression* but preserve
the value (e.g., `2 + 2 → 2 * 2 = 4`) still pass the assertion
because they are mathematically equivalent — these are documented
in the "operand-value equivalents" class below.

Assertions added:

```rust
const _: () = assert!(SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES == 15);
const _: () = assert!(ROUTING_CHILD_ENTRY_FIXED_BYTES == 12);
const _: () = assert!(TOP_GRAPH_OBJECT_BODY_PREFIX_BYTES == 28);
const _: () = assert!(TOP_GRAPH_NODE_FIXED_BYTES == 16);
const _: () = assert!(SPIRE_LEAF_V2_META_BODY_BYTES == 30);
const _: () = assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_META_BODY_BYTES == 22);
const _: () = assert!(SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_PREFIX_BYTES == 14);
```

Net effect:
- 7 previously-MISSED "symmetric constant" mutations are now KILLED
  via compile-time assertion failures (`exit=101`).
- 8 mutations remain MISSED — all genuine equivalents (5 disjoint-flag
  + 3 operand-value).

## Per-mutation verdicts (148 total)

140 KILLED. 8 MISSED, all in two equivalence classes.

## Equivalent mutants (8)

### Disjoint-flag class (5) — lines 18-22

`SPIRE_ASSIGNMENT_KNOWN_FLAGS = PRIMARY | BOUNDARY_REPLICA |
TOMBSTONE | DELTA_INSERT | DELTA_DELETE | STALE_LOCATOR`. All six
operands are single-bit (0x0001, 0x0002, 0x0004, 0x0008, 0x0010,
0x0020 — disjoint). For disjoint operands, `|` and `^` produce
identical bit values. Reviewer-accepted class.

### Operand-value-equivalence class (3) — lines 77, 87, 132

| Line | Column | Mutation | Operands | Why equivalent |
| --- | --- | --- | --- | --- |
| 77 | 57 | `+ -> *` in `TOP_GRAPH_OBJECT_BODY_PREFIX_BYTES` | `2 + 2` | `2 + 2 = 4` and `2 * 2 = 4` — constant value (28) unchanged. |
| 87 | 23 | `+ -> *` in `SPIRE_LEAF_V2_META_BODY_BYTES` | `2 + 2` | Same: `2 + 2 = 2 * 2 = 4` — constant value (30) unchanged. |
| 132 | 7 | `+ -> *` in `SPIRE_PARTITION_OBJECT_V2_CHAIN_META_BODY_BYTES` | `2 + 2` | Same: `2 + 2 = 2 * 2 = 4` — constant value (22) unchanged. |

These three mutations replace `+` with `*` between two `2`s. Both
operations yield the same result, so the byte-layout constant is
unchanged, the compile-time assertion still passes, and downstream
encode/decode behavior is identical. There is no observable
difference — these are real mathematical equivalents, not "test
gap" survivors.

## Verification artifacts

- `artifacts/vec-id-mutants-enumerated.txt` — refreshed 148 enumeration.
- `artifacts/manual-verification.log` — 148/148 per-mutation verdicts
  (140 KILLED + 8 equivalent).
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/vec_id.rs` differs from the
pre-packet baseline only by the addition of 7 compile-time
assertions (no behavioral change).
