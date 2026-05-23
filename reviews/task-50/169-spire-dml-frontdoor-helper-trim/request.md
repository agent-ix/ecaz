# Review Request: SPIRE DML Frontdoor Helper Trim

## Summary

This checkpoint removes two unused unsafe baserel wrappers from the SPIRE DML frontdoor and converts the test-only predicate value-kind helper from a raw-pointer unsafe API to a borrowed `Expr` helper. The production path still uses the shared primitive-plan baserel handoff; the removed update/delete wrappers had no callers.

Code commit: `8aea3f5934623ab58b657909b3cba9db10b5d5c2`

## Scope

- Removed unused `dml_frontdoor_update_primitive_plan_expr_from_baserel`.
- Removed unused `dml_frontdoor_delete_primitive_plan_expr_from_baserel`.
- Changed `dml_frontdoor_value_kind` to accept `&mut pg_sys::Expr` for test-only classification.
- Removed raw-pointer unsafe blocks from DML frontdoor predicate-value tests.

## Counts

Touched-file direct unsafe counts:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 61 | 59 |
| `src/am/ec_spire/dml_frontdoor/tests.rs` | 9 | 2 |

Current packet-local `src/` unsafe ledger: `1876` rows, checked.

## Completion Audit Note

Task 50 is not complete: current ledger output still covers 1876 direct unsafe rows in `src/`, and final closeout still requires residual registration for every remaining unsafe plus hardening/crates/tests/vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check HEAD~1..HEAD`: passed.
- `make unsafe-block-count`: passed.
- `make unsafe-ledger`: generated packet-local ledger.
- `make unsafe-ledger-check`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
