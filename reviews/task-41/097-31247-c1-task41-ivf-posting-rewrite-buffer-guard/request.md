# Review Request: Task 41 IVF posting rewrite buffer guard

## Summary

Task 41 buffer-resource slice for IVF posting rewrite and debug page helpers.

This uses `LockedBufferGuard` in `src/am/ec_ivf/page.rs` for:

- `rewrite_ivf_posting`
- `rewrite_ivf_postings_for_list_block`
- `debug_ivf_posting_block_summary`

Code commit: `da2a77c2`

## Safety Effect

- Moves posting rewrite `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Uses a shared-locked guard for debug page summaries and exclusive-locked
  guards for posting rewrite paths.
- Keeps `GenericXLogTxn` cleanup behavior unchanged for rewrite error paths.
- Updates the unsafe comment baseline from `3894` to `3877`.

## Review Focus

- Confirm rewrite tuple pointer usage stays scoped to the locked buffer
  lifetime.
- Confirm the list-block wrapper still delegates to
  `rewrite_ivf_postings_from_exclusive_buffer` with an exclusive-locked buffer.
- Confirm debug summary no longer needs manual unlock/release after the closure.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
