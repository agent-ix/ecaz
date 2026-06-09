# Artifact Manifest

- Head SHA: `e79f41ac6`
- Task bucket: `reviews/task-91/011-diskann-build-binding-rename`
- Timestamp: `2026-06-09T05:47:15Z`
- Lane / fixture / storage format / rerank mode: DiskANN unit and pg18 routine validation; naming-only build binding cleanup
- Shared-table vs isolated one-index-per-table surface: pg18 routine tests use their built-in SQL fixtures; quantizer tests are in-process unit fixtures

## Commands

### DiskANN quantizer unit tests

Command:

```sh
cargo test --lib am::ec_diskann::quantizer::tests --no-default-features --features pg18
```

Key result lines:

```text
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2022 filtered out
```

### DiskANN routine tests

Command:

```sh
cargo test --lib am::ec_diskann::routine::tests --no-default-features --features pg18
```

Key result lines:

```text
running 24 tests
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 2004 filtered out
```

### Whitespace check

Command:

```sh
git diff --check
```

Key result lines:

```text
passed with no output
```
