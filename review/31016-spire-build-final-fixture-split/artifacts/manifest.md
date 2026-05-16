# Artifact Manifest: 31016 SPIRE Build Final Fixture Split

Head SHA: `63d73a8c8f95c81a6bb973b691be57b94d18130f`
Packet/topic: `31016-spire-build-final-fixture-split`
Timestamp: `2026-05-13T17:31:48-07:00`
Lane: Phase 12b cleanup, build fixture relocation
Fixture: recursive fanout reloption guard, recursive fanout hierarchy build,
large top-graph chain-storage build
Storage format: unchanged existing SPIRE test fixtures
Rerank mode: not applicable
Surface isolation: not a measurement run; existing unit-test fixtures only

## Artifacts

### `cargo-fmt-check.log`

Command:

```sh
cargo fmt --check
```

Key result:

```text
Script done on 2026-05-13 17:26:57-07:00 [COMMAND_EXIT_CODE="0"]
```

Notes: stable rustfmt emitted the repository's existing unstable-option
warnings for `imports_granularity` and `group_imports`.

### `cargo-test-recursive-fanout-one-rejected.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_recursive_fanout_one_rejected -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_recursive_fanout_one_rejected - should panic ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 47.46s
```

### `cargo-test-recursive-fanout-build.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_recursive_fanout_build_hierarchy -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_recursive_fanout_build_hierarchy ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 85.07s
```

### `cargo-test-large-top-graph-chain-storage.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_large_top_graph_uses_chain_storage -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_large_top_graph_uses_chain_storage ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 67.17s
```

### `location-check.log`

Command:

```sh
rg -n 'fn test_ec_spire_recursive_fanout_one_rejected|fn test_ec_spire_recursive_fanout_build_hierarchy|fn test_ec_spire_large_top_graph_uses_chain_storage' src/tests/build.rs src/tests/mod.rs
```

Key result:

```text
src/tests/build.rs
762:    fn test_ec_spire_recursive_fanout_one_rejected() {
778:    fn test_ec_spire_recursive_fanout_build_hierarchy() {
979:    fn test_ec_spire_large_top_graph_uses_chain_storage() {
```

### `line-counts.log`

Command:

```sh
wc -l src/tests/mod.rs src/tests/build.rs src/lib.rs
```

Key result:

```text
  36197 src/tests/mod.rs
   1048 src/tests/build.rs
  17812 src/lib.rs
  55057 total
```

### `git diff --check`

Command:

```sh
git diff --check
```

Key result: clean; command exited with status 0 and no output.
