# Review Request: C1 ADR-030 V2 PqFastScan Small-Dimension Group Size

## Context

The branch already had first-class `PqFastScan` build / scan / insert / vacuum
support, and packet 390 closed the empty-index bootstrap and small-table build
gaps.

But the default grouped build path still had one lingering hardcoded layout
assumption:

- `default_pq_fastscan_flush_output(...)` always used `group_size = 16`

That works for the real large-dimension workloads, but it unnecessarily makes
the default grouped layout less dimension-aware than the metadata format already
allows.

## Problem

ADR-032 and task 15 both call out the hardcoded grouped layout defaults as a
remaining landing gap.

Without this slice, default `PqFastScan` build still relied on a module-level
`16`-wide grouped layout even though:

1. the runtime layout is already metadata-driven after build
2. the grouped codec can support smaller transformed dimensions cleanly
3. small transformed dimensions should not need a fake 16-wide grouping rule

## Planned Slice

One narrow parameterization checkpoint:

1. derive the default `PqFastScan` group size from the transformed dimension
2. keep the existing `16` target for normal workloads
3. add coverage for the small-dimension path

## Implementation

Updated:

- `src/am/build.rs`
- `src/lib.rs`

### 1. Default grouped build now derives a dimension-aware group size

In `src/am/build.rs`:

- renamed the fixed target constant to `PQ_FASTSCAN_TARGET_GROUP_SIZE`
- added `default_pq_fastscan_group_size(dimensions)` which computes:
  - `effective_transform_dim(dimensions)`
  - `min(transform_dim, 16)`
- updated `default_pq_fastscan_flush_output(...)` to use that derived group
  size instead of unconditionally forcing `16`

That preserves the current behavior for typical large dimensions while allowing
small transformed dimensions to use a smaller, metadata-consistent grouped
layout.

### 2. Added pure build coverage for the small-dimension path

Also in `src/am/build.rs`:

- kept the existing default-parameter test for the 16-dimension case
- added
  `default_pq_fastscan_flush_output_derives_small_dimension_group_size`
  which builds an 8-dimension grouped index plan and asserts:
  - `search_subvector_count == 1`
  - `search_subvector_dim == 8`
  - persisted grouped codebooks are still emitted

### 3. Added pg coverage for end-to-end small-dimension grouped build

In `src/lib.rs`:

- added `test_tqhnsw_pq_fastscan_small_dim_build_derives_group_size`

The test:

1. creates an 8-dimension source/embedding table
2. builds a `storage_format = 'pq_fastscan'` index
3. inspects persisted metadata and tuple tags
4. asserts:
   - grouped format version
   - `search_subvector_dim == 8`
   - `search_subvector_count == 1`
   - exactly one grouped codebook tuple
   - valid grouped codebook head metadata

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family, including:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This advances the remaining parameterization cleanup without changing the
default large-dimension shape:

1. default `PqFastScan` build is now dimension-aware for small transformed dims
2. the `16`-wide grouped layout remains the default target where it fits
3. metadata and tests now prove the small-dimension grouped layout works

What this slice intentionally does **not** do:

- expose `PqFastScan` group size as a user reloption
- change the fixed PQ4 search-code width
- remove the remaining runtime tuning env vars in `scan.rs`

## Next Slice

The remaining landing work is now mostly cleanup and proof:

1. continue chipping away at the parameterization / legacy-assumption surface
2. decide whether any of the remaining `scan.rs` env-controlled paths should
   stop looking experimental before merge
3. keep strengthening explicit task-15 parity proof for both formats
