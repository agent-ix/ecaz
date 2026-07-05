# Task 70 / Packet 010: Frontier Profile Smoke Test

## Packet Scope

- Code commit: `1b4ae18aa12d996e37a135f69d3986554d0c869c`
- Review driver: `reviews/task-70/008-frontier-subtiming-profile/feedback/2026-05-31-001-reviewer.md`
- Manifest: `artifacts/manifest.md`

This packet requests review for a test-only follow-up to packet 008. The reviewer accepted packet 008 but noted the profile-aware scan path had no direct unit test coverage.

## Code Change

`src/am/ec_diskann/scan.rs` adds `sc_011b_scan_with_frontier_profile_records_counters`.

The test builds a small persisted chain graph and verifies the profile-aware scan path:

- returns the same results as the default `vamana_scan` path;
- still invokes the prefetch hook with the rerank-budget-sized batch;
- records positive frontier operation counters for candidate heap, visited set, neighbor slots, and retained inserts.

The test asserts operation counters rather than microsecond timers because packet 008 showed per-op timers can quantize to zero on small/fast fixtures.

No production behavior changes and no new `unsafe` were introduced.

## Validation

Commands and logs:

- `cargo fmt --check` -> `artifacts/cargo-fmt-check.log`
- `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` -> `artifacts/cargo-test-diskann-scan.log`
- `cargo check --all-targets --no-default-features --features pg18` -> `artifacts/cargo-check-pg18.log`

The focused scan module now passes 20 tests.
