# Review Request: C1 ADR-030 V2 Gated Raw Page Validation

## Context

Packet `321` added a default-off internal gate that can route source-backed builds through the
experimental ADR-030 v2 grouped rebuild lane.

The next narrow slice is to verify that the gated build path actually writes grouped-v2 pages and
metadata on disk.

## Problem

We now have an internal grouped-v2 rebuild lane, but it has only been validated through builder-side
assembly seams.

There is still no direct proof that a real index build under the gate produces:

1. grouped-v2 metadata
2. grouped-hot tuples
3. rerank tuples
4. no legacy scalar element tuples

## Planned Slice

Add one pg test that:

1. enables the internal build gate for that test only
2. builds a source-backed fixture large enough for grouped-PQ training
3. inspects raw index pages through `debug_index_pages`
4. verifies grouped-v2 metadata and tuple tags on disk

This slice still excludes:

- no runtime grouped scan support
- no query-path validation against the grouped-v2 format
- no user-visible surface for enabling the gate

## Implementation

Added one pg test that exercises the experimental ADR-030 v2 build gate end to end and inspects the
resulting raw index pages.

Test support added:

- a scoped environment-variable helper
- a shared mutex to serialize env-var-sensitive tests

New pg test:

- `test_experimental_grouped_v2_source_build_writes_grouped_pages`

Behavior:

1. enables `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` for that test only
2. creates a source-backed fixture with 16 rows so grouped-PQ training is valid
3. builds the index with `build_source_column`
4. inspects metadata and tuple tags through `debug_index_pages`

Assertions:

1. metadata format version is `v2 grouped`
2. transform kind is `SRHT`
3. search codec kind is grouped PQ
4. rerank codec kind is scalar quantized
5. grouped-search and cold-rerank payload flags are present
6. on-disk tuples contain grouped hot / rerank / neighbor tuples
7. no legacy scalar element tuples are present
8. metadata entry point points at a grouped hot tuple

This is the first packet that proves the experimental builder lane writes grouped-v2 storage on disk
rather than only assembling grouped output internally.

## Measurements

This packet is still a build-path slice, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test test_experimental_grouped_v2_source_build_writes_grouped_pages --no-default-features --features 'pg17 pg_test'`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

ADR-030 v2 now has direct evidence that the gated rebuild path produces real grouped-v2 on-disk
layout.

What this de-risks:

1. the experimental gate is not just choosing a different in-memory seam; it is writing grouped-v2
   metadata and tuple tags to the relation
2. the env-var-scoped test harness is safe enough to validate future experimental build slices
3. the next work can move closer to scan/runtime support, since the build/storage side is now
   visibly real on disk

## Next Slice

The next narrow slice should start the read-side groundwork for grouped-v2 pages:

1. gated metadata dispatch for `v1 scalar` vs `v2 grouped`
2. grouped-hot tuple decode and page read helpers at the scan boundary
3. no grouped scoring/runtime cutover yet, only the read-side format split
