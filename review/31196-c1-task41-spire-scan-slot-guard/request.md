# Task 41 Review Request: SPIRE Scan Slot Guard

## Scope

This checkpoint removes the local `HeapTupleSlot` RAII wrapper from
`src/am/ec_spire/scan/relation.rs` and extends the shared
`TupleTableSlotGuard` with a `single_for_heap` constructor for
`MakeSingleTupleTableSlot`.

Code commit: `cce95df454b67a485ea5fc7c7a1c3863487087ac`

## Safety Invariant

`TupleTableSlotGuard::single_for_heap` receives an open heap relation and
owns the slot returned by `MakeSingleTupleTableSlot`. The shared guard drops
the slot via `ExecDropSingleTupleTableSlot`.

The SPiRE scan relation path keeps the slot guard alive across heap rerank
prefetch/fetch/scoring work and no longer owns a module-local slot drop
implementation.

## Baseline Impact

Unsafe comment baseline decreased:

- before: `4239`
- after: `4238`

This removes the module-local slot drop unsafe site. The allocation unsafe is
centralized in the shared slot guard constructor.

## Validation

See `artifacts/validation.md`.

Commands run:

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Review Focus

- Confirm `single_for_heap` is a clear shared API name for the
  `MakeSingleTupleTableSlot` call shape.
- Confirm the SPiRE scan relation slot lifetime remains wide enough for the
  heap rerank path.
