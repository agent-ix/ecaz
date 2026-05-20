# Task 50 Packet 008 Artifact Manifest

- Head SHA: `3e089ada3276c64d1bd8bcf221fcbb583ac759de`
- Base SHA: `b699cc4ec240bb17ff8b9e32f08ef56239c71f2c`
- Task bucket: `reviews/task-50/008-spire-active-anchor-seed`
- Lane: SPIRE active-epoch anchor seed.
- Fixture/storage/rerank: not applicable; structural diagnostic-path refactor only.
- Table surface: not applicable; no benchmark suite/table load was run.
- Timestamp: `2026-05-19T23:10:31-07:00`

| Artifact | Command | Key result |
| --- | --- | --- |
| `block-count-before.log` | `git show b699cc4ec240bb17ff8b9e32f08ef56239c71f2c:src/am/ec_spire/coordinator/snapshots.rs \| rg -c 'unsafe\s*\{'` | `62` direct unsafe blocks before this slice. |
| `block-count-after.log` | `make unsafe-block-count PATHS='src/am/ec_spire/coordinator/snapshots.rs'` | `52 src/am/ec_spire/coordinator/snapshots.rs`; net `-10`, a 16.1% file reduction for this seed slice. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Passed. Existing warnings remain in `src/am/common/parallel.rs` and `src/am/mod.rs`. |
| `rustfmt-touched-check.log` | `rustfmt --check src/am/ec_spire/coordinator/snapshots.rs` | Passed; stable rustfmt warned that two unstable rustfmt config keys are ignored. |
| `git-diff-check.log` | `git diff --check` | Passed. |
| `cargo-fmt-check.log` | `cargo fmt --all --check` | Failed on existing repo-wide formatting drift outside the touched file. |
| `cargo-clippy-pg18.log` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | Failed on existing repo-wide lint backlog. No diagnostics target `src/am/ec_spire/coordinator/snapshots.rs` after the local type-alias cleanup. |
| `cargo-test-scan-local-heap-gate.log` | `cargo test --lib --no-default-features --features pg18,bench am::ec_spire::scan::tests::runtime_state::local_heap_delivery_gate -- --nocapture` | Built the test binary, then failed at process start with `undefined symbol: LockBuffer`; retained as evidence that the narrow non-PG unit invocation is not runnable outside PostgreSQL linkage. |
