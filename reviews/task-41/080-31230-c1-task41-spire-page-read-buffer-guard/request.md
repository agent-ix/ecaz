# Review Request: Task 41 SPIRE page read buffer guard

## Summary

Task 41 buffer-resource slice for read-only SPIRE object page helpers.

This introduces `LockedBufferGuard` in `src/storage/buffer_guard.rs` and uses
it for three read-only paths in `src/am/ec_spire/page.rs`:

- `read_root_control_page`
- `with_pinned_object_tuple`
- `scan_object_tuples`

Code commit: `764d43ae`

## Safety Effect

- Moves `ReadBufferExtended` / `LockBuffer` / `UnlockReleaseBuffer` ownership
  into a shared guard.
- Keeps buffer locks and pins scoped to the page-read helper stack frame.
- Preserves caller-visible invalid-buffer errors and root-control `pgrx::error!`
  behavior.
- Updates the unsafe comment baseline from `4075` to `4058`.

## Review Focus

- Confirm `LockedBufferGuard::read_main` owns exactly the buffer pin, lock, and
  matching `UnlockReleaseBuffer`.
- Confirm the guard stays live while page pointers and tuple slices derived
  from the buffer are used.
- Confirm the migrated helpers do not leak buffer locks on visitor errors or
  root-control decode errors.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
