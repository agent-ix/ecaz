# Artifacts Manifest

Packet: `30075-task28-ivf-quantizer-cache-audit`

Head SHA: `4e232f5210a4c3e862ad79860cc7f02b5be79f25`

Timestamp: `2026-04-27T18:55:54-07:00`

## Classification

This packet is a code/test audit for Task 28 A5. It does not make benchmark timing, recall, size, memory, or throughput claims, so there are no raw measurement logs in `artifacts/`.

## Validation Commands

- `cargo fmt --check`
- `cargo test --lib cached_quantizer_reuses_instances --no-default-features --features pg18`
- `cargo test --lib cached_with_presence_reports_whether_entry_already_existed --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_rescan_reuses_cached_prod_quantizer`
- `git diff --check`

## Key Result Lines

- `test result: ok. 1 passed; 0 failed` for `cached_quantizer_reuses_instances`
- `test result: ok. 1 passed; 0 failed` for `cached_with_presence_reports_whether_entry_already_existed`
- `test result: ok. 42 passed; 0 failed` for `am::ec_ivf`
- `test tests::pg_test_ec_ivf_rescan_reuses_cached_prod_quantizer ... ok`
- `test result: ok. 1 passed; 0 failed` for the PG18 pgrx regression
