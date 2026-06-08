# Task 89 / Packet 001: TQ+ Format Design ADR

## Summary

This packet lands the Task 89 Phase 1 format-design ADR before any TQ+ porting
work. The decision is:

- keep `storage_format = 'turboquant'` as the durable family identity;
- expose TQ+ as `turboquant_profile = 'tqplus'`;
- keep current behavior as `turboquant_profile = 'standard'`;
- do not promote Task 86's `storage_format = 'turboquant_tqplus'` tag-4 shape
  as the production API;
- allow tag 4 only as a read-only IVF legacy decoder if packet-011 fixtures
  need to be read.

## Files

- `spec/adr/ADR-076-turboquant-tqplus-format-and-validation.md`
- `spec/adr/index.md`

## Validation

Documentation-only checkpoint. No Rust tests were run.

I checked the current AM/storage decision context before writing the ADR:

- ADR-070: on-disk forward-compat posture.
- ADR-071 and ADR-072: shared quantizer math with AM-local codec adapters.
- Task 86 packet-011 format plan from preserved history.
- Current AM source surfaces for IVF, SPIRE, HNSW, and DiskANN storage-format
  and payload binding.

## Reviewer Focus

Please review whether ADR-076 is acceptable as the Phase 1 gate for Task 89:

1. Is `turboquant_profile = 'tqplus'` the right production surface?
2. Is rejecting `turboquant_tqplus` as a preferred production storage format
   correct?
3. Are the compatibility, calibration-storage, and measurement gates strong
   enough before Phase 2 ports begin?
