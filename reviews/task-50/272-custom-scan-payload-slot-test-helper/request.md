# Task 50 Review Request: Custom Scan Payload Slot Test Helper

## Summary

This slice removes direct slot/DATUM unsafe from the custom-scan payload slot test body. The fixture now reads the virtual slot through small local helpers:

- `custom_scan_payload_slot_datum`
- `custom_scan_payload_slot_bigint`
- `custom_scan_payload_slot_text`
- `custom_scan_payload_slot_is_null`

The remaining unsafe is centralized in those helpers with the test fixture contract: the slot is live and the attribute numbers belong to the relation created by the test.

## Unsafe Burndown

- Previous broad count from packet 271: `2210`
- Current broad count: `2208`
- Net: `-2`

## Validation

Artifacts are under `reviews/task-50/272-custom-scan-payload-slot-test-helper/artifacts/`.

- `git-diff-check.log`: passed
- `rustfmt-check.log`: standalone rustfmt skipped because changed file is a module-included test source; syntax/format viability was checked by cargo parsing
- `custom-scan-slot-callsite-grep.log`: remaining slot/DATUM unsafe is centralized in the local helper functions
- `unsafe-count.log`: `2208`
- `cargo-check-pg18-bench.log`: passed with the existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`
- `cargo-test-lib-pg18-pgtest-no-run.log`: passed with existing Hadamard test-only dead-code warnings

