# Review Request: Task 36 SIMD Differential + Task 38 Fault Smoke Harness

## Summary

This checkpoint extends the Task 36 SIMD/scalar differential lane and the
operator-facing Task 38 fault smoke surface after review feedback found that
the first pass overstated coverage.

Task 36 now has scalar reference hooks for product-quantizer scoring and FWHT,
plus test-only AVX2/FMA and NEON entry points where the backend exists.
`tests/simd_diff.rs` covers dispatched-vs-scalar score paths, forced
host-backend score/FWHT paths, deterministic 3-bit and 4-bit width sampling,
pack/unpack roundtrips across 2..=8 bits, and the production 1536/4-bit score
path.

Task 38 adds `crates/ecaz-fault-injection`, an LD_PRELOAD provider for matched
EIO/ENOSPC/slow-disk injection, `ecaz dev fault`, Makefile lanes, and
documentation for the PG-level fault matrix. The provider now hooks `open`,
`open64`, `openat`, `openat2`, read/pread, write/pwrite, fsync, and fdatasync
surfaces. Live lanes run AM-specific fixtures for `ec_hnsw`, `ec_ivf`,
`ec_diskann`, and `ec_spire`: cancellation and statement timeout use repeated
AM KNN scans, lock timeout uses `REINDEX INDEX CONCURRENTLY` while the table is
locked, and provider-backed slow-disk latency runs against a postmaster
restarted through `ecaz dev fault provider-restart`. Those live lanes tag their
sessions and assert postconditions for leftover fault sessions, locks, and
prepared transactions.

## Scope Boundary

Task 38 is still scope-bounded: the LD_PRELOAD provider self-tests matched EIO
and ENOSPC and the provider-backed PG18 slow-disk lane passes, but PG18
EIO/ENOSPC AM sweeps still need mode-specific postmaster orchestration.
palloc-failure sweeps still need a palloc-aware PG test hook or extension-side
injection point. The CLI refuses unsupported non-dry-run lanes instead of
producing a false pass.

Task 36 still does not include the optional negative-control mutation artifact
or AM scan-accumulator differential coverage. The current packet closes the
reviewed backend-pinning and width-coverage gaps for the quantizer/FWHT lane.

## Validation

Artifacts are under `artifacts/` and recorded in `artifacts/manifest.md`.

- `cargo fmt --all --check`
- `cargo test -p ecaz-fault-injection`
- `cargo test -p ecaz-cli cli_parses_fault`
- `cargo test --features bench --test simd_diff -- --test-threads=1`
- Live PG18 fault smoke against `ecaz_fault_probe_36_38`:
  - `ecaz dev fault smoke --lane cancel --rows 64`
  - `ecaz dev fault smoke --lane timeout --rows 64`
  - `ecaz dev fault smoke --lane lock-timeout --rows 64`
  - `ecaz dev fault provider-restart --mode slow-disk ...`
  - `ecaz dev fault smoke --lane slow-disk --rows 64 --provider-marker ...`
  - `ecaz dev fault provider-restore`

## Reviewer Focus

- SIMD tolerance choices and scalar reference isolation.
- Whether the forced backend wrappers are narrow enough for bench/test use.
- Whether the remaining Task 38 provider boundary is explicit enough for
  follow-up work.
- Whether the application-name based postcondition checks are the right minimum
  live leak checks for the built-in fault lanes.
