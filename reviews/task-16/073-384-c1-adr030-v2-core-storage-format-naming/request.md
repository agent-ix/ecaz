# Review Request: C1 ADR-030 V2 Core Storage-Format Naming

## Context

Packet 380 renamed the main runtime storage descriptor surface to:

- `TurboQuant`
- `PqFastScan`

But one lower Rust layer still used the old names:

- `page::GraphStorageFormat::ScalarV1`
- `page::GraphStorageFormat::GroupedV2`

That left the runtime reading awkwardly: the primary dispatch enum used the
first-class names from ADR-032, while the metadata-decoding enum under it
still exported the old feasibility-era names.

## Problem

The wire bytes should stay version-oriented:

- `INDEX_FORMAT_V1_SCALAR`
- `INDEX_FORMAT_V2_GROUPED`

But the Rust runtime types should not keep exposing those version labels if the
product decision is “two first-class peer formats.”

Before this packet, the core path still had mixed terminology:

1. `MetadataPage::graph_storage_format()` returned `ScalarV1` / `GroupedV2`
2. `graph.rs` immediately retranslated those into `TurboQuant` /
   `PqFastScan`
3. a handful of remaining AM helper/test names still used `grouped_v2`
   terminology in code that now models first-class `PqFastScan`

That inconsistency makes the code harder to read and is exactly the sort of
rename debt ADR-032 calls out.

## Planned Slice

One naming checkpoint:

1. rename `page::GraphStorageFormat` variants to `TurboQuant` /
   `PqFastScan`
2. keep the on-disk format-version bytes unchanged
3. update `graph.rs` to consume the renamed enum directly
4. rename the remaining small AM helper/test names that still said
   `grouped_v2` or `experimental_grouped_v2`

No behavior change. No on-disk format change.

## Implementation

Updated:

- `src/am/page.rs`
- `src/am/graph.rs`
- `src/am/scan.rs`
- `src/am/build.rs`
- `src/am/insert.rs`
- `src/am/vacuum.rs`

### 1. Page-level storage-format enum now uses first-class names

`page::GraphStorageFormat` now exposes:

- `TurboQuant`
- `PqFastScan`

`MetadataPage::graph_storage_format()` still decodes the same wire bytes:

- `INDEX_FORMAT_V1_SCALAR` → `GraphStorageFormat::TurboQuant`
- `INDEX_FORMAT_V2_GROUPED` → `GraphStorageFormat::PqFastScan`

So the runtime type name changed, but the persisted format-version contract did
not.

### 2. Graph descriptor resolution no longer rewraps old names

`src/am/graph.rs` now matches the renamed page-level enum directly when
constructing `GraphStorageDescriptor`.

That removes the last core runtime hop from:

- old wire-era names
- into first-class product names

### 3. Small remaining helper names were normalized too

This packet also renames a few leftover helper/test identifiers in AM modules,
for example:

- `experimental_grouped_v2_exact_traversal_enabled(...)` →
  `pq_fastscan_exact_traversal_enabled(...)`
- `grouped_v2_metadata(...)` → `pq_fastscan_metadata(...)`
- `resolve_*_recognizes_grouped_v2` →
  `resolve_*_recognizes_pq_fastscan`

This keeps the core module surface aligned with ADR-032 without touching the
much larger pg-test naming sweep in `src/lib.rs`.

## Measurements

No benchmark or recall work in this slice. Naming-only cleanup.

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

This checkpoint does not change behavior. It does make the core runtime naming
more coherent:

1. page-level storage-format decoding now uses `TurboQuant` / `PqFastScan`
2. runtime dispatch no longer translates through old feasibility-era names
3. the remaining small AM helper names now match the first-class format names

What it intentionally does **not** do:

- rename the wire bytes or persisted format versions
- rename the large `src/lib.rs` pg-test surface yet
- change scan/insert/vacuum behavior

## Next Slice

The next practical cleanup / landing slices are:

1. continue removing remaining `grouped_v2` / experimental naming from the
   wider test/docs surface
2. audit hardcoded `PqFastScan` parameter assumptions like grouped build
   defaults
3. finish the remaining ADR-032/task-15 landing work beyond naming hygiene
