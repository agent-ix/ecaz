# Review Request: Task 25 Slice 2 — Graduate ADR-031 Binary Surface into `src/quant/rabitq.rs`

Scope:
- `src/quant/rabitq.rs` — the skeleton from slice 1 now has real
  `Quantizer::encode_code` / `prepare_scorer` / `QueryScorer::score`
  impls, and absorbs the ADR-031 primitives:
  - `sign_words_from_rotated` (moved from `prod.rs`)
  - `sign_words_from_packed_4bit` (moved from `prod.rs`)
  - `hamming_similarity` (moved from `prod.rs`, was `binary_sign_similarity`)
  - `binary_sign_lookup_4bit` (moved from `prod.rs`)
  - `BinarySignNoQjl4BitQuery` struct (moved from `prod.rs`)
  - `persisted_sidecar_word_count` (moved from `src/am/common/training.rs`,
    was `persisted_binary_sidecar_word_count`)
  - `derive_persisted_sidecar_words` (moved from `src/am/common/training.rs`,
    was `derive_persisted_binary_words`)
- `src/quant/prod.rs` — the three private free fns and the codebook
  sign-lookup helper deleted; `ProdQuantizer::prepare_ip_query_binary_sign_no_qjl_4bit`,
  `binary_sign_words_from_packed_no_qjl_4bit`, and
  `score_binary_sign_words_no_qjl_4bit` now delegate into `crate::quant::rabitq::*`.
  `BinarySignNoQjl4BitQuery` is re-exported via
  `pub use crate::quant::rabitq::BinarySignNoQjl4BitQuery;` so existing
  `crate::quant::prod::BinarySignNoQjl4BitQuery` imports elsewhere keep
  resolving without a second pass of edits.
- `src/am/common/training.rs` — `derive_persisted_binary_words` and
  `persisted_binary_sidecar_word_count` deleted. The unused
  `ProdQuantizer` import dropped.
- `src/am/ec_hnsw/build.rs` — the two `training::derive_persisted_binary_words`
  call sites switched to `crate::quant::rabitq::derive_persisted_sidecar_words`.
  The in-file `persisted_binary_sidecar_word_count` shim retained as a
  single-line wrapper around `rabitq::persisted_sidecar_word_count`
  (it keeps the existing import surface stable for five nearby call
  sites; slice 3 or later can flatten it if reviewers prefer).

Task: `plan/tasks/25-rabitq-quantizer.md` (Phase 1, slice 2 of 6).

Branch: `task25-rabitq-stage1-phase0` (slice 2 builds on `3afbf4b`).

## Problem

Slice 1 left the ADR-031 binary-prefilter primitives where they
originally landed — private free fns in `src/quant/prod.rs`, helper
wrappers in `src/am/common/training.rs`. That made `rabitq.rs` a
skeleton that could not actually encode or score. Slice 2 graduates
those primitives into `rabitq.rs` so the module becomes the single
owner of the binary encode/score surface, and implements the
`Quantizer` / `QueryScorer` trait methods against that surface.

## Approach

### Two encode paths, one module

The module now exposes two encode paths deliberately, because
ADR-031's optimization (deriving sign words from an already-PQ-packed
code, avoiding a second SRHT rotation) is not expressible through the
`Quantizer::encode_code(&[f32])` trait:

1. **Canonical RaBitQ encode** (`Quantizer::encode_code`). Rotates
   the raw vector, extracts sign bits into a `dim/8`-byte payload,
   and appends the rotated vector's L2 norm as a trailing 4-byte
   `f32`. `prepare_scorer` mirrors this on the query side. This is
   the path that Phase 2's feasibility study (slice 5) will exercise
   and that task 27's Stage 2 build will consume.
2. **ADR-031 PQ-derived sidecar** (`derive_persisted_sidecar_words`).
   Takes a `ProdQuantizer` reference and a packed PQ code, looks up
   the sign of each codebook entry via `binary_sign_lookup_4bit`, and
   emits the sign words. This is the AM-side build optimization that
   ADR-031 already ships; it lives in `rabitq.rs` now but remains a
   free function because it needs the PQ codebook state that the
   `Quantizer` trait does not surface.

These two paths produce equivalent sign bits — the PQ-derived path
is the ADR-031 cheat that the canonical path does not take.

