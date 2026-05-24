# Task 50 Review Request: Explain Callback Guards

## Summary

This packet removes the two remaining direct `pgrx_extern_c_guard` call sites from `src/am/common/explain.rs` by routing both PostgreSQL EXPLAIN callbacks through the shared `pg_callback!` macro introduced in the common callback guard slice.

The callback bodies are behavior-preserving. The change only moves the C-boundary unwind guard from hand-written `unsafe { pgrx::pgrx_extern_c_guard(...) }` blocks into the named callback guard helper.

## Code Under Review

- Code commit: `19efa9a5 Centralize explain callback guards`
- File changed: `src/am/common/explain.rs`

## Unsafe Ledger

- `src/am/common/explain.rs`: `unsafe` matches `22 -> 20`
- `src/`: `unsafe` matches `2646 -> 2644`
- `src/am/common/explain.rs`: direct `pgrx_extern_c_guard` matches `2 -> 0`

## Validation

Packet-local artifacts are recorded in `artifacts/manifest.md`.

- `rustfmt --check src/am/common/explain.rs`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass with existing `src/am/mod.rs` unused import warning
- `cargo check --all-targets --no-default-features --features pg18,pg_test`: pass with existing Hadamard helper dead-code warnings
- `cargo test --lib am::common::explain --no-default-features --features pg18,pg_test --no-run`: pass with existing Hadamard helper dead-code warnings
- `git diff --check HEAD`: pass

## Review Focus

- Confirm that `pg_callback!` preserves the same unwind-containment behavior as the removed hand-written guards.
- Confirm previous EXPLAIN hook chaining still occurs in the same control-flow cases.
- Confirm the removed direct unsafe blocks are now covered by the shared callback guard contract rather than being silently hidden in a local helper.
