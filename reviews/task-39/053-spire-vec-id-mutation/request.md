# Task 39 / 053 — SPIRE vec_id.rs mutation campaign (wire-format assertion follow-up)

## Goal

Address reviewer feedback 2026-05-21-03 on this packet: the
"encoder/decoder-symmetric constant" equivalence class could mask
wire-format regressions if any of the cited constants determine
on-disk byte layout.

## Result

**148 mutations enumerated → 140 KILLED + 8 equivalent,
0 non-equivalent survivors.** Improved from the prior back-fill
(133 KILLED + 15 equivalent) by adding 6 compile-time assertions
that pin the byte-layout constants to their expected values.

## Wire-format compile-time assertions

Each cited constant gets a `const _: () = assert!(NAME == N);`
right after its definition. Any mutation that changes the
expression's *value* breaks the assertion at compile time
(`exit=101`), killing the mutation. Mutations that preserve the
value via mathematical equivalence (e.g. `2 + 2 → 2 * 2 = 4`)
still pass the assertion — these are real equivalents.

Assertions added in `src/am/ec_spire/storage/vec_id.rs`:

- `SPIRE_ASSIGNMENT_ROW_FIXED_TAIL_BYTES == 15`
- `ROUTING_CHILD_ENTRY_FIXED_BYTES == 12`
- `TOP_GRAPH_OBJECT_BODY_PREFIX_BYTES == 28`
- `TOP_GRAPH_NODE_FIXED_BYTES == 16`
- `SPIRE_LEAF_V2_META_BODY_BYTES == 30`
- `SPIRE_PARTITION_OBJECT_V2_CHAIN_META_BODY_BYTES == 22`
- `SPIRE_PARTITION_OBJECT_V2_CHAIN_SEGMENT_PREFIX_BYTES == 14`

If the on-disk format intentionally changes, the corresponding
assertion must be updated in the same packet — providing the
"static byte-position assertion" lever the reviewer asked for.

## Net effect

| Class | Before | After |
| --- | ---: | ---: |
| KILLED | 133 | 140 |
| Equivalent (disjoint-flag) | 5 | 5 |
| Equivalent (symmetric-constant) | 10 | 0 |
| Equivalent (operand-value) | 0 | 3 |
| Non-equivalent survivor | 0 | 0 |

7 mutations previously classified as "symmetric-constant
equivalents" are now KILLED at compile time. 3 remaining
`+ -> *` mutations are operand-value-equivalent (`2 + 2 = 2 * 2
= 4`) so the constant value is genuinely unchanged.

## Methodology

Full per-mutation apply/test/revert with `CARGO_TARGET_DIR`
isolation. `cargo mutants --list` was re-run after the
assertions were added so line numbers in the enumeration file
match the patched source (previous PATCH-FAIL events were caused
by stale line numbers).

## Code change

- `src/am/ec_spire/storage/vec_id.rs`: 7 `const _: () = assert!(...)`
  lines added after each byte-layout constant.

No behavioral change.

## Validation

Artifacts under `reviews/task-39/053-spire-vec-id-mutation/artifacts/`:

- `vec-id-mutants-enumerated.txt` — refreshed 148 enumeration.
- `manual-verification.log` — 148/148 per-mutation verdicts.
- `post-verification-tests.log` — clean re-run after revert.

`triage.md` documents each surviving equivalent with line-level
justification.
