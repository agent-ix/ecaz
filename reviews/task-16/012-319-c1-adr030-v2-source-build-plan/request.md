# Review Request: C1 ADR-030 V2 Source Build Plan

## Context

Packet `318` connected generated grouped search codes to the alternate v2 page staging path.

The next narrow slice is to compute a full source-backed v2 build result in-memory, including the
same high-level outputs the real builder will need, without yet switching the live build path or
metadata format.

## Problem

We can now stage v2-shaped pages, but the builder still does not have a single seam that returns the
whole alternate build result:

1. staged data pages
2. entry point
3. max level

## Planned Slice

Add an in-memory source-backed v2 build planner that:

1. builds the graph
2. stages generated-code v2 pages
3. computes entry point and max level from the staged hot tids

This slice still excludes:

- no live build switchover
- no v2 metadata writes
- no scan/runtime use

## Implementation

Added a source-backed alternate v2 build planner that returns the full in-memory build result
shape the eventual rebuild path will need.

New seam:

- `plan_v2_grouped_source_build(...)`

New result type:

- `V2GroupedBuildPlan`

Returned outputs:

1. staged grouped v2 data pages
2. chosen entry point
3. computed max level

Behavior:

1. builds the HNSW graph from build state
2. stages grouped hot / rerank / neighbor tuples from source-backed grouped-code generation
3. computes entry point from staged hot tids and graph nodes
4. computes max level from the built graph

Test added:

- validates that the planner returns a non-empty staged chain, a valid entry point, and a bounded
  max level for a source-backed build state

This is the first packet where ADR-030 v2 has a single builder-side seam that can hand back the
whole alternate build result without depending on the live current-format writer.

## Measurements

This packet is still a build-path slice, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test plan_v2_grouped_source_build_reports_entry_point_and_levels --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

ADR-030 v2 now has a source-backed in-memory build planner that can:

1. build the graph
2. derive grouped search codes
3. stage grouped v2 pages
4. return entry point and max level alongside those pages

What this de-risks:

1. the alternate v2 path now has the same top-level outputs a real builder flush path will need
2. grouped-code generation, staged page layout, and entry-point selection are now proven to compose
3. the next slice can focus on flushing this plan through a guarded alternate path instead of
   inventing more intermediate seams

## Next Slice

The next narrow slice should add a guarded alternate flush path that consumes
`V2GroupedBuildPlan`:

1. reuse the source-backed grouped build planner during build
2. flush the staged plan through a real builder-side write path behind an explicit gate
3. keep the default current-format metadata and runtime unchanged while validating the alternate
   v2 storage path
