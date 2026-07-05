# Review Request: Task 41 IVF debug heap scan guards

## Summary

Task 41 follow-up for IVF debug heap-backed scans in `src/am/ec_ivf/scan.rs`.

This slice replaces the debug helper's manual snapshot and index scan cleanup
with existing RAII guards:

- `ActiveSnapshotGuard::latest_after_command_counter`
- `IndexScanGuard`

`DebugHeapBackedScan` now stores guarded scan, snapshot, heap relation, and index
relation fields in cleanup order.

Code commit: `8939a99b`

## Safety Effect

- Removes debug helper manual `RegisterSnapshot` / `PushActiveSnapshot` setup.
- Removes debug helper manual `PopActiveSnapshot` / `UnregisterSnapshot`
  cleanup.
- Removes debug helper manual `index_beginscan` / `index_endscan` cleanup.
- Removes the null-scan cleanup branch because created guards now clean up before
  the error is raised.
- Updates the unsafe comment baseline from `4086` to `4078`.

## Review Focus

- Confirm `DebugHeapBackedScan` field order drops the index scan before the
  active snapshot and relations.
- Confirm the debug helpers use `state.scan.as_ptr()` only while the scan guard
  is live.
- Confirm `latest_after_command_counter` preserves the previous command counter
  increment plus latest snapshot semantics.
- Confirm `IndexScanGuard::begin` receives the same key/order-by counts as the
  previous raw `index_beginscan` calls.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
