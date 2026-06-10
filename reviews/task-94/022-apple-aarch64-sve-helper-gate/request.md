# Task 94 Packet 022: Apple aarch64 SVE Helper Gate

## Summary

This packet fixes the follow-on Apple aarch64 warning after packet 021. Once the
SVE implementation was gated off Apple aarch64, its `centroid_index` helper
became dead code on that platform under `-D warnings`.

The helper now uses the same cfg as the SVE implementation:

```rust
#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
```

Linux aarch64 keeps the grouped-PQ SVE/SVE2 path for Graviton 4; Apple aarch64
keeps the scalar fallback path.

## Code

- Code checkpoint: `8dabe603f35efa49055eb5d75ecd6fbffb77c298`
- Changed file: `src/quant/grouped_pq_block/sve.rs`

## Evidence

- Existing failed CI job inspected, no rerun requested:
  - `pg18 / stable`: https://github.com/agent-ix/ecaz/actions/runs/27230037274/job/80407424509
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

## Out of Scope

- No manual CI rerun was started.
- No AWS instance, benchmark, or smoke test was started.
