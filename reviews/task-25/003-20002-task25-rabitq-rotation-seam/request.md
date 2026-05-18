# Review Request: Task 25 Slice 3 — Rotation Front-End Seam

Scope:
- `src/quant/rabitq.rs`:
  - Introduces the `Rotation` trait (`dimensions()` + `apply(&[f32])`),
    `Send + Sync` so it can be shared via `Arc` between build and
    scan paths.
  - Adds `SrhtRotation` — the default impl for ADR-045 Stage 1. Wraps
    an `Arc<ProdQuantizer>` and delegates to `rotation::srht_padded`
    against the quantizer's sign vector.
  - `RaBitQQuantizer` now stores `rotation: Arc<dyn Rotation>`. New
    constructor `RaBitQQuantizer::new(rotation)` takes a prebuilt
    rotation; `RaBitQQuantizer::with_srht(dimensions, prod)` is the
    convenience path that mirrors the slice-2 call signature.
  - Adds a `custom_rotation_plugs_into_seam` unit test (tiny identity
    rotation inside the test module) demonstrating the trait object
    is the only contract an alternative rotation has to meet.

No change to any other file. The ADR-031 PQ-derived sidecar helpers
(`derive_persisted_sidecar_words`, `persisted_sidecar_word_count`)
still take a `ProdQuantizer` reference because they look at codebook
entries rather than rotating a vector — those helpers are not a
consumer of the `Rotation` seam.

Task: `plan/tasks/25-rabitq-quantizer.md` (Phase 1, slice 3 of 6).

Branch: `task25-rabitq-stage1-phase0` (slice 3 builds on `68a050b`).

## Problem

Slice 2 implemented `Quantizer::encode_code` / `prepare_scorer` by
reaching into `Arc<ProdQuantizer>` for SRHT state directly. That
coupled two concerns that ADR-045 Stage 1 deliberately wants
separated:

1. **The rotation.** Today SRHT; tomorrow ADR-036 OPQ (task 20) or a
   learned rotation — any decorrelating linear map that leaves the
   sign-bit distribution favorable.
2. **The encoder / scorer / estimator.** The RaBitQ math is
   independent of which rotation produced the input.

Without the seam, swapping rotations means editing `rabitq.rs`. With
the seam, swapping rotations means introducing a new `impl Rotation`
and handing it to `RaBitQQuantizer::new`. Task 20 (OPQ) and the
Phase 2 recall study both benefit: OPQ lands as a second `Rotation`
impl; Phase 2's feasibility binary can A/B the same RaBitQ scorer
across SRHT vs. identity vs. (if we want) a fixed random Gaussian
projection, without touching the encode/score path.

## Approach

### Why a trait object, not a type parameter

`Arc<dyn Rotation>` rather than `RaBitQQuantizer<R: Rotation>`. Two
reasons:

1. The AM's `TqScanOpaque` and build context hold concrete
   quantizer types. Generic propagation across those structs would
   cascade through the scan lifecycle — not free, and not needed
   until there is real call-site polymorphism.
2. Rotation is called once per vector at encode and once per query
   at `prepare_scorer`. The virtual call is amortized across the
   per-coordinate sign loop that follows. There is no hot inner
   loop where the indirection shows up.

If Phase 2 or Phase 3 ever needs monomorphized dispatch (it should
not — see the cost model above), the trait-object seam is a
compatible step toward adding a generic variant later.

### `SrhtRotation::new` asserts matching dimensions

The constructor asserts `prod.original_dim == dimensions` up front
so misconfigurations fail at construction rather than at the first
`apply`. This matches the existing invariant in `ProdQuantizer`'s
`prepare_ip_query_binary_sign_no_qjl_4bit`.

### `SrhtRotation::prod()` accessor

`SrhtRotation` exposes the underlying `Arc<ProdQuantizer>` through a
small accessor. ADR-031's PQ-derived sidecar path
(`derive_persisted_sidecar_words`) still needs the full quantizer to
reach the codebook; when we hold a `RaBitQQuantizer` backed by
`SrhtRotation`, we can recover the `ProdQuantizer` for that path
without storing it twice. Non-SRHT rotations have nothing to expose
here — they would simply not support the ADR-031 optimization and
fall back to the canonical encode.

## Verification

- `cargo check --lib` clean.
- `cargo test --lib quant::rabitq` — five tests pass. The new
  `custom_rotation_plugs_into_seam` test defines an `Identity`
  rotation in the test module and verifies the first sign byte of
  the encode output matches the expected bit pattern (`0x55` for a
  `[+, -, +, -, ...]` input at D=16). That proves an external
  `Rotation` impl lands cleanly without touching `rabitq.rs`
  internals.

## What this slice does NOT do

- No OPQ implementation (that is task 20's `plan/tasks/20-opq-rotation.md`).
- No change to the ADR-031 PQ-derived sidecar path. It keeps taking
  `&ProdQuantizer`; the `Rotation` seam is only on the canonical
  encode.
- No AM caller change. Slice 2 left AM code calling
  `ProdQuantizer::binary_sign_*` methods; slice 3 adds no new AM
  consumers.

## Open questions for reviewer

1. Is `Rotation::apply(&self, v: &[f32]) -> Vec<f32>` the right
   signature, or would you rather `apply_into(&self, v: &[f32], out:
   &mut [f32])` to let callers pool buffers? The allocation is one
   `Vec<f32>` per encode (1536 × 4 B = 6 KB at D=1536) — probably
   negligible against the per-vector work, but measurable under
   million-row builds. Happy to add `apply_into` in slice 4 if you
   want it sized against the estimator work.
2. `SrhtRotation::prod()` exposes an implementation detail. I could
   narrow it to `fn as_pq_codebook_source(&self) -> Option<&ProdQuantizer>`
   on the trait so only PQ-backed rotations surface the quantizer
   reference. Overkill for now; raising it so you can flag if the
   current accessor leaks too much.
