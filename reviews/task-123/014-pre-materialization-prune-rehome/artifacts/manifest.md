# Manifest: Task 123 Pre-Materialization Prune Rehome

- Head SHA when prepared: `8bdbf7b0b`
- Task bucket: `reviews/task-123/014-pre-materialization-prune-rehome/`
- Date: 2026-06-28
- Lane: code validation only; no benchmark measurement in this packet
- Fixture / storage format / rerank mode: not applicable
- Isolation: focused unit diagnostics

## Code Under Review

- `a5d03abc1` - `Skip doomed SPIRE candidate materialization`
- `ea036b542` - `Prune SPIRE batched TQ materialization`
- `8bdbf7b0b` - `Gate SPIRE pre-materialization prune`

These commits were cherry-picked from draft PR #43 without the historical Task
122 packet artifacts.

## Artifacts

### `cargo-test-spire-diagnostics.log`

- Command:

```text
cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics -- --nocapture
```

- Key result lines:

```text
running 4 tests
test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_skips_degraded_unavailable_leaf ... ok
test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_reports_candidate_truncation ... ok
test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_reports_boundary_dedupe_and_winners ... ok
test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_counts_routed_store_rows ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2223 filtered out; finished in 0.06s
```
