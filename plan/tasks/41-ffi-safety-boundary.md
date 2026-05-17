# Task 41: FFI Safety Boundary (Panic, `pg_guard`, Memory Context Lifetimes)

Status: **in progress** — closes a class of latent bugs that no existing lane
catches: Rust code crossing the C boundary into PostgreSQL in a way that is
formally UB or that leaks Postgres-side resources.

## Current PG Resource Wrapper Track

The active first subtrack is structural removal of PostgreSQL relation-resource
unsafe sites. `AccessShareIndexRelation` is the current RAII wrapper for
`index_open(... AccessShareLock)` / `index_close`. It depends on the pgrx ERROR
contract that `pgrx::error!` unwinds Rust frames under `pg_guard`, so destructors
run before PostgreSQL observes the ERROR. Re-audit this assumption on every pgrx
bump or `pg_guard` behavior change.

Closeout requirements for this subtrack:

- migrate all raw `open_valid_ec_*_index` callers to guard-owning code,
- delete the compatibility helpers
  (`open_valid_ec_hnsw_index`, `open_valid_ec_ivf_index`,
  `open_valid_ec_spire_index`, `open_valid_ec_diskann_index`) once their last
  callers are gone,
- consolidate validation-only callsites behind a small helper that opens,
  validates, and drops when no AM read is needed,
- keep AM helper calls scoped so raw relation pointers never escape the guard,
- prefer re-opening short guards over one long guard when a long guard would
  span environment-variable lookups, non-PostgreSQL FFI, SPI, or broad
  diagnostic control flow,
- keep SPI and local heap helper work outside relation-guard scopes unless the
  AM explicitly requires the relation to remain open.

The next Task 41 slice should be selected from the survey packet at
`review/31161-c1-task41-unsafe-surface-strategy/`. Prefer structural cluster
removal over per-site annotation: wrappers first for relation/buffer/snapshot
resources, Task 40 for synchronization primitives, Task 43 for proof coverage,
and Task 35 comments only after the relevant structural refactor lands or is
abandoned.

## Scope

Audit and enforce three invariants at the Rust↔Postgres boundary:

1. **Panic safety.** Every Rust function reachable from Postgres as a C
   callback must either be `#[pg_guard]` (pgrx-managed unwind catch) or call
   `std::panic::catch_unwind` itself before any code that can panic. A panic
   that unwinds across the FFI boundary is UB.
2. **Memory context lifetimes.** Any Rust value that borrows from a Postgres
   memory context must not outlive that context. Specifically: borrowed
   `&str` / `&[u8]` from `text *` / `bytea *` palloc'd memory, `Buffer`
   contents that hold a pin, `Datum`s aliasing palloc'd storage.
3. **Resource release on early return.** Buffer pins, LWLocks, ResourceOwner
   handles, snapshots, and SPI tuptables must be released on every exit path,
   including the error path that pgrx converts into a PG ERROR.

## Why

These three invariants are language- and ecosystem-specific UB sources that
no general Rust tool catches:

- **Miri** does not model pgrx callbacks; it cannot see the FFI boundary.
- **Sanitizers** detect heap corruption from leaks/UAF but only if the
  workload triggers the bad allocation pattern; structural review catches it
  every time.
- **Clippy** lints exist for some patterns but not for "panic across FFI."
- **Rudra/MIRAI** focus on `Send`/`Sync`/aliasing; they do not understand PG
  memory contexts at all.

Pgrx provides `#[pg_guard]` and helper types but does not enforce their use.
Today the only protection is convention. A single `extern "C" fn` without
`pg_guard` that calls a panicking helper is silent UB until something else
makes the panic fire — at which point the postmaster's child crashes and
recovery starts (best case) or memory is corrupted (worst case).

## Approach

1. **Inventory.** Generate the complete list of FFI entry points:
   - `rg -n 'extern "C" fn' src/` + every `#[pg_extern]`, `#[pg_aggregate]`,
     `#[pg_operator]`, `IndexAmRoutine` field, `CustomScanState` callback,
     `RegisterXactCallback`, `RegisterSubXactCallback`, etc.
   - Cross-reference against `#[pg_guard]` (or pgrx-managed equivalents).
   - The diff is the panic-unsafe surface; first packet eliminates it.
2. **dylint enforcement.** Author a `dylint` lint:
   `ecaz_panic_across_ffi` — denies any `extern "C" fn` body that can reach
   a panic without a guard frame. Pair with a `#[allow(...)]` and review-
   packet note for the rare exception.
3. **Memory context lifetimes.** Newtype palloc'd lifetimes: introduce
   `PallocCtx<'cx>` and require any `&'cx [u8]` / `&'cx str` to carry it,
   so the compiler tracks "this borrow comes from memory context `cx` and
   must not escape it." Where pgrx already does this, ensure ECAZ never
   bypasses with raw `from_raw_parts`.
4. **Resource release.** Wrap PG resources in RAII types:
   - `BufferPin` (drops via `ReleaseBuffer`),
   - `LwLockGuard` (drops via `LWLockRelease`),
   - `Snapshot` (drops via `UnregisterSnapshot`),
   - `SpiTuptable` (drops via `SPI_freetuptable`).
   Where pgrx already provides these, audit ECAZ for raw `pg_sys::` calls
   that bypass them. Add a lint that flags raw `LWLockAcquire` /
   `BufferGetBlock` calls outside the wrapper modules.
5. **Drop-order audit.** Pgrx unwinds the Rust stack on PG ERROR via
   `longjmp` — drop order matters. Tests:
   - force ERROR mid-scan and assert no leaked buffer pin,
   - force ERROR mid-build and assert no leaked maintenance memory context.
   This dovetails with Task 38 (fault injection).
6. **Make lanes:**
   - `make ffi-audit` — runs the inventory script and fails if any FFI
     entry point is unguarded; emits a report at
     `review/<packet>/artifacts/ffi-inventory.md`.
   - `make ffi-lint` — runs the `dylint` lint suite over the workspace.
   - `make ffi-leak-smoke` — paired with Task 38, forces ERROR at each
     reachable site and checks for leaks.

## Validation

- `make ffi-audit` produces an empty unguarded-entry list. The inventory is
  committed to `docs/ffi-inventory.md` and updated whenever entry points
  change.
- `make ffi-lint` is clean across the workspace.
- A deliberately added `extern "C" fn` without `#[pg_guard]` (in a test
  fixture) is caught by the lint and the audit.
- A deliberately leaked `Buffer` pin under forced ERROR is caught by the
  leak smoke lane.

## Exit Criteria

- Every FFI entry point is `#[pg_guard]` or has a documented exception with
  a `catch_unwind` frame.
- All raw `pg_sys::` resource handles (buffer pins, locks, snapshots) are
  funneled through RAII wrappers; lint enforces this.
- `docs/ffi-inventory.md` is authoritative and verified weekly.
- `make ffi-audit` runs in PR CI per Task 49 governance.

## Dependencies

- Independent of Tasks 36–40; can land in parallel.
- Pairs with Task 35 (unsafe burndown) — many unguarded entry points are
  also unsafe sites awaiting review.
- The leak-smoke component depends on Task 38 (fault injection plumbing).
