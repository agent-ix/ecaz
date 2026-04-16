# Review Request: C1 ADR-030 V2 Storage Format Reloption Build Selection

## Context

Task 15 / ADR-032 moves PqFastScan selection from a process-wide build env var
to a per-index reloption:

- old: `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD`
- new: `WITH (storage_format = 'pq_fastscan')`

Before this slice, grouped build selection still depended on the process
environment, so the branch could not express format choice in index metadata,
tests, or SQL-facing configuration.

## Problem

The old grouped build gate had three concrete problems:

1. format choice was not persisted in reloptions
2. dev/test flows still depended on a process-global env var
3. empty metadata initialization always started as scalar-v1, even when the
   intended format was grouped

That is incompatible with ADR-032's "two first-class formats" direction.

## Planned Slice

One checkpoint:

1. add a `storage_format` reloption
2. route build selection through that reloption instead of the env gate
3. persist grouped-format intent in initial metadata for grouped indexes
4. update grouped build tests/scripts to use SQL reloptions instead of the
   removed build env

Scan, insert, and vacuum parity are explicitly out of scope for this packet.

## Implementation

Updated:

- `src/am/options.rs`
- `src/am/build.rs`
- `src/am/graph.rs`
- `src/lib.rs`
- `scripts/restart_adr030_scratch.sh`

### 1. Added `storage_format` reloption parsing

`src/am/options.rs` now adds:

- `StorageFormat::TurboQuant`
- `StorageFormat::PqFastScan`

and exposes it through `TqHnswOptions.storage_format`.

Accepted SQL values:

- `turboquant` (default)
- `pq_fastscan`

Invalid values now error through reloption parsing rather than silently falling
back to TurboQuant.

### 2. Build selection now uses reloptions, not the build env

`src/am/build.rs` now switches `flush_build_state(...)` on
`state.options.storage_format`:

- `TurboQuant` => existing scalar flush path
- `PqFastScan` => grouped flush path

The old `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` gate and its helper were
deleted from code and from the scratch restart script.

This slice keeps the current grouped defaults:

- `group_size = 16`
- `train_size <= 1024`
- `kmeans_iters = 8`

but they now live as default PqFastScan build parameters instead of an
experimental env-gated path.

### 3. Empty grouped indexes now initialize as grouped

`BuildState::initial_metadata()` now writes grouped-format metadata when the
index reloption selects `pq_fastscan`, even before build output is flushed.

To keep that metadata readable, `src/am/graph.rs` now accepts the "empty grouped
metadata" shape and decodes it as a grouped descriptor with zero-length layout
fields.

This does not implement empty-index live insert for PqFastScan; it only makes
the intended on-disk format explicit from index creation onward.

### 4. Grouped tests now select PqFastScan in SQL

Grouped pg tests in `src/lib.rs` now build grouped indexes with:

- `storage_format = 'pq_fastscan'`

instead of relying on the removed process-global build env.

The grouped runtime settings debug probe now reports grouped build availability
as always-on, since selection moved from env gating to reloptions.

## Measurements

No new benchmark or recall measurements in this slice. This is configuration and
metadata wiring.

## Validation

Passed:

- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`, and
  `errstart`

## Outcome

This checkpoint moves grouped build selection onto the ADR-032 path:

1. format choice is now represented in SQL reloptions
2. grouped build no longer depends on a process-wide build gate
3. grouped metadata intent is explicit from initial metadata creation
4. grouped build tests now exercise the SQL selection mechanism directly

What it still does **not** do:

- grouped scan is still runtime-gated
- grouped insert still rejects
- grouped vacuum still rejects
- grouped build parameters are still defaulted, not fully metadata-driven

## Next Slice

The next practical checkpoint is to remove the grouped ordered-scan runtime gate
now that build selection is reloption-based, so grouped build + scan becomes a
fully SQL-selected path while insert/vacuum parity continues separately.
