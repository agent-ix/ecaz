# Manifest: 449-c1-native-build-harness-fix-and-hnsw-rs-removal

- Head SHA at validation time: `8ac9bda`
- Packet: `449-c1-native-build-harness-fix-and-hnsw-rs-removal`
- Scope: external-summary harness fix plus crate-level `hnsw_rs` removal
- Validation commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `rg -n "hnsw_rs|probe_hnsw_rs|test_hnsw_rs" src/lib.rs Cargo.toml`
- Validation date: `2026-04-19`
- Measurement status: no new measurement claim in this packet
- Notes:
  - `vendor/hnsw_rs/` remains on disk by task constraint, but the crate does not
    reference it anymore.
  - This packet closes the packet-448 WIP harness fix and the packet-446
    dependency cleanup follow-up without changing persisted index layout.
