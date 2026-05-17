# Manifest: 458-c1-ecvector-extension-upgrade-path

- Head SHA at validation time: `913c907`
- Packet: `458-c1-ecvector-extension-upgrade-path`
- Scope: extension upgrade path for `ecvector` SQL objects
- Validation commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Validation date: `2026-04-19`
- Measurement status: no new measurement claim in this packet
- Notes:
  - This packet fixes extension upgrade semantics for existing databases.
  - It was discovered while preparing a stable cached source-build measurement
    surface on the long-lived scratch cluster.
