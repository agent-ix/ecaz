# Review Request: C1 ADR-030 V2 Shared Grouped PQ Scorer

## Context

ADR-030 already had one shared grouped helper for nibble packing, but the grouped PQ score path was
still split:

- study harness had its own packed-nibble decode and f32 LUT scorer
- runtime preparation was starting to grow toward its own grouped scorer path

That duplication risk is similar to the earlier shared-packer problem: study-vs-runtime divergence
would look like quality drift instead of an implementation bug.

## Problem

If the study harness and runtime grouped scorer decode packed grouped codes differently, ADR-030
will be hard to trust during the first real runtime measurements.

The packed-nibble decode and simple f32 LUT score path should be shared before the runtime grouped
approximate scorer lands.

## Planned Slice

Create one shared grouped-PQ scorer primitive:

1. shared packed-nibble decode helper
2. shared f32 LUT scorer over packed grouped codes
3. route the study harness through that helper
4. add direct unit tests at the shared quant layer

This slice intentionally excludes:

- no runtime grouped scorer cutover yet
- no gate lift
- no new measurements

## Implementation

Updated:

- `src/quant/grouped_pq.rs`
- `src/lib.rs`
- `src/bin/approx_score_study.rs`

Concrete changes:

1. added `grouped_pq_nibble(...)`
2. added `grouped_pq_score_f32(...)`
3. exported those helpers through `bench_api`
4. changed the study harness f32 grouped-PQ scorer to call the shared helper
5. updated the study harness u8 scorer to use the shared nibble decode
6. added unit tests for:
   - packed nibble decode
   - shared f32 LUT score aggregation

## Measurements

This packet is shared scorer extraction only, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test pack_grouped_pq_nibbles_packs_even_count --lib`: passed
  - `cargo test grouped_pq_score_f32_sums_lut_rows_by_nibble --lib`: passed
  - `cargo test grouped_pq_u8_score_tracks_f32_for_same_code --bin approx_score_study`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 now has one shared grouped-PQ packed-code decode and f32 LUT scorer instead of separate
study-only logic.

What this de-risks:

1. study-vs-runtime grouped score math is less likely to drift
2. the first real runtime grouped approximate scorer can reuse the same packed-code decode path
3. future measurement disagreements are more likely to reflect real algorithmic differences instead
   of duplicated scorer code

## Next Slice

The next runtime slice should use this shared scorer primitive inside the grouped-v2 scan path while
still keeping the external grouped runtime gate in place.
