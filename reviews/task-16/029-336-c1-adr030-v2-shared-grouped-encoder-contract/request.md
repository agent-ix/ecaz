# Review Request: C1 ADR-030 V2 Shared Grouped Encoder Contract

## Context

Reviewer feedback on packets `311`, `315`, `317`, `318`, and `333` all repeated the same concern:

- grouped PQ nibble packing existed in both `src/am/build.rs` and
  `src/bin/approx_score_study.rs`

That duplication was especially risky because ADR-030 currently depends on the study harness to
justify the grouped-v2 lane, while the build path writes those same grouped codes into the v2 hot
tuples.

## Problem

If the study harness and build path pack grouped centroid indices differently, the resulting bug
would look like recall noise rather than a codec bug. That is exactly the wrong failure mode for the
first real grouped scorer packet.

The duplicate packing logic needs to be removed before the scorer lands.

## Planned Slice

Create one shared grouped-code packing helper and route both current packers through it:

1. move grouped nibble packing into a shared quant module
2. keep build-side nearest-centroid search local to `build.rs`
3. keep study-side nearest-centroid search local to `approx_score_study.rs`
4. make both paths use the same nibble-packing implementation

This still excludes:

- no grouped scorer implementation yet
- no insert/vacuum grouped-v2 safety changes yet
- no cold rerank fetch path yet

## Implementation

Added:

- `src/quant/grouped_pq.rs`

Updated:

- `src/quant/mod.rs`
- `src/lib.rs`
- `src/am/build.rs`
- `src/bin/approx_score_study.rs`

Concrete changes:

1. added shared helper `pack_grouped_pq_nibbles(indices: &[u8]) -> Vec<u8>`
2. exported that helper through `bench_api` so the study binary can use the same implementation
3. changed the build-side `encode_grouped_pq(...)` to:
   - compute centroid indices
   - pack via `pack_grouped_pq_nibbles(...)`
4. changed the study-side `encode_grouped_pq(...)` to do the same
5. added direct unit tests for shared grouped nibble packing

New tests:

- `quant::grouped_pq::tests::pack_grouped_pq_nibbles_packs_even_count`
- `quant::grouped_pq::tests::pack_grouped_pq_nibbles_packs_odd_count`

Existing tests still covering downstream behavior:

- `tests::grouped_pq_encode_packs_two_nibbles_per_byte` in `approx_score_study`
- `grouped_build_model_trains_and_derives_codes_from_source_vectors` in `build.rs`

## Measurements

This packet is codec-contract cleanup, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test pack_grouped_pq_nibbles_packs_even_count --lib`: passed
  - `cargo test grouped_pq_encode_packs_two_nibbles_per_byte --bin approx_score_study`: passed
  - `cargo test grouped_build_model_trains_and_derives_codes_from_source_vectors --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 no longer has two independent grouped nibble-packers in the build path and study harness.

What this de-risks:

1. study-vs-build grouped code bytes now share one packing implementation
2. the first real grouped scorer packet no longer has to guess whether codec drift is coming from
   runtime or from duplicate packers
3. future grouped insert/runtime work can reuse the same helper rather than introducing a third
   packer

## Next Slice

The next feedback-driven slice should pick one of the remaining explicit blocker items:

1. grouped-v2 insert-path rejection
2. grouped-v2 vacuum-path rejection
3. cold `reranktid -> rerank tuple` fetch seam

Those are now more important than adding another scorer-only seam packet.
