# Task 67 Review Request: cloud bench SIMD env propagation

## Summary

This checkpoint fixes the runner issue found while processing packet 036 feedback. `ecaz cloud bench --simd-mode` was setting `ECAZ_SIMD` for PostgreSQL, but not for the remote `ecaz bench suite` CLI process.

That made packet 036's sidecar-rerank scalar-vs-auto framing too weak: sidecar scoring happens in the CLI process. With this change, cloud bench exports `ECAZ_SIMD` into both PostgreSQL service state and the remote suite process.

## Scope

- Changed `crates/ecaz-cloud/src/commands/bench.rs`.
- Added a focused unit test proving the generated SSM script contains:
  - PostgreSQL systemd `ECAZ_SIMD` setup.
  - CLI-process `export ECAZ_SIMD=...` before `/usr/local/bin/ecaz bench suite run`.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz-cloud remote_suite_script_exports_simd_for_cli_and_postgres`

Logs are under `artifacts/`.

## Follow-up

Packet 036's 100k scalar/auto numbers should be replaced or superseded by a rerun using this fixed cloud bench wrapper.
