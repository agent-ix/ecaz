# Review Request: surface the DiskANN unit-norm precondition in the AM

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `src/am/ec_diskann/mod.rs`
- `src/am/ec_diskann/ambuild.rs`
- `src/am/ec_diskann/routine.rs`
- `src/am/ec_diskann/scan.rs`

## What this packet is

This is the first response slice to reviewer feedback on packet `11084`.

The reviewer called out one AM-local handoff blocker: the V0 DiskANN exact
distance wrapper uses `1 - ip`, which assumes unit-normalized source vectors,
but the code did not validate or even name that precondition anywhere durable.

This packet closes that gap inside `src/am/ec_diskann/` without widening into
loader or corpus tooling work. It also folds in two tiny cleanup nits from the
same review:

- remove the duplicated `maybe_check_for_interrupts()` body
- rename the shadowed vacuum repair neighbor-set binding so the local intent is
  visible

## What changed

### `src/am/ec_diskann/mod.rs`

Centralized the V0 unit-norm contract:

```rust
pub(super) const ECDISKANN_UNIT_NORM_DISTANCE_BIAS: f32 = 1.0;
pub(super) const ECDISKANN_UNIT_NORM_EPSILON: f32 = 0.01;
pub(super) const ECDISKANN_UNIT_NORM_BUILD_SAMPLE_CAP: usize = 1024;
```

and added shared helpers to:

- compute `||v||`
- validate a single source vector
- validate a bounded prefix sample of build-time source vectors
- emit a warning from real runtime paths when the V0 precondition is violated

The constant now carries the missing comment explaining why `C = 1` exists:
the V0 exact-distance wrapper only preserves the `<#>` ordering when the input
vectors are unit-normalized.

The same file now also owns the shared `maybe_check_for_interrupts()` helper,
which replaces the duplicated copies in `routine.rs` and `scan.rs`.

Added pure unit coverage for the new validation helpers.

### `src/am/ec_diskann/ambuild.rs`

`flush_build_state(...)` now samples up to `1024` source vectors and warns if
their norms drift outside the V0 unit-norm band before Vamana build starts:

```rust
warn_on_non_unit_source_vector_sample(
    &source_refs,
    ECDISKANN_UNIT_NORM_BUILD_SAMPLE_CAP,
    "ambuild",
);
```

This is intentionally a warning, not a hard error, to match the review's
allowed v0-permissive version while still making non-unit corpora visible to
the operator instead of silently building a misleading graph.

### `src/am/ec_diskann/routine.rs`

`ec_diskann_aminsert` now warns on non-unit incoming source vectors before the
bootstrap or live-insert path runs:

```rust
warn_on_non_unit_source_vector(&source_vector, "aminsert");
```

This makes the same V0 precondition visible on the insert path without moving
the validation into the lower-level payload helpers that unit tests use to
exercise unrelated metadata/error cases.

The same file also now imports the shared interrupt helper from `mod.rs`, and
the final `existing_neighbor_set` shadow in
`plan_vacuum_fill_candidates_for_target(...)` is renamed to
`prior_neighbor_set`.

### `src/am/ec_diskann/scan.rs`

Dropped the duplicate local interrupt helper and now reuses the shared one from
`mod.rs`.

## Why this slice

- It closes the main reviewer blocker in the DiskANN AM itself.
- It makes the V0 unit-norm assumption explicit and operator-visible at the two
  runtime entrypoints that matter for handoff: `ambuild` and `aminsert`.
- It keeps scope tight: no loader changes, no SQL wrapper changes, no new
  corpus-prep surface.
- It folds two trivial ec_diskann-local cleanup nits into the same slice
  instead of carrying them as separate follow-ups.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also passed on `pg18` for this checkpoint:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Notable coverage in that run:

- `am::ec_diskann::tests::validate_source_vector_unit_norm_accepts_unit_vectors`
- `am::ec_diskann::tests::validate_source_vector_unit_norm_rejects_non_unit_vectors`
- `am::ec_diskann::tests::validate_source_vector_unit_norm_sample_reports_sample_stats`
- `am::ec_diskann::routine::tests::pg_test_ec_diskann_build_coalesces_duplicate_vectors`
- `am::ec_diskann::routine::tests::pg_test_ec_diskann_vacuum_refills_broken_neighbor_slot`

## Follow-ups intentionally not in this packet

- Task-file / handoff updates for the remaining reviewer items. Those are a
  separate docs slice.
- Any attempt to lift the unit-norm precondition itself. This packet only makes
  the existing V0 contract explicit and visible.
- The unrelated `vamana.rs` reserved-import nit.
