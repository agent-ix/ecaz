# Review Request: Task 36 SIMD Differential + Task 38 Fault Smoke Harness

## Summary

This checkpoint implements the Task 36 SIMD/scalar differential lane and the
operator-facing Task 38 fault smoke surface.

Task 36 adds scalar reference hooks for product-quantizer scoring and FWHT,
plus `tests/simd_diff.rs` coverage for dispatched-vs-scalar score paths, FWHT,
3-bit packing, and the production 1536/4-bit score path.

Task 38 adds `crates/ecaz-fault-injection`, `ecaz dev fault`, Makefile lanes,
and documentation for the PG-level fault matrix. Live provider-free lanes now
run AM-specific fixtures for `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire`
across cancellation, statement timeout, lock timeout, and resource pressure.
Those live lanes tag their sessions and assert postconditions for leftover
fault sessions, locks, and prepared transactions.

## Scope Boundary

Provider-backed Task 38 injection is intentionally not claimed complete here:
true EIO/ENOSPC, palloc-failure sweeps, and slow-disk latency injection still
need an LD_PRELOAD/FUSE/PG-test-hook provider. The CLI refuses non-dry-run
execution for those lanes instead of producing a false pass.

## Validation

Artifacts are under `artifacts/` and recorded in `artifacts/manifest.md`.

- `cargo fmt --all --check`
- `cargo test -p ecaz-fault-injection`
- `cargo test -p ecaz-cli`
- `cargo test --features bench --test simd_diff -- --test-threads=1`
- Live PG18 fault smoke against `ecaz_fault_probe_36_38`:
  - `ecaz dev fault smoke --lane cancel --rows 16`
  - `ecaz dev fault smoke --lane timeout --rows 16`
  - `ecaz dev fault smoke --lane lock-timeout --rows 16`
  - `ecaz dev fault smoke --lane resource --rows 16`

## Reviewer Focus

- SIMD tolerance choices and scalar reference isolation.
- Whether the Task 38 provider boundary is explicit enough for follow-up work.
- Whether the application-name based postcondition checks are the right minimum
  live leak checks for the built-in fault lanes.
