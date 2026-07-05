# SPIRE Code Layout

SPIRE modules use directory modules once they carry non-trivial unit-test
coverage. Keep production code in `mod.rs` or concern-specific sibling files,
and keep Rust unit tests in `tests.rs` or a `tests/` subdirectory included from
the module.

Do not add new inline `#[cfg(test)] mod tests` blocks to SPIRE production
source files. For small modules, prefer:

```text
module/
  mod.rs
  tests.rs
```

For larger modules, prefer:

```text
module/
  mod.rs
  concern.rs
  tests/
    concern.rs
```

PostgreSQL `pg_test` SQL fixtures may stay in `src/lib.rs` until the Phase
12b fixture-sink split moves them into `src/tests/`.

