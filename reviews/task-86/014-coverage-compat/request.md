# Task 86 Packet 014: Coverage Compatibility

## Summary

This packet addresses the GitHub `Test Quality Coverage` failure observed after packet 013.

Changes:

- `src/am/ec_ivf/page.rs`: add `StorageFormat::TurboQuantTqPlus = 4` to the non-pg hardening enum so the decode tests compile outside `pg17`/`pg18` builds.
- `hardening/careful/src/spire.rs`: import existing SPIRE RaBitQ/block-summary symbols into the careful storage-test include scope.

This does not change the TQ+ benchmark results or production scoring behavior. It aligns the hardening/coverage compile surface with the storage-format and SPIRE storage symbols already used by included tests.

## Validation

Artifact manifest:

- `reviews/task-86/014-coverage-compat/artifacts/manifest.md`

Validation logs:

- `reviews/task-86/014-coverage-compat/artifacts/careful-coverage-lib.log`
- `reviews/task-86/014-coverage-compat/artifacts/careful-coverage-lib-rustflags.log`

Commands:

```sh
cargo test --manifest-path hardening/careful/Cargo.toml --target-dir target/llvm-cov-target --lib
RUSTFLAGS="-C instrument-coverage --cfg=coverage" cargo test --manifest-path hardening/careful/Cargo.toml --target-dir target/llvm-cov-target --lib
```

Results:

```text
test result: ok. 603 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.55s
test result: ok. 603 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 6.49s
```

## Review Focus

- Confirm the non-pg IVF enum mirrors the pg storage-format tag set for TQ+.
- Confirm the careful SPIRE imports are limited to symbols already referenced by included storage tests.
