# Review Request: SPIRE AM Tuple-Path Dedupe Blockers

## Summary

This packet adds Stage D final AM tuple-path coverage for the remaining
dedupe/blocker cases before remote row materialization exists.

Code checkpoint: `695dcaca66b6d81cd6a68f64c9b348d05b694e7b`

The new Rust-side tests drive heap-resolution candidates through global/local
vec-id merge, production scan output construction, and AM delivery
classification. The tests prove:

- global vec IDs dedupe before AM delivery;
- a remote-origin global dedupe winner still blocks on
  `remote_row_materialization` instead of being returned as a coordinator TID;
- node-scoped local vec IDs remain distinct across nodes;
- stale remote locators preserve the `remote_heap_resolution` blocker and do
  not produce AM-deliverable outputs.

This does not implement remote row materialization. It narrows the final tuple
path invariant around the existing fail-closed boundary.

## Evidence

Artifacts are stored under `artifacts/`:

- `am-tuple-path-dedupe-blockers.log`
- `manifest.md`

Key result lines:

```text
running 4 tests
test am::ec_spire::tests::production_scan_am_tuple_path_keeps_node_scoped_local_vec_ids_distinct ... ok
test am::ec_spire::tests::production_scan_am_tuple_path_blocks_remote_global_dedupe_winner ... ok
test am::ec_spire::tests::production_scan_am_tuple_path_dedupes_global_vec_ids_before_delivery ... ok
test am::ec_spire::tests::production_scan_am_tuple_path_preserves_stale_locator_blocker ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1590 filtered out
```

## Validation

- `cargo fmt --check`
- `git diff --check -- src/am/ec_spire/root/tests.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo test production_scan_am_tuple_path --no-default-features --features pg18`
- `script -q -c "cargo test production_scan_am_tuple_path --no-default-features --features pg18" review/30796-spire-am-tuple-path-dedupe-blockers/artifacts/am-tuple-path-dedupe-blockers.log`

## Review Focus

- Whether these tests cover the intended final AM tuple-path edge cases from
  heap-candidate merge through AM delivery classification.
- Whether remote-origin winners and node-scoped duplicate-looking IDs remain
  fail-closed until materialization exists.
- Whether the task note scopes this as coverage rather than claiming remote row
  materialization implementation.
