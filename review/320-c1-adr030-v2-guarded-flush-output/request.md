# Review Request: C1 ADR-030 V2 Guarded Flush Output

## Context

Packet `319` added a source-backed in-memory v2 build planner that can return:

1. staged grouped v2 data pages
2. entry point
3. max level

The next narrow slice is to turn that in-memory plan into a real builder-side flush output shape
without switching the default build path yet.

## Problem

We can now plan the full v2 build result in memory, but the builder still only has one real flush
lane:

1. current-format data-page staging
2. current-format metadata assembly
3. immediate write to relation

There is still no reusable flush-output seam that can hold either:

1. current-format build output
2. grouped v2 build output

## Planned Slice

Add a guarded alternate flush-output seam that:

1. defines a reusable build flush output of `data_pages + metadata`
2. refactors the current build flush path to use that output
3. adds a grouped v2 flush-output constructor from `V2GroupedBuildPlan`

This slice still excludes:

- no live build switchover
- no runtime grouped scan path
- no user-visible reloption or SQL surface for selecting v2 builds

## Implementation

Added a reusable builder flush-output seam that separates:

1. build output assembly
2. relation flush

New result type:

- `BuildFlushOutput`

New helpers:

- `current_format_flush_output(...)`
- `grouped_v2_flush_output(...)`
- `flush_build_output(...)`

Refactor:

- `flush_build_state(...)` now builds a `BuildFlushOutput` for the current scalar format and
  flushes that output through the shared write helper

Grouped v2 behavior:

1. consumes `V2GroupedBuildPlan`
2. reuses the staged grouped v2 pages from the in-memory planner
3. assembles grouped v2 metadata with:
   - format version `v2`
   - `SRHT` transform tag
   - grouped PQ search codec tag
   - grouped search-code payload flag
   - cold rerank payload flag
   - optional binary sidecar flag when the underlying 4-bit lane supports it

Test added:

- validates that grouped v2 flush output carries grouped metadata and grouped-hot / rerank /
  neighbor tuples, with no scalar element tuples

This is the first packet where both the current build lane and the alternate ADR-030 v2 lane can be
described as the same builder output shape: `data_pages + metadata`.

## Measurements

This packet is still a build-path slice, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test grouped_v2_flush_output_marks_grouped_metadata_and_pages --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

ADR-030 v2 now has a real builder-side flush-output contract instead of only in-memory staging.

What this de-risks:

1. the current builder flush path is now factored around a reusable output seam
2. grouped v2 build output can now be assembled in the same form a real writer expects
3. the next slice can focus on a guarded relation-flush or rebuild switch instead of more builder
   refactoring

## Next Slice

The next narrow slice should add an explicit guarded alternate rebuild path that uses
`grouped_v2_flush_output(...)` end to end:

1. plan grouped v2 output from source-backed build state
2. flush that output through the shared writer behind an explicit internal gate
3. keep the default `ambuild` path on current-format output until scan/runtime support exists
