# Review Request: SPIRE Custom Scan BeginExec Setup

## Summary

This slice reduces repeated unsafe setup in SPIRE custom scan BeginCustomScan handling.

The change:

- consolidates vector-order-limit tuple payload and query extraction into one BeginCustomScan boundary,
- merges the three DML custom-scan modes into one shared DML initialization arm, and
- inlines the single-use DML tuple-payload initializer into the existing DML setup boundary.

That removes repeated unsafe call sites while keeping all plan-private metadata copied into Rust-owned executor state before the callback returns.

## Unsafe Burn-Down

- `rg -n "unsafe" src | wc -l`: `2548 -> 2544`
- `rg -n "unsafe" src/am/ec_spire/custom_scan/begin_exec.rs | wc -l`: `30 -> 26`
- `rg -n "unsafe fn" src/am/ec_spire/custom_scan/begin_exec.rs | wc -l`: `4 -> 3`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/custom_scan/begin_exec.rs` passed with the existing stable-channel import-grouping warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the existing `src/am/mod.rs` unused-import warning.
- `artifacts/cargo-test-custom-scan-pg18-pgtest-no-run.log`: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run` passed with the existing Hadamard test-helper dead-code warnings.

