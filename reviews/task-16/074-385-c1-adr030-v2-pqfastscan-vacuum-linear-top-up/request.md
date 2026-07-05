# Review Request: C1 ADR-030 V2 PqFastScan Vacuum Linear Top-Up

## Context

Packet 383 removed the blanket `PqFastScan` vacuum reject and taught grouped
vacuum how to:

- strip dead heap TIDs from grouped hot tuples
- unlink stale grouped graph refs
- finalize fully dead grouped hot tuples

That still left one repair seam asymmetric.

When graph-search replacement planning found too few candidates, vacuum could
top up from a linear scan only for `TurboQuant`. `PqFastScan` still stopped at
the graph-search result set, even though grouped vacuum no longer rejected the
format at the top level.

## Problem

Before this packet, `plan_repair_replacement(...)` had this shape:

1. run graph-search replacement planning for both formats
2. if replacements are short, create a `LinearRepairPlanner` only for
   `TurboQuant`
3. if storage is `PqFastScan`, return whatever graph search found, even when a
   linear top-up could supply additional live candidates

That means grouped repair could under-fill broken neighbor slots relative to
the scalar path.

## Planned Slice

One repair-top-up checkpoint:

1. make linear repair planning storage-aware instead of scalar-only
2. teach the linear scan path to score `PqFastScan` candidates with exact cold
   rerank payloads
3. avoid re-locking the current hot-tuple page when the rerank tuple lives on
   the same block
4. add grouped layer-0 repair coverage

This is a repair-quality parity slice, not a new vacuum lifecycle.

## Implementation

Updated:

- `src/am/vacuum.rs`
- `src/lib.rs`

### 1. Linear repair planning now runs for both formats

`LinearRepairPlanner` now carries a `GraphStorageDescriptor` instead of a
scalar `code_len`.

`plan_repair_replacement(...)` now always constructs that planner, so grouped
repair no longer bails out early when graph-search-only replacements are
short.

### 2. Linear candidate collection is storage-aware

`collect_linear_repair_candidates_on_page(...)` now:

- still decodes `TqElementTuple` directly for `TurboQuant`
- decodes `TqGroupedHotTuple` for `PqFastScan`
- reconstructs an exact `GraphElement` for grouped candidates by loading the
  cold rerank payload

That means the same linear top-up path can now score grouped repair
candidates using the exact payload vacuum repair already expects elsewhere.

### 3. Same-page rerank payload reads avoid re-locking the current page

Grouped linear top-up needs the rerank tuple for exact scoring, but the
rerank tuple may live on the same block as the hot tuple currently being
scanned.

This packet adds:

- `load_grouped_rerank_payload_for_linear_repair_candidate(...)`

That helper:

- decodes the rerank tuple directly from the current page when it shares the
  current block
- falls back to the existing graph read helper only when the rerank tuple is
  on another block

So the grouped linear top-up path does not reopen and relock the same buffer
just to recover the cold payload.

### 4. Added grouped layer-0 replacement coverage

`src/lib.rs` now adds grouped vacuum coverage for broken layer-0 repair:

- a `PqFastScan` fixture with `m = 2`
- delete one row with inbound grouped layer-0 refs
- run vacuum
- confirm the deleted grouped element tid is fully unlinked
- confirm at least one affected grouped neighbor tuple gains a new live
  replacement candidate

The grouped fixture helper was extended to allow a custom `m` value for this
coverage.

## Measurements

No benchmark or recall measurements in this slice. This is repair-quality
groundwork only.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`,
  `CopyErrorData`, and `errstart`

## Outcome

This checkpoint closes the remaining scalar-only linear-repair fallback seam:

1. grouped repair planning now gets the same linear top-up path as scalar
   repair
2. grouped linear candidates are scored from exact cold rerank payloads
3. same-block grouped rerank reads avoid a redundant relock path
4. grouped layer-0 replacement now has explicit regression coverage in code

What it still does **not** do:

- prove the new grouped pg regression in this workstation environment, because
  the PostgreSQL linker boundary still blocks test execution
- add grouped upper-layer replacement coverage
- finish the broader ADR-032 landing work outside vacuum repair

## Next Slice

The next practical slices are:

1. continue broader PqFastScan-first-class cleanup outside vacuum, especially
   remaining naming and task-15 parity gaps
2. add grouped upper-layer replacement coverage if we want symmetric repair
   coverage with the scalar path
3. keep reducing remaining `grouped_v2` / experimental naming debt in the
   wider test/docs surface
