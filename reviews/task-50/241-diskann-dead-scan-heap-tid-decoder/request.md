---
task: 50
packet: 241
topic: diskann-dead-scan-heap-tid-decoder
role: coder
status: ready-for-review
created: 2026-05-21T07:02:39-07:00
head_sha: 9e16cb69f5674fe958005689e06aa97f8ec972ac
---

# Review Request: DiskANN Dead Scan Heap TID Decoder

## Summary

This packet removes an unreferenced DiskANN scan-state heap TID decoder.

Changes:

- Deleted `scan_state::decode_heap_tid`, which had no callers.
- Removed the now-unused `item_pointer_get_both` import from `scan_state.rs`.
- Left the live ambuild heap TID decoder intact; it is still used by `ec_diskann_aminsert` and ambuild callback code.

## Safety Notes

- This is dead-code removal only.
- The remaining DiskANN heap-TID decoder is `ambuild::decode_heap_tid`, which still handles PostgreSQL callback-supplied item pointers.

## Unsafe Count

- `src/am/ec_diskann/scan_state.rs`: `18 -> 16`
- Previous repo count: `2484`
- Current repo count: `2482`
- Delta: `-2`

The packet-local count log is:

- `artifacts/unsafe-counts.log`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_diskann/scan_state.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-lib-ec-diskann-pg18-no-run.log`: `cargo test --lib ec_diskann --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.
