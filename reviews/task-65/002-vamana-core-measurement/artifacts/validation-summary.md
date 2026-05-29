# Task 65 Validation Summary

Head: `a8b0b87893a7868023b0ef49cbb00cc9225a7ac8`

## Commands

### `cargo fmt --check`

Result: passed.

### `cargo check -p ecaz --lib --no-default-features --features pg18`

Result: passed.

Key output:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 21s
```

### `cargo check -p ecaz-cli`

Result: passed.

Key output:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 8m 12s
```

### `cargo check -p ecaz-cli --bin ecaz`

Result: passed.

Key output:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 16s
```

### `cargo test -p ecaz --features pg18 ec_diskann`

Result: passed.

Key output:

```text
running 182 tests
...
test am::ec_diskann::routine::tests::pg_test_ec_diskann_build_keeps_duplicate_vectors_as_distinct_nodes ... ok
...
test result: ok. 182 passed; 0 failed; 0 ignored; 0 measured; 1735 filtered out; finished in 87.33s
...
process exited with code 0
```

This also verifies the macOS standalone-loader fix: the command reached and
ran the DiskANN tests instead of aborting before test execution with the prior
`_BufferBlocks` dyld error.

### `/Users/peter/.cargo/bin/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/install-ecaz-pg-test-after-loader-fix.log`

Result: passed.

Key output:

```text
[install] installed_backend=/opt/homebrew/lib/postgresql@18/ecaz.dylib
[install] sha256=fbe83817a4e22b919c98f89ccb9e207ca33ff76c5699b0ee8e1a6ebd2f952f05
```

### `cargo run -p ecaz-cli --bin ecaz -- ... corpus load --prefix task65_lfix_r10k ...`

Result: passed after one rejected too-long prefix attempt.

Key output:

```text
[loader] copied corpus table task65_lfix_r10k_corpus in 5.73s
[loader] encoding ecvector embeddings via encode_to_ecvector(source, 4, 42) ...
[loader] encoded corpus table task65_lfix_r10k_corpus in 1.84s
[loader] built task65_lfix_r10k_pq_fastscan_idx in 7.62s
[loader] completed prefix task65_lfix_r10k in 24.95s
```

The first loaderfix run used prefix
`task65_real10k_diskann_pq_rel_r32_l100_loaderfix`; it proved the new
stage-and-insert loader path but failed before index build because the
generated index identifier exceeded PostgreSQL's 63-byte limit.

### `cargo run -p ecaz-cli --bin ecaz -- ... bench recall --prefix task65_lfix_r10k ...`

Result: passed.

Key output:

```text
list_size=64  recall@k=0.9965  mean q-time=0.69 ms
list_size=128 recall@k=0.9970  mean q-time=0.78 ms
list_size=200 recall@k=0.9975  mean q-time=0.91 ms
```

## Notes

`cargo pgrx test pg18 ec_diskann` was previously blocked in this sandbox by
Homebrew extension-directory write permissions when run directly. The direct
`cargo test -p ecaz --features pg18 ec_diskann` lane did perform the pg_test
extension build/install and passed, so this packet treats the dyld issue as
fixed and the old direct-pgrx failure as an environment/sandbox lane issue.
