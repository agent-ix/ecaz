# Artifact Manifest

- head SHA: `d29bfb7a73b036a80fe0a8cc4fefffbb20794625`
- task bucket: `reviews/task-91/019-careful-turboquant-mirror`
- timestamp: `2026-06-09T00:41:50-07:00`
- lane / fixture / storage format / rerank mode: local hardening/careful unit test; no SQL fixture; no index storage format; no rerank mode
- table surface: not applicable; Rust hardening harness unit test

## Artifacts

### `focused-careful-turboquant-test.log`

- command:

```bash
cargo test --manifest-path hardening/careful/Cargo.toml --target-dir target/llvm-cov-target --lib careful_diskann_build::tests::turboquant_build_params_use_direct_search_code_without_sidecar_flags
```

- key result:

```text
test careful_diskann_build::tests::turboquant_build_params_use_direct_search_code_without_sidecar_flags ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 621 filtered out
```

### `git-diff-check.log`

- command:

```bash
git diff --check
```

- key result: command exited 0 with no whitespace findings.
