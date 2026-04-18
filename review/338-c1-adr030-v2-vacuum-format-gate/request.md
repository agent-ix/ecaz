# Review Request: C1 ADR-030 V2 Vacuum Format Gate

## Context

Reviewer feedback on the ADR-030 grouped-v2 lane called out vacuum safety as a gate-lift blocker,
alongside insert safety.

Packet `337` already made live `INSERT` reject grouped-v2 indexes explicitly. Vacuum still needed the
same protection because the current implementation only knows how to:

- decode scalar-v1 element tuples
- score scalar-v1 codes during repair search
- rewrite scalar-v1 element tuples during pass-1/pass-2 maintenance

## Problem

Once a grouped-v2 index exists on disk, the vacuum callbacks can still be invoked through:

- `ambulkdelete`
- `amvacuumcleanup`
- debug vacuum helpers that exercise those callbacks directly
- SQL paths like `ANALYZE` that reach the AM vacuum interface

There is no grouped-v2 maintenance implementation yet, so those paths must fail explicitly instead
of trying to interpret grouped-hot / rerank tuples as scalar element tuples.

## Planned Slice

Add a vacuum-time storage-format gate:

1. read metadata at vacuum callback startup
2. classify storage as scalar-v1 or grouped-v2
3. allow scalar-v1 vacuum unchanged
4. reject grouped-v2 vacuum with a dedicated error

This slice intentionally excludes:

- no grouped-v2 vacuum implementation
- no cold rerank fetch work yet
- no grouped scorer implementation yet
- no gate lift

## Implementation

Updated:

- `src/am/vacuum.rs`
- `src/lib.rs`

Concrete changes:

1. added `ADR030_GROUPED_V2_VACUUM_UNSUPPORTED`
2. added `validate_vacuum_storage_format(...) -> Result<(), String>`
3. made both `tqhnsw_ambulkdelete` and `tqhnsw_amvacuumcleanup` reject grouped-v2 metadata before
   entering scalar maintenance logic
4. added unit tests for scalar-v1 accept / grouped-v2 reject
5. added a pg test that:
   - enables the experimental grouped-v2 build gate
   - builds a source-backed grouped-v2 index
   - invokes the debug vacuum path
   - verifies the dedicated grouped-v2 vacuum error
6. updated the existing grouped-v2 ordered-scan rejection test to stop running `ANALYZE`, because
   that now correctly trips the new vacuum gate before scan startup

## Measurements

This packet is a safety gate, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test validate_vacuum_storage_format_accepts_scalar_v1 --lib`: passed
  - `cargo test validate_vacuum_storage_format_rejects_grouped_v2 --lib`: passed
  - `cargo test test_experimental_grouped_v2_ordered_scan_rejects_runtime --lib`: passed after
    removing the now-invalid `ANALYZE` call
  - `cargo test test_tqhnsw_vacuum_rejects_grouped_v2_index --lib`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17 test_tqhnsw_vacuum_rejects_grouped_v2_index`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

Grouped-v2 indexes are now protected from accidental maintenance through the scalar vacuum path.

What this de-risks:

1. grouped-v2 experimental builds cannot silently corrupt themselves during vacuum/analyze activity
2. insert-path and vacuum-path gate-lift blockers are now both covered explicitly
3. future grouped maintenance work has a single format gate to replace when real grouped-v2 vacuum
   support exists

## Next Slice

The next review-driven blocker should move to data access needed by the eventual pipeline:

1. cold `reranktid -> rerank tuple` fetch seam
2. stronger grouped metadata/runtime validation
3. end-to-end `binary -> grouped -> rerank` measurement once the read/scorer path is ready
