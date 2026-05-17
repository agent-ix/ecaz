# Review Request: Task 36 SIMD Differential + Task 38 Fault Smoke Harness

## Summary

This checkpoint extends the Task 36 SIMD/scalar differential lane and the
operator-facing Task 38 fault smoke surface after review feedback found that
the first pass overstated coverage.

Task 36 now has scalar reference hooks for product-quantizer scoring and FWHT,
AM source inner-product hooks for HNSW/DiskANN, plus test-only AVX2/FMA and
NEON entry points where the backend exists. `tests/simd_diff.rs` covers
dispatched-vs-scalar score paths, forced host-backend score/FWHT paths,
deterministic 3-bit and 4-bit width sampling, pack/unpack roundtrips across
2..=8 bits, AM source inner-product SIMD, and the production 1536/4-bit score
path. The CI workflow now runs a focused `simd-diff` matrix on `ubuntu-24.04`
x64 and `ubuntu-24.04-arm` arm64 runners so AVX2/FMA and NEON coverage are
PR-visible when the hosted CPU exposes those features. Miri covers the scalar
reference path, and the packet includes a mutation-control artifact that proves
the lane fails on a deliberate score perturbation.

Task 38 adds `crates/ecaz-fault-injection`, an LD_PRELOAD provider for matched
EIO/ENOSPC/slow-disk injection, `ecaz dev fault`, Makefile lanes, and
documentation for the PG-level fault matrix. The provider now hooks `open`,
`open64`, `openat`, `openat2`, read/pread, write/pwrite, fsync, and fdatasync
surfaces. Live lanes run AM-specific fixtures for `ec_hnsw`, `ec_ivf`,
`ec_diskann`, and `ec_spire`: cancellation and backend termination use repeated
AM KNN scans, statement timeout uses repeated AM KNN scans,
`idle_in_transaction_session_timeout` terminates sessions after each AM fixture
is touched inside an open transaction, lock timeout covers blocked
`REINDEX INDEX CONCURRENTLY`, `CREATE INDEX`, and `VACUUM (FULL)` while rolling
back the lock holder even if waiter cleanup errors, memory smoke uses
`ecaz.fault_palloc_nth` and sweeps the currently instrumented scan allocation
points plus build/insert/vacuum callback boundaries for each AM and now
SIGKILLs worker backends during build/scan/insert as an OOM-kill proxy,
resource smoke covers calibrated accumulator `work_mem` pressure with
pressure-sized AM fixtures plus built-in `temp_file_limit` temp-spill failure
and provider-backed ENOSPC on `pgsql_tmp`,
provider-backed slow-disk latency runs against a postmaster restarted through
`ecaz dev fault provider-restart`, and provider-backed I/O smoke now supports
prebuilt relation-path fixtures through `ecaz dev fault prepare` plus
`--assume-prepared`. Provider-backed I/O assertions now accept only expected
provider SQLSTATEs, plus PostgreSQL's specific checkpoint-failure `XX000`
surface for ENOSPC, instead of accepting any database error. WAL-path ENOSPC
smoke now targets `match=pg_wal`, records the backend disconnect as a
crash-recovery surface, and uses `ecaz dev fault provider-restore` to recover
the postmaster, including an immediate stop/start fallback when fast restart
cannot shut down the faulting postmaster. The existing SPIRE Stage E remote
fault surface now has a
packet-local `remote_oom` run through
`ecaz dev spire-multicluster fault-pg18`; that run also exposed and fixed a
fixture bug where long run ids placed Unix sockets under paths longer than
PostgreSQL allows. Those live lanes tag their sessions and assert
postconditions for leftover fault sessions, locks, prepared transactions,
optional `pg_buffercache` fixture pins, and optional `pg_stat_io`
non-decreasing operation counters. Buffer-pin probes now also print a
structured `pg_buffercache_fixture_pins_ok=<bool>` marker for packet checks.
The Task 38 interrupt-site inventory is documented in `docs/hardening.md` under
"Current interrupt inventory".

## Scope Boundary

Task 38 is still scope-bounded to smoke coverage. It now has live PG18
EIO/ENOSPC provider probes and a palloc-failure smoke lane for all four AMs,
but exhaustive per-allocation sweeps inside each build/insert/vacuum callback,
true kernel/cgroup OOM pressure campaigns, WAL rotation edge accounting beyond
WAL-path ENOSPC smoke, SPIRE remote-object fetch faulting, and full
expected-vs-forced WAL/temp-spill accounting remain follow-on expansion.

Task 36 covers the SIMD paths that exist in this tree. There is no AVX-512
product-quantizer implementation, SIMD `unpack_mse_indices` implementation,
arch-specific `rotation.rs` implementation, or IVF/SPIRE scan SIMD accumulator
to exercise yet.

## Validation

Artifacts are under `artifacts/` and recorded in `artifacts/manifest.md`.

- `cargo fmt --all --check`
- `cargo test -p ecaz-fault-injection`
- `cargo test -p ecaz-cli cli_parses_fault`
- `cargo test --features bench --test simd_diff -- --test-threads=1`
- `.github/workflows/ci.yml` `simd-diff` matrix:
  - `ubuntu-24.04` / `avx2`
  - `ubuntu-24.04-arm` / `neon`
- `cargo +nightly miri test --lib -- miri_`
- SIMD mutation control: the production 1536/4 score assertion fails when
  perturbed by `0.01`.
- Live PG18 fault smoke against `ecaz_fault_probe_36_38`:
  - `ecaz dev fault smoke --lane cancel --rows 64`
  - `ecaz dev fault smoke --lane timeout --rows 64` including statement timeout
    and `idle_in_transaction_session_timeout`
  - `ecaz dev fault smoke --lane lock-timeout --rows 64`
  - `ecaz dev fault smoke --lane resource --rows 64` including calibrated
    accumulator pressure, tiny memory settings, and `temp_file_limit`
    temp-spill failure
  - `ecaz dev fault provider-restart --mode enospc-write --path-match pgsql_tmp ...`
  - `ecaz dev fault smoke --lane resource --rows 64 --provider-marker ...`
  - `ecaz dev fault smoke --lane memory --rows 64` including palloc smoke and
    backend-SIGKILL OOM proxy build/scan/insert checks
  - `ecaz dev fault provider-restart --mode slow-disk ...`
  - `ecaz dev fault smoke --lane slow-disk --rows 64 --provider-marker ...`
  - `ecaz dev fault provider-restart --mode eio-read/enospc-write --path-match <relation path> ...`
  - `ecaz dev fault smoke --lane io --am <hnsw|ivf|diskann|spire> --assume-prepared --provider-marker ...`
  - focused HNSW provider SQLSTATE guard for EIO and ENOSPC
  - focused HNSW WAL-path ENOSPC smoke with `--path-match pg_wal`
  - `ecaz dev fault provider-restore`
  - `ecaz dev spire-multicluster fault-pg18 --case remote_oom ...`

## Reviewer Focus

- SIMD tolerance choices and scalar reference isolation.
- Whether the forced backend wrappers are narrow enough for bench/test use.
- Whether the major-callback memory smoke is enough for this checkpoint before
  expanding into exhaustive per-allocation sweeps.
- Whether the optional `pg_buffercache`/`pg_stat_io` probes are enough live
  leak/accounting coverage for this smoke checkpoint.
