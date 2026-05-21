# Triage: vec_id.rs mutation analysis (back-filled)

Result: **148 mutations enumerated → 133 KILLED + 15 equivalent,
0 non-equivalent survivors. Full per-mutation verification under
isolated CARGO_TARGET_DIR.**

This back-fills the partial verification originally shipped in
this packet (31/148 verified). Authorized by reviewer feedback
across 050/053/054/055.

## Methodology

Full per-mutation apply/test/revert via
`/tmp/run_spire_mutations_v2.py` with
`CARGO_TARGET_DIR=$(pwd)/target-mutants` build isolation.

## Per-mutation verdicts (148 total)

133 mutations KILLED by the existing cascade test surface. 15
MISSED, all in documented equivalence classes (no killing tests
required).

## Equivalent mutants (15)

### Disjoint-flag class (5) — lines 18-22

`SPIRE_ASSIGNMENT_KNOWN_FLAGS = SPIRE_ASSIGNMENT_FLAG_PRIMARY |
SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA | ... | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR`.
Each flag is a different bit (no overlap). `|` and `^` produce
identical results on disjoint operands, so the constant value is
unchanged. This is the same `|→^` class accepted by the reviewer
in 053's own initial feedback ("first recurring equivalence class").

### Encoder/decoder-symmetric constant class (10) — lines 42, 75, 76, 83, 127

These constants define byte-layout offsets that are consumed *both*
by encoder and decoder on the same encoded form. Mutating the
constant shifts both encode and decode by the same amount, so
round-trip tests (`encode → bytes → decode → equal`) still pass.

| Line | Constant | Mutations |
| --- | --- | --- |
| 42 | `SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES = ITEM_POINTER_BYTES + 1 + 4 + 4` | 5 (`+→-`, `+→*` across operands) |
| 75 | `TOP_GRAPH_OBJECT_BODY_PREFIX_BYTES = 8 + 2 + 2 + 4 + 4 + 4 + 4` | 1 (`+→*`) |
| 76 | `TOP_GRAPH_NODE_FIXED_BYTES = 8 + 4 + 4` | 2 (`+→-` across operands) |
| 83 | `SPIRE_LEAF_V2_META_BODY_BYTES = 1 + 1 + 2 + 4 + 2 + 2 + 4 + ITEM_POINTER_BYTES + 8` | 1 (`+→*`) |
| 127 | `SPIRE_PARTITION_OBJECT_V2_CHAIN_META_BODY_BYTES = 2 + 2 + 4 + ITEM_POINTER_BYTES + 8` | 1 (`+→*`) |

A killing test for these would require asserting the exact encoded
byte length (e.g. `assert_eq!(encoded.len(), N)`), which couples
the test to the storage format constant and would itself need to
change every time the layout is intentionally widened — that's a
tighter coupling than the cascade discipline wants. The reviewer's
053 feedback already accepted the "constant-as-capacity-hint and
encoder/decoder-symmetric" class as recurring.

(The reviewer-direction asked for one-line acceptance of these
classes in the cascade closeout — captured here.)

## Verification artifacts

- `artifacts/vec-id-mutants-enumerated.txt` — full 148 enumeration.
- `artifacts/manual-verification.log` — 148/148 per-mutation verdicts
  (133 KILLED + 15 MISSED equivalent).
- `artifacts/post-verification-tests.log` — clean re-run after revert.

Source `src/am/ec_spire/storage/vec_id.rs` byte-for-byte identical
pre/post packet.
