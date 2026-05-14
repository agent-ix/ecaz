# Review Request: SPIRE Build Final Fixture Split

## Summary

Packet 31016 closes the `tests/build.rs` cleanup row by moving the remaining
SPIRE build fixtures out of `src/tests/mod.rs` and into the concern file:

- `test_ec_spire_recursive_fanout_one_rejected`
- `test_ec_spire_recursive_fanout_build_hierarchy`
- `test_ec_spire_large_top_graph_uses_chain_storage`

`src/tests/mod.rs` now retains only the build concern include for this area.
The tracker records `tests/build.rs` as closed for Phase 12b cleanup.

Code checkpoint: `63d73a8c8f95c81a6bb973b691be57b94d18130f`

## Review Focus

- Confirm this is a mechanical fixture relocation with no test behavior
  changes.
- Confirm the moved fixtures remain under the `#[pg_schema] mod tests`
  namespace through the existing textual include.
- Confirm the task tracker correctly marks only `tests/build.rs` as closed.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_recursive_fanout_one_rejected -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_recursive_fanout_build_hierarchy -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_large_top_graph_uses_chain_storage -- --nocapture`
- `rg -n 'fn test_ec_spire_recursive_fanout_one_rejected|fn test_ec_spire_recursive_fanout_build_hierarchy|fn test_ec_spire_large_top_graph_uses_chain_storage' src/tests/build.rs src/tests/mod.rs`
- `wc -l src/tests/mod.rs src/tests/build.rs src/lib.rs`
- `git diff --check`

Artifacts and key result lines are recorded in
`review/31016-spire-build-final-fixture-split/artifacts/manifest.md`.
