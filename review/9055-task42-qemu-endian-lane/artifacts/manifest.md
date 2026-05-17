# Artifact Manifest

- head SHA: `2e2c77caa2ebc3de4b2e43203472321e9d641ba8`
- packet/topic: `9055-task42-qemu-endian-lane`
- timestamp: `2026-05-17T21:41:23Z`
- storage surface: qemu cross-arch decode lane plus existing Task 42 fixture and matrix checks
- rerank mode: not applicable

## Artifacts

### `make-endian-qemu-dry-run.log`

- lane: qemu cross-arch on-disk fixture decode command shape
- fixture: `tests/on_disk_fixtures.rs` fixture suite
- storage format: current on-disk fixture formats
- command used: `script -q -c "make -n endian-qemu" review/9055-task42-qemu-endian-lane/artifacts/make-endian-qemu-dry-run.log`
- key result lines:
  - `CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER="s390x-linux-gnu-gcc" \`
  - `CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_RUNNER="qemu-s390x -L /usr/s390x-linux-gnu" \`
  - `cargo test --target s390x-unknown-linux-gnu --features bench --test on_disk_fixtures`

### `cargo-fmt-check.log`

- lane: formatting check
- fixture: not applicable
- storage format: source tree formatting
- command used: `script -q -c "cargo fmt --all -- --check" review/9055-task42-qemu-endian-lane/artifacts/cargo-fmt-check.log`
- key result lines:
  - `Script done on 2026-05-17 14:40:06-07:00 [COMMAND_EXIT_CODE="0"]`

### `make-on-disk-fixtures.log`

- lane: on-disk golden fixture decode checks
- fixture: HNSW/DiskANN/IVF/SPIRE fixture suite
- storage format: current on-disk fixture formats
- command used: `script -q -c "make on-disk-fixtures" review/9055-task42-qemu-endian-lane/artifacts/make-on-disk-fixtures.log`
- key result lines:
  - `running 45 tests`
  - `test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

### `make-upgrade-smoke.log`

- lane: on-disk format-version compatibility matrix smoke
- fixture: `fixtures/upgrade/matrix.csv` and referenced on-disk fixtures
- storage format: current HNSW, DiskANN, IVF, and SPIRE partition object format tags
- command used: `script -q -c "make upgrade-smoke" review/9055-task42-qemu-endian-lane/artifacts/make-upgrade-smoke.log`
- key result lines:
  - `running 2 tests`
  - `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

### `make-layout-check.log`

- lane: static size/layout assertion check
- fixture: not applicable
- storage format: static layout assertions under `tests/size_of_assertions.rs`
- command used: `script -q -c "make layout-check" review/9055-task42-qemu-endian-lane/artifacts/make-layout-check.log`
- key result lines:
  - `running 13 tests`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

The executable qemu lane is wired into GitHub Actions for scheduled,
manual, and push-to-main runs. It was not executed locally because this host
does not have the `s390x-unknown-linux-gnu` Rust target, qemu user emulator,
or s390x sysroot installed. The fixture, matrix, and layout logs emit the
pre-existing `src/am/mod.rs` unused-import warning for SPIRE DML frontdoor
exports.
