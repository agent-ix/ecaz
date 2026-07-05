# Review Request: Task 41 IVF directory buffer guard

## Summary

Task 41 buffer-resource slice for IVF directory tuple updates.

This uses `LockedBufferGuard` in `src/am/ec_ivf/page.rs` for:

- `rewrite_ivf_list_directory`
- `update_ivf_list_directory`

Code commit: `81b84e8c`

## Safety Effect

- Moves directory page `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Keeps directory pages under exclusive locks while tuple bounds are checked,
  decoded, and rewritten.
- Keeps `GenericXLogTxn` as the WAL cleanup owner; early errors drop the WAL
  transaction before the locked buffer guard drops.
- Updates the unsafe comment baseline from `3912` to `3894`.

## Review Focus

- Confirm all early error paths still abort unfinished WAL state before the
  buffer guard drops.
- Confirm directory tuple pointer usage stays scoped to the locked buffer
  lifetime.
- Confirm size and bounds checks remain unchanged.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
