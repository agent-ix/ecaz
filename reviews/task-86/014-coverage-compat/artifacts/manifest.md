# Task 86 Packet 014 Artifact Manifest

- Head SHA: `2817e3b3996a4042da77a71f372d4db7dabb716b`
- Task bucket: `reviews/task-86/014-coverage-compat`
- Timestamp: `2026-06-07T22:59:40Z`
- Scope: Coverage/hardening compile compatibility after the Task 86 TQ+ storage-format addition.

## Artifacts

### `careful-coverage-lib.log`

- Command:

  ```sh
  cargo test --manifest-path hardening/careful/Cargo.toml --target-dir target/llvm-cov-target --lib > reviews/task-86/014-coverage-compat/artifacts/careful-coverage-lib.log 2>&1
  ```

- Lane / fixture / storage format / rerank mode: hardening/careful lib coverage compile repro; no benchmark fixture; no runtime storage format; no rerank mode.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result cited by request:

  ```text
  test result: ok. 603 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.55s
  ```

### `careful-coverage-lib-rustflags.log`

- Command:

  ```sh
  RUSTFLAGS="-C instrument-coverage --cfg=coverage" cargo test --manifest-path hardening/careful/Cargo.toml --target-dir target/llvm-cov-target --lib > reviews/task-86/014-coverage-compat/artifacts/careful-coverage-lib-rustflags.log 2>&1
  ```

- Lane / fixture / storage format / rerank mode: hardening/careful lib coverage-mode compile repro; no benchmark fixture; no runtime storage format; no rerank mode.
- Isolated one-index-per-table or shared-table surface: not applicable.
- Result cited by request:

  ```text
  test result: ok. 603 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 6.49s
  ```

## CI Failure Mapping

GitHub `Test Quality Coverage` failed compiling `ecaz-careful-hardening` with:

- missing `SPIRE_PAYLOAD_FORMAT_RABITQ` import in the careful SPIRE storage-test include scope;
- missing `SpireLeafBlockSummary` import in that same scope;
- missing non-pg `StorageFormat::TurboQuantTqPlus` variant in `src/am/ec_ivf/page.rs`.

Packet 014 fixes those compile-surface mismatches and validates the careful hardening lib target locally, including the coverage cfg mode.
