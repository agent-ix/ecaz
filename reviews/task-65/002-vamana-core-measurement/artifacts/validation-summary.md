# Task 65 Validation Summary

Head: `8e860324c1fa2a009bab209502962375f0207642`

## Commands

### `cargo fmt --check`

Result: passed.

Repeated after adding `dhat_vamana_build`.

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

### `cargo check --features bench,dhat-heap --bin dhat_vamana_build`

Result: passed.

Key output:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.36s
```

### `cargo test -p ecaz --features pg18 ec_diskann`

Result: passed.

Key output:

```text
running 182 tests
...
test am::ec_diskann::routine::tests::pg_test_ec_diskann_build_keeps_duplicate_vectors_as_distinct_nodes ... ok
...
test result: ok. 182 passed; 0 failed; 0 ignored; 0 measured; 1735 filtered out; finished in 70.42s
...
process exited with code 0
```

This also verifies the macOS standalone-loader fix: the command reached and
ran the DiskANN tests instead of aborting before test execution with the prior
`_BufferBlocks` dyld error.

This run was repeated after `de2ef72e4` so the test evidence matches the
committed Vamana hot-path cleanup rather than a dirty worktree.

### `/Users/peter/.cargo/bin/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-65/002-vamana-core-measurement/artifacts/install-ecaz-pg-test-after-hotpath-trim.log`

Result: passed.

Key output:

```text
[install] installed_backend=/opt/homebrew/lib/postgresql@18/ecaz.dylib
[install] sha256=b36dac9a7f8900dc38fa398bc3bcbb1080341baf76394401b25bfa476ca3c1c1
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

### `cargo run -p ecaz-cli --bin ecaz -- ... corpus load --prefix task65_real_l200 ...`

Result: passed.

Key output:

```text
[loader] copied corpus table task65_real_l200_corpus in 5.67s
[loader] encoded corpus table task65_real_l200_corpus in 1.78s
[loader] built task65_real_l200_pq_fastscan_idx in 14.92s
[loader] completed prefix task65_real_l200 in 32.10s
```

### `cargo run -p ecaz-cli --bin ecaz -- ... bench recall --prefix task65_real_l200 ...`

Result: passed.

Key output:

```text
list_size=64  recall@k=0.9975  mean q-time=0.72 ms
list_size=128 recall@k=0.9975  mean q-time=0.82 ms
list_size=200 recall@k=0.9975  mean q-time=0.96 ms
```

### `cargo run -p ecaz-cli --bin ecaz -- ... corpus load --prefix task65_syn_l200 ...`

Result: passed.

Key output:

```text
[loader] copied corpus table task65_syn_l200_corpus in 4.90s
[loader] encoded corpus table task65_syn_l200_corpus in 2.15s
[loader] built task65_syn_l200_pq_fastscan_idx in 35.67s
[loader] completed prefix task65_syn_l200 in 50.38s
```

### `cargo run -p ecaz-cli --bin ecaz -- ... bench recall --prefix task65_syn_l200 ...`

Result: passed.

Key output:

```text
list_size=64  recall@k=0.1610  mean q-time=1.09 ms
list_size=200 recall@k=0.2625  mean q-time=1.75 ms
list_size=800 recall@k=0.3270  mean q-time=3.21 ms
```

### `cargo run --release --features bench,dhat-heap --bin dhat_vamana_build -- ...`

Result: passed on the first 1,000 rows from the real10k m5 fixture.

Key output is in `dhat-vamana-build-real1k-r32-l200-summary.md`:

```text
rows=1000
graph_degree=32
build_list_size=200
elapsed_ms=14050
greedy_search_ms=924
robust_prune_ms=1880
backlink_ms=9735
dhat_output=reviews/task-65/002-vamana-core-measurement/artifacts/dhat-vamana-build-real1k-r32-l200.json
```

## Notes

`cargo pgrx test pg18 ec_diskann` was previously blocked in this sandbox by
Homebrew extension-directory write permissions when run directly. The direct
`cargo test -p ecaz --features pg18 ec_diskann` lane did perform the pg_test
extension build/install and passed, so this packet treats the dyld issue as
fixed and the old direct-pgrx failure as an environment/sandbox lane issue.
