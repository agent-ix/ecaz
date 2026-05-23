# Task 54 Packet 003 — HNSW `build.rs` Migration to P3 Wrappers

Status: **proposed**

## What this packet does

Migrates the HNSW build-time page/WAL chain onto the §Scope P3 wrappers
landed in packet 002. Touches three files:

- `src/am/ec_hnsw/build.rs` — `write_data_pages`,
  `flush_build_output`, `flush_build_state_with_timing`, and the
  ambuild parallel path at line 257.
- `src/am/ec_hnsw/shared.rs` — adds a safe
  `initialize_metadata_page_handle(RelationHandle, MetadataPage)`
  variant of the existing `unsafe fn initialize_metadata_page` so
  `flush_build_output` can drop its `unsafe { ... }` wrapper.
- `src/am/ec_hnsw/insert.rs` — caller-side update to pass a
  `RelationHandle` into the migrated `write_data_pages`.

## Migration

### `write_data_pages`

| Before | After |
| --- | --- |
| `unsafe fn write_data_pages(index_relation: pg_sys::Relation, ...)` | `fn write_data_pages(handle: RelationHandle, ...)` |
| `unsafe { LockedBufferGuard::read_main_locked(...) }` | `LockedBufferGuard::read_main_locked_handle(...)` |
| `unsafe { wal::GenericXLogTxn::start(index_relation) }` | `wal::WalTxnScope::start_handle(handle)` |
| `let page_ptr = wal_txn.register_locked_buffer_full_image(&buffer);` + `unsafe { pg_sys::PageInit(page_ptr, page_size, 0) }` | `let mut page = wal_txn.register_page(&buffer); page.init(0);` |
| `unsafe { pg_sys::PageAddItemExtended(...) }` + open-coded `if offset == InvalidOffsetNumber { error!() }` | `page.add_item(tuple).unwrap_or_else(\|err\| ...)` |

Result: write_data_pages goes from 4 inline `unsafe { ... }` blocks
to 0; the function signature itself is now safe `fn`.

### `flush_build_output`

| Before | After |
| --- | --- |
| `unsafe fn flush_build_output(index_relation: pg_sys::Relation, ...)` | `fn flush_build_output(handle: RelationHandle, ...)` |
| `unsafe { write_data_pages(index_relation, ...) }` + `unsafe { shared::initialize_metadata_page(...) }` | `write_data_pages(handle, ...)` + `shared::initialize_metadata_page_handle(handle, ...)` |

### `flush_build_state_with_timing`

Now constructs `RelationHandle` once at function entry and passes it
to `flush_build_output`. Drops the `unsafe { flush_build_output(...) }`
wrapper because the callee is now a safe `fn`.

### `shared::initialize_metadata_page`

Becomes a thin `unsafe fn` shim that validates the relation pointer
into a `RelationHandle` and delegates to a new safe
`initialize_metadata_page_handle(RelationHandle, MetadataPage)`. The
new safe variant uses `LockedBufferGuard::read_main_handle{,_locked}`
and the existing `rewrite_metadata_buffer` (unchanged).

## Per-file unsafe block counts

| File | Pre (HEAD before this packet) | Post | Delta | Notes |
| --- | ---: | ---: | ---: | --- |
| `src/am/ec_hnsw/build.rs` | 18 | **11** | **-7** | **Target ≤ 12 — met with margin (-1).** |
| `src/am/ec_hnsw/shared.rs` | 21 | 21 | 0 | New safe `initialize_metadata_page_handle` uses safe wrapper methods; no new `unsafe { ... }` blocks introduced. |
| `src/am/ec_hnsw/insert.rs` | 25 | 25 | 0 | Caller-side rewrite at L658 only — same call site, now passes a `RelationHandle`. |
| `src/` total | 966 (post-002) | 963 | -3 | Three blocks removed from `build.rs` net the +6 wrapper additions in packet 002. |

`build.rs` ≤ 12 §Exit Criterion: **met**.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` — passes.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — passes.
- No behavioral change: the wrappers are thin and PostgreSQL semantics
  (PageInit/PageAddItemExtended/GenericXLog) are unchanged.

## Files touched

- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/shared.rs`
- `src/am/ec_hnsw/insert.rs`

## References

- `plan/tasks/54-common-p3-page-wal-wrappers.md` §Scope, §Migration Targets, §Exit Criteria.
- `reviews/task-54/002-p3-wrappers/request.md` — wrapper surface added in packet 002.
