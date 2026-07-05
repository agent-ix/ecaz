# Task 50 Review Request: DiskANN Routine Rewrite Unsafe Reduction

## Summary

This packet handles `src/am/ec_diskann/routine.rs`, the DiskANN top-15 file.

Code commit:

- `6a31f01a Reduce DiskANN routine unsafe rewrite blocks`

The change consolidates repeated unsafe regions around DiskANN vacuum tuple rewrites, heap-source extraction, prefetch stream ownership, and test-only vacuum callback helpers.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Follow-up count | Target | Status |
| --- | ---: | ---: | ---: | --- |
| `src/am/ec_diskann/routine.rs` | 92 | 64 | <=64 | met |

## Review Notes

- Vacuum rewrite behavior is unchanged: expected bytes are still checked before GenericXLog registration mutates tuples, and replacement length is still validated.
- Heap source extraction still fetches one row version, reads the required ecvector datum, runs the visitor, and clears the caller-owned slot.
- Test vacuum helpers now expose safe wrappers because they construct the callback-duration vacuum state internally.

## Validation

- `make unsafe-block-count PATHS='src/am/ec_diskann/routine.rs'` passed with count 64.
- `rustfmt --edition 2021 --check src/am/ec_diskann/routine.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

No benchmark result is claimed in this packet. The slice changes unsafe ownership structure but not DiskANN scoring, candidate ordering, tuple encoding, or rewrite ordering.
