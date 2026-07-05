# Review Request: SPIRE unsafe boundary audit

## Summary

This checkpoint closes the Phase 12b.6 unsafe/business-logic audit.

The code checkpoint removes avoidable unsafe from two relation publish object
writer helpers:

- `write_relation_replacement_objects`
- `write_relation_scheduled_replacement_objects`

Those helpers only call the safe generic replacement-object writer. The actual
relation mutation remains concentrated in the relation object-store
implementation.

Code checkpoint: `1ea3b750c29a60627f8c3e196afd7110ba252887`

## Audit Result

The current path for the tracker's old `dml_frontdoor.rs` name is
`src/am/ec_spire/dml_frontdoor/mod.rs`.

Scoped count for `dml_frontdoor` plus `update`:

- Before: 247 unsafe-bearing lines.
- After: 244 unsafe-bearing lines.

Full `src/am/ec_spire` count:

- Before: 1430 unsafe-bearing lines.
- After: 1427 unsafe-bearing lines.

The remaining scoped sites are classified in `artifacts/classification.md` as
FFI/SPI or storage/relation boundary sites. No remaining category (b) avoidable
unsafe or business-layer-only unsafe site was identified in this pass.

## Validation

- `cargo test -p ecaz relation_scheduled_replacement_execution_input_uses_publish_plan`
- `cargo fmt --check`

Raw logs and command metadata are in `artifacts/manifest.md`.

## Reviewer Focus

- Confirm the two helpers made safe are genuinely covered by the generic writer
  abstraction.
- Confirm the remaining scoped sites are boundary sites rather than business
  logic that should move in 12b.
- Confirm the count reduction and tracker closure are acceptable without a
  larger mechanical `dml_frontdoor/ffi.rs` split.
