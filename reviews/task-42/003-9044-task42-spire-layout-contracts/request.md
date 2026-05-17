# Review Request: Task 42 SPIRE layout contracts

## Summary

Task 42 follow-up for static on-disk layout invariants.

This slice extends `tests/size_of_assertions.rs` to cover the current SPIRE
encoded storage and metadata contracts:

- partition-object header field offsets;
- legacy assignment-row dynamic offsets;
- leaf V2 meta body offsets;
- leaf V2 segment prefix and dynamic-region offset helpers;
- partition-object V2 chain meta and segment prefixes;
- local-store config and descriptor offsets;
- placement entry and placement-directory offsets;
- epoch manifest offsets;
- object manifest and manifest-entry offsets.

`docs/on-disk-format.md` now records SPIRE storage and metadata as covered by
`layout-check`.

Code commit: `a2f12000c505c11b46daac9627d657b1ca071324`

## Review Focus

- Confirm the SPIRE constants mirror the existing `encode` / `decode` slice
  indexes in `src/am/ec_spire/storage/**` and `src/am/ec_spire/meta/**`.
- Confirm the dynamic offset helpers for assignment rows and leaf V2 segments
  match the variable-length regions.
- Confirm this slice does not alter runtime encoding or decoding behavior.

## Validation

See `artifacts/manifest.md`.

- `cargo test --features bench --test size_of_assertions`
  - Result: 13 passed, 0 failed.
  - Note: the run emitted one existing unused-import warning in `src/am/mod.rs`.

## Remaining Task 42 Gaps

- Golden fixtures under `fixtures/on-disk/`.
- Byte-swapped fixture rejection tests.
- Additional SPIRE routing/top-graph body-prefix assertions if those become
  durable page-buffer contracts beyond the current partition-object codecs.
- qemu cross-arch decode lane with Task 48.
- `(format_version, AM, can_read, can_write)` upgrade matrix.
- WAL record version tags with Task 37.
- pg_upgrade smoke with ECAZ data present.