### Slice 2 score is a surrogate, not the estimator

`RaBitQScorer::score` in this slice computes
`hamming_similarity(q, c) * q_norm * c_norm / dim`. That is a
coarse cosine-surrogate, **not** the RaBitQ unbiased estimator. The
estimator lands in slice 4; the slice 2 body exists so the trait
surface is exercised end-to-end in a round-trip unit test. The AM
hot paths still call `ProdQuantizer::score_binary_sign_words_no_qjl_4bit`
(which now delegates to `rabitq::hamming_similarity`), so this
surrogate score is not on any production path.

### Why `ProdQuantizer` keeps its `binary_sign_*` methods

Two reasons:

1. Those methods need `self.codebook`, `self.signs`, and
   `self.original_dim` to interpret a packed code; the `Quantizer`
   trait surface does not carry those. Keeping them on
   `ProdQuantizer` colocates them with their state.
2. Rewriting the ~12 AM call sites to go through `&dyn Quantizer`
   dispatch is a real orchestration change (the AM would need to
   hold a `RaBitQQuantizer` next to its existing `ProdQuantizer`,
   which changes the slot layout in `TqScanOpaque` and the build
   context). That is the slice that comes with the Phase 3
   reloption wiring, gated on the Phase 2 recall study passing.

The method bodies are one-liners that delegate into `rabitq::` — the
logic is in `rabitq.rs`, the method is a thin call-through with
the `binary_sign_no_qjl_4bit_supported()` assertion.

### Diagnostic GUCs kept

`ec_hnsw.disable_binary_prefilter` and `ec_hnsw.force_binary_derivation`
stayed in `src/am/ec_hnsw/options.rs`. They are scan-orchestration
knobs ("should the AM use the prefilter? should it derive from the
packed code or the persisted sidecar?"), not quantizer-level knobs.
The quantizer module has no opinion about either question. Deleting
them would remove working A/B diagnostic surface; keeping them costs
nothing since the "deprecate = delete" rule targets dead paths, not
live toggles.

## Verification

- `cargo check --lib` clean, zero warnings.
- `cargo test --lib` — **539 passed, 0 failed** (same count as
  pre-slice). No existing test changed expectations; slice 2 only
  added four new unit tests in `quant::rabitq::tests`:
  - `code_len_matches_dimension` (slice 1 carry-over)
  - `encode_then_score_same_vector_is_nonnegative` — round-trip smoke
  - `sign_words_from_rotated_matches_manual_pack` — bit-layout invariant
  - `hamming_similarity_identity_equals_dim` — identity-case invariant

## What this slice does NOT do

- No AM orchestration rewire to `&dyn Quantizer`. Scan and build
  still hold `ProdQuantizer` and call its `binary_sign_*` methods
  (which now delegate into `rabitq::`). The trait dispatch switch
  lives with Phase 3 reloption wiring.
- No rotation trait (slice 3).
- No unbiased distance estimator (slice 4). The slice-2 surrogate
  score is a placeholder.
- No `INDEX_FORMAT_*` constant for RaBitQ. `wire_format_version()`
  still returns 0 — the AM has no reader yet.

## Open questions for reviewer

1. `RaBitQScorer::score` returns the surrogate rather than
   `unimplemented!()`. Preference? I chose a surrogate so the
   round-trip smoke test is meaningful (it forces the encode/decode
   bit layout to be self-consistent); the downside is a misrouted
   caller would silently see bad scores instead of a loud panic.
2. Function renames at the module boundary: `binary_sign_similarity`
   → `hamming_similarity`, `binary_sign_words_from_packed` →
   `sign_words_from_packed_4bit`, `derive_persisted_binary_words` →
   `derive_persisted_sidecar_words`, etc. Cleaner inside `rabitq`,
   but breaks from the historical `binary_sign_*` naming carried in
   `ProdQuantizer` method names (those I kept — renaming them would
   have rippled to ~12 AM call sites for no immediate gain).
3. The `persisted_binary_sidecar_word_count` shim in `build.rs:326`
   kept as a one-liner. Happy to flatten it into direct
   `crate::quant::rabitq::persisted_sidecar_word_count` at the five
   call sites if you'd rather not carry the wrapper.
