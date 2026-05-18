# Review Request: Dylint FFI Guard Lint

## Scope

Code commit: `402885505408c4f3a0f7e75c8fe8175793f95ee5`

This packet adds the Task 41 Dylint lane for invariant #1:

- adds `crates/ecaz-lints` with `ecaz_panic_across_ffi`
- adds a negative fixture that proves an unguarded C ABI callback is rejected
- routes Dylint through `scripts/run_dylint.sh` and `scripts/run_dylint_self_test.sh`
- wires `make ffi-lint` to run the existing Python gates plus the Dylint suite
- excludes `crates/ecaz-lints` from the normal Cargo workspace so ordinary `cargo test` does not build a rustc-private nightly lint crate

The lint is syntactic by design: direct `extern "C"` / `extern "C-unwind"` Rust function bodies must use `#[pg_guard]`, `pgrx::pgrx_extern_c_guard`, or `std::panic::catch_unwind`. Generated `pg_finfo_*` metadata functions are excluded.

## Validation

`make ffi-lint` passed on the code commit.

Key lines:

- `ffi audit passed: 101 direct C ABI functions, 288 pgrx-managed SQL entrypoints`
- `ffi audit self-test passed`
- `ffi lint self-test passed`
- `ffi lint passed: raw PostgreSQL resource APIs are confined to guard modules`
- `dylint self-test passed: /Users/peter/dev/tqvector/crates/ecaz-lints/target/panic_across_ffi.self-test.log`
- `Finished dev profile ...`

Artifacts:

- `artifacts/make-ffi-lint.log`
- `artifacts/dylint-self-test.log`
- `artifacts/manifest.md`
