# Review Request: C1 ADR-030 V2 Insert Format Gate

## Context

Reviewer feedback on the ADR-030 grouped-v2 lane repeatedly called out live-write safety as a gate
lift blocker.

Before grouped-v2 has a real insert path, `aminsert` must fail explicitly instead of falling into the
scalar-v1 write logic by accident.

That matters now because packet `321` already added a default-off grouped-v2 build gate, and packet
`322` proved that gate can write real grouped-v2 pages to disk.

## Problem

Once a grouped-v2 index exists on disk, a normal live `INSERT` can reach `tqhnsw_aminsert`.

Today there is no grouped-v2 live-write implementation:

- no grouped search-code derivation at insert time
- no grouped hot tuple write path
- no grouped cold rerank payload write path
- no grouped-v2 graph update contract

So the safe behavior is an explicit storage-format rejection.

## Planned Slice

Add an insert-time storage-format gate:

1. read metadata at `tqhnsw_aminsert` startup
2. classify storage as scalar-v1 or grouped-v2
3. allow scalar-v1 inserts unchanged
4. reject grouped-v2 inserts with a dedicated error

This slice intentionally excludes:

- no grouped-v2 live insert implementation
- no vacuum-path grouped-v2 guard yet
- no rerank fetch work yet
- no grouped scorer changes

## Implementation

Updated:

- `src/am/insert.rs`
- `src/lib.rs`

Concrete changes:

1. added `ADR030_GROUPED_V2_INSERT_UNSUPPORTED`
2. added `validate_insert_storage_format(...) -> Result<(), String>`
3. made `tqhnsw_aminsert` read the metadata page and reject grouped-v2 before entering scalar insert
   logic
4. added unit tests for scalar-v1 accept / grouped-v2 reject
5. added a pg test that:
   - enables the experimental grouped-v2 build gate
   - builds a source-backed grouped-v2 index
   - attempts a live `INSERT`
   - verifies the dedicated grouped-v2 insert error

## Measurements

This packet is a safety gate, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test validate_insert_storage_format_accepts_scalar_v1 --lib`: passed
  - `cargo test validate_insert_storage_format_rejects_grouped_v2 --lib`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17 test_tqhnsw_insert_rejects_grouped_v2_index`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - initial `cargo test`: invalid due to overlap with a concurrent `cargo pgrx test pg17` run; failed in
    the embedded pg test mutex layer, not in the insert gate itself
  - rerun `cargo test` with no overlapping `pgrx` run: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

Grouped-v2 indexes are now protected from accidental live writes through the scalar insert path.

What this de-risks:

1. grouped-v2 remains experimental without silent format corruption risk from normal inserts
2. future grouped insert work now has a single place to replace when the real live-write path exists
3. gate lift blockers from reviewer feedback are now being handled in the runtime/write path, not only
   in scorer prep

## Next Slice

The next review-driven blocker should stay on safety and storage support:

1. grouped-v2 vacuum-path rejection
2. cold `reranktid -> rerank tuple` fetch seam
3. stronger grouped metadata/runtime validation
