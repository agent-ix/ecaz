# Task 41 Invariant #2 Strategy: Memory-Context Lifetimes

Base head for this survey: `7fef354f16ebd44eca63700ff666ee4b479ac189`

## Objective

Task 41 invariant #2 says Rust values borrowing from PostgreSQL-owned memory
must not outlive the owning memory context or resource lifetime. The concrete
surfaces are:

- borrowed `&str` / `&[u8]` from `text *`, `bytea *`, `ArrayType`, or other
  varlena Datums;
- `Datum` values or tuple-slot values that alias palloc'd storage;
- buffer/page bytes whose validity depends on a live pin/lock;
- palloc-backed scan-state arrays exposed as Rust slices;
- C strings owned by PostgreSQL catalog/type APIs.

The goal is not to annotate every unsafe block. The goal is to make the default
API shape prevent lifetime escape: scoped closures, guard-owned borrows, owned
copies, or narrow local wrappers.

## High-Level Model

### 1. Datum / Varlena Memory

Rule: any detoasted varlena borrow must be backed by an owning guard or copied
immediately before the guard drops.

Preferred local pattern:

```rust
struct Detoasted...Datum {
    varlena: *mut pg_sys::varlena,
    owned: bool,
}

impl Drop for Detoasted...Datum {
    fn drop(&mut self) {
        if self.owned {
            unsafe { pg_sys::pfree(self.varlena.cast()) };
        }
    }
}
```

Then either:

- expose `with_..._slice(datum, |slice| ...)` so borrows cannot escape; or
- expose `to_vec()` only, so callers receive owned bytes.

Do this locally per AM/file unless a repeated shape proves worth extracting.
Avoid a broad shared detoast abstraction until at least two reviewed packets
show the exact same semantics and error model.

### 2. Tuple-Slot Datums

Rule: a `Datum` read from `TupleTableSlot.tts_values` is valid only while the
slot still owns the tuple contents and before `ExecClearTuple` / slot drop.

Preferred local pattern:

- `with_slot_datum(slot, attnum, label, |datum| ...)` for borrowed access;
- copy/decode inside the closure;
- do not return `Datum`, `&[u8]`, `&[f32]`, or wrappers backed by slot storage.

The HNSW source closure packet started this pattern for source vectors. Apply
it only to call paths where a real borrow can escape or where the current API
hands raw Datums outward.

### 3. Buffer / Page Bytes

Rule: page tuple byte slices are valid only while the relevant buffer guard and
pin/lock are live.

This overlaps the other agent's invariant #3 resource-release track. Do not
race broad page/buffer rewrites. The invariant #2 follow-up should be:

- after or alongside established `LockedBufferGuard` ownership in a file,
  introduce local page-view helpers that take `&LockedBufferGuard` and invoke a
  closure with tuple bytes;
- avoid changing buffer acquisition/release semantics in invariant #2 packets;
- leave pure buffer resource ownership to invariant #3 packets.

### 4. Palloc-Backed Scan State

Rule: palloc arrays stored in scan opaque state must not become long-lived Rust
references disconnected from the owning opaque state.

Preferred pattern:

- return slices through methods on the scan-state/opaque wrapper;
- keep slice use expression-local or closure-scoped;
- document owner fields (`query_values`, `selected_lists`, candidate arrays)
  where palloc lifetime intentionally equals scan lifetime.

This is a later phase because it has more behavioral blast radius than the
local detoast packets.

### 5. C Strings From PostgreSQL

Rule: `CStr::from_ptr` over catalog/type-name memory must be converted to owned
Rust data before the owning PG allocation is freed or before the surrounding
PG object can go away.

Most current sites already call `.to_string_lossy().into_owned()` before
`pfree` or use relation/attribute names synchronously. Treat these as audit
packets unless a borrow crosses a drop/free boundary.

## Ground-Level Strategy

### Phase A: Finish Non-Buffer Varlena Sites

Current state after packets 114-120:

- HNSW source views have higher-ranked closure consumption and guard-owned
  detoast state.
- HNSW build tqvector decode has a local detoast guard.
- IVF build decode has a local detoast guard.
- DiskANN build ecvector decode has a local detoast guard.
- SPIRE build and SPIRE scan decode have local detoast guards.

Remaining detoast inventory is expected to contain guard-internal calls plus
`ecvector_typmod_in` in `src/lib.rs`.

Next local slice:

1. `src/lib.rs` `ecvector_typmod_in`
   - Add a tiny local typmod-array detoast guard or use a closure helper.
   - Preserve exact error strings.
   - Validation: `cargo fmt --all --check`, `cargo check --no-default-features --features pg18`,
     `git diff --check HEAD~1 HEAD`.

Stop condition for Phase A:

- `rg -n "pg_detoast_datum(_packed)?\\(|varlena_to_byte_slice" src/am src/lib.rs -g '*.rs'`
  shows only guard-internal calls and no open-coded detoast/copy/free sequence.

### Phase B: Slot-Datum APIs

Inventory source: `artifacts/slot-datum-inventory.log`.

