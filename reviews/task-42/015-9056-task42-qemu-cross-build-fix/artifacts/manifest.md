# Artifact Manifest

- head SHA: `3b8e7a1d65c7259cb7b835948c18edbfb957e033`
- packet/topic: `9056-task42-qemu-cross-build-fix`
- timestamp: `2026-05-17T21:46:55Z`
- storage surface: qemu cross-arch decode lane setup
- rerank mode: not applicable

## Artifacts

### `make-endian-qemu-dry-run.log`

- lane: qemu cross-arch on-disk fixture decode command shape
- fixture: `tests/on_disk_fixtures.rs` fixture suite
- storage format: current on-disk fixture formats
- command used: `script -q -c "make -n endian-qemu" reviews/task-42/015-9056-task42-qemu-cross-build-fix/artifacts/make-endian-qemu-dry-run.log`
- key result lines:
  - `CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER="s390x-linux-gnu-gcc" \`
  - `CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_RUNNER="qemu-s390x -L /usr/s390x-linux-gnu" \`
  - `CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-Wl,--unresolved-symbols=ignore-all" \`
  - `cargo test --target s390x-unknown-linux-gnu --features bench --test on_disk_fixtures`

### `cargo-fmt-check.log`

- lane: formatting check
- fixture: not applicable
- storage format: source tree formatting
- command used: `script -q -c "cargo fmt --all -- --check" reviews/task-42/015-9056-task42-qemu-cross-build-fix/artifacts/cargo-fmt-check.log`
- key result lines:
  - `Script done on 2026-05-17 14:46:39-07:00 [COMMAND_EXIT_CODE="0"]`

### `git-diff-check.log`

- lane: whitespace check for the qemu cross-build setup patch
- fixture: not applicable
- storage format: source tree patch
- command used: `script -q -c "git diff --check HEAD^ HEAD" reviews/task-42/015-9056-task42-qemu-cross-build-fix/artifacts/git-diff-check.log`
- key result lines:
  - `Script done on 2026-05-17 14:46:39-07:00 [COMMAND_EXIT_CODE="0"]`

The prior pushed qemu CI run `26003525647` failed in the new
`On-disk fixtures under qemu s390x` job before this fix. The failure showed
the cross build inheriting host-specific Linux rustflags, with x86 CPU
features reported as unrecognized for the s390x target. This checkpoint
overrides the s390x target rustflags and provides `PGRX_PG_CONFIG_PATH`
from PGDG's PostgreSQL 18 development package for the pgrx build script.
