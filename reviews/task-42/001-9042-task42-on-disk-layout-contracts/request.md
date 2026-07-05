# Review Request: Task 42 on-disk layout contracts

## Summary

Task 42 first hardening slice for on-disk format invariants.

This slice publishes named byte-offset constants for the clearest current
encoded layouts and pins them in `tests/size_of_assertions.rs`:

- generic `ItemPointer` wire fields;
- HNSW legacy/current metadata payload sizes and current metadata offsets;
- HNSW element, grouped-hot, turbo-hot, rerank, grouped-codebook, and neighbor
  tuple fixed offsets;
- DiskANN Vamana metadata payload size and field offsets.

It also adds `docs/on-disk-format.md` as the starting inventory for Task 42,
including current version-tag behavior and explicit remaining gaps.

Code commit: `6badde59124944181a6b8e10624ff8afffc1d061`

## Review Focus

- Confirm the new constants reflect the existing codecs without changing runtime
  encoding or decoding behavior.
- Confirm the layout assertions cover encoded byte contracts rather than host
  Rust struct layouts where the persisted form is a `Vec<u8>` or borrowed slice
  view.
- Confirm the documented gaps are accurate for follow-up Task 42 slices.

## Validation

See `artifacts/manifest.md`.

- `cargo test --features bench --test size_of_assertions`
  - Result: 13 passed, 0 failed.
  - Note: the run emitted one existing unused-import warning in `src/am/mod.rs`.
- Post-merge rerun after integrating `origin/main`: 13 passed, 0 failed, with
  the same existing unused-import warning.

## Remaining Task 42 Gaps

- Golden fixtures under `fixtures/on-disk/`.
- Byte-swapped fixture rejection tests.
- Static offset coverage for IVF and SPIRE storage codecs.
- qemu cross-arch decode lane with Task 48.
- `(format_version, AM, can_read, can_write)` upgrade matrix.
- WAL record version tags with Task 37.
- pg_upgrade smoke with ECAZ data present.
