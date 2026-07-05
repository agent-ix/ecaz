# Review Request: C1 ADR-030 V2 Runtime Adapters and Shared Grouped Encoder

## Context

ADR-032 and task 15 changed the branch objective: grouped-v2 is no longer just
an experiment to benchmark. It now needs a credible landing path as
`PqFastScan`, with insert and vacuum parity instead of permanent runtime
rejections.

The immediate architecture question was whether to fork insert/vacuum per
format or keep one lifecycle with narrow format-specific seams. In parallel,
the live grouped insert path still lacked one reusable primitive the build path
already had: shared grouped-PQ code packing from source vectors and persisted
codebooks.

This packet captures the first runtime-architecture checkpoint after ADR-032.

## Problem

Two concrete blockers were still in the way:

1. `aminsert` and vacuum had only scalar-shaped internal pipelines, even after
   the grouped-v2 gate packets.
   - the top-level functions could reject grouped-v2, but the actual write and
     repair flows still assumed scalar tuple decode/write everywhere
2. grouped-PQ search-code derivation still lived in duplicate local helpers
   instead of one shared implementation.
   - `src/am/build.rs` had one grouped encoder
   - `src/bin/approx_score_study.rs` had another
   - live grouped insert will need a third caller that derives search codes
     from persisted codebooks, not from the build-only training model

Without fixing the second point first, real grouped insert work would either
duplicate encoding again or couple runtime code to build-only model structs.

## Planned Slice

Two narrow groundwork slices in one checkpoint:

1. make insert and vacuum dispatch through explicit per-format runtime adapters
   while keeping one shared lifecycle
2. move grouped-PQ packing/centroid selection into one shared quant helper and
   add a persisted-codebook search-code derivation seam for future live insert

This packet intentionally does **not** claim grouped insert/vacuum parity yet.
Grouped `PqFastScan` still errors at the real data-mutation points.

## Implementation

Updated:

- `spec/adr/ADR-033-shared-graph-lifecycle-format-adapters.md`
- `src/am/insert.rs`
- `src/am/vacuum.rs`
- `src/quant/grouped_pq.rs`
- `src/am/build.rs`
- `src/am/graph.rs`
- `src/bin/approx_score_study.rs`
- `src/lib.rs`

### 1. ADR-033 records the runtime architecture

Added ADR-033 to make the design choice durable:

- one shared insert lifecycle
- one shared vacuum lifecycle
- format-specific adapters only at payload/storage/scoring seams

That ADR is the rationale for the code changes below, not a separate paper
exercise.

### 2. Insert now dispatches through an explicit format adapter

`src/am/insert.rs` now resolves `InsertFormatAdapter` from
`GraphStorageDescriptor::from_metadata(...)` and routes the existing shared
insert flow through adapter hooks for:

- duplicate detection
- forward-neighbor discovery
- node append
- backlink mutation

Current behavior:

- `TurboQuant` routes through the existing scalar implementation
- `PqFastScan` still errors with the dedicated grouped insert unsupported
  message at those hook points

This preserves current safety while replacing the old one-off format gate with
an actual extension seam.

### 3. Vacuum now dispatches through an explicit format adapter

`src/am/vacuum.rs` now resolves `VacuumFormatAdapter` from metadata and routes
bulkdelete/cleanup through format-owned hooks.

Concrete changes:

- `tqhnsw_ambulkdelete` now uses `run_bulkdelete_with_adapter(...)`
- `tqhnsw_amvacuumcleanup` dispatches through `VacuumFormatAdapter`
- scalar-only repair/finalize helpers were renamed to
  `repair_turboquant_graph_connections(...)` and
  `finalize_turboquant_fully_dead_elements(...)`

Current behavior:

- `TurboQuant` keeps the existing scalar vacuum behavior
- `PqFastScan` still errors with the dedicated grouped vacuum unsupported
  message at adapter hook points

Again, the grouped path is still blocked on real implementation, but the code
now has a clean place to put it without forking the whole vacuum algorithm.

### 4. Grouped-PQ packing is now shared

`src/quant/grouped_pq.rs` now owns the reusable grouped encoder pieces:

- `nearest_centroid_l2(...)`
- `encode_grouped_pq(...)`

Then both existing callers were moved onto that shared implementation:

- `src/am/build.rs`
- `src/bin/approx_score_study.rs`

That removes the duplicated grouped-code packing logic the earlier reviewer
feedback called out.

### 5. Runtime can now derive grouped search codes from persisted codebooks

`src/am/graph.rs` now exposes
`derive_grouped_search_code_from_source(metadata, model, source_vector)`.

That helper:

- validates source dimension against metadata
- rebuilds the SRHT sign vector from persisted metadata seed
- rotates the source vector
- encodes grouped search codes from the persisted grouped codebook model

This is the missing runtime seam needed by live grouped insert. It is not wired
into `aminsert` yet, but it removes the build-only model dependency.

### 6. Bench/test surface exports the shared encoder helpers

`src/lib.rs` now re-exports the new grouped encoder helpers through `bench_api`
so the study binary can keep using the narrow public surface instead of
reaching into internal modules.

## Measurements

This checkpoint is architecture + shared-code groundwork only. There are no new
runtime or recall measurements yet.

## Validation

Local build-only checkpoint commands:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All three passed.

Required full checkpoint commands still fail on this workstation at the same
known PostgreSQL/pgrx linker layer:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode for both commands is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`, and
  `errstart`

One environment detail changed:

- `/tmp/tqvector_pgrx_home/config.toml` was restored from the existing
  `~/.pgrx/config.toml`, so `cargo pgrx test pg17` no longer dies immediately
  on a missing config file and now reaches the same linker failure as
  `cargo test`

## Outcome

This checkpoint does three useful things without pretending grouped parity is
done:

1. insert/vacuum now have explicit runtime format seams instead of only top
   level guardrails
2. grouped-PQ packing is shared instead of duplicated between build and study
   code
3. runtime code can now derive grouped search codes from persisted grouped
   codebooks, which is a real prerequisite for live grouped insert

What it does **not** do yet:

- no live `PqFastScan` insert success path
- no `PqFastScan` vacuum success path
- no reloption work yet (`storage_format`)
- no grouped runtime rename cleanup yet (`GroupedV2` / `ScalarV1` still exist)

## Next Slice

The next practical runtime slices should be:

1. fetch `build_source_column` for live insert so grouped insert can obtain the
   source vector it needs
2. use the new persisted-codebook encoder seam to build real grouped hot/cold
   insert payloads
3. wire grouped insert/vacuum graph search and repair through exact cold-rerank
   scoring first, then optimize later if needed
