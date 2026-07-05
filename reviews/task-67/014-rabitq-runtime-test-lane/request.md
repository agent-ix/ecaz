# Task 67 Review Request: RaBitQ Runtime Test Lane

## Summary

This packet restores the local plain-`cargo test` runtime lane needed to make
Task 67's functional RaBitQ checks executable outside PostgreSQL. The native
standalone PostgreSQL stub list now covers the backend symbols that the current
test binary needs to load. Backend execution symbols are deliberately
backend-only panic stubs, preserving the existing contract that plain Rust tests
must not fake SPI, heap, catalog, executor, or buffer behavior.

It also aligns `PreparedEstimator::estimate_ip_batch` with the already-landed
and reviewed bits=4 batch implementation, allowing bits=1, bits=4, and bits=8
batch scoring. This fixes the local `bits4_batch_estimator_matches_scalar_order`
runtime failure exposed once the test binary could execute.

## Code Under Review

- Commit: `59e899a41205825ef44f0925bea06701bc3353f3`
- Files:
  - `csrc/standalone_pg_backend_stubs.c`
  - `src/quant/rabitq.rs`

## Validation

See `artifacts/validation.log` and `artifacts/manifest.md`.

- `cargo fmt` passed.
- `cargo test -p ecaz task67_sum_query_dequant_for_test_scaffold_matches_scalar_when_available -- --nocapture` passed.
- `cargo test -p ecaz quant::rabitq -- --nocapture` passed: 46 tests.
- `git diff --check` passed.

This advances Task 67's functional gate with real local runtime evidence on the
available AVX2+FMA Intel host. AVX-512 runtime, recall, benchmark, and Slice J
throughput evidence remain pending on suitable hardware.
