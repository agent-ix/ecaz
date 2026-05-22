# Current-Head PG18 RaBitQ / IVF / SPIRE Sweep Prep

## Scope

This packet records the merge-validation cleanup and bench handoff state after
the upstream merge and the SPIRE wrapper checkpoints.

Code cleanup committed in `e21d0dd42`:

- Mechanical `cargo fmt --all` cleanup in `crates/ecaz-cli`,
  `hardening/careful`, and `src/quant/simd.rs`.
- Test-only SPIRE DML re-export cfg cleanup in `src/am/mod.rs` and
  `src/am/ec_spire/mod.rs`, so normal PG18 bench builds no longer emit the
  unused-import warning.

## Validation Summary

- `cargo fmt --all -- --check`: passed after cleanup.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed cleanly after cleanup.
- `cargo test --no-run --all-targets --no-default-features --features pg18,bench`:
  passed after cleanup.
- `cargo build -p ecaz-cli --bin ecaz`: passed; current CLI is built.
- `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings`:
  failed on broad existing lint backlog; captured as evidence, not cleared in
  this merge prep slice.

## Bench Handoff

Benchmarks were not started in this packet. The local PG18 scratch cluster was
already running on `localhost:28818`, database `tqvector_bench`, and the suite
config for the next agent is:

`artifacts/rabitq-ivf-spire-local-suite.json`

## Artifacts

See `artifacts/manifest.md`.
