# Validation

## Commands

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `rg -n "RegisterSnapshot|UnregisterSnapshot|RegisteredSnapshotGuard|snapshot_guard|GetTransactionSnapshot" src/am/ec_hnsw/build_parallel.rs src/storage/snapshot_guard.rs`
- `make fmt-check`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Key Results

- Unsafe baseline after the code slice: `3698` entries.
- Baseline movement for the code slice: `3701 -> 3698`.
- `bash scripts/unsafe_baseline_report.sh`: `src/am/ec_hnsw/build_parallel.rs`
  dropped from `214` to `211` entries.
- Targeted `rg` shows HNSW parallel build now stores
  `Option<RegisteredSnapshotGuard>` and calls
  `RegisteredSnapshotGuard::transaction`; raw snapshot register/unregister
  remains only inside `src/storage/snapshot_guard.rs`.
- `git diff --check`: clean.
- `bash scripts/check_unsafe_comments.sh`: clean.
- `make fmt-check`: passed; rustfmt emitted the existing stable-toolchain
  warnings for unstable `imports_granularity` and `group_imports` settings.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed; output included the existing PG18 C header unused-parameter warnings
  and the existing unused re-export warning in `src/am/mod.rs`.
