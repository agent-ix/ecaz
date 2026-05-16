# Review Request: SPIRE Stable Source Identity Contract

Status: open
Owner: coder1
Head SHA: `942efcae1becb14ffcfccb4e38504e206645829e`

## Summary

This Phase 11.2 slice defines the stable source-identity contract needed before
live build/insert writers can emit global `0x02` IDs.

Key changes:

- Added `SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES = 16`.
- Added `SpireVecIdSourceIdentity::StableFixedGlobalPayload([u8; 16])` and a
  checked slice constructor.
- Kept the existing variable `StableGlobalPayload(Vec<u8>)` path for generic
  diagnostics and row-encoded compatibility helpers.
- Updated the SQL-visible vector identity contract to report
  `writer_global_source_identity =
  fixed_16_byte_source_identity_required_not_heap_tid`.
- Added `plan/design/spire-stable-source-identity-contract.md` to make the
  live-writer provider requirement explicit.
- Updated Phase 11 task docs to separate the now-defined contract from the
  still-open live provider/plumbing work.

## Deliberate Limit

This does not make live writers emit global IDs. The current AM build/insert
callbacks only provide the indexed vector datum and heap TID, and the current
SPIRE index definition rejects multi-column, expression, and partial indexes.
Heap TID is not a cross-node stable identity, so the remaining work is choosing
and implementing a production source-identity provider.

## Validation

- `cargo test fixed_global_source_identity --lib`
  - 2 passed, 0 failed
- `cargo test assign --lib`
  - 61 passed, 0 failed
- `cargo test remote_search_final_contract --lib`
  - `test tests::pg_test_ec_spire_remote_search_final_contract ... ok`
- `cargo fmt --check`
  - passed; rustfmt still prints the existing stable-toolchain warnings for
    unstable import-grouping settings
- `git diff --check`
  - passed

## Review Focus

- Is 16 bytes the right Phase 11 fixed source payload width for Leaf V2
  production writers?
- Is it correct to keep variable global payloads available for diagnostics
  while requiring the fixed variant for the live writer contract?
- Does the provider deferral accurately capture the current AM limitation, or
  should the next slice target an identity-column DDL contract immediately?
