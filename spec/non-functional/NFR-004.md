---
id: NFR-004
title: Safety and Stability
type: non-functional-requirement
status: APPROVED
traces:
  - StR-002
---
# NFR-004: Safety and Stability

## Requirement

### No Backend Crashes

The extension SHALL NOT cause a PostgreSQL backend to crash under any input. All errors SHALL be reported via `ereport(ERROR)`, not `panic!` or segfault.

Rust panics in pgrx are caught and converted to PostgreSQL ERRORs — this is acceptable. Uncaught panics that bypass pgrx's catch mechanism are NOT acceptable.

### Memory Safety

- No use of `unsafe` code outside of pgrx FFI wrappers and GenericXLog calls
- All `unsafe` blocks SHALL have a `// SAFETY:` comment explaining the invariant
- No memory leaks in scan state, build state, or vacuum state (all freed in end/cleanup callbacks)

### WAL Correctness

- All index mutations are WAL-logged via GenericXLog
- After crash + WAL replay, the index SHALL be usable without REINDEX

### Licensing

- Extension code: MIT
- All transitive Cargo dependencies: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, or ISC
- `cargo deny check licenses` SHALL pass

## Measurement

- Fuzz testing: feed random byte sequences to `tqvector_in` — no crashes
- `cargo deny check licenses` in CI
- Code review for `unsafe` blocks
