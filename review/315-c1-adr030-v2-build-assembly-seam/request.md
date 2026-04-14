# Review Request: C1 ADR-030 V2 Build Assembly Seam

## Context

Packet `314` added concrete page and page-chain placement helpers for the ADR-030 v2 tuple kinds:

- grouped hot tuples
- rerank tuples

The next narrow slice is to add the first builder-side seam that can assemble those v2 tuples from
existing build inputs, without switching the live build path over yet.

## Problem

The builder still only knows how to materialize current scalar element tuples directly inside
`flush_build_state`.

That means there is still no isolated seam for:

1. taking one logical build tuple
2. assigning its hot-path payload versus cold rerank payload
3. producing the exact v2 tuple pair the future write path will persist

Without that seam, moving to a real v2 builder would require entangling format redesign with the
existing flush path all at once.

## Planned Slice

Add a builder-side staging helper that:

1. accepts one `BuildTuple`
2. accepts an explicit grouped search-code payload
3. derives the optional persisted binary sidecar
4. emits:
   - one grouped hot tuple
   - one rerank tuple

This slice still excludes:

- no live builder switchover yet
- no v2 metadata writes yet
- no scan/runtime changes yet

## Implementation

Added a builder-side staging helper that assembles one logical build tuple into the intended ADR-030
v2 hot/cold storage split without changing the live build path.

New seam:

- `stage_v2_grouped_build_payload(...) -> V2GroupedBuildPayload`

Output types:

1. grouped hot tuple
   - level
   - deleted flag
   - duplicate heap tids
   - neighbor tuple ref
   - rerank tuple ref
   - optional persisted binary sidecar
   - grouped search-code payload
2. rerank tuple
   - gamma
   - rerank payload bytes

The helper:

1. accepts an existing `BuildTuple`
2. accepts an explicit grouped search-code payload
3. derives the persisted binary sidecar only when the quantizer supports the no-QJL 4-bit path
4. preserves the current scalar payload as the rerank code

Tests added:

- supported-binary-sidecar case on the real `1536 x 4-bit` lane
- unsupported-binary-sidecar case on a non-supported lane

## Measurements

This packet is a builder seam slice, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test stage_v2_grouped_build_payload_keeps_hot_and_cold_split --lib`: passed
- `cargo test stage_v2_grouped_build_payload_skips_binary_sidecar_when_unsupported --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

The v2 builder no longer needs to invent the hot/cold split inline inside a future flush rewrite.

What this de-risks:

1. there is now a concrete handoff point between logical build tuples and v2 storage tuples
2. the persisted binary-sidecar rule is centralized in one seam
3. the eventual builder switchover can reuse a tested assembly step instead of rewriting tuple
   construction ad hoc

## Next Slice

The next narrow slice should use this assembly seam in a minimal alternate write path:

1. build grouped hot tuples plus rerank tuples into a `DataPageChain`
2. keep metadata and live build behavior unchanged
3. validate that the alternate chain layout is internally consistent before any runtime switchover
