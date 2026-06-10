# Task 94 Packet 021: Apple aarch64 SVE Assembly Gate

## Summary

This packet fixes the latest PR failure on `macos-14-arm64`. The grouped-PQ SVE
`global_asm!` used ELF-only directives (`.hidden`, `.type`) and was compiled on
Apple aarch64 during `ecaz-cli compile`.

The fix gates grouped-PQ SVE assembly, externs, runtime detection, and the SVE
implementation helper to non-Apple aarch64:

```rust
#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
```

Apple aarch64 now uses the scalar fallback path. Linux aarch64 keeps the SVE/SVE2
path required for Graviton 4.

## Code

- Code checkpoint: `20856cfb4d69e9a554acbe47cb9e7c4c78ac2dbc`
- Changed file: `src/quant/grouped_pq_block/sve.rs`

## Evidence

- Existing failed CI job inspected, no rerun requested:
  - `pg18 / stable`: https://github.com/agent-ix/ecaz/actions/runs/27229478676/job/80405533632
- Failure source: `artifacts/ci-failure-source.md`
- Local format: `artifacts/cargo-fmt-check.log`
  - `cargo fmt --check`
  - Passed
- Local clippy: `artifacts/cargo-clippy-pg18-bench.log`
  - `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings`
  - Passed
- Local grouped-PQ tests: `artifacts/cargo-test-grouped-pq-lib.log`
  - `cargo test grouped_pq --lib`
  - Passed: 35 passed, 0 failed
- Local Linux aarch64 cross-check attempt: `artifacts/cargo-check-aarch64-linux-pg18.log`
  - Blocked by missing local `aarch64-linux-gnu-gcc` before crate validation.

## Out of Scope

- No manual CI rerun was started.
- No AWS instance, benchmark, or smoke test was started.
