# Review Request: C1 ADR-030 V2 Runtime Descriptor Rename

## Context

ADR-032 and task 15 rename the two first-class index formats to:

- `TurboQuant`
- `PqFastScan`

Packet 379 finished the reloption-driven grouped build+scan cutover, but the
runtime type surface still exposed the older feasibility names:

- `GraphStorageDescriptor::ScalarV1`
- `GraphStorageDescriptor::GroupedV2`
- `GroupedGraphLayout`

That mismatch made the branch harder to reason about: SQL/config/docs were
speaking in `TurboQuant` / `PqFastScan`, while the runtime matching code and
error paths were still speaking in `ScalarV1` / `GroupedV2`.

## Problem

The remaining naming gap had three concrete costs:

1. runtime match sites in scan/insert/vacuum/debug still used the old
   feasibility-era names
2. human-facing error strings still surfaced `grouped-v2` in places where
   ADR-032 now wants `PqFastScan`
3. the layout type name still encoded the old experiment label instead of the
   now-intended first-class format

This did not block functionality directly, but it made every follow-on insert /
vacuum parity slice harder to read and review.

## Planned Slice

One rename-only checkpoint:

1. rename the runtime descriptor variants to `TurboQuant` / `PqFastScan`
2. rename `GroupedGraphLayout` to `PqFastScanLayout`
3. update the touched runtime error strings to use `PqFastScan`
4. leave the on-disk wire/version tags unchanged

The `page.rs` storage-format enum is intentionally out of scope. Its
`ScalarV1` / `GroupedV2` names remain the disk-versioning layer, matching
ADR-032's "wire tags are not renamed" rule.

## Implementation

Updated:

- `src/am/graph.rs`
- `src/am/insert.rs`
- `src/am/vacuum.rs`
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/lib.rs`

### 1. Renamed the runtime descriptor and layout types

`src/am/graph.rs` now exposes:

- `GraphStorageDescriptor::TurboQuant { code_len }`
- `GraphStorageDescriptor::PqFastScan(PqFastScanLayout)`

instead of:

- `ScalarV1`
- `GroupedV2(GroupedGraphLayout)`

This rename is purely at the runtime descriptor layer. Metadata decoding still
maps the existing disk-format enum variants:

- `page::GraphStorageFormat::ScalarV1` -> `GraphStorageDescriptor::TurboQuant`
- `page::GraphStorageFormat::GroupedV2` -> `GraphStorageDescriptor::PqFastScan`

### 2. Updated runtime match sites to use the renamed descriptors

The rename is carried through the actual runtime consumers:

- scan dispatch and grouped-score shape derivation in `src/am/scan.rs`
- insert format resolution in `src/am/insert.rs`
- vacuum format resolution in `src/am/vacuum.rs`
- grouped-storage debug detection in `src/am/scan_debug.rs`

That means new work no longer has to mentally translate "GroupedV2" while
implementing the `PqFastScan` landing plan.

### 3. Renamed the small runtime constants and error strings that were touched

This packet also updated the visible runtime wording in the same touched paths:

- live rerank window constants now use `PQ_FASTSCAN_*`
- grouped exact-score error text now says `PqFastScan`
- insert/vacuum unsupported errors now say:
  - `tqhnsw aminsert does not support PqFastScan indexes yet`
  - `tqhnsw vacuum does not support PqFastScan indexes yet`
- scan env-validation errors now say `PqFastScan` instead of `grouped-v2`
- graph metadata validation errors now say `PqFastScan metadata ...`

This is still rename-only work. The unsupported insert/vacuum paths remain
unsupported after the rename.

### 4. Updated pg/unit expectations for the renamed runtime surface

`src/lib.rs` and the local unit tests in the AM modules now expect:

- `TurboQuant` / `PqFastScan` descriptor names
- the renamed `PqFastScan` runtime error strings

No behavioral assertions changed beyond the wording/type names.

## Measurements

No new benchmark or recall measurements in this slice. This is runtime-surface
cleanup only.

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

This checkpoint aligns the runtime type surface with ADR-032:

1. runtime descriptors now say `TurboQuant` / `PqFastScan`
2. the runtime layout type now says `PqFastScanLayout`
3. touched runtime errors now speak the same naming language as task 15
4. the on-disk format/version layer remains unchanged, as intended

What it still does **not** do:

- insert parity for `PqFastScan`
- vacuum parity for `PqFastScan`
- wholesale renaming of legacy helper/test function names that still use
  `grouped_v2`
- any on-disk tag rename

## Next Slice

The next practical slices are:

1. continue removing the remaining legacy `grouped_v2` naming from the runtime
   helper/test surface where it still obscures intent
2. more importantly, start replacing the `PqFastScan` insert/vacuum unsupported
   branches with real append/repair implementations
