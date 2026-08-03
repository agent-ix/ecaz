# Validation

Head SHA: `3fb1319af4dc0a1ebe1dc2c94138ad767fb05593`

Commands run with `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`, `CARGO_TARGET_DIR=target`, and Cargo offline mode:

- `cargo check --offline --all-targets --no-default-features --features pg18`
  - `Finished dev profile`
- `cargo test --offline --lib --no-default-features --features pg18 ec_distann::shard_build::tests`
  - 16 tests passed; 0 failed
- `cargo test --offline --lib --no-default-features --features pg18 ec_distann::head_sample::tests::partition_union`
  - 2 tests passed; 0 failed
- `cargo test --offline -p ecaz-cli commands::bench::suite::tests::distann_local_multinode`
  - 8 tests passed; 0 failed
- `git diff --check`
  - no output

Benchmark gate status: not run. The staged corpus directory inspected for the
required 10k/50k/100k matrix was empty.
