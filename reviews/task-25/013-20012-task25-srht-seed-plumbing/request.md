# Review Request: Task 25 Slice 13 — SRHT Rotation Seed Plumbing

Scope:
- `src/quant/rabitq.rs`:
  - `SrhtRotation` now stores its sign vector directly
    (`Arc<Vec<f32>>`) rather than borrowing from a `ProdQuantizer`.
  - New constructor `SrhtRotation::with_seed(dim, seed)` —
    produces the sign vector via `rotation::sign_vector(…, seed)`,
    no `ProdQuantizer` dependency. This is the recommended path
    for prod call sites.
  - Existing constructor `SrhtRotation::new(dim, prod)` kept for
    the ADR-031 PQ-derived sidecar path. Internally clones the
    quantizer's signs into the rotation's own `Arc<Vec<f32>>`; the
    rotation is now self-sufficient if the quantizer is freed.
  - `SrhtRotation::prod()` signature changed from
    `&Arc<ProdQuantizer>` to `Option<&Arc<ProdQuantizer>>`.
    Returns `Some` only when built via `::new`. There are zero
    external callers today; task 27 would have been the first.
  - New `SrhtRotation::seed() -> Option<u64>` accessor; returns
    `Some` only when built via `::with_seed`.
  - New `RaBitQQuantizer::with_seeded_srht_bits(dim, seed, bits)`
    convenience constructor combining seed-based rotation + q-bit
    encoder.
  - Module docstring updated to describe seed discipline:
    "Tests across the crate pin the canonical seed at
    `DEFAULT_QUANT_SEED = 42` for reproducibility; prod
    deployments should pass a fresh seed per index build."
  - New unit test
    `srht_seeded_rotation_is_deterministic_and_independent_of_prod`
    — same seed → same rotation output; different seeds → different
    outputs; `seed()`/`prod()` return the expected `Some`/`None`
    pattern for each constructor.

Task: `plan/tasks/25-rabitq-quantizer.md` (slice 13 of the extended
plan; cleans up the prod-hygiene gap raised in the slice-12
discussion).

Branch: `task25-rabitq-stage1-phase0` (slice 13 builds on `7edc1e8`).

## Problem

Before this slice every `SrhtRotation` inherited its signs from a
`ProdQuantizer::cached(dim, bits, seed)` handle, and in practice
that meant `seed = DEFAULT_QUANT_SEED = 42` everywhere. Good for
test reproducibility; bad for prod where every index should have
its own statistically independent rotation (otherwise a
SRHT-degenerate input for one index would degrade every index on
the box).

The math does not care which seed we use — RaBitQ's concentration
guarantees only require that the rotation be independent of the
input data. But `seed = 42` everywhere trivially violates that for
multi-index deployments.

## Approach

### Two constructors, one semantic

- `with_seed(dim, seed)` is the "clean" path. Seed is first-class,
  stored via `seed()` accessor, and no `ProdQuantizer` appears
  anywhere in the rotation's state.
- `new(dim, prod)` is the "ADR-031 compat" path. It exists
  because the PQ-derived sidecar path (`derive_persisted_sidecar_words`)
  needs to reach the PQ codebook through the rotation handle. If
  a future refactor moves that accessor to its own seam, `new`
  can be collapsed away entirely.

Both produce a rotation that behaves identically under `apply`.
The difference is purely which state the rotation carries.

### Why return type change on `prod()` is safe

`prod()` now returns `Option<&Arc<ProdQuantizer>>` instead of
`&Arc<ProdQuantizer>`. Zero external callers today (grepped `src/`
and `crates/`); the access pattern was designed for task 27 which
has not started. Changing now rather than later avoids a
compatibility shim when that consumer lands.

### Storage lives in the rotation, not the quantizer

Previously `apply` called `rotation::srht_padded(v, &self.prod.signs)`.
Now it calls `rotation::srht_padded(v, &self.signs)` where
`self.signs: Arc<Vec<f32>>` is owned by the rotation. The clone at
construction time is a one-shot cost; the `Arc` wrapper means the
signs are shared across all RaBitQ encode / prepare_scorer calls
without further allocation.

### What prod call sites should look like now

```rust
// Per-index build (pseudocode):
let rotation_seed = random_u64();                     // store in metadata page
let quant = RaBitQQuantizer::with_seeded_srht_bits(dim, rotation_seed, bits)?;
```

At scan time, read `rotation_seed` from the metadata page,
reconstruct via the same constructor. Deterministic, independent
per index, no shared global state.

The AM plumbing to actually write the seed into the metadata page
is out of scope for slice 13; this slice only exposes the seam.
When task 27 (Symphony AM) starts, the seed lands in its
`INDEX_FORMAT_*` constant alongside the other RaBitQ scalars.

## Verification

- `cargo check --lib` clean.
- `cargo test --lib` — 546 passed (545 pre-slice + the new
  `srht_seeded_rotation_is_deterministic_and_independent_of_prod`).
  0 regressions.

## What this slice does NOT do

- No changes to AM code. The production `ec_hnsw` still uses
  `ProdQuantizer`'s signs directly (through its own PQ path); the
  rotation seed plumbing lands only on `src/quant/rabitq.rs`'s
  surface. A real prod deployment of RaBitQ will carry this
  seed in the AM's metadata page, but that is task-27 work.
- No change to the feasibility harness CLI. The harness still
  takes `--seed` and passes it to `ProdQuantizer::cached`. The
  harness could be moved to `SrhtRotation::with_seed` for
  cleanliness; low priority since the harness is deterministic
  by design.
- No containment impact on anything else — still purely
  `src/quant/rabitq.rs`.

## Open questions for reviewer

1. `SrhtRotation::new(dim, prod)` is kept exclusively for the
   ADR-031 PQ-derived sidecar compat path. When task 27 lands its
   own Symphony AM, it will most likely go through `with_seed`
   and the `new` constructor can be deprecated (deleted per the
   project rule). Worth scheduling that removal on the task-27
   checklist now.
2. `SrhtRotation::prod()` return type is now `Option`. Any
   downstream that grabs it panics today (zero such callers), but
   a future careless `.prod().signs` would need `.prod().unwrap().signs`.
   Acceptable since the callsite is intrinsically path-dependent
   on the rotation's construction mode.
