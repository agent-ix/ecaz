# Task 50 Packet 009 Artifact Manifest

- Head SHA: `5ede3fe2bb34abc03d1920bedb8c6464722ff74a`
- Base SHA: `33936fb6d207a9ffda032132e74f47bc377e0101`
- Task bucket: `reviews/task-50/009-spire-hierarchy-anchor-rollout`
- Lane: SPIRE hierarchy/read-path active-anchor rollout.
- Fixture/storage/rerank: not applicable; structural coordinator read-path refactor only.
- Table surface: not applicable; no benchmark suite/table load was run.
- Timestamp: `2026-05-19T23:18:35-07:00`

| Artifact | Command | Key result |
| --- | --- | --- |
| `block-count-before.log` | `git show 33936fb6d207a9ffda032132e74f47bc377e0101:src/am/ec_spire/coordinator/hierarchy_snapshots.rs \| rg -c 'unsafe\s*\{'` | `71` direct unsafe blocks before this slice. |
| `block-count-after.log` | `make unsafe-block-count PATHS='src/am/ec_spire/coordinator/hierarchy_snapshots.rs'` | `48 src/am/ec_spire/coordinator/hierarchy_snapshots.rs`; net `-23`, a 32.4% file reduction. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Passed. Existing warnings remain in `src/am/common/parallel.rs` and `src/am/mod.rs`. |
| `rustfmt-touched-check.log` | `rustfmt --check src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | Passed; stable rustfmt warned that two unstable rustfmt config keys are ignored. |
| `git-diff-check.log` | `git diff --check` | Passed. |
| `cargo-fmt-check.log` | `cargo fmt --all --check` | Failed on existing repo-wide formatting drift outside the touched file. |
| `cargo-clippy-pg18.log` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | Failed on existing repo-wide lint backlog. No diagnostics target `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`. |
| `cargo-test-spire-coordinator-filter.log` | `cargo test --lib --no-default-features --features pg18,bench am::ec_spire::coordinator:: -- --nocapture` | Built the test binary, then failed at process start with `undefined symbol: LockBuffer`; retained as evidence that this coordinator filter cannot run in the non-PostgreSQL unit-test process. |