Target files, in local slices:

1. `src/am/ec_diskann/scan_state.rs`
   - `required_slot_datum` returns `Result<Datum, String>`.
   - If every caller copies immediately, document and leave. If any caller
     defers the Datum past `ExecClearTuple`, convert to scoped closure.

2. `src/am/ec_spire/scan/relation.rs`
   - Already decodes immediately after `required_slot_datum`.
   - Consider converting to a local `with_required_slot_datum` only if review
     finds an actual escape path.

3. `src/am/ec_hnsw/source.rs`
   - Already migrated the source-vector path to closure helpers.
   - Leave remaining `required_slot_datum` public only if callers consume
     immediately; otherwise narrow visibility or add closure helpers.

4. SPIRE CustomScan tuple payload modules
   - These mostly write slot Datums rather than borrow from them.
   - Treat as audit-only unless a Datum is read and borrowed.

Stop condition for Phase B:

- Each `tts_values` read is one of:
  - copied/decoded before slot clear;
  - consumed through a scoped closure;
  - documented as scalar by-value Datum.

### Phase C: Palloc Scan-State Slices

Inventory source: `artifacts/palloc-inventory.log` and query slice entries in
`artifacts/raw-slice-inventory.log`.

Target files, in local slices:

1. `src/am/ec_ivf/scan.rs`
   - `query_values`, `selected_lists`, candidate arrays.
   - Introduce methods on the opaque state for scoped or owner-tied slice
     access.
   - Preserve scan-lifetime ownership and existing pfree points.

2. `src/am/ec_hnsw/scan.rs`
   - `query_values` and grouped heap rerank interactions.
   - Same owner-tied method pattern.

3. `src/am/ec_hnsw/scan_debug.rs`
   - Test/debug surface; do after production scan paths unless it blocks a
     shared helper.

4. `src/am/ec_hnsw/build_parallel.rs`
   - DSM/shared-memory arrays are not ordinary palloc memory. Coordinate with
     Task 40 before rewriting. For invariant #2, produce an audit packet that
     classifies ownership/lifetime and defers synchronization proof where
     appropriate.

Stop condition for Phase C:

- Raw slices over scan opaque fields are only created by owner methods or
  closures, and no `&[T]` escapes the owning scan/build state.

### Phase D: Buffer/Page Byte Views

Inventory source: `artifacts/raw-slice-inventory.log`.

Coordinate with invariant #3. Do not change buffer release semantics here.

Target files, after confirming current invariant #3 progress:

1. `src/am/ec_ivf/page.rs`
   - Introduce local tuple-view helpers that take the existing buffer guard and
     parse/copy inside a closure.

2. `src/am/ec_hnsw/insert.rs`, `src/am/ec_hnsw/vacuum.rs`,
   `src/am/ec_hnsw/graph.rs`, `src/am/ec_hnsw/shared.rs`
   - Apply the same local view pattern only where the buffer guard already
     exists.

3. `src/am/ec_diskann/insert.rs`, `src/am/ec_diskann/routine.rs`,
   `src/am/ec_diskann/scan_state.rs`
   - Prefer local page-view closures near existing `LockedBufferGuard` use.

4. `src/am/ec_spire/page.rs`
   - Local page tuple view helpers; no cross-AM abstraction until patterns
     stabilize.

Stop condition for Phase D:

- Any slice of page memory is bounded by a live buffer guard/pin in the type
  signature or closure call shape.
- Packets cite invariant #3 packet dependencies where buffer ownership was
  already established.

### Phase E: C String and Catalog Borrow Audit

Inventory source: `artifacts/palloc-inventory.log`.

Target files:

- `src/lib.rs`
- AM `options.rs` files
- SPIRE DML/custom-scan helpers
- `src/am/common/explain.rs`

Most sites should remain audit-only because they convert to owned `String`
before freeing or use names synchronously. Write findings only if a borrowed
`&str` can outlive a PG-owned C string.

Stop condition for Phase E:

- Every `CStr::from_ptr` either converts to owned data before the PG owner can
  be freed/dropped, or is documented as synchronous non-escaping use.

## Completion Criteria For Invariant #2

Invariant #2 is complete when:

1. Detoast inventory has no open-coded detoast/borrow/free sequence outside
   local guard implementations.
2. Slot-Datum reads are copied/decoded, closure-scoped, or documented by-value.
3. Palloc scan-state slices are owner-tied and cannot escape scan/build state.
4. Page/buffer slices are tied to live buffer guards or explicitly covered by
   invariant #3 packets.
5. C string uses are owned or synchronous and non-escaping.
6. A final review packet includes fresh inventories, explains remaining
   accepted raw sites, and records validation.

## Coordination Rules

- Keep slices local: one file or one tightly owned module per code commit.
- Pair every code commit with its own Task 41 packet and packet-local artifacts.
- Do not push until requested.
- Do not edit invariant #1 panic/`pg_guard` surfaces in this lane.
- Do not alter buffer acquisition/release ownership in invariant #2 packets
  unless that exact local file already has the invariant #3 guard work landed.
