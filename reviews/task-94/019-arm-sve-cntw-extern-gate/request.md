# Task 94 Packet 019: ARM SVE `cntw` Extern Gate

## Summary

This packet fixes the existing PR failures on aarch64 caused by the test-only
SVE vector-lane helper leaving its `ecaz_grouped_pq_sve_cntw` extern declaration
visible in non-test builds.

The production SVE accumulator extern remains available for the SVE block
kernel. Only the `cntw` extern declaration is gated to
`#[cfg(all(test, target_arch = "aarch64"))]`, matching its only Rust caller.

## Code

- Code checkpoint: `7631fb8cb854c6f8c94d7f71a10b957986478ffa`
- Changed file: `src/quant/grouped_pq_block/sve.rs`

## Evidence

- Existing failed CI jobs inspected, no rerun requested:
  - `pg18 / stable / compile`: https://github.com/agent-ix/ecaz/actions/runs/27228790771/job/80403035467
  - `pg18 / stable`: https://github.com/agent-ix/ecaz/actions/runs/27228791172/job/80403036447
- Failure source: `artifacts/ci-failure-source.md`
- Local clippy: `artifacts/cargo-clippy-pg18-bench.log`
  - `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings`
  - Passed
- Local grouped-PQ tests: `artifacts/cargo-test-grouped-pq-lib.log`
  - `cargo test grouped_pq --lib`
  - Passed: 35 passed, 0 failed
- Local format check: `artifacts/cargo-fmt-check.log`
  - `cargo fmt --check`
  - Passed

## Out of Scope

- No CI rerun was started.
- No AWS instance, benchmark, or smoke test was started.
