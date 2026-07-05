# Review Request: Task 25 — RaBitQ Quantizer Module Skeleton (ADR-045 Stage 1, Slice 1)

Scope:
- `src/quant/rabitq.rs` — new module. Skeleton only: `RaBitQQuantizer`
  struct (holds `dimensions` + rotation `Arc<ProdQuantizer>` seam),
  `RaBitQScorer` struct, `DistanceEstimate { estimate, bound }` public
  type, and `Quantizer` / `QueryScorer` trait impls. `encode_code`,
  `prepare_scorer`, and `score` are `unimplemented!()` stubs with
  slice-2 pointers; `code_len` and `sign_bytes` are real (sign-bit
  portion + `RABITQ_NORM_LEN = 4`). `wire_format_version` returns `0`
  pending a dedicated `INDEX_FORMAT_*` constant in slice 2.
- `src/quant/mod.rs` — `pub mod rabitq;` registration, inserted in the
  existing alphabetical block next to `qjl` / `rotation`.

Task: `plan/tasks/25-rabitq-quantizer.md` (Phase 1, slice 1 of 6).
Supersedes ADR-031 in scope per ADR-045 Stage 1.

Branch: `task25-rabitq-stage1-phase0` cut from `main` at `2e664d8`.

## Problem

The ADR-031 binary-prefilter surface (sign-bit encode, persisted
sidecar, cached runtime scorer, SIMD POPCNT) already lives on `main`,
but it lives **inside** `src/am/ec_hnsw/` and `src/quant/prod.rs`. That
placement is accidental: ADR-031 originally shipped the work as an
inline prefilter, not as a standalone quantizer family. ADR-045
Stage 1 asks for the same scoring kernel to be promoted to a
first-class `Quantizer` under the ADR-041 module-seam discipline, with
three net-new pieces on top: an explicit rotation seam, per-vector
f32 norm storage (the RaBitQ scalar), and an unbiased distance
estimator with an error-bound API that task 27's Stage 3 no-rerank
query path will consume.

This slice lands the module file and the trait-surface outline only —
no behavior is moved, no call sites change. The point is to make the
public API shape visible in `src/quant/` from the first commit so the
subsequent slices can be reviewed as concrete, self-contained moves
against a known target.

## Approach

- `RaBitQQuantizer` is parameterized by `dimensions` and holds the
  rotation as `Arc<ProdQuantizer>`. Slice 3 introduces a proper
  `Rotation` trait; until then, keeping `Arc<ProdQuantizer>` matches
  how `PqFastScanQuantizer` (`src/quant/grouped_pq.rs:23`) already
  handles its rotation seam and avoids committing to a layout before
  the move in slice 2 constrains it.
- `code_len()` returns `sign_bytes() + RABITQ_NORM_LEN`
  (`dim.div_ceil(8) + 4`). At D=1536 that is 196 B, vs. 768 B for PQ4 —
  the storage point the ADR-045 Stage 1 gate asks recall to hold at.
- `DistanceEstimate { estimate, bound }` is declared empty-bodied in
  slice 1 so task 27's contract has a named type to reference in
  advance of the slice-4 estimator work. It is `Copy` on purpose:
  Stage 3's hot loop is one struct per scored candidate.
- `unimplemented!()` rather than a trivial fallback body in
  `encode_code` / `prepare_scorer` / `score`. The skeleton is not a
  shipping quantizer yet and nothing dispatches to it; a loud panic at
  the call site is the right failure mode if a misrouted slice-2
  change lands prematurely.
- `wire_format_version() -> 0` is a placeholder. Slice 2 adds a
  dedicated `INDEX_FORMAT_*` constant in `src/am/page.rs` and returns
  it; no AM path reads this value yet because no AM code holds a
  `&dyn Quantizer` pointing at `RaBitQQuantizer`.

## What this slice does NOT do

Explicitly out of scope for slice 1 — each lands in a later slice
listed in the module-level doc comment:

- No move of `ProdQuantizer::binary_sign_words_from_packed_no_qjl_4bit`
  or `training::derive_persisted_binary_words` (slice 2).
- No change to `src/am/ec_hnsw/scan.rs` binary prefilter entry points
  (slice 2 — `disable_binary_prefilter` / `force_binary_derivation`
  GUCs evaluated there; keep if still useful as quantizer-level
  diagnostics, otherwise delete per "deprecate = delete" rule).
- No rotation trait (slice 3).
- No estimator implementation — `DistanceEstimate` is declared but not
  produced anywhere (slice 4).
- No `Family::RaBitQ` reloption variant. Reloption wiring is a Phase 3
  concern, gated on the Phase 2 recall study passing.

## Verification

- `cargo check --lib` clean.
- `cargo test --lib quant::rabitq` — one unit test
  (`code_len_matches_dimension`) exercising the only non-stubbed
  method. Passes at D=1536: `sign_bytes = 192`, `code_len = 196`.

## Open questions for reviewer

1. `wire_format_version() -> 0` in the skeleton — acceptable, or
   would you prefer the slice be blocked until `INDEX_FORMAT_*` is
   allocated? (I'd rather land slice 2 with the constant, since slice 1
   has no reader.)
2. `DistanceEstimate` field naming: `estimate` / `bound` vs. the
   RaBitQ-paper-standard `ip_hat` / `epsilon`. I took the plain-English
   pair so task 27's docs read naturally; easy to rename before slice 4.
3. Should `RaBitQQuantizer::new` reject non-multiple-of-8 dimensions
   up front? Current behavior via `div_ceil(8)` tolerates any D ≥ 1,
   which matches how ADR-031 already treats odd dims.
