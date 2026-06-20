# Focused PG18 Test Log

Command:

```sh
cargo pgrx test pg18 test_ec_ivf_index_placement_update_snapshot_payload
```

Result:

```text
test tests::pg_test_ec_ivf_index_placement_update_snapshot_payload ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2206 filtered out; finished in 86.25s
```

Notes:

- This direct rerun was used after the packet-local `script` wrapper captured
  the initial compile failure from the too-long test identifier.
- The initial failed attempt is retained in
  `cargo-pgrx-test-pg18-update-snapshot.log`.
