# Current Focused DiskANN Test Rerun

Head: `8e355577f45d57ab8b573a421f5b62f87863d92a5`

Command:

```text
cargo test -p ecaz --features pg18 ec_diskann
```

Result: passed on 2026-05-28.

Key output:

```text
running 182 tests
...
test am::ec_diskann::routine::tests::pg_test_ec_diskann_build_keeps_duplicate_vectors_as_distinct_nodes ... ok
...
test result: ok. 182 passed; 0 failed; 0 ignored; 0 measured; 1735 filtered out; finished in 90.68s
...
process exited with code 0
```

The rerun completed the pgrx extension build/install phase and did not hit the
previous macOS `_BufferBlocks` dyld failure.
