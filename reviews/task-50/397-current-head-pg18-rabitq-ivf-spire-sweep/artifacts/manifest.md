# Manifest: Current-Head PG18 RaBitQ / IVF / SPIRE Sweep Prep

- Task bucket: `reviews/task-50`
- Packet: `reviews/task-50/397-current-head-pg18-rabitq-ivf-spire-sweep`
- Code cleanup commit: `e21d0dd42`
- Branch: `task-50-unsafe-closeout`
- Timestamp: `2026-05-21T21:48:25-07:00`
- Primary target: PG18
- Local scratch connection for follow-on benches: `localhost:28818`, database
  `tqvector_bench`

## Commands And Evidence

- `cargo-fmt-check.log`
  - Command: `cargo fmt --all -- --check`
  - Result: failed before cleanup due repo-wide formatting drift.
- `cargo-fmt-apply.log`
  - Command: `cargo fmt --all`
  - Result: applied mechanical formatting.
- `cargo-fmt-check-clean-final.log`
  - Command: `cargo fmt --all -- --check`
  - Result: passed with stable-rustfmt warnings about nightly-only import
    grouping options.
- `cargo-check-all-targets-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed before final warning cleanup, with unused SPIRE DML re-export
    warnings.
- `cargo-check-all-targets-pg18-bench-clean-final.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed cleanly.
- `cargo-test-no-run-all-targets-pg18-bench.log`
  - Command: `cargo test --no-run --all-targets --no-default-features --features pg18,bench`
  - Result: passed before final cleanup.
- `cargo-test-no-run-all-targets-pg18-bench-clean-final.log`
  - Command: `cargo test --no-run --all-targets --no-default-features --features pg18,bench`
  - Result: passed after cleanup.
- `cargo-build-ecaz-cli.log`
  - Command: `cargo build -p ecaz-cli --bin ecaz`
  - Result: passed.
- `cargo-clippy-all-targets-pg18-bench-d-warnings.log`
  - Command:
    `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings`
  - Result: failed with broad existing lint backlog; this was not resolved in
    the merge prep cleanup.
- `rabitq-ivf-spire-local-suite.json`
  - Follow-on local suite config for IVF/RaBitQ and SPIRE/RaBitQ benches.

## Bench Status

Benchmarks have not been run from this packet yet.
