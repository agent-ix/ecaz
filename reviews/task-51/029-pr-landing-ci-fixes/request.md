# Review Request: PR Landing CI Fixes

- PR: `https://github.com/agent-ix/ecaz/pull/4`
- Scope: resolve `main` merge conflict and CI setup failures blocking Task 51 landing

## Change

After opening the Task 51 PR, GitHub reported the branch as conflicting with
`main`. The merge conflict was in `src/am/ec_ivf/scan.rs`; the resolution keeps
main's `HeapSlotReader` path and Task 51's deduplicated heap-block prefetch
counter/prefetch behavior.

CI then failed before meaningful tests because:

- `cargo fmt --check` wanted formatting in merged main files.
- Ubuntu 24.04 runners could not install `postgresql-server-dev-17` /
  `postgresql-server-dev-18` without the PGDG apt repository.
- Non-pgrx Rust jobs that compile pgrx-backed crates had no PG18
  `pg_config` path or `PGRX_HOME`.

The workflow now installs PGDG headers for Rust/SIMD/coverage jobs that need
pgrx bindgen, and for pgrx-backed jobs before installing PG17/PG18 dev
packages.

## Validation

- `artifacts/cargo-fmt-check.log`: `cargo fmt --all -- --check` passed.
- `artifacts/cargo-test-ecaz-cloud-install.log`: `cargo test -p ecaz-cloud --no-default-features install` passed.
- `artifacts/cargo-test-ecaz-cli-sidecar.log`: `cargo test -p ecaz-cli sidecar --no-default-features` passed, `7 passed; 0 failed`.

No AWS instances were started for this fix.
