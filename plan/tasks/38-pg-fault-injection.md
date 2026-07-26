# Task 38: PG-Level Fault Injection (I/O, OOM, Cancellation, Timeouts)

Status: **open; Apple-M5 local PG18 evidence and reviewed source slices
complete, authoritative `fault-full` aggregation and designated Intel/Linux
execution pending** — the historical Linux packet proved the four-AM
`ec_hnsw`/`ec_ivf`/`ec_diskann`/`ec_spire` surface. The current Task 38 branch
adds `ec_distann` as a first-class AM with separate RaBitQ, TurboQuant, and
grouped-PQ fixtures, exact-peer socket reset/latency provider modes, measured
slow-disk comparison inputs, stronger accumulator markers, reset-safe palloc
handling, armed exact-peer DistANN TCP and SPIRE Unix-socket drills, an
executable seven-fixture systemd-scoped cgroup OOM drill, and repeatable
seven-fixture cancellation/palloc mutation controls.

The seven current fixtures flow through real build, KNN scan, insert,
delete/vacuum, DDL, cancel/terminate, statement/idle/lock timeout, palloc,
RLIMIT/SIGKILL proxy, file I/O, WAL/temp/accounting, and shared cleanup
inventory. DistANN uses 64-D RaBitQ, supported 1536-D no-QJL TurboQuant, and
64-D grouped-PQ shapes so interruption and pressure occur around actual
physical generation/traversal/owner scoring/payload and codec batch work.

Evidence is intentionally split. The old four-AM live Linux results remain
historical background. The current macOS arm64 host can validate Rust/CLI
planning and local PostgreSQL behavior, but cannot load the Linux LD_PRELOAD
provider or execute cgroup-v2 user scopes. Therefore live DistANN provider
EIO/ENOSPC/slow-disk, SPIRE/DistANN socket faults, and cgroup OOM remain
unavailable on this host until Linux evidence lands. SPIRE remote SQL transport
does exist and is actionable; only future SPIRE object-store reads are
nonexistent. No CI/nightly execution is claimed, and this task stays open for
operator-scheduled execution on the designated Intel/Linux host. Review
packets 001 through 007 have outside approval for their stated implementation,
source, inventory, and M5-local evidence scopes; those approvals do not
substitute for the missing live Linux evidence.

The three Apple-M5-verifiable gaps identified by packet 005 are complete.
Packet 006 inventories every explicit ECAZ interrupt boundary and files Task
200 for unpolled-loop classification/remediation. Packet 007 proves the
production cancellation oracle rejects a deliberate wrong AM failure and
proves the real-AM recovery oracle rejects an unrecovered palloc state before
accepting the identical scan after disarm, across all seven fixtures.

Task 38 still has an implementation gap that can be source-reviewed on M5:
`make fault-full` must become a genuinely authoritative aggregate rather than
the current generic dry-run dependency list. It must include the approved
mutation, remote-socket, and cgroup operators and must orchestrate distinct
provider modes, restart/restore, exact markers/arm files, and same-run slow
baselines. After that implementation is reviewed, the designated Intel/Linux
host must execute and evidence all of the following:

- all three DistANN codecs across applicable heap/index/WAL/temp paths for
  provider-backed EIO and ENOSPC, plus a same-run measured slow-disk baseline
  delta, with exact mode/path markers, accepted outcomes, recovery, and shared
  postconditions;
- DistANN owner/payload TCP reset with an accepted clean outcome, and TCP slow
  with the expected source identity and baseline-plus-latency threshold,
  followed by exact expected-source recovery;
- SPIRE participant named-Unix reset/slow with a validated healthy baseline,
  accepted reset error/degraded result, slow stable profile equal to baseline,
  baseline-plus-latency timing, and recovered stable profile equal to
  baseline; and
- seven systemd/cgroup-v2 OOM cases with `Result=oom-kill`, dead scoped
  postmaster, exact row-count equality, valid/ready index, forced AM scan,
  shared postconditions, and clean post-recovery stop.

Every executed lane must retain a postmaster log with no `PANIC` and pass the
buffer/lock cleanup criteria.

## Scope

Add fault-injection harnesses for failure modes that happen during normal PG
operation and that ECAZ AMs must survive cleanly:

- **I/O faults**: EIO and ENOSPC on heap reads, index reads, WAL writes, and
  temp file spill. SPIRE object-store fetch is deferred because the production
  path does not exist.
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

Task 38 now exercises the local in-process cleanup paths and supplies armed
Linux provider/cgroup operators. Task 37 remains the historical source of the
general `SIGKILL` recovery model; the live Task 38 Linux provider, socket, and
cgroup matrix remains pending as stated above.

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
  - `make fault-full` — full locally authoritative sweep when host
    prerequisites are reachable.

## Validation

- Each lane completes without `PANIC` in the postmaster log and without
  leaked buffers/locks.
- A deliberately introduced bug — e.g., `BufferGetPage(buf).unwrap()` where
  `buf` is `InvalidBuffer` — is caught by the cancellation lane.
- Resource-exhaustion smoke catches an accumulator that hits `palloc`
  failure without recovery.

## Exit Criteria

- All five ECAZ AMs, including all three supported DistANN codec fixtures,
  survive the applicable smoke lanes cleanly.
- Documented inventory of `CHECK_FOR_INTERRUPTS` sites in every long-running
  ECAZ loop; missing sites are filed as follow-ups.
- `make fault-full` is locally authoritative; any later nightly/CI eligibility
  is owned by Task 49 governance and is not claimed here.
- `docs/hardening.md` gains a "fault injection" section describing the model.

## Dependencies

- Requires the same live PG18 environment as Task 37.
- Inherits from Task 37 the "crash probe" model — fault injection adds in-
  process variants alongside the SIGKILL variant.
- Sits below Task 49 (CI promotion governance) for the nightly cadence.
