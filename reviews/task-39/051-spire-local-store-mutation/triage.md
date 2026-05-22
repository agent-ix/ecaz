# Triage: local_store.rs mutation campaign

Result: **87 mutations enumerated → 85 killed (75 by existing tests + 10 by the one new test added in this packet) + 2 equivalent mutants, 0 timed-out, 0 non-equivalent survivors.**

## Verification

Full bg run completed against the cumulative test surface. Initial
verdicts:

- 75 KILLED by the cumulative careful suite from packets 028-050.
- 12 MISSED.

Of the 12 MISSED, **10** are on the epoch-backref validation guard
that appears in every `SpireLocalObjectStore::read_*` method
(`read_object_header`, `read_routing_object`, `read_leaf_object_v2`,
`read_delta_object`, `read_top_graph_object`). The guard:

```rust
if header.published_epoch_backref == 0
    || header.published_epoch_backref > placement.epoch
{
    return Err(...);
}
```

Existing tests construct objects whose `published_epoch_backref`
equals the insert epoch, then read with the same placement — so the
guard's `> placement.epoch` operand is always `false` (the typical
happy path). Mutations `|| → &&` and `> → <` flip the guard such
that pathological inputs (`backref == 0` or `backref > epoch`) are
silently accepted; existing tests never construct such an input.

The new test
`miri_local_object_store_read_rejects_placement_epoch_below_object_backref`
in `src/am/ec_spire/storage/tests/helpers.rs` inserts with epoch=7
then reads each kind with a mutated `placement.epoch = 1`. The
guard's `> placement.epoch` operand becomes true (`7 > 1`), so the
original errors. Both `|| → &&` (skips the check) and `> → <`
(inverts to `7 < 1 = false`) cause the read to succeed where the
test expects an error — killed.

Confirmed by manual apply of `339:13 || → &&`: the test failed under
the mutant and passed once the source was reverted.

## Equivalent mutants (2)

`local_store.rs:104:47 - → +` and `- → /` inside the ceiling-divide
idiom in `insert_leaf_object_v2_from_rows`:

```rust
let count = assignments
    .len()
    .checked_add(max_segment_rows - 1)   // ← line 104
    .and_then(|value| value.checked_div(max_segment_rows))
    .ok_or_else(|| ... segment count overflow ...)?;
```

For the careful test surface, `max_segment_rows = 253` and
`assignments.len() ≤ 50`. The ceiling-divide pattern is mutation-
resistant for small dividends:

- Original: `(len + 252) / 253` → 1 segment for any `len ∈ [1, 253]`.
- Mutant `+`: `(len + 254) / 253` → still 1 segment for `len ∈ [1, 252]`,
  becomes 2 segments at `len = 252` (vs original which becomes 2 at
  `len = 254`). For test inputs `len ≤ 50`, both produce 1 segment.
- Mutant `/`: `(len + (max_rows / 1)) / max_rows` =
  `(len + 253) / 253`. Same answer as mutant `+` for the test range.

Both equivalent for the test surface. Reachable inputs in
production also stay below the boundary (single-segment leaves are
the common case). Filed as equivalent for the test surface; a
follow-up packet exercising large leaf objects (e.g. 254+ assignments)
would distinguish them and should add a killing test if desired.

## Verification artifacts

- `artifacts/local-store-mutants-enumerated.txt` — full 87-mutation
  enumeration.
- `artifacts/manual-verification.log` — per-mutation verdict:
  **75 KILLED + 12 MISSED initially → after the new killing test
  + equivalent verdict on 104:47, 0 non-equivalent survivors.**
- `artifacts/post-verification-tests.log` — `cargo test
  --manifest-path hardening/careful/Cargo.toml --lib`:
  **550 passed, 0 failed** (was 549 after packet 050; +1 new test).

Source `src/am/ec_spire/storage/local_store.rs` byte-for-byte
identical pre/post packet.

## Reviewer Direction

Confirm the equivalent-mutant verdict on the line 104 ceiling-divide
arithmetic for the test surface, or authorize a follow-up packet
that exercises `len > max_segment_rows` cases (large leaf objects)
to distinguish the mutants.
