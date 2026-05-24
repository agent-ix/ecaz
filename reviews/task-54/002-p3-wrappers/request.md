# Task 54 Packet 002 — P3 Page / WAL / Buffer Typed Wrappers

Status: **proposed**

## What this packet does

Adds the §Scope P3 typed wrappers in `src/storage/` (Task 54 plan).
**No consumer migration yet** — packets 003 and 004 move HNSW
`build.rs` / `vacuum.rs` onto the wrappers.

## Surface added

### `src/storage/wal.rs`

- `WalTxnScope<'rel>` — safe-surface name for `GenericXLogTxn` with
  the relation-liveness invariant encoded in `'rel`. Two
  constructors:
  - `unsafe fn start(relation: pg_sys::Relation)` — raw-pointer entry
    point retained for callers that don't yet hold a
    `RelationHandle`.
  - `fn start_handle(handle: RelationHandle)` — safe variant
    inheriting the handle's "live relation" SAFETY contract.
  - Safe `register_page(&mut self, &LockedBufferGuard)` and
    `finish(self) -> XLogRecPtr` operations.
- `RegisteredBufferPage<'txn, 'buffer>` gains safe `init(special_size)`
  (wraps `PageInit`) and `add_item(payload) -> Result<OffsetNumber,
  PageAddItemError>` (wraps `PageAddItemExtended`).
- `PageAddItemError { block_number }` carries the block number so
  callers can format the same `pgrx::error!` messages without
  reaching for `self.buffer` after the page is moved.

### `src/storage/buffer_guard.rs`

- `LockedBufferGuard::read_main_handle(...)` — safe variant of
  `read_main` taking `RelationHandle`.
- `LockedBufferGuard::read_main_locked_handle(...)` — safe variant
  of `read_main_locked` taking `RelationHandle`.

### `PinnedBufferGuard` (§Scope alias)

The task-plan names `BufferPinScope<'rel>` as a separate wrapper.
The repository already has `PinnedBufferGuard` covering the same
operations (`read_main` + `from_pinned` + `lock` + Drop release).
This packet treats `PinnedBufferGuard` as the in-repo equivalent
and adds no separate type; the alias is documented in the packet
manifest. If a `'rel`-bound variant becomes load-bearing, a future
packet adds it.

## Per-file unsafe block counts

| File | Before | After | Delta | Notes |
| --- | ---: | ---: | ---: | --- |
| `src/storage/wal.rs` | 5 | 9 | +4 | +4 wrapper-internal: `WalTxnScope::start`, `start_handle`, `RegisteredBufferPage::init`, `add_item`. **Category shift** — these are absorbed when consumers migrate. |
| `src/storage/buffer_guard.rs` | 22 | 24 | +2 | +2 wrapper-internal: `read_main_handle`, `read_main_locked_handle`. **Category shift** — same disposition. |
| `src/` total | 960 | 966 | +6 | All six new blocks are wrapper-internal. |

The `src/` total transiently rises in this packet by design (per
§Slice and Packet Rules → "Wrapper-side blocks in `src/storage/`
are counted but recorded as the intended category shift"). Packets
003 and 004 absorb these gains by removing caller-side `unsafe {}`
blocks in `build.rs` and `vacuum.rs`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` — passes (clean compile, exit 0).
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — passes.
- No behavior change: new code paths are dead until packet 003 wires consumers.
- `#[allow(dead_code)]` attributes on the new items are scoped to "Consumed by HNSW migration in Task 54 packets 003/004" and will be removed automatically by clippy when migration lands.

## Files touched

- `src/storage/wal.rs`
- `src/storage/buffer_guard.rs`

No consumer files touched.

## References

- `plan/tasks/54-common-p3-page-wal-wrappers.md` §Scope, §Slice and Packet Rules.
- `reviews/task-54/001-execution-plan/request.md` — slice plan.
