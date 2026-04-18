# Review Request: C1 Standalone Cargo Test PG Backend Stubs

Current head: `a4ccba9`

This packet covers local uncommitted work on top of that head.

## Context

Recent ADR-030 v2 packets kept adding pure Rust and `#[pg_test]` coverage, but
the normal `cargo test` lane was still too weak as a checkpoint because the
unit-test binary linked extension code that expects PostgreSQL backend symbols
to exist only when the extension is loaded inside a backend.

That left an awkward split:

1. `cargo pgrx test` exercises the real backend, but depends on a writable
   `pgrx-install`
2. plain `cargo test` should still be able to cover Rust/unit/property work
3. without a small linker/runtime bridge, plain `cargo test` remained hostage
   to PostgreSQL backend linkage details instead of the Rust code under test

## Problem

Before this slice, plain `cargo test` was not a dependable checkpoint for this
repo because the test binaries pulled in unresolved PostgreSQL backend symbols
through error-reporting and test-framework paths.

The missing pieces were:

1. a minimal standalone shim for the handful of backend globals and error APIs
   referenced by unit-test binaries
2. build wiring that links that shim only for local test binaries
3. a clear wrapper surface for the real `cargo pgrx test pg17` lane

## Planned Slice

Enable the normal Rust test lane without pretending it replaces the backend
lane:

1. add a Linux/x86_64-only native shim for the PostgreSQL symbols reached by
   test binaries
2. link it into test builds and route backend-style errors back into Rust
   panics
3. keep the `cargo pgrx test pg17` wrapper direct and honest
4. leave the real `pgrx` install/test lane unchanged so any remaining failure
   is an environment problem, not a hidden wrapper policy

## Implementation

Updated:

- `.cargo/config.toml`
- `Cargo.toml`
- `build.rs`
- `csrc/standalone_pg_backend_stubs.c`
- `src/lib.rs`
- `src/standalone_pg_backend_stubs.rs`
- `scripts/run_pgrx_pg17_test.sh`

Concrete changes:

1. added `cc` as a build dependency
2. added a small `build.rs` that compiles
   `csrc/standalone_pg_backend_stubs.c` on Linux/x86_64
3. added a Linux-target linker flag so test binaries can tolerate backend-only
   unresolved symbols that are resolved at extension-load time in PostgreSQL
4. added `src/standalone_pg_backend_stubs.rs` and gated it behind
   `#[cfg(all(test, target_arch = "x86_64", target_os = "linux"))]`
5. implemented minimal backend globals / error helpers in
   `csrc/standalone_pg_backend_stubs.c`, including:
   - `CurrentMemoryContext`
   - `ErrorContext`
   - `PG_exception_stack`
   - `error_context_stack`
   - `errstart` / `errfinish`
   - `errmsg` / `errdetail` / `errhint`
   - `CopyErrorData` / `FreeErrorData`
6. wired backend-reported test failures back into Rust panics through
   `tqvector_test_pg_backend_panic(...)`
7. simplified `scripts/run_pgrx_pg17_test.sh` to a direct
   `cargo pgrx test pg17 "$@"` wrapper so the wrapper no longer masks what the
   underlying lane is doing

## Validation

Passed:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Still failing in this environment:

- `bash scripts/run_pgrx_pg17_test.sh`

Observed failure is now during `cargo pgrx install --test`, not at the old
linker boundary:

- failed writing
  `/home/peter/.pgrx/17.9/pgrx-install/share/postgresql/extension/tqvector.control`
- `Read-only file system (os error 30)`

## Outcome

This slice restores the value of plain `cargo test` as a real checkpoint:

1. unit/property/recall-smoke coverage runs locally again
2. the remaining `pgrx` gap is now the install destination in this sandbox
3. the wrapper stays transparent about that gap instead of hiding it behind
   extra logic

## Next Slice

Use that restored checkpoint lane to package the current pq-fastscan fixes in
smaller reviewable packets:

1. repair the debug/vacuum helper surfaces so they follow the real heap-backed
   source-backed scan shape
2. then align the pq-fastscan runtime fixtures and test expectations with the
   current source-backed behavior
