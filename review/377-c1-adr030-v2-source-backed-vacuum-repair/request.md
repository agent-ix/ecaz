# Review Request: C1 ADR-030 V2 Source-Backed Vacuum Repair

## Context

Packet 376 introduced a shared source helper module and moved live insert onto
source-space graph maintenance for `build_source_column` indexes.

That still left one architectural mismatch:

- build and live insert could maintain a source-backed graph in source space
- vacuum repair still ranked replacement edges in code space only

So source-backed scalar indexes would immediately diverge again after deletes,
and future `PqFastScan` maintenance would still need a separate vacuum-only
source seam.

## Problem

The remaining gap was entirely in vacuum repair:

1. repair search used `score_code_inner_product(...)` everywhere
   - entry seeding
   - upper/layer0 repair search
   - linear top-up fallback
2. vacuum did not inspect `build_source_column` at all
3. there was no regression coverage proving source-backed repair could choose a
   different replacement than code-backed repair

That made packet 376 only half the maintenance story.

## Planned Slice

One narrow follow-on checkpoint:

1. introduce a vacuum search metric that mirrors the new insert metric
2. switch scalar vacuum repair onto source-space scoring when
   `build_source_column` is configured
3. add a regression that proves the source-backed repair path can prefer a
   different candidate than the old code-space ranking

Grouped vacuum is still out of scope here.

## Implementation

Updated:

- `src/am/vacuum.rs`
- `src/lib.rs`

### 1. Vacuum now resolves a maintenance metric from reloptions

`src/am/vacuum.rs` now adds `VacuumSearchMetric`:

- `Code`
- `Source(VacuumHeapSourceScorer)`

When `build_source_column` is present, vacuum now:

- resolves the heap source column through the shared `src/am/source.rs` helper
- allocates one reusable heap tuple slot
- uses `SnapshotAny` to fetch source rows by heap TID during repair ranking
- averages duplicate representatives across stored heap TIDs before scoring

So vacuum now uses the same maintenance-space decision as build/live-insert:

- no source column => code-space repair
- source column => source-space repair

### 2. Source-backed scoring now flows through the whole repair path

The new metric is threaded through:

- `repair_turboquant_graph_connections(...)`
- `plan_repair_replacements(...)`
- `search_repair_candidates_for_layer(...)`
- `load_vacuum_entry_candidate(...)`
- `top_up_repair_replacements_from_linear_scan(...)`

That means source-backed vacuum repair no longer silently falls back to
quantized code similarity in either graph search or the linear top-up path.

### 3. Added regression coverage for source-vs-code divergence

`src/lib.rs` now adds
`test_vacuum_source_backed_repair_prefers_source_candidate`.

The fixture intentionally builds:

- source vectors in one similarity order
- quantized embeddings in the reverse order

The test:

1. finds a real broken layer-0 repair case after deletion
2. computes the best eligible replacement under source-space ranking
3. computes the best eligible replacement under code-space ranking
4. asserts those candidates differ
5. runs vacuum
6. verifies the repaired edge now contains the source-space winner

That is the checkpoint’s key behavioral proof.

## Measurements

No new benchmark or recall measurements in this slice. This is maintenance-path
correctness.

## Validation

Passed:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`, and
  `errstart`

## Outcome

This checkpoint closes the immediate maintenance mismatch from packet 376:

1. source-backed scalar indexes now use one consistent metric across build,
   live insert, and vacuum repair
2. the vacuum repair path reuses the same shared source-column seam instead of
   growing a third bespoke helper copy
3. there is now direct coverage that source-backed repair can make a
   materially different choice than code-backed repair

What it still does **not** do:

- no grouped append path yet
- no grouped vacuum path yet
- no format reloption cutover yet

## Next Slice

The next practical checkpoint is to reuse these shared source-backed
maintenance seams for the grouped append/vacuum implementation itself, instead
of adding grouped-only heap fetch logic beside them.
