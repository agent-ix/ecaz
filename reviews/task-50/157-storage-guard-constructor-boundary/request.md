# Review Request: Storage Guard Constructor Boundary

## Summary

This checkpoint addresses the storage guard feedback at the first boundary level.

Code commit: `7920c13306d004b676c7761d5ec7906fcb4317de`

The reviewer was correct that the previous safe constructors leaked raw PostgreSQL relation/snapshot/slot lifetime contracts. This slice marks the scan and tuple-slot guard constructors unsafe and updates the remaining safe call sites to acknowledge the caller-owned lifetime preconditions.

## Scope

- Marked `IndexScanGuard::begin` and `HeapScanGuard::begin` unsafe.
- Marked `TupleTableSlotGuard::create` and `TupleTableSlotGuard::single_for_heap` unsafe.
- Updated the remaining direct call sites in IVF debug scan setup, DiskANN debug fixture scan, and custom scan tests.

## Remaining RAII Work

The reviewer’s stronger recommendation is still correct: a follow-up RAII pass should add lifetime-bearing guard/view types. The blocking shape is the existing debug/helper structs that own relation/snapshot guards and scan guards together; converting those safely requires restructuring those owners rather than only adding `PhantomData`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
