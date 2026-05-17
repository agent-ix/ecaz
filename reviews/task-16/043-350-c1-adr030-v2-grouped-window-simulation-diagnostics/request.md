# Review Request: C1 ADR-030 V2 Grouped Window Simulation Diagnostics

## Context

Packet `349` exposed small-window evidence from emitted grouped scan rows behind
`TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`:

- emitted grouped rows already carried approximate traversal score and exact rerank comparison score
- grouped runtime diagnostics now summarize exact-best coverage for approximate windows `1`, `2`,
  `4`, and `8`
- scalar scans already prove those grouped-only window metrics stay inert

That is enough to see whether the exact-best emitted row is nearby, but it still leaves the next
decision awkward: what would a concrete sliding rerank window actually do to the emitted order?

## Problem

The next runtime milestone is a narrow grouped rerank window, but wiring that directly into the
live scan path would immediately blur the measurement seams from packets `346-349`:

1. current row/order diagnostics intentionally treat emitted order as approximate search order
2. changing runtime first would force a larger semantic rewrite of those diagnostics before the
   window size itself is justified
3. there was still no stable SQL-visible way to simulate a concrete rerank prefix and compare it
   against the current emitted order

Before changing live grouped output, ADR-030 needs a parameterized window simulation seam that can
show what a sliding rerank prefix would do to emitted rows and summary metrics.

## Planned Slice

Batch the next related measurement slices together:

1. add a sliding grouped rerank-window row simulation over emitted comparison rows
2. expose the simulated windowed row order through a debug SQL wrapper
3. summarize before/after rank-drift metrics for the same window through a second SQL wrapper
4. prove grouped SQL results match the simulated rows exactly
5. prove scalar scans keep the grouped-only window simulation surfaces inert

This slice intentionally excludes:

- no live scan behavior change yet
- no output-score cutover yet
- no gate lift
- no grouped recall / latency claims yet

## Implementation

Updated:

- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Concrete changes:

1. added shared debug helpers in `src/am/scan_debug.rs` to:
   - validate positive window sizes
   - compute grouped rank-drift metrics from observed-vs-exact ranks
   - simulate a sliding rerank window over emitted grouped comparison rows
2. added `debug_grouped_scan_windowed_rows(index_oid, query, window_size)` returning:
   - approximate rank
   - simulated windowed rank
   - approximate score
   - exact comparison score
   - exact rank
   - baseline and windowed rank shifts
3. added `debug_grouped_scan_windowed_summary(index_oid, query, window_size)` returning:
   - emitted / grouped / compared counts
   - exact-best and exact-top4 before/after ranks
   - mean/max absolute rank shift before and after
   - Spearman rank correlation before and after
4. refactored the existing order-drift summary to reuse the shared grouped rank-metric helper
5. exported both new helpers through `src/am/mod.rs`
6. added SQL debug wrappers:
   - `tests.tqhnsw_debug_grouped_scan_windowed_rows(...)`
   - `tests.tqhnsw_debug_grouped_scan_windowed_summary(...)`
7. added grouped pg coverage that:
   - reconstructs the sliding-window simulation for a concrete `window_size = 4`
   - proves the row wrapper matches that simulation exactly
   - proves the summary wrapper matches row-derived aggregation exactly
8. added scalar pg coverage that proves both windowed wrappers stay inert on a normal scalar index

## Measurements

This packet remains diagnostic only. It simulates a grouped rerank prefix, but it does not yet
change live scan behavior.

Validation results for this checkpoint:

- focused validation:
  - `cargo test windowed -- --nocapture`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 now has a parameterized sliding-window simulation seam for grouped emitted rows without
changing the live grouped scan runtime.

What this de-risks:

1. the next runtime packet can pick a concrete rerank prefix from measured before/after row data
   instead of only window-coverage booleans
2. grouped window behavior can now be compared against baseline exact-rank drift using one stable
   SQL/debug surface
3. live runtime changes can stay smaller because the measurement semantics are preserved first

## Next Slice

The next grouped runtime batch should use packet `350` to keep the actual live cutover narrow:

1. choose a concrete rerank prefix from the new simulation evidence
2. wire that prefix into the grouped-v2 live scan path behind `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`
3. preserve a stable baseline-vs-window comparison seam while the live grouped runtime starts using
   the selected rerank window
