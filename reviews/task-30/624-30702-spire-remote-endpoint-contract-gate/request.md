# Review Request: SPIRE Remote Endpoint Contract Gate

Status: open
Owner: coder1
Head SHA: `79ea08aa0423dd90fac5aa0d44b51e30bee3dd45`

## Summary

This Phase 11.3 / Stage B slice adds a production-readiness contract gate for
the current remote search endpoint without claiming the endpoint is production
complete.

Key changes:

- Adds `ec_spire_remote_search_endpoint_contract()` as a SQL-visible contract
  surface for endpoint function, protocol version, request columns, response
  columns, selected-PID semantics, RaBitQ-only support, extension version, and
  remaining endpoint blockers.
- Marks scoring-option binding, quantizer/index fingerprint binding, and
  opclass-binary binding as non-ready rows that must be resolved before remote
  candidate batches are accepted as production merge inputs.
- Updates the libpq result contract `pid` validator to reflect the Stage A
  delta-row invariant: selected leaf PIDs may yield leaf-derived delta PIDs.
- Adds the new endpoint contract surface to the operator entrypoint contract.
- Records conservative Stage B progress in the Phase 11 task file.

## Deliberate Limits

- This does not add fingerprint, opclass, or explicit scoring/rerank fields to
  the actual remote candidate row yet.
- This does not change the nine-column `ec_spire_remote_search` candidate batch
  shape or libpq decoder.
- This keeps PQ/PQFastScan unsupported and reserves wording around those paths;
  RaBitQ remains the first supported quantized scoring family.

## Validation

- `cargo fmt`
  - passed; rustfmt still prints existing stable-toolchain warnings for
    unstable import-grouping settings
- `cargo test remote_search_endpoint_contract --lib`
  - passed: 0 tests run after filtering, library compiled successfully
- `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`
  - passed: 1 passed, 0 failed
- `git diff --check`
  - passed

## Review Focus

- Does the endpoint contract gate expose the right non-ready blockers before
  Stage B changes the candidate row shape?
- Is the selected-PID / leaf-derived delta PID wording consistent across the
  parameter, result, and endpoint contracts?
- Are the RaBitQ-only and PQ/PQFastScan-reserved contract rows conservative
  enough for the current Phase 11 scope?
