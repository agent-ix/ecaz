# Artifacts Manifest

Packet: `30079-task28-ivf-streaming-vacuum`

Head SHA: `7b4e23281acdc9a6a38402a0d45ced2b4e7ca8b9`

Timestamp: `2026-04-27T19:54:34-07:00`

Lane: Task 28 A2 streaming IVF vacuum code checkpoint.

## Classification

This is a code/test packet. It makes no wall-time, peak-memory, index-size, recall, or latency measurement claims, so there are no raw benchmark logs in `artifacts/`.

## Validation Commands

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::page::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_vacuum`
- `git diff --check`

## Key Result Lines

- `test result: ok. 11 passed; 0 failed` for `am::ec_ivf::page::tests`
- `test result: ok. 42 passed; 0 failed` for `am::ec_ivf`
- `test tests::pg_test_ec_ivf_vacuum_repeated_bulkdelete_is_idempotent ... ok`
- `test tests::pg_test_ec_ivf_vacuum_bulkdelete_removes_dead_heap_tid ... ok`
- `test tests::pg_test_ec_ivf_vacuum_callbacks_keep_live_count_noop ... ok`
- `test tests::pg_test_ec_ivf_vacuum_repairs_empty_list_directory_refs ... ok`
