# Artifact Manifest: SPIRE AM Tuple-Path Dedupe Blockers

Head SHA: `695dcaca66b6d81cd6a68f64c9b348d05b694e7b`
Packet: `30796-spire-am-tuple-path-dedupe-blockers`
Timestamp: `2026-05-10`

## Fixture

- Lane: SPIRE Stage D final AM tuple-path coverage
- Fixture: standalone Rust test target
- Storage format: synthetic heap-resolution candidates using SPIRE vec-id
  encodings
- Rerank mode: production heap-candidate merge into AM delivery classifier
- Surface: Rust-side final tuple-path helpers, no PostgreSQL server
- Command:

```text
script -q -c "cargo test production_scan_am_tuple_path --no-default-features --features pg18" review/30796-spire-am-tuple-path-dedupe-blockers/artifacts/am-tuple-path-dedupe-blockers.log
```

## Artifacts

### `am-tuple-path-dedupe-blockers.log`

Raw focused standalone Rust test log.

Key result lines:

```text
running 4 tests
test am::ec_spire::tests::production_scan_am_tuple_path_keeps_node_scoped_local_vec_ids_distinct ... ok
test am::ec_spire::tests::production_scan_am_tuple_path_blocks_remote_global_dedupe_winner ... ok
test am::ec_spire::tests::production_scan_am_tuple_path_dedupes_global_vec_ids_before_delivery ... ok
test am::ec_spire::tests::production_scan_am_tuple_path_preserves_stale_locator_blocker ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1590 filtered out; finished in 0.00s
```
