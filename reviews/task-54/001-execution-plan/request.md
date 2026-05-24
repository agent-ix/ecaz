# Task 54 Packet 001 — Execution Plan: P3 Page/WAL/Buffer Typed Wrappers

Status: **plan**

## Scope of this packet

Lay out the slice plan, target file inventory, and pre-task baseline
unsafe-block counts for the Task 54 P3 burndown. No code lands in this
packet — packet 002 begins the wrapper module.

## Baseline (HEAD = task-54 branch tip)

`grep -c "unsafe {" <file>` against `src/`:

| File | Unsafe blocks | Notes |
| --- | ---: | --- |
| `src/am/ec_hnsw/build.rs` | 18 | §Migration target — write_data_pages / flush_build_state_with_timing / flush_build_output |
| `src/am/ec_hnsw/vacuum.rs` | 18 | §Migration target — VacuumPageRewrite chain |
| `src/storage/buffer_guard.rs` | 22 | §Migration target — self-narrow (task-plan calls this `locked_buffer.rs`; the in-repo module name is `buffer_guard.rs`, same surface) |
| `src/storage/wal.rs` | 5 | §Migration target — `WalTxnScope` swallows start/finish unsafe |
| `src/am/ec_hnsw/insert.rs` | 25 (informational) | HNSW consumer that uses the same primitives; not in §Migration Targets — listed here as a stretch consumer for free deltas |
| `src/am/ec_hnsw/shared.rs` | 21 (informational) | `rewrite_metadata_buffer` uses the same WAL/PageInit chain |
| `src/` total | 960 |  |

## Targets (§Exit Criteria)

- `src/am/ec_hnsw/build.rs` ≤ **12** (-6 minimum)
- `src/am/ec_hnsw/vacuum.rs` ≤ **14** (-4 minimum)
- `src/storage/buffer_guard.rs` ≤ **16** (-6 minimum)
- `src/storage/wal.rs` ≤ **3** (-2 minimum)

## Wrapper Surface (§Scope)

The task-plan names map to in-repo surface as follows:

| Task-plan name | Concrete addition | Module |
| --- | --- | --- |
| `WalTxnScope<'rel>` | thin wrapper over existing `GenericXLogTxn`; encodes relation lifetime via `PhantomData<&'rel ()>` and exposes safe `register_page` / `finish` / `abort` | `src/storage/wal.rs` (extend) |
| `PageInitGuard<'buf>` | safe `init(special_size: usize)` method on the WAL-registered writable page | `src/storage/wal.rs` (extend `RegisteredBufferPage`) — gated behind `WalTxnScope::register_page` |
| `PageAddItemExtendedGuard<'page>` | safe `add_item(payload: &[u8]) -> Result<OffsetNumber, PageAddItemError>` on the WAL-registered writable page | same — extends `RegisteredBufferPage` |
| `BufferPinScope<'rel>` | already lives in-repo as `PinnedBufferGuard` (`src/storage/buffer_guard.rs`); the `'rel` lifetime convention is the existing "live relation handle" SAFETY contract. No new type needed; the doc-comment will record this equivalence | `src/storage/buffer_guard.rs` (doc) |
| Extend `LockedBufferGuard` | self-narrow remaining caller-side `unsafe { ... }` wraps where the buffer-guard invariant already proves safety | `src/storage/buffer_guard.rs` |

`PageAddItemError` records the block number for caller-side `pgrx::error!`
formatting; this replaces the open-coded `if offset == InvalidOffsetNumber
{ pgrx::error!(...) }` repetition at every call site.

## Slice plan

1. **002 — wrappers (no consumer change)**: add `WalTxnScope`, extend
   `RegisteredBufferPage` with `init` / `add_item`, add
   `PageAddItemError`. No call-site moves; src/ total unsafe count
   should rise only inside `wal.rs` (new internal `unsafe` is offset
   by lifting unsafe out of consumers in subsequent slices). Compile
   gate: `cargo check --features pg18,bench` + clippy.
2. **003 — migrate HNSW `build.rs`**: convert `write_data_pages`,
   `flush_build_state_with_timing`, `flush_build_output` to consume
   the wrappers. Target: `build.rs` ≤ 12.
3. **004 — migrate HNSW `vacuum.rs` + buffer_guard self-narrow**:
   convert `VacuumPageRewrite::start` to consume `WalTxnScope`;
   self-narrow `buffer_guard.rs` interior unsafe wraps where the
   invariant is provable. Target: `vacuum.rs` ≤ 14, `buffer_guard.rs`
   ≤ 16, `wal.rs` ≤ 3.
4. **005 — benchmark + closeout**: run `ecaz bench suite` for HNSW
   latency + storage on `benchmarks/task-50-m5-hnsw-baseline/`
   prefixes; write closeout packet citing per-file deltas, src/ total
   change, and SPIRE / IVF / DiskANN consumer-site handoff list.

## Migration handoff (deferred to Tasks 55/56/57)

Same wrappers will absorb the following non-HNSW call sites once their
owning task lands:

- IVF: `src/am/ec_ivf/page.rs`, `src/am/ec_ivf/build.rs`
- DiskANN: `src/am/ec_diskann/insert.rs`, `src/am/ec_diskann/ambuild.rs`
- SPIRE: `src/am/ec_spire/page.rs`

This packet writes the handoff list into `005-closeout` as part of the
§Exit Criteria deliverable.

## Out of scope

- Non-HNSW consumer migration (locked by reviewer scope rule).
- WAL record format changes (§Non-Goals).
- AM-callback `unsafe extern` shells (§Non-Goals).
- `src/am/ec_hnsw/insert.rs` / `shared.rs` consumer migration is *not
  in* §Migration Targets and is treated as a deferred stretch
  consumer; this packet plans only the §Migration-target files.

## Validation gates (per slice)

Per CLAUDE.md *Common Rules → Checkpoint Rules*, primary validation is
static (`cargo check`, `cargo clippy`, unsafe-block counts). Bench
gate runs once in slice 005 against the post-Task-50 M5 baseline.

- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- per-file `grep -c "unsafe {" …` snapshot

## References

- `plan/tasks/54-common-p3-page-wal-wrappers.md`
- `reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md` (§P3 disposition)
- `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md` (§"`build.rs` ceiling")
- `benchmarks/task-50-m5-hnsw-baseline/manifest.md` (pre-state for §Performance Gate)
