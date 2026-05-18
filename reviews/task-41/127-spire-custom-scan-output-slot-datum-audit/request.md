# Review Request: Task 41 Invariant #2 SPIRE CustomScan output slot audit

Audit head: `a927f145377440f3adc1f2a00dddd9a75f835091`

## Summary

This packet covers the remaining Phase B strategy item: SPIRE CustomScan tuple
payload modules.

The relevant slot access is write-only output-slot population. The code clears
the virtual slot, writes `tts_isnull` and `tts_values`, sets `tts_nvalid`, and
stores the virtual tuple. It does not read a Datum out of
`TupleTableSlot.tts_values`, does not call `slot_getsomeattrs_int`, and does
not create a Rust borrow backed by slot storage.

No invariant #2 slot-Datum code change is needed for this surface.

## Scope

- Audit-only packet; no code change.
- Covered `src/am/ec_spire/custom_scan`.
- Did not audit unrelated CustomScan executor state lifetimes, palloc arrays,
  buffer/page views, or C string ownership beyond the slot-Datum question.

## Evidence

- `artifacts/custom-scan-slot-inventory.log` shows CustomScan slot access sites.
- `artifacts/json-payload-slot-output-excerpt.log` captures JSON payload output
  slot population.
- `artifacts/typed-payload-slot-output-excerpt.log` captures typed payload
  output slot population.
- `artifacts/git-status.log` records the audit head worktree context.

## Validation

No tests were run because this is an audit-only packet with no code change.

## Reviewer Focus

- Confirm CustomScan tuple payload handling writes output Datums rather than
  borrowing from slot Datums.
- Confirm Phase B can treat this surface as audit-only for invariant #2.
