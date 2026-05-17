# Task 38: PG-Level Fault Injection (I/O, OOM, Cancellation, Timeouts)

Status: **operator smoke surface implemented locally** — extends Task 37 from
"crash mid-write" to the broader class of operational faults that real
production clusters hit. The local implementation adds
`crates/ecaz-fault-injection`, an LD_PRELOAD provider for matched-path EIO,
ENOSPC, and slow-disk latency injection, extension-side palloc smoke injection
through `ecaz.fault_palloc_nth`, `ecaz dev fault`, Makefile smoke lanes, and
`docs/hardening.md` coverage. Current validation passed provider self-tests,
the full dry-run matrix, and live PG18 probes for cancellation, statement
timeout, `pg_cancel_backend` and `pg_terminate_backend`, lock timeout across
`REINDEX INDEX CONCURRENTLY`, `CREATE INDEX`, and `VACUUM (FULL)`, resource
settings, memory/palloc smoke across build, scan, insert, and vacuum AM
callbacks, provider-backed slow-disk operation, and provider-backed EIO/ENOSPC
against AM-specific `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire` fixtures.
The smoke surface is now in place;
exhaustive per-allocation palloc sweeps, OOM-kill campaigns, WAL
rotation/temp-spill targeting, SPIRE remote-object fetch faulting, and richer
`pg_buffercache`/`pg_stat_io` accounting remain follow-on expansion beyond this
smoke checkpoint.

## Scope

Add fault-injection harnesses for failure modes that happen during normal PG
operation and that ECAZ AMs must survive cleanly:

- **I/O faults**: EIO and ENOSPC on heap reads, index reads, WAL writes, temp
  file spill, and SPIRE remote object fetch.
- **Memory pressure**: palloc failures (`MemoryContextStats`), OOM kills mid
  build/insert/scan, `work_mem` exhaustion in scan accumulators.
- **Query cancellation**: `pg_cancel_backend` and `pg_terminate_backend`
  delivered at every CHECK_FOR_INTERRUPTS site reachable from ECAZ code paths.
- **Statement timeout**: long-running build/scan/vacuum operations cut off by
  `statement_timeout` and `idle_in_transaction_session_timeout`.
- **Lock timeouts**: `lock_timeout` interrupting `CREATE INDEX`, `REINDEX
  CONCURRENTLY`, and `VACUUM (FULL)` mid-acquire.
- **Disk full** during page extension and WAL segment rotation.
- **Slow disk** (latency injection) to surface timing-dependent assumptions.

## Why

Custom AMs frequently have failure-handling gaps that only show under
adversarial operational conditions:

- Cancellation: failure to call `CHECK_FOR_INTERRUPTS` inside long scan or
  build loops leaves operators unable to cancel queries. Conversely, calling
  it inside a half-built page mutation leaks buffer pins or leaves the AM in
  an inconsistent state.
- I/O failure handling: a bare `unwrap()` on a read error converts an EIO
  into a backend crash (and recovery cycle) instead of a clean ERROR.
- Memory: an unhandled `palloc` failure inside an `extern "C"` callback is
  caught by Postgres and converted to ERROR, but Rust state on the stack may
  not have been cleaned up (drop order via `pg_guard`).
- `statement_timeout` fires asynchronously; any code path that holds a buffer
  pin or LWLock when it fires can leak.

None of these are exercised today. Task 37 covers `SIGKILL` recovery, but not
the in-process cleanup paths.

## Approach

1. **I/O injection layer.** Two options, pick one and document the choice:
   - LD_PRELOAD shim (e.g. `libfiu` or a custom one) that injects EIO/ENOSPC
     at configurable byte offsets in matched paths.
   - Filesystem-level: a FUSE filter that returns errors on configurable
     inodes, used only in CI.
   The shim must distinguish ECAZ paths from PG core paths so we exercise
   ECAZ error handling without flagrantly breaking PG.
2. **Memory injection.** Wire a `MemoryContextAlloc` failure hook (PG18 has
   `palloc_extended` with `MCXT_ALLOC_NO_OOM`). For each ECAZ allocation
   site, run a sweep that forces failure at the Nth allocation in a
   workload; assert clean ERROR, no leaked buffers (`pg_buffercache`), no
   leaked LWLocks (`pg_locks`), no double-free.
3. **Cancellation harness.** Use `pg_sleep` markers and a side-channel that
   issues `pg_cancel_backend` at sub-second intervals while a workload runs.
   For each ECAZ entry point, assert:
   - the query terminates within an expected window,
   - no buffer pin remains (`pg_buffercache_pin_count`),
   - no LWLock held (assert via `pg_stat_activity.wait_event`).
4. **Timeout sweeps.** `SET statement_timeout = '50ms'` against
   intentionally-long operations and assert clean ERROR and no leaked state.
   Same for `lock_timeout` against contended DDL.
5. **Resource-exhaustion smoke.** Run builds at `work_mem = 64kB`, scans at
   tiny `effective_cache_size`, and inserts at `maintenance_work_mem = 1MB`
   to surface accumulator bugs.
6. **Leak detectors.** Each test verifies post-conditions:
   - `pg_buffercache_summary` shows no pinned buffers from the test backend,
   - `pg_locks` shows no surviving locks,
   - `pg_stat_io` shows expected vs. forced read/write counts,
   - `pgstat` extension counters do not show negative deltas.

## Implementation

- New crate `crates/ecaz-fault-injection` housing the shim, harness, and
  workload definitions, exposed via `ecaz dev fault` (subcommand of the
  existing operator CLI).
- Make lanes:
  - `make fault-io-smoke` — one EIO/ENOSPC per AM per code path, 30s budget.
  - `make fault-mem-smoke` — palloc-failure sweep capped at first 100 sites.
  - `make fault-cancel-smoke` — cancel sweep across documented entry points.
  - `make fault-full` — full sweep, nightly.

## Validation

- Each lane completes without `PANIC` in the postmaster log and without
  leaked buffers/locks.
- A deliberately introduced bug — e.g., `BufferGetPage(buf).unwrap()` where
  `buf` is `InvalidBuffer` — is caught by the cancellation lane.
- Resource-exhaustion smoke catches an accumulator that hits `palloc`
  failure without recovery.

## Exit Criteria

- All four ECAZ AMs survive the smoke lanes cleanly.
- Documented inventory of `CHECK_FOR_INTERRUPTS` sites in every long-running
  ECAZ loop; missing sites are filed as follow-ups.
- `make fault-full` is nightly-CI-eligible (per Task 49 governance).
- `docs/hardening.md` gains a "fault injection" section describing the model.

## Dependencies

- Requires the same live PG18 environment as Task 37.
- Inherits from Task 37 the "crash probe" model — fault injection adds in-
  process variants alongside the SIGKILL variant.
- Sits below Task 49 (CI promotion governance) for the nightly cadence.
