# ecaz_lints

Project-local Dylint lints for the ECAZ hardening lanes.

## Lints

### `ecaz_panic_across_ffi`

Finds direct Rust `extern "C"` and `extern "C-unwind"` function bodies that are
not protected by `#[pg_guard]`, `pgrx::pgrx_extern_c_guard`, or
`std::panic::catch_unwind`.

Rust panics must not unwind across PostgreSQL C callback boundaries. This lint
keeps new callback bodies from bypassing the guard policy enforced by Task 41.

The lint intentionally uses a syntactic guard search. It excludes generated
`pg_finfo_*` metadata symbols.

## Running

Use the repository targets so the pinned toolchain, Dylint driver path, and lint
deny flags stay consistent:

```sh
make ffi-dylint-self-test
make ffi-dylint
make ffi-lint
```
